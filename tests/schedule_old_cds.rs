//! Independent public OldCDS domain, vectors, provenance, guards and errors.
//!
//! Every expected date, flag, count and error context below is derived by hand
//! from `docs/M7_OLD_CDS_CONTRACT.md`, or recomputed by the small day-walk and
//! month-number helpers in this file. No expectation reuses production OldCDS
//! generation, and business-day expectations implement the seven conventions
//! from their requirement definitions.

use sabiquant::calendar::{
    BusinessDayConvention as Bdc, Calendar, CalendarError, EndOfMonth, JointRule, TargetCalendar,
    TargetPolicy,
};
use sabiquant::error::{Error as RootError, IdentifierError};
use sabiquant::schedule::{
    DateGenerationRule as Generation, ExplicitStubDates, OldCdsProvenance,
    PeriodRegularity as Regularity, Schedule, ScheduleBuilder, ScheduleCalendar, ScheduleError,
    SpecialRollRule as Roll, StubPolicy,
};
use sabiquant::time::{Date, DateError, Period, TimeUnit, Weekday};
use std::collections::{HashSet, hash_map::DefaultHasher};
use std::error::Error;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// The contract's uniform boundary-cap maximum.
const MAXIMUM_CAP: u32 = 3_652_059;

#[expect(
    clippy::expect_used,
    reason = "literal dates are hand-checked valid ISO dates"
)]
fn d(text: &str) -> Date {
    text.parse().expect("valid literal date")
}

/// Parses a hand-written, whitespace-separated list of ISO dates.
fn dates(text: &str) -> Vec<Date> {
    text.split_whitespace().map(d).collect()
}

#[expect(
    clippy::panic,
    reason = "hand-written flag literals contain only R (Regular) and S (Stub)"
)]
fn flags(text: &str) -> Vec<Regularity> {
    text.chars()
        .map(|flag| match flag {
            'R' => Regularity::Regular,
            'S' => Regularity::Stub,
            other => panic!("unknown literal flag {other}"),
        })
        .collect()
}

/// The raw vector: start, the listed interiors, then the terminal.
fn joined(start: &str, interiors: &str, terminal: &str) -> Vec<Date> {
    let mut raw = vec![d(start)];
    raw.extend(dates(interiors));
    raw.push(d(terminal));
    raw
}

#[expect(
    clippy::panic,
    reason = "a rejected public request must not return a value"
)]
fn rejected<T: Debug>(result: Result<T, ScheduleError>) -> ScheduleError {
    match result {
        Ok(value) => panic!("expected rejection, got {value:?}"),
        Err(error) => error,
    }
}

fn months(length: i32) -> Period {
    Period::new(length, TimeUnit::Months)
}

fn years(length: i32) -> Period {
    Period::new(length, TimeUnit::Years)
}

/// Stored representation; Period equality itself normalizes 12M and 1Y.
fn stored(period: Period) -> (i32, TimeUnit) {
    (period.length(), period.unit())
}

fn plain(start: Date, end: Date) -> Result<ScheduleBuilder, ScheduleError> {
    ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Plain(Calendar::null()),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )
}

fn on(
    start: Date,
    end: Date,
    calendar: Calendar,
    ordinary: Bdc,
    terminal: Bdc,
) -> Result<ScheduleBuilder, ScheduleError> {
    ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Plain(calendar),
        ordinary,
        terminal,
    )
}

fn guarded(
    start: Date,
    end: Date,
    calendar: TargetCalendar,
) -> Result<ScheduleBuilder, ScheduleError> {
    ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Target(calendar),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )
}

fn hash<T: Hash>(value: &T) -> u64 {
    let mut state = DefaultHasher::new();
    value.hash(&mut state);
    state.finish()
}

fn cloned<T: Clone>(value: &T) -> T {
    value.clone()
}

/// A calendar without weekend rules whose only holidays fill the inclusive ranges.
fn closed_ranges(ranges: &[(&str, &str)]) -> Result<Calendar, DateError> {
    let mut closed = Vec::new();
    for &(first, last) in ranges {
        let (first, last) = (d(first), d(last));
        for offset in 0..=last.days_since(first) {
            closed.push(first.checked_add_days(i64::from(offset))?);
        }
    }
    Ok(Calendar::custom([], closed))
}

/// One hand-derived OldCDS definition and its expected public projection.
#[derive(Clone, Debug)]
struct Case {
    start: Date,
    end: Date,
    tenor: Period,
    raw: Vec<Date>,
    flags: Vec<Regularity>,
    initial: Date,
    anchor: Date,
    terminal: Date,
    omitted: Option<Date>,
}

/// Builds under the all-business Null calendar with moving conventions, so the
/// adjusted vector must equal the raw vector. Also checks the metadata, every
/// provenance inspector, cap identity and the exact count at the cap boundary.
fn verify(case: &Case) -> Result<Schedule, Box<dyn Error>> {
    let builder = ScheduleBuilder::new(
        case.start,
        case.end,
        ScheduleCalendar::Plain(Calendar::null()),
        Bdc::Following,
        Bdc::Preceding,
    )?;
    let count = u32::try_from(case.raw.len())?;
    let schedule = builder.build_old_cds(case.tenor, count)?;
    assert_eq!(schedule.contractual_dates(), case.raw, "{case:?}");
    assert_eq!(schedule.adjusted_dates(), case.raw, "{case:?}");
    assert_eq!(schedule.regularity(), case.flags, "{case:?}");
    assert!(!schedule.regularity().contains(&Regularity::NonPeriodic));
    assert_eq!(schedule.generation_rule(), Generation::OldCds);
    assert_eq!(schedule.tenor().map(stored), Some(stored(case.tenor)));
    assert_eq!(schedule.stub_policy(), None);
    assert_eq!(schedule.end_of_month(), None);
    assert_eq!(schedule.explicit_stub_dates(), None);
    assert!(schedule.roll_provenance().is_none());
    assert_eq!(schedule.lattice_anchor(), None);
    let provenance = schedule
        .old_cds_provenance()
        .ok_or("missing OldCDS provenance")?;
    assert_eq!(provenance.generation_start(), case.start);
    assert_eq!(provenance.requested_end(), case.end);
    assert_eq!(provenance.initial_quarter(), case.initial, "{case:?}");
    assert_eq!(provenance.recurrence_anchor(), case.anchor, "{case:?}");
    assert_eq!(provenance.resolved_end(), case.terminal, "{case:?}");
    assert_eq!(provenance.omitted_quarter(), case.omitted, "{case:?}");
    assert_eq!(stored(provenance.tenor()), stored(case.tenor));
    assert_eq!(builder.build_old_cds(case.tenor, MAXIMUM_CAP)?, schedule);
    if count > ScheduleBuilder::MIN_BOUNDARIES {
        assert!(
            matches!(builder.build_old_cds(case.tenor, count - 1),
                Err(ScheduleError::BoundaryLimitExceeded { maximum, required_at_least, .. })
                if maximum == count - 1 && required_at_least == count),
            "{case:?}"
        );
    }
    Ok(schedule)
}

fn month_days(year: i32, month: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Independent Gregorian day number; day 0 (0001-01-01) was a Monday.
fn ordinal(date: Date) -> i64 {
    let (year, month, day) = date.to_ymd();
    let y = i64::from(year - 1);
    let before: u32 = (1..month).map(|m| month_days(year, m)).sum();
    365 * y + y / 4 - y / 100 + y / 400 + i64::from(before + day - 1)
}

fn weekend(date: Date) -> bool {
    ordinal(date).rem_euclid(7) >= 5
}

/// The seven conventions, written from their requirement definitions, for a
/// weekends-only calendar: Following/Preceding are the nearest business days on
/// or after/before the date, and the other rules choose between them.
fn expected_adjust(date: Date, convention: Bdc) -> Result<Date, DateError> {
    if convention == Bdc::Unadjusted || !weekend(date) {
        return Ok(date);
    }
    let mut following = date;
    while weekend(following) {
        following = following.checked_add_days(1)?;
    }
    let mut preceding = date;
    while weekend(preceding) {
        preceding = preceding.checked_sub_days(1)?;
    }
    let same_month = |other: Date| other.year() == date.year() && other.month() == date.month();
    Ok(match convention {
        Bdc::Following => following,
        Bdc::Preceding => preceding,
        Bdc::ModifiedFollowing => {
            if same_month(following) {
                following
            } else {
                preceding
            }
        }
        Bdc::ModifiedPreceding => {
            if same_month(preceding) {
                preceding
            } else {
                following
            }
        }
        Bdc::HalfMonthModifiedFollowing => {
            if !same_month(following) || (date.day() <= 15 && following.day() > 15) {
                preceding
            } else {
                following
            }
        }
        Bdc::Nearest => {
            if following.days_since(date) <= date.days_since(preceding) {
                following
            } else {
                preceding
            }
        }
        Bdc::Unadjusted => date,
    })
}

/// Independent expectation written from the contract text.
#[derive(Debug)]
struct Expected {
    raw: Vec<Date>,
    flags: Vec<Regularity>,
    initial: Date,
    anchor: Date,
    terminal: Date,
    omitted: Option<Date>,
}

/// Q(x): walks forward one civil day at a time to a quarter-month twentieth.
fn quarter_on_or_after(from: Date) -> Result<Date, DateError> {
    let mut date = from;
    while date.day() != 20 || !date.month().is_multiple_of(3) {
        date = date.checked_add_days(1)?;
    }
    Ok(date)
}

/// A strictly increasing civil month number (year * 12 + month).
fn month_number(date: Date) -> i64 {
    i64::from(date.year()) * 12 + i64::from(date.month())
}

fn twentieth(number: i64) -> Result<Date, Box<dyn Error>> {
    let zero_based = number - 1;
    let year = i32::try_from(zero_based.div_euclid(12))?;
    let month = u32::try_from(zero_based.rem_euclid(12) + 1)?;
    Ok(Date::from_ymd(year, month, 20)?)
}

fn stride_months(tenor: Period) -> i64 {
    let length = i64::from(tenor.length());
    if tenor.unit() == TimeUnit::Years {
        length * 12
    } else {
        length
    }
}

/// The contract's raw definition: B is the quarter after A by a day walk when
/// A < R and A is fewer than thirty days after S; points B + k·m strictly before
/// E; then R. Regular means both endpoints on the B lattice one stride apart.
fn expected_old_cds(start: Date, end: Date, stride: i64) -> Result<Expected, Box<dyn Error>> {
    let initial = quarter_on_or_after(start)?;
    let terminal = quarter_on_or_after(end)?;
    let anchor = if initial < terminal && initial.days_since(start) < 30 {
        quarter_on_or_after(initial.checked_add_days(1)?)?
    } else {
        initial
    };
    let mut raw = vec![start];
    let mut number = month_number(anchor);
    while number <= month_number(end) {
        let point = twentieth(number)?;
        if point >= end {
            break;
        }
        raw.push(point);
        number += stride;
    }
    raw.push(terminal);
    let on_lattice = |date: Date| {
        date.day() == 20 && (month_number(date) - month_number(anchor)).rem_euclid(stride) == 0
    };
    let flags = raw
        .windows(2)
        .map(|pair| {
            if on_lattice(pair[0])
                && on_lattice(pair[1])
                && month_number(pair[1]) - month_number(pair[0]) == stride
            {
                Regularity::Regular
            } else {
                Regularity::Stub
            }
        })
        .collect();
    let omitted = (anchor != initial && initial != start).then_some(initial);
    Ok(Expected {
        raw,
        flags,
        initial,
        anchor,
        terminal,
        omitted,
    })
}

/// FR-SCHED-33: `OldCDS` is the only spelling, `DateGenerationRule::ALL` has the
/// ten canonical labels in contract order, and OldCDS is not a civil roll.
#[test]
fn old_cds_label_is_canonical_exact_and_distinct_from_civil_rolls() -> Result<(), Box<dyn Error>> {
    assert_eq!(Generation::OldCds.canonical_identifier(), "OldCDS");
    assert_eq!(Generation::OldCds.to_string(), "OldCDS");
    assert_eq!("OldCDS".parse::<Generation>()?, Generation::OldCds);
    for text in [
        " OldCDS",
        "OldCDS ",
        "oldcds",
        "OLDCDS",
        "Old CDS",
        "OldCds",
        "Old-CDS",
        "OldCDS2015",
        "",
    ] {
        assert!(
            matches!(text.parse::<Generation>(),
                Err(IdentifierError::Unknown { text: rejected_text, .. }) if rejected_text == text),
            "{text:?}"
        );
        assert!(text.parse::<Roll>().is_err());
    }
    let names = [
        "Forward",
        "Backward",
        "Zero",
        "ThirdWednesday",
        "ThirdWednesdayInclusive",
        "Twentieth",
        "TwentiethIMM",
        "OldCDS",
        "CDS",
        "CDS2015",
    ];
    assert_eq!(Generation::ALL.len(), names.len());
    for (rule, name) in Generation::ALL.into_iter().zip(names) {
        assert_eq!(rule.canonical_identifier(), name);
        assert_eq!(rule.to_string(), name);
        assert_eq!(name.parse::<Generation>()?, rule);
    }
    assert_eq!(Generation::ALL[7], Generation::OldCds);
    assert_eq!(
        Generation::ALL.into_iter().collect::<HashSet<_>>().len(),
        10
    );
    assert_eq!(Roll::ALL.len(), 4);
    assert!("OldCDS".parse::<Roll>().is_err());
    assert!(Roll::ALL.into_iter().all(|rule| {
        Generation::Roll(rule) != Generation::OldCds && rule.canonical_identifier() != "OldCDS"
    }));
    Ok(())
}

/// FR-SCHED-33/37: an unsupported cap is reported first, before an invalid
/// tenor, an uncovered TARGET span or an unrepresentable terminal.
#[test]
fn boundary_cap_domain_precedes_tenor_coverage_and_terminal_checks() -> Result<(), Box<dyn Error>> {
    assert_eq!(ScheduleBuilder::MIN_BOUNDARIES, 2);
    assert_eq!(ScheduleBuilder::MAX_BOUNDARIES, MAXIMUM_CAP);
    let builders = [
        guarded(d("2025-12-30"), d("9999-12-31"), TargetCalendar::new())?,
        plain(d("2026-01-05"), d("9999-12-31"))?,
        plain(d("2026-02-19"), d("2026-06-01"))?,
    ];
    for builder in &builders {
        for cap in [0, 1, MAXIMUM_CAP + 1, u32::MAX] {
            for tenor in [
                months(0),
                Period::new(1, TimeUnit::Days),
                Period::new(1, TimeUnit::Weeks),
                months(i32::MIN),
                months(3),
                years(i32::MAX),
            ] {
                let error = rejected(builder.build_old_cds(tenor, cap));
                assert_eq!(error.kind(), "schedule.invalid_boundary_limit");
                assert!(
                    matches!(error, ScheduleError::InvalidBoundaryLimit {
                        requested, minimum: 2, maximum: MAXIMUM_CAP, .. } if requested == cap),
                    "{error:?}"
                );
            }
        }
    }
    // The exact minimum and maximum caps are valid budgets.
    let builder = plain(d("2026-02-19"), d("2026-06-01"))?;
    assert_eq!(
        builder.build_old_cds(months(3), 2)?,
        builder.build_old_cds(months(3), MAXIMUM_CAP)?
    );
    Ok(())
}

/// FR-SCHED-33/37: zero, negative (including i32::MIN), Days and Weeks tenors fail
/// with InvalidOldCdsTenor carrying start, requested end and the stored tenor,
/// before original or expanded TARGET coverage and before terminal resolution.
#[test]
fn invalid_stored_tenors_fail_with_unnormalized_context_before_guards_and_terminal()
-> Result<(), Box<dyn Error>> {
    let tenors = [
        months(0),
        years(0),
        months(-1),
        years(-1),
        months(i32::MIN),
        years(i32::MIN),
        Period::new(0, TimeUnit::Days),
        Period::new(1, TimeUnit::Days),
        Period::new(7, TimeUnit::Days),
        Period::new(-7, TimeUnit::Days),
        Period::new(365, TimeUnit::Days),
        Period::new(i32::MAX, TimeUnit::Days),
        Period::new(0, TimeUnit::Weeks),
        Period::new(1, TimeUnit::Weeks),
        Period::new(-1, TimeUnit::Weeks),
        Period::new(52, TimeUnit::Weeks),
        Period::new(i32::MAX, TimeUnit::Weeks),
        Period::new(i32::MIN, TimeUnit::Weeks),
    ];
    let mut cases = 0;
    for (start, end, calendar) in [
        (
            d("2026-02-19"),
            d("2026-06-01"),
            ScheduleCalendar::Plain(Calendar::null()),
        ),
        // The strict original span starts in the unverified 2003-2025 gap.
        (
            d("2025-12-30"),
            d("2026-03-01"),
            ScheduleCalendar::Target(TargetCalendar::new()),
        ),
        // Both endpoints precede TARGET inception.
        (
            d("1998-01-05"),
            d("1998-12-31"),
            ScheduleCalendar::Target(TargetCalendar::with_policy(TargetPolicy::AllowProjection)),
        ),
        // The original span is verified, but R = 2029-03-20 is not.
        (
            d("2028-01-05"),
            d("2028-12-31"),
            ScheduleCalendar::Target(TargetCalendar::new()),
        ),
        // The terminal quarter after 9999-12-20 is unrepresentable.
        (
            d("2026-01-05"),
            d("9999-12-31"),
            ScheduleCalendar::Plain(Calendar::custom(Weekday::ALL, [])),
        ),
    ] {
        let builder = ScheduleBuilder::new(start, end, calendar, Bdc::Following, Bdc::Following)?;
        for tenor in tenors {
            for cap in [2, MAXIMUM_CAP] {
                let error = rejected(builder.build_old_cds(tenor, cap));
                assert_eq!(error.kind(), "schedule.invalid_old_cds_tenor");
                assert!(
                    matches!(error, ScheduleError::InvalidOldCdsTenor {
                        generation_start, requested_end, tenor: rejected_tenor, .. }
                    if generation_start == start && requested_end == end
                        && stored(rejected_tenor) == stored(tenor)),
                    "{error:?}"
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 5 * 18 * 2);
    Ok(())
}

/// FR-SCHED-33/34: every positive stored Months/Years length is accepted without
/// a divisibility rule, Years stride as twelve months, and huge lengths simply
/// generate no further point.
#[test]
fn positive_month_and_year_lengths_need_no_divisibility_or_size_bound() -> Result<(), Box<dyn Error>>
{
    let (start, end) = (d("2026-02-19"), d("2031-07-01"));
    for (tenor, interiors, expected_flags) in [
        (
            months(8),
            "2026-06-20 2027-02-20 2027-10-20 2028-06-20 2029-02-20 2029-10-20 2030-06-20 2031-02-20",
            "SRRRRRRRS",
        ),
        (
            months(13),
            "2026-06-20 2027-07-20 2028-08-20 2029-09-20 2030-10-20",
            "SRRRRS",
        ),
        (months(60), "2026-06-20 2031-06-20", "SRS"),
        (years(5), "2026-06-20 2031-06-20", "SRS"),
        (months(i32::MAX), "2026-06-20", "SS"),
        (years(i32::MAX), "2026-06-20", "SS"),
    ] {
        verify(&Case {
            start,
            end,
            tenor,
            raw: joined("2026-02-19", interiors, "2031-09-20"),
            flags: flags(expected_flags),
            initial: d("2026-03-20"),
            anchor: d("2026-06-20"),
            terminal: d("2031-09-20"),
            omitted: Some(d("2026-03-20")),
        })?;
    }
    for length in [
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8,
        10,
        11,
        13,
        17,
        23,
        25,
        59,
        60,
        61,
        100,
        1_000,
        119_987,
        i32::MAX,
    ] {
        for unit in [TimeUnit::Months, TimeUnit::Years] {
            let tenor = Period::new(length, unit);
            let expected = expected_old_cds(start, end, stride_months(tenor))?;
            verify(&Case {
                start,
                end,
                tenor,
                raw: expected.raw,
                flags: expected.flags,
                initial: expected.initial,
                anchor: expected.anchor,
                terminal: expected.terminal,
                omitted: expected.omitted,
            })?;
        }
    }
    Ok(())
}

/// FR-SCHED-33: the recurrent and explicit builders reject the OldCDS label with
/// InvalidRule, but only after their own cap check.
#[test]
fn ordinary_builders_reject_the_old_cds_label_after_their_cap_check() -> Result<(), Box<dyn Error>>
{
    let builder = plain(d("2026-01-05"), d("2026-12-31"))?;
    let roles = ExplicitStubDates::new(Some(d("2026-03-20")), None)?;
    for tenor in [months(3), months(0), Period::new(1, TimeUnit::Days)] {
        for eom in EndOfMonth::ALL {
            for stub in StubPolicy::ALL {
                let error =
                    rejected(builder.build_recurrent(tenor, Generation::OldCds, stub, eom, 10));
                assert_eq!(error.kind(), "schedule.invalid_rule");
                assert!(matches!(
                    error,
                    ScheduleError::InvalidRule {
                        rule: Generation::OldCds,
                        ..
                    }
                ));
            }
            let error = rejected(builder.build_explicit(tenor, Generation::OldCds, roles, eom, 10));
            assert!(matches!(
                error,
                ScheduleError::InvalidRule {
                    rule: Generation::OldCds,
                    ..
                }
            ));
            for cap in [0, 1, MAXIMUM_CAP + 1, u32::MAX] {
                assert!(
                    matches!(builder.build_recurrent(tenor, Generation::OldCds, StubPolicy::None, eom, cap),
                    Err(ScheduleError::InvalidBoundaryLimit { requested, .. }) if requested == cap)
                );
                assert!(
                    matches!(builder.build_explicit(tenor, Generation::OldCds, roles, eom, cap),
                    Err(ScheduleError::InvalidBoundaryLimit { requested, .. }) if requested == cap)
                );
            }
        }
    }
    Ok(())
}

/// FR-SCHED-34/35/36: distances 0, 1, 29, 30 and 31 days before each 2026
/// quarter date, requested end 2026-12-31 (R = 2027-03-20), strides 3M and 1M.
/// Zero retains S once and starts at the next quarter, 1-29 omit A, 30-31 keep A.
#[test]
fn distance_classes_before_every_2026_quarter_have_exact_hand_vectors() -> Result<(), Box<dyn Error>>
{
    const MAR_DEC: &str = "2026-03-20 2026-04-20 2026-05-20 2026-06-20 2026-07-20 \
                           2026-08-20 2026-09-20 2026-10-20 2026-11-20 2026-12-20";
    const JUN_DEC: &str = "2026-06-20 2026-07-20 2026-08-20 2026-09-20 2026-10-20 \
                           2026-11-20 2026-12-20";
    const SEP_DEC: &str = "2026-09-20 2026-10-20 2026-11-20 2026-12-20";
    // (start, A - S, A, B, omitted A, 3M interiors, 3M flags, 1M interiors, 1M flags)
    let rows = [
        (
            "2026-03-20",
            0,
            "2026-03-20",
            "2026-06-20",
            None,
            "2026-06-20 2026-09-20 2026-12-20",
            "RRRR",
            JUN_DEC,
            "SRRRRRRS",
        ),
        (
            "2026-03-19",
            1,
            "2026-03-20",
            "2026-06-20",
            Some("2026-03-20"),
            "2026-06-20 2026-09-20 2026-12-20",
            "SRRR",
            JUN_DEC,
            "SRRRRRRS",
        ),
        (
            "2026-02-19",
            29,
            "2026-03-20",
            "2026-06-20",
            Some("2026-03-20"),
            "2026-06-20 2026-09-20 2026-12-20",
            "SRRR",
            JUN_DEC,
            "SRRRRRRS",
        ),
        (
            "2026-02-18",
            30,
            "2026-03-20",
            "2026-03-20",
            None,
            "2026-03-20 2026-06-20 2026-09-20 2026-12-20",
            "SRRRR",
            MAR_DEC,
            "SRRRRRRRRRS",
        ),
        (
            "2026-02-17",
            31,
            "2026-03-20",
            "2026-03-20",
            None,
            "2026-03-20 2026-06-20 2026-09-20 2026-12-20",
            "SRRRR",
            MAR_DEC,
            "SRRRRRRRRRS",
        ),
        (
            "2026-06-20",
            0,
            "2026-06-20",
            "2026-09-20",
            None,
            "2026-09-20 2026-12-20",
            "RRR",
            SEP_DEC,
            "SRRRS",
        ),
        (
            "2026-06-19",
            1,
            "2026-06-20",
            "2026-09-20",
            Some("2026-06-20"),
            "2026-09-20 2026-12-20",
            "SRR",
            SEP_DEC,
            "SRRRS",
        ),
        (
            "2026-05-22",
            29,
            "2026-06-20",
            "2026-09-20",
            Some("2026-06-20"),
            "2026-09-20 2026-12-20",
            "SRR",
            SEP_DEC,
            "SRRRS",
        ),
        (
            "2026-05-21",
            30,
            "2026-06-20",
            "2026-06-20",
            None,
            "2026-06-20 2026-09-20 2026-12-20",
            "SRRR",
            JUN_DEC,
            "SRRRRRRS",
        ),
        // A day-20 start one month before B lies on the 1M lattice.
        (
            "2026-05-20",
            31,
            "2026-06-20",
            "2026-06-20",
            None,
            "2026-06-20 2026-09-20 2026-12-20",
            "SRRR",
            JUN_DEC,
            "RRRRRRRS",
        ),
        (
            "2026-09-20",
            0,
            "2026-09-20",
            "2026-12-20",
            None,
            "2026-12-20",
            "RR",
            "2026-12-20",
            "SS",
        ),
        (
            "2026-09-19",
            1,
            "2026-09-20",
            "2026-12-20",
            Some("2026-09-20"),
            "2026-12-20",
            "SR",
            "2026-12-20",
            "SS",
        ),
        (
            "2026-08-22",
            29,
            "2026-09-20",
            "2026-12-20",
            Some("2026-09-20"),
            "2026-12-20",
            "SR",
            "2026-12-20",
            "SS",
        ),
        (
            "2026-08-21",
            30,
            "2026-09-20",
            "2026-09-20",
            None,
            "2026-09-20 2026-12-20",
            "SRR",
            SEP_DEC,
            "SRRRS",
        ),
        (
            "2026-08-20",
            31,
            "2026-09-20",
            "2026-09-20",
            None,
            "2026-09-20 2026-12-20",
            "SRR",
            SEP_DEC,
            "RRRRS",
        ),
        // In December, B = A + 3M is already the terminal 2027-03-20.
        (
            "2026-12-20",
            0,
            "2026-12-20",
            "2027-03-20",
            None,
            "",
            "R",
            "",
            "S",
        ),
        (
            "2026-12-19",
            1,
            "2026-12-20",
            "2027-03-20",
            Some("2026-12-20"),
            "",
            "S",
            "",
            "S",
        ),
        (
            "2026-11-21",
            29,
            "2026-12-20",
            "2027-03-20",
            Some("2026-12-20"),
            "",
            "S",
            "",
            "S",
        ),
        (
            "2026-11-20",
            30,
            "2026-12-20",
            "2026-12-20",
            None,
            "2026-12-20",
            "SR",
            "2026-12-20",
            "RS",
        ),
        (
            "2026-11-19",
            31,
            "2026-12-20",
            "2026-12-20",
            None,
            "2026-12-20",
            "SR",
            "2026-12-20",
            "SS",
        ),
    ];
    for (start, distance, initial, anchor, omitted, quarterly, q_flags, monthly, m_flags) in rows {
        assert_eq!(d(initial).days_since(d(start)), distance, "{start}");
        for (tenor, interiors, expected_flags) in [
            (months(3), quarterly, q_flags),
            (months(1), monthly, m_flags),
        ] {
            verify(&Case {
                start: d(start),
                end: d("2026-12-31"),
                tenor,
                raw: joined(start, interiors, "2027-03-20"),
                flags: flags(expected_flags),
                initial: d(initial),
                anchor: d(anchor),
                terminal: d("2027-03-20"),
                omitted: omitted.map(d),
            })?;
        }
    }
    Ok(())
}

/// FR-SCHED-34/35: when Q(start) = Q(end) the terminal is retained once with no
/// +3M shift and no extension, at any distance; A = S starts at the next quarter.
/// This also covers the first and last civil quarters without any shift overflow.
#[test]
fn same_quarter_and_minimal_spans_keep_one_terminal_without_extension() -> Result<(), Box<dyn Error>>
{
    // (start, end, A, B, R, 3M flag, 1M flag); every raw vector is [start, R].
    for (start, end, initial, anchor, terminal, quarterly, monthly) in [
        (
            "2026-03-17",
            "2026-03-19",
            "2026-03-20",
            "2026-03-20",
            "2026-03-20",
            "S",
            "S",
        ),
        (
            "2026-03-19",
            "2026-03-20",
            "2026-03-20",
            "2026-03-20",
            "2026-03-20",
            "S",
            "S",
        ),
        (
            "2026-03-01",
            "2026-03-20",
            "2026-03-20",
            "2026-03-20",
            "2026-03-20",
            "S",
            "S",
        ),
        (
            "2026-02-19",
            "2026-03-20",
            "2026-03-20",
            "2026-03-20",
            "2026-03-20",
            "S",
            "S",
        ),
        // Feb 20 is on the B-anchored 1M lattice one month before R.
        (
            "2026-02-20",
            "2026-03-20",
            "2026-03-20",
            "2026-03-20",
            "2026-03-20",
            "S",
            "R",
        ),
        (
            "2026-03-20",
            "2026-03-21",
            "2026-03-20",
            "2026-06-20",
            "2026-06-20",
            "R",
            "S",
        ),
        (
            "2026-12-30",
            "2026-12-31",
            "2027-03-20",
            "2027-03-20",
            "2027-03-20",
            "S",
            "S",
        ),
        (
            "2026-12-20",
            "2026-12-21",
            "2026-12-20",
            "2027-03-20",
            "2027-03-20",
            "R",
            "S",
        ),
        (
            "0001-01-01",
            "0001-01-02",
            "0001-03-20",
            "0001-03-20",
            "0001-03-20",
            "S",
            "S",
        ),
        (
            "9999-09-20",
            "9999-12-20",
            "9999-09-20",
            "9999-12-20",
            "9999-12-20",
            "R",
            "S",
        ),
        (
            "9999-11-25",
            "9999-12-01",
            "9999-12-20",
            "9999-12-20",
            "9999-12-20",
            "S",
            "S",
        ),
        // A = R = 9999-12-20 at distance 1: A + 3M is never requested.
        (
            "9999-12-19",
            "9999-12-20",
            "9999-12-20",
            "9999-12-20",
            "9999-12-20",
            "S",
            "S",
        ),
    ] {
        for (tenor, flag) in [
            (months(3), quarterly),
            (months(1), monthly),
            (years(i32::MAX), "S"),
        ] {
            verify(&Case {
                start: d(start),
                end: d(end),
                tenor,
                raw: joined(start, "", terminal),
                flags: flags(flag),
                initial: d(initial),
                anchor: d(anchor),
                terminal: d(terminal),
                omitted: None,
            })?;
        }
    }
    Ok(())
}

/// FR-SCHED-34: the thirty-calendar-day cut moves with February: in leap years
/// (2024, 2028, 2000) Feb 19 is exactly 30 days before Mar 20 and keeps A, in
/// common years (2026, 2027, 2100) Feb 18 does; one day later omits A.
#[test]
fn leap_february_moves_the_thirty_day_cut_by_exactly_one_start_day() -> Result<(), Box<dyn Error>> {
    for (year, days) in [
        (2024, [(18, 31), (19, 30), (20, 29), (21, 28)]),
        (2028, [(18, 31), (19, 30), (20, 29), (21, 28)]),
        (2000, [(18, 31), (19, 30), (20, 29), (21, 28)]),
        (2026, [(17, 31), (18, 30), (19, 29), (20, 28)]),
        (2027, [(17, 31), (18, 30), (19, 29), (20, 28)]),
        (2100, [(17, 31), (18, 30), (19, 29), (20, 28)]),
    ] {
        let march = Date::from_ymd(year, 3, 20)?;
        let june = Date::from_ymd(year, 6, 20)?;
        let september = Date::from_ymd(year, 9, 20)?;
        for (day, distance) in days {
            let start = Date::from_ymd(year, 2, day)?;
            assert_eq!(march.days_since(start), distance);
            let (raw, expected_flags, anchor, omitted) = if distance >= 30 {
                (vec![start, march, june, september], "SRR", march, None)
            } else {
                (vec![start, june, september], "SR", june, Some(march))
            };
            verify(&Case {
                start,
                end: Date::from_ymd(year, 7, 1)?,
                tenor: months(3),
                raw,
                flags: flags(expected_flags),
                initial: march,
                anchor,
                terminal: september,
                omitted,
            })?;
        }
    }
    Ok(())
}

/// FR-SCHED-34/35: generation stops at the requested end E (stride points in
/// [E, R) are never boundaries) and the terminal is R = Q(E), for every listed
/// stride with ends just before, on and after a stride point. S = 2026-01-05 is
/// 74 days before A = B = 2026-03-20.
#[test]
fn every_stride_stops_before_the_requested_end_and_projects_the_quarter_terminal()
-> Result<(), Box<dyn Error>> {
    const MAR_JUL: &str = "2026-03-20 2026-04-20 2026-05-20 2026-06-20 2026-07-20";
    for (tenor, end, interiors, terminal, expected_flags) in [
        (months(1), "2026-08-19", MAR_JUL, "2026-09-20", "SRRRRS"),
        (months(1), "2026-08-20", MAR_JUL, "2026-09-20", "SRRRRS"),
        (
            months(1),
            "2026-08-21",
            "2026-03-20 2026-04-20 2026-05-20 2026-06-20 2026-07-20 2026-08-20",
            "2026-09-20",
            "SRRRRRR",
        ),
        // Oct 20 and Nov 20 lie in [E, R): the final interval skips them.
        (
            months(1),
            "2026-09-21",
            "2026-03-20 2026-04-20 2026-05-20 2026-06-20 2026-07-20 2026-08-20 2026-09-20",
            "2026-12-20",
            "SRRRRRRS",
        ),
        // Nov 20 lies in [E, R): Oct 20 -> Dec 20 is a two-month Stub.
        (
            months(1),
            "2026-10-21",
            "2026-03-20 2026-04-20 2026-05-20 2026-06-20 2026-07-20 2026-08-20 \
             2026-09-20 2026-10-20",
            "2026-12-20",
            "SRRRRRRRS",
        ),
        (months(2), "2026-05-19", "2026-03-20", "2026-06-20", "SS"),
        (months(2), "2026-05-20", "2026-03-20", "2026-06-20", "SS"),
        (
            months(2),
            "2026-05-21",
            "2026-03-20 2026-05-20",
            "2026-06-20",
            "SRS",
        ),
        (months(4), "2026-07-19", "2026-03-20", "2026-09-20", "SS"),
        (months(4), "2026-07-20", "2026-03-20", "2026-09-20", "SS"),
        (
            months(4),
            "2026-07-21",
            "2026-03-20 2026-07-20",
            "2026-09-20",
            "SRS",
        ),
        (months(5), "2026-08-19", "2026-03-20", "2026-09-20", "SS"),
        (months(5), "2026-08-20", "2026-03-20", "2026-09-20", "SS"),
        (
            months(5),
            "2026-08-21",
            "2026-03-20 2026-08-20",
            "2026-09-20",
            "SRS",
        ),
        (months(6), "2026-09-19", "2026-03-20", "2026-09-20", "SR"),
        (months(6), "2026-09-20", "2026-03-20", "2026-09-20", "SR"),
        (
            months(6),
            "2026-09-21",
            "2026-03-20 2026-09-20",
            "2026-12-20",
            "SRS",
        ),
        (months(7), "2026-10-19", "2026-03-20", "2026-12-20", "SS"),
        (months(7), "2026-10-20", "2026-03-20", "2026-12-20", "SS"),
        (
            months(7),
            "2026-10-21",
            "2026-03-20 2026-10-20",
            "2026-12-20",
            "SRS",
        ),
        (months(9), "2026-12-19", "2026-03-20", "2026-12-20", "SR"),
        (months(9), "2026-12-20", "2026-03-20", "2026-12-20", "SR"),
        (
            months(9),
            "2026-12-21",
            "2026-03-20 2026-12-20",
            "2027-03-20",
            "SRS",
        ),
        (months(12), "2027-03-19", "2026-03-20", "2027-03-20", "SR"),
        (months(12), "2027-03-20", "2026-03-20", "2027-03-20", "SR"),
        (
            months(12),
            "2027-03-21",
            "2026-03-20 2027-03-20",
            "2027-06-20",
            "SRS",
        ),
        (years(1), "2027-03-19", "2026-03-20", "2027-03-20", "SR"),
        (years(1), "2027-03-20", "2026-03-20", "2027-03-20", "SR"),
        (
            years(1),
            "2027-03-21",
            "2026-03-20 2027-03-20",
            "2027-06-20",
            "SRS",
        ),
        (months(18), "2027-09-19", "2026-03-20", "2027-09-20", "SR"),
        (months(18), "2027-09-20", "2026-03-20", "2027-09-20", "SR"),
        (
            months(18),
            "2027-09-21",
            "2026-03-20 2027-09-20",
            "2027-12-20",
            "SRS",
        ),
        (years(2), "2028-03-19", "2026-03-20", "2028-03-20", "SR"),
        (years(2), "2028-03-20", "2026-03-20", "2028-03-20", "SR"),
        (
            years(2),
            "2028-03-21",
            "2026-03-20 2028-03-20",
            "2028-06-20",
            "SRS",
        ),
        (
            months(24),
            "2028-03-21",
            "2026-03-20 2028-03-20",
            "2028-06-20",
            "SRS",
        ),
    ] {
        verify(&Case {
            start: d("2026-01-05"),
            end: d(end),
            tenor,
            raw: joined("2026-01-05", interiors, terminal),
            flags: flags(expected_flags),
            initial: d("2026-03-20"),
            anchor: d("2026-03-20"),
            terminal: d(terminal),
            omitted: None,
        })?;
    }
    Ok(())
}

/// FR-SCHED-35: Regular exactly when both raw endpoints lie on the B-anchored
/// stride lattice one stride apart: A = S is Regular for 3M but not for 1M, a
/// day-20 start on the lattice is Regular, off-lattice starts and off-stride
/// terminals are Stub; never NonPeriodic. An interval over an omitted quarter
/// is Stub unless its endpoints are lattice points one stride apart: 4M from
/// 20 February is Regular to 20 June in both common and leap years.
#[test]
fn regular_flags_mean_exact_b_anchored_lattice_steps_and_all_else_is_stub()
-> Result<(), Box<dyn Error>> {
    // (start, end, tenor, interiors, R, flags, A, B, omitted A)
    for (start, end, tenor, interiors, terminal, expected_flags, initial, anchor, omitted) in [
        (
            "2026-03-20",
            "2026-07-01",
            months(3),
            "2026-06-20",
            "2026-09-20",
            "RR",
            "2026-03-20",
            "2026-06-20",
            None,
        ),
        (
            "2026-03-20",
            "2026-07-01",
            months(1),
            "2026-06-20",
            "2026-09-20",
            "SS",
            "2026-03-20",
            "2026-06-20",
            None,
        ),
        // Jan 20 is two months before B on the 2M lattice.
        (
            "2026-01-20",
            "2026-06-01",
            months(2),
            "2026-03-20 2026-05-20",
            "2026-06-20",
            "RRS",
            "2026-03-20",
            "2026-03-20",
            None,
        ),
        // Feb 20 is on the 1M lattice, but its interval spans the omitted Mar 20.
        (
            "2026-02-20",
            "2026-09-01",
            months(1),
            "2026-06-20 2026-07-20 2026-08-20",
            "2026-09-20",
            "SRRR",
            "2026-03-20",
            "2026-06-20",
            Some("2026-03-20"),
        ),
        (
            "2026-03-01",
            "2026-08-21",
            months(2),
            "2026-06-20 2026-08-20",
            "2026-09-20",
            "SRS",
            "2026-03-20",
            "2026-06-20",
            Some("2026-03-20"),
        ),
        // A day-20 start off the quarterly lattice.
        (
            "2026-05-20",
            "2026-12-31",
            months(3),
            "2026-06-20 2026-09-20 2026-12-20",
            "2027-03-20",
            "SRRR",
            "2026-06-20",
            "2026-06-20",
            None,
        ),
        (
            "2026-08-20",
            "2026-12-31",
            months(1),
            "2026-09-20 2026-10-20 2026-11-20 2026-12-20",
            "2027-03-20",
            "RRRRS",
            "2026-09-20",
            "2026-09-20",
            None,
        ),
        // Feb 20 is B - 4M: Regular to Jun 20 although Mar 20 is omitted.
        (
            "2026-02-20",
            "2026-12-01",
            months(4),
            "2026-06-20 2026-10-20",
            "2026-12-20",
            "RRS",
            "2026-03-20",
            "2026-06-20",
            Some("2026-03-20"),
        ),
        // Leap year: Feb 20 is 29 days before Mar 20, so A is still omitted.
        (
            "2024-02-20",
            "2024-07-01",
            months(4),
            "2024-06-20",
            "2024-09-20",
            "RS",
            "2024-03-20",
            "2024-06-20",
            Some("2024-03-20"),
        ),
        // Sep 20 -> Dec 20 is off the 6M stride.
        (
            "2026-01-05",
            "2026-09-21",
            months(6),
            "2026-03-20 2026-09-20",
            "2026-12-20",
            "SRS",
            "2026-03-20",
            "2026-03-20",
            None,
        ),
        (
            "2026-01-05",
            "2028-03-21",
            months(12),
            "2026-03-20 2027-03-20 2028-03-20",
            "2028-06-20",
            "SRRS",
            "2026-03-20",
            "2026-03-20",
            None,
        ),
    ] {
        verify(&Case {
            start: d(start),
            end: d(end),
            tenor,
            raw: joined(start, interiors, terminal),
            flags: flags(expected_flags),
            initial: d(initial),
            anchor: d(anchor),
            terminal: d(terminal),
            omitted: omitted.map(d),
        })?;
    }
    Ok(())
}

/// FR-SCHED-33/34: i32::MAX Months and Years are valid; only S, B (when B < E)
/// and R are constructed, so no unused stride candidate can fail.
#[test]
fn huge_months_and_years_construct_only_start_anchor_and_terminal() -> Result<(), Box<dyn Error>> {
    for tenor in [months(i32::MAX), years(i32::MAX)] {
        // (start, end, interiors, R, A, B, omitted A)
        for (start, end, interiors, terminal, initial, anchor, omitted) in [
            (
                "2026-01-05",
                "2030-01-01",
                "2026-03-20",
                "2030-03-20",
                "2026-03-20",
                "2026-03-20",
                None,
            ),
            (
                "0001-01-01",
                "9999-12-20",
                "0001-03-20",
                "9999-12-20",
                "0001-03-20",
                "0001-03-20",
                None,
            ),
            (
                "9999-01-05",
                "9999-12-20",
                "9999-03-20",
                "9999-12-20",
                "9999-03-20",
                "9999-03-20",
                None,
            ),
            (
                "2026-02-19",
                "2026-06-01",
                "",
                "2026-06-20",
                "2026-03-20",
                "2026-06-20",
                Some("2026-03-20"),
            ),
            (
                "2026-03-20",
                "2026-03-21",
                "",
                "2026-06-20",
                "2026-03-20",
                "2026-06-20",
                None,
            ),
        ] {
            let raw = joined(start, interiors, terminal);
            let stubs = vec![Regularity::Stub; raw.len() - 1];
            verify(&Case {
                start: d(start),
                end: d(end),
                tenor,
                raw,
                flags: stubs,
                initial: d(initial),
                anchor: d(anchor),
                terminal: d(terminal),
                omitted: omitted.map(d),
            })?;
        }
    }
    Ok(())
}

/// FR-SCHED-34/37: near the civil maximum every stride stops before E without
/// constructing a post-end candidate such as 10000-03-20.
#[test]
fn strides_near_the_civil_maximum_never_construct_post_end_candidates() -> Result<(), Box<dyn Error>>
{
    for (tenor, interiors, expected_flags) in [
        (
            months(1),
            "9999-03-20 9999-04-20 9999-05-20 9999-06-20 9999-07-20 9999-08-20 \
             9999-09-20 9999-10-20 9999-11-20",
            "SRRRRRRRRR",
        ),
        (months(3), "9999-03-20 9999-06-20 9999-09-20", "SRRR"),
        (months(4), "9999-03-20 9999-07-20 9999-11-20", "SRRS"),
        (months(5), "9999-03-20 9999-08-20", "SRS"),
        (months(6), "9999-03-20 9999-09-20", "SRS"),
        (months(9), "9999-03-20", "SR"),
        (months(12), "9999-03-20", "SS"),
        (years(1), "9999-03-20", "SS"),
        (months(13), "9999-03-20", "SS"),
    ] {
        verify(&Case {
            start: d("9999-01-05"),
            end: d("9999-12-20"),
            tenor,
            raw: joined("9999-01-05", interiors, "9999-12-20"),
            flags: flags(expected_flags),
            initial: d("9999-03-20"),
            anchor: d("9999-03-20"),
            terminal: d("9999-12-20"),
            omitted: None,
        })?;
    }
    Ok(())
}

/// FR-SCHED-34/36: when the +3M anchor already equals the terminal, every stride
/// returns [S, R] once and records the omitted A.
#[test]
fn shifted_anchor_equal_to_the_terminal_is_retained_once_for_every_stride()
-> Result<(), Box<dyn Error>> {
    let tenors = [
        months(1),
        months(2),
        months(3),
        months(4),
        months(5),
        months(6),
        months(7),
        months(9),
        months(12),
        years(1),
        months(18),
        years(2),
        months(i32::MAX),
        years(i32::MAX),
    ];
    for end in ["2026-03-21", "2026-06-01", "2026-06-20"] {
        for tenor in tenors {
            verify(&Case {
                start: d("2026-02-19"),
                end: d(end),
                tenor,
                raw: dates("2026-02-19 2026-06-20"),
                flags: flags("S"),
                initial: d("2026-03-20"),
                anchor: d("2026-06-20"),
                terminal: d("2026-06-20"),
                omitted: Some(d("2026-03-20")),
            })?;
        }
    }
    Ok(())
}

/// FR-SCHED-36: the immutable provenance records an omission only for an actual
/// interior cutoff, is Copy/Clone/Debug/Eq/Hash/Send/Sync/'static, and is absent
/// from every other Schedule constructor.
#[test]
fn provenance_omission_traits_and_absence_from_other_constructors() -> Result<(), Box<dyn Error>> {
    fn immutable<T: Copy + Clone + Debug + Eq + Hash + Send + Sync + 'static>() {}
    fn shared<T: Clone + Debug + Eq + Hash + Send + Sync + 'static>() {}
    immutable::<OldCdsProvenance>();
    shared::<Schedule>();
    for (start, end, omitted) in [
        ("2026-03-20", "2026-12-31", None),               // A = S
        ("2026-03-19", "2026-03-20", None),               // A = R at distance 1
        ("2026-03-01", "2026-03-20", None),               // A = R at distance 19
        ("2026-02-18", "2026-12-31", None),               // distance 30
        ("2026-02-17", "2026-12-31", None),               // distance 31
        ("2026-03-19", "2026-12-31", Some("2026-03-20")), // distance 1
        ("2026-02-19", "2026-12-31", Some("2026-03-20")), // distance 29
        ("2026-02-19", "2026-06-01", Some("2026-03-20")), // distance 29 and B = R
    ] {
        for tenor in [months(3), months(1), years(1)] {
            let schedule = plain(d(start), d(end))?.build_old_cds(tenor, 20)?;
            let provenance = schedule.old_cds_provenance().ok_or("missing provenance")?;
            assert_eq!(
                provenance.omitted_quarter(),
                omitted.map(d),
                "{start} {end}"
            );
        }
    }
    let schedule = plain(d("2026-02-19"), d("2026-12-31"))?.build_old_cds(months(4), 10)?;
    let provenance = *schedule.old_cds_provenance().ok_or("missing provenance")?;
    let copied = provenance;
    assert_eq!(copied, provenance);
    assert_eq!(cloned(&provenance), provenance);
    assert_eq!(hash(&copied), hash(&provenance));
    assert!(!format!("{provenance:?}").is_empty());
    assert_eq!(cloned(&schedule), schedule);
    assert_eq!(hash(&cloned(&schedule)), hash(&schedule));
    let builder = plain(d("2026-01-01"), d("2027-01-01"))?;
    assert!(builder.build_zero(2)?.old_cds_provenance().is_none());
    for rule in [Generation::Forward, Generation::Backward] {
        let recurrent =
            builder.build_recurrent(months(3), rule, StubPolicy::Short, EndOfMonth::Ignore, 10)?;
        assert!(recurrent.old_cds_provenance().is_none());
    }
    let explicit = builder.build_explicit(
        months(6),
        Generation::Forward,
        ExplicitStubDates::new(Some(d("2026-07-01")), None)?,
        EndOfMonth::Ignore,
        3,
    )?;
    assert!(explicit.old_cds_provenance().is_none());
    for rule in Roll::ALL {
        let rolled = builder.build_roll(months(3), rule, 10)?;
        assert!(rolled.old_cds_provenance().is_none());
    }
    Ok(())
}

/// FR-SCHED-36: stored 1Y and 12M aliases and different requested ends with
/// identical vectors are distinct definitions (Eq and Hash include provenance),
/// while caps are budgets that never affect identity.
#[test]
fn stored_aliases_and_requested_ends_are_identity_while_caps_are_not() -> Result<(), Box<dyn Error>>
{
    let builder = plain(d("2026-01-05"), d("2028-03-21"))?;
    let monthly = builder.build_old_cds(months(12), 5)?;
    let yearly = builder.build_old_cds(years(1), 5)?;
    assert_eq!(months(12), years(1));
    assert_eq!(
        monthly.contractual_dates(),
        dates("2026-01-05 2026-03-20 2027-03-20 2028-03-20 2028-06-20")
    );
    assert_eq!(monthly.contractual_dates(), yearly.contractual_dates());
    assert_eq!(monthly.adjusted_dates(), yearly.adjusted_dates());
    assert_eq!(monthly.regularity(), flags("SRRS"));
    assert_eq!(monthly.regularity(), yearly.regularity());
    assert_ne!(monthly, yearly);
    assert_ne!(hash(&monthly), hash(&yearly));
    let by_month = monthly.old_cds_provenance().ok_or("missing provenance")?;
    let by_year = yearly.old_cds_provenance().ok_or("missing provenance")?;
    assert_ne!(by_month, by_year);
    assert_ne!(hash(by_month), hash(by_year));
    assert_eq!(by_month.tenor(), by_year.tenor());
    assert_eq!(stored(by_month.tenor()), (12, TimeUnit::Months));
    assert_eq!(stored(by_year.tenor()), (1, TimeUnit::Years));
    assert_eq!(
        HashSet::from([monthly.clone(), yearly.clone(), monthly.clone()]).len(),
        2
    );
    assert_eq!(HashSet::from([*by_month, *by_year]).len(), 2);

    // Requested ends 2026-11-17 and 2026-11-18 both project to R = 2026-12-20.
    let early = plain(d("2026-01-05"), d("2026-11-17"))?.build_old_cds(months(3), 5)?;
    let late = plain(d("2026-01-05"), d("2026-11-18"))?.build_old_cds(months(3), 5)?;
    let raw = dates("2026-01-05 2026-03-20 2026-06-20 2026-09-20 2026-12-20");
    assert_eq!(early.contractual_dates(), raw);
    assert_eq!(late.contractual_dates(), raw);
    assert_eq!(early.adjusted_dates(), late.adjusted_dates());
    assert_eq!(early.regularity(), late.regularity());
    assert_ne!(early, late);
    assert_ne!(hash(&early), hash(&late));
    let first = early.old_cds_provenance().ok_or("missing provenance")?;
    let second = late.old_cds_provenance().ok_or("missing provenance")?;
    assert_ne!(first, second);
    assert_eq!(first.requested_end(), d("2026-11-17"));
    assert_eq!(second.requested_end(), d("2026-11-18"));
    assert_eq!(first.resolved_end(), second.resolved_end());
    assert_eq!(first.recurrence_anchor(), second.recurrence_anchor());

    // Caps are operation budgets only.
    let generous =
        plain(d("2026-01-05"), d("2026-11-17"))?.build_old_cds(months(3), MAXIMUM_CAP)?;
    assert_eq!(early, generous);
    assert_eq!(hash(&early), hash(&generous));
    assert_eq!(early.old_cds_provenance(), generous.old_cds_provenance());
    assert_eq!(
        HashSet::from([early.clone(), late.clone(), generous.clone()]).len(),
        2
    );

    // The calendar keeps its existing identity role; provenance holds no calendar.
    let weekends = on(
        d("2026-01-05"),
        d("2026-11-17"),
        Calendar::weekends_only(),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )?
    .build_old_cds(months(3), 5)?;
    assert_eq!(weekends.adjusted_dates(), early.adjusted_dates());
    assert_ne!(weekends, early);
    assert_eq!(weekends.old_cds_provenance(), early.old_cds_provenance());
    Ok(())
}

/// FR-SCHED-36: builders, schedules and Copy provenance are shared immutably
/// across threads, and concurrent rebuilds are equal with equal hashes.
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "integration tests exercise immutable sharing; the library starts no threads"
)]
fn concurrent_rebuilds_and_shared_provenance_are_identical() -> Result<(), Box<dyn Error>> {
    fn shared<T: Send + Sync + 'static>() {}
    shared::<ScheduleBuilder>();
    shared::<Schedule>();
    shared::<OldCdsProvenance>();
    let builder = Arc::new(on(
        d("2026-02-07"),
        d("2026-12-31"),
        Calendar::weekends_only(),
        Bdc::Following,
        Bdc::ModifiedFollowing,
    )?);
    let tenors = [
        months(1),
        months(3),
        months(12),
        years(1),
        months(5),
        months(i32::MAX),
    ];
    let baseline = tenors
        .iter()
        .map(|tenor| builder.build_old_cds(*tenor, 64))
        .collect::<Result<Vec<_>, _>>()?;
    let expected = Arc::new(baseline);
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let builder = Arc::clone(&builder);
            let expected = Arc::clone(&expected);
            std::thread::spawn(move || {
                let mut built = 0_usize;
                for _ in 0..8 {
                    for (tenor, reference) in tenors.into_iter().zip(expected.iter()) {
                        let schedule = builder
                            .build_old_cds(tenor, 64)
                            .expect("valid threaded OldCDS schedule");
                        assert_eq!(&schedule, reference);
                        assert_eq!(hash(&schedule), hash(reference));
                        assert_eq!(
                            schedule.old_cds_provenance(),
                            reference.old_cds_provenance()
                        );
                        built += 1;
                    }
                }
                built
            })
        })
        .collect();
    let mut total = 0;
    for worker in workers {
        total += worker.join().map_err(|_| "OldCDS worker thread failed")?;
    }
    assert_eq!(total, 8 * 8 * tenors.len());
    let provenance = *expected[1]
        .old_cds_provenance()
        .ok_or("missing provenance")?;
    let echoed = std::thread::spawn(move || provenance)
        .join()
        .map_err(|_| "provenance thread failed")?;
    assert_eq!(echoed, provenance);
    assert_eq!(hash(&echoed), hash(&provenance));
    assert_eq!(builder.start(), d("2026-02-07"));
    assert_eq!(builder.end(), d("2026-12-31"));
    Ok(())
}

/// FR-SCHED-34/37: for representative shapes the exact final count succeeds,
/// count - 1 and the minimum cap report `required_at_least` equal to that exact
/// count, and the cap check precedes business-day adjustment.
#[test]
fn exact_final_counts_are_reported_and_cap_checks_precede_adjustment() -> Result<(), Box<dyn Error>>
{
    let closed = Calendar::custom(Weekday::ALL, []);
    for (start, end, tenor, count) in [
        ("2026-03-20", "2026-12-31", months(3), 5_u32),
        ("2026-03-20", "2026-12-31", months(1), 9),
        ("2026-02-18", "2026-12-31", months(1), 12),
        ("2026-01-05", "2026-10-21", months(1), 10),
        ("2026-01-05", "2026-08-21", months(1), 8),
        ("2026-01-05", "2026-08-20", months(1), 7),
        ("2026-01-05", "2027-03-21", months(12), 4),
        ("2026-01-05", "2027-03-21", years(1), 4),
        ("2026-01-05", "2028-03-21", years(2), 4),
        ("2026-01-05", "2026-10-21", months(7), 4),
        ("2026-01-20", "2026-06-01", months(2), 4),
        ("2026-01-05", "2036-01-01", months(1), 120),
        ("9999-01-05", "9999-12-20", months(1), 11),
        ("9999-01-05", "9999-12-20", months(4), 5),
        ("2026-01-05", "2030-01-01", months(i32::MAX), 3),
        ("0001-01-01", "0001-01-02", months(1), 2),
        ("2026-02-19", "2026-06-01", months(5), 2),
        ("2026-12-30", "2026-12-31", months(3), 2),
    ] {
        let (start, end) = (d(start), d(end));
        let builder = plain(start, end)?;
        let schedule = builder.build_old_cds(tenor, count)?;
        assert_eq!(schedule.contractual_dates().len(), usize::try_from(count)?);
        assert_eq!(schedule.regularity().len(), usize::try_from(count - 1)?);
        assert_eq!(builder.build_old_cds(tenor, MAXIMUM_CAP)?, schedule);
        for cap in [ScheduleBuilder::MIN_BOUNDARIES, count - 1] {
            if (ScheduleBuilder::MIN_BOUNDARIES..count).contains(&cap) {
                let error = rejected(builder.build_old_cds(tenor, cap));
                assert_eq!(error.kind(), "schedule.boundary_limit_exceeded");
                assert!(
                    matches!(error, ScheduleError::BoundaryLimitExceeded {
                        maximum, required_at_least, .. }
                    if maximum == cap && required_at_least == count),
                    "{start} {end} {tenor}: {error:?}"
                );
                assert!(error.source().is_none());
            }
        }
        assert!(matches!(
            builder.build_old_cds(tenor, 1),
            Err(ScheduleError::InvalidBoundaryLimit { requested: 1, .. })
        ));
        // A closed calendar fails the first adjustment, but only after the cap.
        let closed_builder = on(start, end, closed.clone(), Bdc::Following, Bdc::Following)?;
        if count > ScheduleBuilder::MIN_BOUNDARIES {
            assert!(matches!(closed_builder.build_old_cds(tenor, count - 1),
                Err(ScheduleError::BoundaryLimitExceeded { required_at_least, .. })
                if required_at_least == count));
        }
        assert!(matches!(
            closed_builder.build_old_cds(tenor, count),
            Err(ScheduleError::Calendar {
                index: Some(0),
                phase: "adjustment",
                cause: CalendarError::NoBusinessWeekday,
                ..
            })
        ));
    }
    Ok(())
}

/// FR-SCHED-34/37: 1M from 0001-01-01 to 9999-12-20 needs exactly the civil
/// maximum of 119,987 boundaries; one fewer is rejected with that exact count.
/// The same span needs 39,997 boundaries for 3M and 10,001 for 12M/1Y.
#[test]
fn civil_maximum_needs_exactly_119_987_boundaries() -> Result<(), Box<dyn Error>> {
    let end = d("9999-12-20");
    let builder = plain(Date::MIN, end)?;
    let schedule = builder.build_old_cds(months(1), 119_987)?;
    let raw = schedule.contractual_dates();
    assert_eq!(raw.len(), 119_987);
    assert_eq!(
        &raw[..3],
        dates("0001-01-01 0001-03-20 0001-04-20").as_slice()
    );
    assert_eq!(
        &raw[raw.len() - 3..],
        dates("9999-10-20 9999-11-20 9999-12-20").as_slice()
    );
    let mut number = month_number(d("0001-03-20"));
    for date in &raw[1..] {
        assert_eq!(*date, twentieth(number)?);
        number += 1;
    }
    assert_eq!(number, month_number(end) + 1);
    assert_eq!(schedule.adjusted_dates(), raw);
    assert_eq!(schedule.regularity()[0], Regularity::Stub);
    assert!(
        schedule.regularity()[1..]
            .iter()
            .all(|flag| *flag == Regularity::Regular)
    );
    let provenance = schedule.old_cds_provenance().ok_or("missing provenance")?;
    assert_eq!(provenance.generation_start(), Date::MIN);
    assert_eq!(provenance.requested_end(), end);
    assert_eq!(provenance.initial_quarter(), d("0001-03-20"));
    assert_eq!(provenance.recurrence_anchor(), d("0001-03-20"));
    assert_eq!(provenance.resolved_end(), end);
    assert_eq!(provenance.omitted_quarter(), None);
    assert_eq!(stored(provenance.tenor()), (1, TimeUnit::Months));
    let error = rejected(builder.build_old_cds(months(1), 119_986));
    assert!(matches!(
        error,
        ScheduleError::BoundaryLimitExceeded {
            maximum: 119_986,
            required_at_least: 119_987,
            ..
        }
    ));
    for (tenor, count, penultimate, regular) in [
        (months(3), 39_997_u32, "9999-09-20", 39_995_usize),
        (months(12), 10_001, "9999-03-20", 9_998),
        (years(1), 10_001, "9999-03-20", 9_998),
    ] {
        let schedule = builder.build_old_cds(tenor, count)?;
        let raw = schedule.contractual_dates();
        assert_eq!(raw.len(), usize::try_from(count)?);
        assert_eq!(raw[1], d("0001-03-20"));
        assert_eq!(raw[raw.len() - 2], d(penultimate));
        assert_eq!(raw[raw.len() - 1], end);
        assert_eq!(
            schedule
                .regularity()
                .iter()
                .filter(|flag| **flag == Regularity::Regular)
                .count(),
            regular
        );
        assert!(matches!(builder.build_old_cds(tenor, count - 1),
            Err(ScheduleError::BoundaryLimitExceeded { maximum, required_at_least, .. })
            if maximum == count - 1 && required_at_least == count));
    }
    Ok(())
}

/// FR-SCHED-37: every requested end after 9999-12-20 needs the unrepresentable
/// 10000-03-20, so it fails first as `terminal quarter` with input E and the
/// checked +3 month cause from E, even for a start after 9999-12-20, before any
/// cap or adjustment failure; there is no truncation.
#[test]
fn ends_after_9999_12_20_fail_as_the_terminal_quarter_without_truncation()
-> Result<(), Box<dyn Error>> {
    let closed = Calendar::custom(Weekday::ALL, []);
    let mut cases = 0;
    for end in ["9999-12-21", "9999-12-25", "9999-12-31"] {
        let end = d(end);
        let Err(cause) = end.checked_add_months(3) else {
            return Err("the quarter after December 9999 must be unrepresentable".into());
        };
        assert!(
            matches!(&cause, DateError::OutOfRange { date, operation: "add months", count: 3, .. }
            if *date == end)
        );
        for start in [
            Date::MIN,
            d("2026-01-05"),
            d("9999-09-20"),
            d("9999-12-19"),
            d("9999-12-20"),
            d("9999-12-21"),
            d("9999-12-30"),
        ] {
            if start >= end {
                continue;
            }
            for tenor in [
                months(1),
                months(3),
                months(12),
                years(1),
                months(i32::MAX),
                years(i32::MAX),
            ] {
                for calendar in [Calendar::null(), closed.clone()] {
                    for cap in [2, MAXIMUM_CAP] {
                        let error = rejected(
                            on(start, end, calendar.clone(), Bdc::Following, Bdc::Following)?
                                .build_old_cds(tenor, cap),
                        );
                        assert_eq!(error.kind(), "schedule.old_cds_date");
                        assert!(
                            matches!(&error, ScheduleError::OldCdsDate {
                                generation_start, requested_end, tenor: rejected_tenor,
                                role: "terminal quarter", input,
                                cause: DateError::OutOfRange { date, operation: "add months", count: 3, .. },
                                .. }
                            if *generation_start == start && *requested_end == end
                                && stored(*rejected_tenor) == stored(tenor)
                                && *input == end && *date == end),
                            "{error:?}"
                        );
                        assert_eq!(
                            error
                                .source()
                                .and_then(|source| source.downcast_ref::<DateError>()),
                            Some(&cause)
                        );
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 18 * 6 * 2 * 2);
    // 9999-12-20 itself is the last representable terminal.
    verify(&Case {
        start: d("9999-12-01"),
        end: d("9999-12-20"),
        tenor: months(3),
        raw: dates("9999-12-01 9999-12-20"),
        flags: flags("S"),
        initial: d("9999-12-20"),
        anchor: d("9999-12-20"),
        terminal: d("9999-12-20"),
        omitted: None,
    })?;
    Ok(())
}

/// FR-SCHED-37: both new variants have stable kinds, Display naming their whole
/// context (start, requested end, stored tenor, role and input), the documented
/// source chain, and identical root `Error::Schedule` kind, text and source.
#[test]
fn new_variants_have_stable_kinds_contextual_display_sources_and_root_wrapping()
-> Result<(), Box<dyn Error>> {
    let (start, end) = (d("2026-02-19"), d("2026-06-01"));
    for tenor in [
        Period::new(-7, TimeUnit::Weeks),
        months(0),
        years(i32::MIN),
        Period::new(7, TimeUnit::Days),
    ] {
        let error = rejected(plain(start, end)?.build_old_cds(tenor, 2));
        assert_eq!(error.kind(), "schedule.invalid_old_cds_tenor");
        assert!(error.source().is_none());
        let text = error.to_string();
        for part in [start.to_string(), end.to_string(), tenor.to_string()] {
            assert!(text.contains(&part), "{text:?} lacks {part}");
        }
        let root = RootError::from(error.clone());
        assert_eq!(root.kind(), "schedule.invalid_old_cds_tenor");
        assert_eq!(root.to_string(), text);
        assert!(matches!(&root, RootError::Schedule(inner) if *inner == error));
        assert_eq!(
            root.source()
                .and_then(|source| source.downcast_ref::<ScheduleError>()),
            Some(&error)
        );
        assert!(root.source().and_then(|source| source.source()).is_none());
    }
    let (start, end) = (d("2026-01-05"), d("9999-12-25"));
    let error = rejected(plain(start, end)?.build_old_cds(months(12), 2));
    assert_eq!(error.kind(), "schedule.old_cds_date");
    let text = error.to_string();
    for part in [
        "2026-01-05",
        "9999-12-25",
        "12M",
        "terminal quarter",
        "required terminal quarter from 9999-12-25",
    ] {
        assert!(text.contains(part), "{text:?} lacks {part}");
    }
    let cause = error
        .source()
        .and_then(|source| source.downcast_ref::<DateError>())
        .ok_or("OldCdsDate must expose its DateError")?;
    assert_eq!(cause.kind(), "date.out_of_range");
    assert!(
        matches!(cause, DateError::OutOfRange { date, operation: "add months", count: 3, .. }
        if *date == end)
    );
    let root = RootError::from(error.clone());
    assert_eq!(root.kind(), "schedule.old_cds_date");
    assert_eq!(root.to_string(), text);
    assert_eq!(
        root.source()
            .and_then(|source| source.downcast_ref::<ScheduleError>()),
        Some(&error)
    );
    assert_eq!(
        root.source()
            .and_then(|source| source.source())
            .and_then(|source| source.downcast_ref::<DateError>()),
        Some(cause)
    );
    assert!(
        root.source()
            .and_then(|source| source.source())
            .and_then(|source| source.source())
            .is_none()
    );
    Ok(())
}

/// FR-SCHED-35: all 49 ordinary/termination pairs on a weekends-only calendar
/// with weekend starts (Sat 2026-01-31, Sat 02-07, Sun 03-01, Sun 03-22, Sat
/// 08-15), weekend interiors (Sat 06-20, Sun 09-20) and a Sunday terminal
/// (12-20): S and interiors use the ordinary rule, R the termination rule, and
/// raw dates and flags never change.
#[test]
fn all_49_convention_pairs_adjust_start_and_interiors_ordinarily_and_r_separately()
-> Result<(), Box<dyn Error>> {
    let weekends = Calendar::weekends_only();
    for (text, weekday) in [
        ("2026-01-31", Weekday::Saturday),
        ("2026-02-07", Weekday::Saturday),
        ("2026-03-01", Weekday::Sunday),
        ("2026-03-20", Weekday::Friday),
        ("2026-03-22", Weekday::Sunday),
        ("2026-06-20", Weekday::Saturday),
        ("2026-08-15", Weekday::Saturday),
        ("2026-09-20", Weekday::Sunday),
        ("2026-12-20", Weekday::Sunday),
    ] {
        assert_eq!(d(text).weekday(), weekday);
        assert_eq!(
            weekend(d(text)),
            matches!(weekday, Weekday::Saturday | Weekday::Sunday)
        );
    }
    // Literal anchors in Bdc::ALL order: F, MF, P, MP, Unadjusted, HMMF, Nearest.
    for (raw, expected) in [
        (
            "2026-01-31",
            [
                "2026-02-02",
                "2026-01-30",
                "2026-01-30",
                "2026-01-30",
                "2026-01-31",
                "2026-01-30",
                "2026-01-30",
            ],
        ),
        (
            "2026-02-07",
            [
                "2026-02-09",
                "2026-02-09",
                "2026-02-06",
                "2026-02-06",
                "2026-02-07",
                "2026-02-09",
                "2026-02-06",
            ],
        ),
        (
            "2026-03-01",
            [
                "2026-03-02",
                "2026-03-02",
                "2026-02-27",
                "2026-03-02",
                "2026-03-01",
                "2026-03-02",
                "2026-03-02",
            ],
        ),
        ("2026-03-20", ["2026-03-20"; 7]),
        (
            "2026-03-22",
            [
                "2026-03-23",
                "2026-03-23",
                "2026-03-20",
                "2026-03-20",
                "2026-03-22",
                "2026-03-23",
                "2026-03-23",
            ],
        ),
        (
            "2026-06-20",
            [
                "2026-06-22",
                "2026-06-22",
                "2026-06-19",
                "2026-06-19",
                "2026-06-20",
                "2026-06-22",
                "2026-06-19",
            ],
        ),
        // HMMF from the 15th cannot cross into the second half-month.
        (
            "2026-08-15",
            [
                "2026-08-17",
                "2026-08-17",
                "2026-08-14",
                "2026-08-14",
                "2026-08-15",
                "2026-08-14",
                "2026-08-14",
            ],
        ),
        (
            "2026-09-20",
            [
                "2026-09-21",
                "2026-09-21",
                "2026-09-18",
                "2026-09-18",
                "2026-09-20",
                "2026-09-21",
                "2026-09-21",
            ],
        ),
        (
            "2026-12-20",
            [
                "2026-12-21",
                "2026-12-21",
                "2026-12-18",
                "2026-12-18",
                "2026-12-20",
                "2026-12-21",
                "2026-12-21",
            ],
        ),
    ] {
        for (convention, wanted) in Bdc::ALL.into_iter().zip(expected) {
            assert_eq!(
                expected_adjust(d(raw), convention)?,
                d(wanted),
                "{raw} {convention:?}"
            );
            assert_eq!(
                weekends.adjust(d(raw), convention)?,
                d(wanted),
                "{raw} {convention:?}"
            );
        }
    }
    let end = d("2026-12-01");
    let mut pairs = 0;
    for (start, tenor, raw, expected_flags) in [
        (
            "2026-01-31",
            months(3),
            "2026-01-31 2026-03-20 2026-06-20 2026-09-20 2026-12-20",
            "SRRR",
        ),
        (
            "2026-02-07",
            months(3),
            "2026-02-07 2026-03-20 2026-06-20 2026-09-20 2026-12-20",
            "SRRR",
        ),
        (
            "2026-03-01",
            months(3),
            "2026-03-01 2026-06-20 2026-09-20 2026-12-20",
            "SRR",
        ),
        (
            "2026-03-22",
            months(3),
            "2026-03-22 2026-06-20 2026-09-20 2026-12-20",
            "SRR",
        ),
        (
            "2026-08-15",
            months(3),
            "2026-08-15 2026-09-20 2026-12-20",
            "SR",
        ),
        (
            "2026-02-07",
            months(1),
            "2026-02-07 2026-03-20 2026-04-20 2026-05-20 2026-06-20 2026-07-20 \
             2026-08-20 2026-09-20 2026-10-20 2026-11-20 2026-12-20",
            "SRRRRRRRRR",
        ),
        (
            "2026-03-01",
            months(1),
            "2026-03-01 2026-06-20 2026-07-20 2026-08-20 2026-09-20 2026-10-20 \
             2026-11-20 2026-12-20",
            "SRRRRRR",
        ),
    ] {
        let raw = dates(raw);
        for ordinary in Bdc::ALL {
            for terminal in Bdc::ALL {
                let schedule = on(d(start), end, weekends.clone(), ordinary, terminal)?
                    .build_old_cds(tenor, 20)?;
                assert_eq!(schedule.contractual_dates(), raw);
                assert_eq!(schedule.regularity(), flags(expected_flags));
                let expected = raw
                    .iter()
                    .enumerate()
                    .map(|(index, date)| {
                        let convention = if index + 1 == raw.len() {
                            terminal
                        } else {
                            ordinary
                        };
                        expected_adjust(*date, convention)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                assert_eq!(
                    schedule.adjusted_dates(),
                    expected,
                    "{start} {tenor} {ordinary:?}/{terminal:?}"
                );
                assert_eq!(schedule.convention(), ordinary);
                assert_eq!(schedule.termination_convention(), terminal);
                assert_eq!(
                    schedule.calendar(),
                    &ScheduleCalendar::Plain(weekends.clone())
                );
                let provenance = schedule.old_cds_provenance().ok_or("missing provenance")?;
                assert_eq!(provenance.resolved_end(), d("2026-12-20"));
                pairs += 1;
            }
        }
    }
    assert_eq!(pairs, 7 * 49);
    Ok(())
}

/// FR-SCHED-35: adjacent adjusted collisions and reversals are rejected with
/// the second boundary's index and both raw and adjusted dates (start/first
/// interior, interior/interior, last interior/terminal and start/terminal);
/// nothing is sorted, merged or dropped.
#[test]
fn adjusted_collisions_and_reversals_are_rejected_with_raw_and_adjusted_context()
-> Result<(), Box<dyn Error>> {
    // (start, end, tenor, closed range, ordinary, terminal, index, previous raw,
    // raw, previous adjusted, adjusted)
    for (start, end, tenor, closed, ordinary, terminal, index, previous, current, earlier, later) in [
        (
            "2026-02-18",
            "2026-06-01",
            months(3),
            ("2026-02-18", "2026-03-20"),
            Bdc::Following,
            Bdc::Following,
            1_usize,
            "2026-02-18",
            "2026-03-20",
            "2026-03-21",
            "2026-03-21",
        ),
        (
            "2026-01-05",
            "2026-08-21",
            months(1),
            ("2026-04-20", "2026-05-20"),
            Bdc::Following,
            Bdc::Following,
            3,
            "2026-04-20",
            "2026-05-20",
            "2026-05-21",
            "2026-05-21",
        ),
        (
            "2026-01-05",
            "2026-08-21",
            months(1),
            ("2026-08-20", "2026-09-19"),
            Bdc::Following,
            Bdc::Unadjusted,
            7,
            "2026-08-20",
            "2026-09-20",
            "2026-09-20",
            "2026-09-20",
        ),
        (
            "2026-01-05",
            "2026-08-21",
            months(1),
            ("2026-08-20", "2026-09-20"),
            Bdc::Following,
            Bdc::Preceding,
            7,
            "2026-08-20",
            "2026-09-20",
            "2026-09-21",
            "2026-08-19",
        ),
        (
            "2026-03-19",
            "2026-03-20",
            months(3),
            ("2026-03-19", "2026-03-19"),
            Bdc::Following,
            Bdc::Following,
            1,
            "2026-03-19",
            "2026-03-20",
            "2026-03-20",
            "2026-03-20",
        ),
        (
            "2026-03-19",
            "2026-03-20",
            months(3),
            ("2026-03-19", "2026-03-20"),
            Bdc::Following,
            Bdc::Preceding,
            1,
            "2026-03-19",
            "2026-03-20",
            "2026-03-21",
            "2026-03-18",
        ),
        (
            "2026-03-19",
            "2026-03-20",
            months(3),
            ("2026-03-19", "2026-03-20"),
            Bdc::Nearest,
            Bdc::ModifiedPreceding,
            1,
            "2026-03-19",
            "2026-03-20",
            "2026-03-18",
            "2026-03-18",
        ),
    ] {
        let error = rejected(
            on(
                d(start),
                d(end),
                closed_ranges(&[closed])?,
                ordinary,
                terminal,
            )?
            .build_old_cds(tenor, 20),
        );
        assert_eq!(error.kind(), "schedule.adjusted_order");
        assert!(error.source().is_none());
        assert!(
            matches!(error, ScheduleError::AdjustedOrder {
                index: actual_index, previous_contractual, contractual,
                previous_adjusted, adjusted, .. }
            if actual_index == index && previous_contractual == d(previous)
                && contractual == d(current) && previous_adjusted == d(earlier)
                && adjusted == d(later)),
            "{error:?}"
        );
        // Without the closure the same raw pair is retained in place.
        let open =
            on(d(start), d(end), Calendar::null(), ordinary, terminal)?.build_old_cds(tenor, 20)?;
        assert_eq!(open.contractual_dates()[index - 1], d(previous));
        assert_eq!(open.contractual_dates()[index], d(current));
    }
    // One closed day fewer keeps the adjusted dates strictly increasing.
    let schedule = on(
        d("2026-02-18"),
        d("2026-06-01"),
        closed_ranges(&[("2026-02-18", "2026-03-18")])?,
        Bdc::Following,
        Bdc::Following,
    )?
    .build_old_cds(months(3), 3)?;
    assert_eq!(
        schedule.contractual_dates(),
        dates("2026-02-18 2026-03-20 2026-06-20")
    );
    assert_eq!(
        schedule.adjusted_dates(),
        dates("2026-03-19 2026-03-20 2026-06-20")
    );
    assert_eq!(schedule.regularity(), flags("SR"));
    Ok(())
}

/// FR-SCHED-35/37: adjustment failures keep phase `adjustment`, the zero-based
/// boundary index and typed CalendarError/DateError causes through the root
/// source chain; boundaries are adjusted in order, so the earliest failure wins.
#[test]
fn adjustment_search_failures_report_the_earliest_failing_boundary_index()
-> Result<(), Box<dyn Error>> {
    let closed = Calendar::custom(Weekday::ALL, []);
    let (start, end) = (d("2026-02-18"), d("2026-06-01")); // raw Feb 18, Mar 20, Jun 20
    for (ordinary, terminal, index) in [
        (Bdc::Following, Bdc::Following, 0_usize),
        (Bdc::Nearest, Bdc::Unadjusted, 0),
        (Bdc::Unadjusted, Bdc::Following, 2),
        (Bdc::Unadjusted, Bdc::HalfMonthModifiedFollowing, 2),
    ] {
        let error = rejected(
            on(start, end, closed.clone(), ordinary, terminal)?.build_old_cds(months(3), 3),
        );
        assert!(
            matches!(&error, ScheduleError::Calendar { index: Some(actual), phase: "adjustment",
                cause: CalendarError::NoBusinessWeekday, .. } if *actual == index),
            "{error:?}"
        );
        assert_eq!(error.kind(), "schedule.calendar");
        assert!(
            error
                .source()
                .is_some_and(|source| source.is::<CalendarError>())
        );
    }
    let unadjusted =
        on(start, end, closed, Bdc::Unadjusted, Bdc::Unadjusted)?.build_old_cds(months(3), 3)?;
    assert_eq!(
        unadjusted.adjusted_dates(),
        dates("2026-02-18 2026-03-20 2026-06-20")
    );

    // A collision at index 1 precedes the search failure of the terminal.
    let error = rejected(
        on(
            start,
            end,
            closed_ranges(&[("2026-02-18", "2026-03-20"), ("2026-06-20", "2027-06-20")])?,
            Bdc::Following,
            Bdc::Following,
        )?
        .build_old_cds(months(3), 3),
    );
    assert!(
        matches!(error, ScheduleError::AdjustedOrder { index: 1, previous_adjusted, adjusted, .. }
        if previous_adjusted == d("2026-03-21") && adjusted == d("2026-03-21")),
        "{error:?}"
    );
    // A search failure at index 1 precedes the terminal that would also fail.
    let error = rejected(
        on(
            start,
            end,
            closed_ranges(&[("2026-03-20", "2027-03-20")])?,
            Bdc::Following,
            Bdc::Following,
        )?
        .build_old_cds(months(3), 3),
    );
    assert!(
        matches!(error, ScheduleError::Calendar { index: Some(1), phase: "adjustment",
            cause: CalendarError::SearchLimit { start: from, direction: 1, rejected: 366, .. }, .. }
        if from == d("2026-03-20")),
        "{error:?}"
    );

    // 365 rejected candidates are accepted; the 366th ends the search.
    let (start, end) = (d("2026-01-05"), d("2026-03-01")); // raw Jan 5, Mar 20
    let accepted = on(
        start,
        end,
        closed_ranges(&[("2026-03-20", "2027-03-19")])?,
        Bdc::Following,
        Bdc::Following,
    )?
    .build_old_cds(months(3), 2)?;
    assert_eq!(accepted.adjusted_dates(), dates("2026-01-05 2027-03-20"));
    let error = rejected(
        on(
            start,
            end,
            closed_ranges(&[("2026-03-20", "2027-03-20")])?,
            Bdc::Following,
            Bdc::Following,
        )?
        .build_old_cds(months(3), 2),
    );
    assert!(
        matches!(error, ScheduleError::Calendar { index: Some(1), phase: "adjustment",
            cause: CalendarError::SearchLimit { start: from, direction: 1, rejected: 366, .. }, .. }
        if from == d("2026-03-20")),
        "{error:?}"
    );
    let error = rejected(
        on(
            start,
            end,
            closed_ranges(&[("2025-01-05", "2026-01-05")])?,
            Bdc::Preceding,
            Bdc::Following,
        )?
        .build_old_cds(months(3), 2),
    );
    assert!(
        matches!(error, ScheduleError::Calendar { index: Some(0), phase: "adjustment",
            cause: CalendarError::SearchLimit { start: from, direction: -1, rejected: 366, .. }, .. }
        if from == start),
        "{error:?}"
    );

    // A modified rule finds no business day anywhere in February.
    let february = (1..=28)
        .map(|day| Date::from_ymd(2026, 2, day))
        .collect::<Result<Vec<_>, _>>()?;
    let error = rejected(
        on(
            d("2026-02-01"),
            d("2026-03-01"),
            Calendar::custom([Weekday::Saturday, Weekday::Sunday], february),
            Bdc::ModifiedFollowing,
            Bdc::Unadjusted,
        )?
        .build_old_cds(months(3), 2),
    );
    assert!(
        matches!(error, ScheduleError::Calendar { index: Some(0), phase: "adjustment",
            cause: CalendarError::NoBusinessDayInMonth { date, .. }, .. }
        if date == d("2026-02-01")),
        "{error:?}"
    );

    // The adjustment search itself leaves the supported civil range.
    for (start, end, closed, ordinary, terminal, index) in [
        (
            "0001-01-01",
            "0001-01-02",
            ("0001-01-01", "0001-01-01"),
            Bdc::Preceding,
            Bdc::Unadjusted,
            0_usize,
        ),
        (
            "9999-12-19",
            "9999-12-20",
            ("9999-12-20", "9999-12-31"),
            Bdc::Unadjusted,
            Bdc::Following,
            1,
        ),
    ] {
        let error = rejected(
            on(
                d(start),
                d(end),
                closed_ranges(&[closed])?,
                ordinary,
                terminal,
            )?
            .build_old_cds(months(3), 2),
        );
        assert!(
            matches!(&error, ScheduleError::Calendar { index: Some(actual), phase: "adjustment",
                cause: CalendarError::Date(_), .. } if *actual == index),
            "{error:?}"
        );
        let root = RootError::from(error);
        assert_eq!(root.kind(), "schedule.calendar");
        assert!(
            root.source()
                .and_then(|source| source.source())
                .is_some_and(|source| source.is::<CalendarError>())
        );
        assert!(
            root.source()
                .and_then(|source| source.source())
                .and_then(|source| source.source())
                .is_some_and(|source| source.is::<DateError>())
        );
    }
    Ok(())
}

/// FR-SCHED-37: the original start..end guard (phase `coverage`) precedes
/// terminal resolution; projection admits the span, so the unrepresentable
/// terminal is next. Inception and the 2003-2025 gap survive overrides and
/// joint members.
#[test]
fn target_original_span_guard_precedes_terminal_resolution() -> Result<(), Box<dyn Error>> {
    let (start, end) = (d("2026-01-05"), d("9999-12-31"));
    for tenor in [months(1), months(3), years(i32::MAX)] {
        let error = rejected(guarded(start, end, TargetCalendar::new())?.build_old_cds(tenor, 2));
        assert!(
            matches!(error, ScheduleError::Calendar { index: None, phase: "coverage",
                cause: CalendarError::UnverifiedCoverage { date, .. }, .. } if date == end),
            "{error:?}"
        );
        let error = rejected(
            guarded(
                start,
                end,
                TargetCalendar::with_policy(TargetPolicy::AllowProjection),
            )?
            .build_old_cds(tenor, 2),
        );
        assert!(
            matches!(error, ScheduleError::OldCdsDate { role: "terminal quarter", input, .. }
            if input == end),
            "{error:?}"
        );
    }
    let strict = TargetCalendar::new();
    let mut calendars = vec![
        strict.clone(),
        strict.without_holiday(d("2003-01-01")),
        strict.with_holiday(d("2003-01-01")),
    ];
    for joint in JointRule::ALL {
        calendars.push(strict.joint_with(vec![Calendar::null()], joint)?);
    }
    // Inception survives projection, holiday overrides and joint members.
    let inception = [TargetPolicy::VerifiedOnly, TargetPolicy::AllowProjection]
        .into_iter()
        .map(TargetCalendar::with_policy)
        .chain(calendars.iter().cloned());
    for calendar in inception {
        let error = rejected(guarded(d("1998-12-31"), end, calendar)?.build_old_cds(months(3), 2));
        assert!(
            matches!(error, ScheduleError::Calendar { index: None, phase: "coverage",
                cause: CalendarError::OutsideValidity { date, .. }, .. } if date == d("1998-12-31")),
            "{error:?}"
        );
    }
    for calendar in calendars {
        let error = rejected(
            guarded(d("2002-12-31"), d("2026-01-05"), calendar)?.build_old_cds(months(3), 200),
        );
        assert!(
            matches!(error, ScheduleError::Calendar { index: None, phase: "coverage",
                cause: CalendarError::UnverifiedCoverage { date, .. }, .. }
            if date == d("2003-01-01")),
            "{error:?}"
        );
    }
    Ok(())
}

/// FR-SCHED-37: when E is verified but R = Q(E) is not, the expanded start..R
/// guard (phase `old CDS coverage`) fails before the count/cap check, under
/// strict TARGET, holiday overrides and joint calendars with plain members;
/// explicit projection builds the same request.
#[test]
fn target_expanded_span_guard_precedes_the_cap_and_cannot_be_bypassed() -> Result<(), Box<dyn Error>>
{
    // (start, end, R, 1M count)
    for (start, end, terminal, count) in [
        ("2028-01-05", "2028-12-31", "2029-03-20", 12_u32),
        ("2002-01-07", "2002-12-21", "2003-03-20", 12),
        ("2028-12-01", "2028-12-31", "2029-03-20", 2),
    ] {
        let (start, end, terminal) = (d(start), d(end), d(terminal));
        let strict = TargetCalendar::new();
        let mut calendars = vec![
            strict.clone(),
            strict.without_holiday(terminal),
            strict.with_holiday(terminal),
        ];
        for joint in JointRule::ALL {
            calendars.push(strict.joint_with(vec![Calendar::null()], joint)?);
            calendars
                .push(strict.joint_with(vec![Calendar::weekends_only(), Calendar::null()], joint)?);
        }
        for calendar in calendars {
            for cap in [2, count, MAXIMUM_CAP] {
                let error =
                    rejected(guarded(start, end, calendar.clone())?.build_old_cds(months(1), cap));
                assert!(
                    matches!(&error, ScheduleError::Calendar { index: None, phase: "old CDS coverage",
                        cause: CalendarError::UnverifiedCoverage { date, .. }, .. }
                    if *date == terminal),
                    "{error:?}"
                );
                let root = RootError::from(error);
                assert_eq!(root.kind(), "schedule.calendar");
                assert!(
                    root.source()
                        .and_then(|source| source.source())
                        .is_some_and(|source| source.is::<CalendarError>())
                );
            }
        }
        for projected in [
            TargetCalendar::with_policy(TargetPolicy::AllowProjection),
            TargetCalendar::with_policy(TargetPolicy::AllowProjection)
                .joint_with(vec![Calendar::null()], JointRule::JoinHolidays)?,
        ] {
            let builder = guarded(start, end, projected)?;
            if count > ScheduleBuilder::MIN_BOUNDARIES {
                assert!(matches!(builder.build_old_cds(months(1), count - 1),
                    Err(ScheduleError::BoundaryLimitExceeded { maximum, required_at_least, .. })
                    if maximum == count - 1 && required_at_least == count));
            }
            let schedule = builder.build_old_cds(months(1), count)?;
            assert_eq!(schedule.contractual_dates().len(), usize::try_from(count)?);
            assert_eq!(schedule.contractual_dates().last(), Some(&terminal));
            let provenance = schedule.old_cds_provenance().ok_or("missing provenance")?;
            assert_eq!(provenance.requested_end(), end);
            assert_eq!(provenance.resolved_end(), terminal);
        }
    }
    // Projected B = R with an omitted A across the 2028/2029 coverage edge.
    let schedule = guarded(
        d("2028-12-01"),
        d("2028-12-31"),
        TargetCalendar::with_policy(TargetPolicy::AllowProjection),
    )?
    .build_old_cds(months(3), 2)?;
    assert_eq!(schedule.contractual_dates(), dates("2028-12-01 2029-03-20"));
    let provenance = schedule.old_cds_provenance().ok_or("missing provenance")?;
    assert_eq!(provenance.initial_quarter(), d("2028-12-20"));
    assert_eq!(provenance.recurrence_anchor(), d("2029-03-20"));
    assert_eq!(provenance.omitted_quarter(), Some(d("2028-12-20")));
    // E = 2028-12-20 is its own verified terminal; E = 2028-12-21 is not.
    let schedule = guarded(d("2028-01-05"), d("2028-12-20"), TargetCalendar::new())?
        .build_old_cds(months(1), 11)?;
    assert_eq!(
        schedule.contractual_dates(),
        joined(
            "2028-01-05",
            "2028-03-20 2028-04-20 2028-05-20 2028-06-20 2028-07-20 2028-08-20 \
             2028-09-20 2028-10-20 2028-11-20",
            "2028-12-20"
        )
    );
    assert_eq!(schedule.regularity(), flags("SRRRRRRRRR"));
    let error = rejected(
        guarded(d("2028-01-05"), d("2028-12-21"), TargetCalendar::new())?
            .build_old_cds(months(1), 12),
    );
    assert!(
        matches!(error, ScheduleError::Calendar { index: None, phase: "old CDS coverage",
            cause: CalendarError::UnverifiedCoverage { date, .. }, .. } if date == d("2029-03-20")),
        "{error:?}"
    );
    Ok(())
}

/// FR-SCHED-35/37: inside verified TARGET coverage the schedule adjusts with
/// TARGET business days, and an adjustment search that reaches unverified
/// coverage fails with the terminal's index while projection accepts it.
#[test]
fn target_verified_spans_adjust_and_adjustment_candidates_stay_guarded()
-> Result<(), Box<dyn Error>> {
    let target = TargetCalendar::new();
    let schedule = ScheduleBuilder::new(
        d("2026-01-05"),
        d("2028-12-20"),
        ScheduleCalendar::Target(target.clone()),
        Bdc::Following,
        Bdc::ModifiedFollowing,
    )?
    .build_old_cds(months(3), 13)?;
    let raw = joined(
        "2026-01-05",
        "2026-03-20 2026-06-20 2026-09-20 2026-12-20 2027-03-20 2027-06-20 \
         2027-09-20 2027-12-20 2028-03-20 2028-06-20 2028-09-20",
        "2028-12-20",
    );
    let adjusted = dates(
        "2026-01-05 2026-03-20 2026-06-22 2026-09-21 2026-12-21 2027-03-22 2027-06-21 \
         2027-09-20 2027-12-20 2028-03-20 2028-06-20 2028-09-20 2028-12-20",
    );
    assert_eq!(schedule.contractual_dates(), raw);
    assert_eq!(schedule.adjusted_dates(), adjusted);
    assert_eq!(schedule.regularity(), flags("SRRRRRRRRRRR"));
    for (index, (date, wanted)) in raw.iter().zip(&adjusted).enumerate() {
        let convention = if index + 1 == raw.len() {
            Bdc::ModifiedFollowing
        } else {
            Bdc::Following
        };
        assert_eq!(target.adjust(*date, convention)?, *wanted);
    }
    let mut strict = TargetCalendar::new();
    let mut projected = TargetCalendar::with_policy(TargetPolicy::AllowProjection);
    for day in 20..=31 {
        let holiday = Date::from_ymd(2028, 12, day)?;
        strict = strict.with_holiday(holiday);
        projected = projected.with_holiday(holiday);
    }
    let error = rejected(
        ScheduleBuilder::new(
            d("2028-11-01"),
            d("2028-12-20"),
            ScheduleCalendar::Target(strict),
            Bdc::Unadjusted,
            Bdc::Following,
        )?
        .build_old_cds(months(3), 2),
    );
    assert!(
        matches!(error, ScheduleError::Calendar { index: Some(1), phase: "adjustment",
            cause: CalendarError::UnverifiedCoverage { date, .. }, .. } if date == d("2029-01-01")),
        "{error:?}"
    );
    let schedule = ScheduleBuilder::new(
        d("2028-11-01"),
        d("2028-12-20"),
        ScheduleCalendar::Target(projected),
        Bdc::Unadjusted,
        Bdc::Following,
    )?
    .build_old_cds(months(3), 2)?;
    assert_eq!(schedule.contractual_dates(), dates("2028-11-01 2028-12-20"));
    assert_eq!(schedule.adjusted_dates(), dates("2028-11-01 2029-01-02"));
    Ok(())
}

/// FR-SCHED-34/35/36: for every start in 2027 and leap 2028, thirteen end
/// offsets and fifteen stored strides, an independent day-walk oracle written
/// from the contract agrees on dates, flags, quarters, omission and exact caps.
#[test]
fn independent_day_walk_oracle_agrees_for_every_start_in_a_common_and_a_leap_year()
-> Result<(), Box<dyn Error>> {
    let tenors = [
        months(1),
        months(2),
        months(3),
        months(4),
        months(5),
        months(6),
        months(7),
        months(11),
        months(12),
        years(1),
        months(18),
        months(24),
        years(2),
        months(i32::MAX),
        years(i32::MAX),
    ];
    let offsets = [1_i64, 2, 28, 29, 30, 31, 32, 61, 91, 92, 93, 183, 400];
    let first = d("2027-01-01");
    let mut cases = 0_usize;
    for day in 0..731_i64 {
        let start = first.checked_add_days(day)?;
        for offset in offsets {
            let end = start.checked_add_days(offset)?;
            let builder = plain(start, end)?;
            for tenor in tenors {
                let expected = expected_old_cds(start, end, stride_months(tenor))?;
                let count = u32::try_from(expected.raw.len())?;
                let schedule = builder.build_old_cds(tenor, count)?;
                assert_eq!(
                    schedule.contractual_dates(),
                    expected.raw,
                    "{start} {end} {tenor}"
                );
                assert_eq!(schedule.adjusted_dates(), expected.raw);
                assert_eq!(
                    schedule.regularity(),
                    expected.flags,
                    "{start} {end} {tenor}"
                );
                let provenance = schedule.old_cds_provenance().ok_or("missing provenance")?;
                assert_eq!(
                    (
                        provenance.initial_quarter(),
                        provenance.recurrence_anchor(),
                        provenance.resolved_end(),
                        provenance.omitted_quarter(),
                    ),
                    (
                        expected.initial,
                        expected.anchor,
                        expected.terminal,
                        expected.omitted,
                    ),
                    "{start} {end} {tenor}"
                );
                assert!(start < expected.anchor && expected.anchor <= expected.terminal);
                assert!(expected.raw.windows(2).all(|pair| pair[0] < pair[1]));
                if count > ScheduleBuilder::MIN_BOUNDARIES {
                    assert!(
                        matches!(builder.build_old_cds(tenor, count - 1),
                        Err(ScheduleError::BoundaryLimitExceeded { maximum, required_at_least, .. })
                        if maximum == count - 1 && required_at_least == count),
                        "{start} {end} {tenor}"
                    );
                }
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 731 * 13 * 15);
    Ok(())
}
