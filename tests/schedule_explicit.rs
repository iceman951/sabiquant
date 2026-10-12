//! Explicit coupon roles checked against civil-span lattice congruences.
//!
//! The oracle visits civil dates and tests integer day/month/year congruences;
//! it never calls Date's recurrence/advancement helpers or Schedule's internals.

use sabiquant::calendar::{
    BusinessDayConvention as Bdc, Calendar, CalendarError, EndOfMonth, JointRule, TargetCalendar,
    TargetPolicy,
};
use sabiquant::error::Error as CrateError;
use sabiquant::schedule::{
    DateGenerationRule as Rule, ExplicitStubDates, PeriodRegularity as Regularity, Schedule,
    ScheduleBuilder, ScheduleCalendar, ScheduleError, StubPolicy,
};
use sabiquant::time::{Date, DateError, Period, TimeUnit, Weekday};
use std::collections::{HashSet, hash_map::DefaultHasher};
use std::error::Error;
use std::hash::{Hash, Hasher};

#[expect(
    clippy::expect_used,
    reason = "tests supply explicitly valid civil components"
)]
fn civil(y: i32, m: u32, d: u32) -> Date {
    Date::from_ymd(y, m, d).expect("valid civil test date")
}

fn month_days(y: i32, m: u32) -> u32 {
    match m {
        2 => {
            if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn ordinal(y: i32, m: u32, d: u32) -> i64 {
    let previous = i64::from(y - 1);
    365 * previous + previous / 4 - previous / 100
        + previous / 400
        + (1..m)
            .map(|month| i64::from(month_days(y, month)))
            .sum::<i64>()
        + i64::from(d - 1)
}

fn next_civil((y, m, d): (i32, u32, u32)) -> (i32, u32, u32) {
    if d < month_days(y, m) {
        (y, m, d + 1)
    } else if m < 12 {
        (y, m + 1, 1)
    } else {
        (y + 1, 1, 1)
    }
}

fn previous_civil((y, m, d): (i32, u32, u32)) -> (i32, u32, u32) {
    if d > 1 {
        (y, m, d - 1)
    } else if m > 1 {
        (y, m - 1, month_days(y, m - 1))
    } else {
        (y - 1, 12, 31)
    }
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

fn hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

// Mathematical lattice membership, including negative quasi-boundary indices.
// It scans no recurrence sequence and does not infer a tenor from date gaps.
fn lattice_index(
    candidate: (i32, u32, u32),
    anchor: (i32, u32, u32),
    tenor: Period,
    eom: EndOfMonth,
) -> Option<i64> {
    let (y, m, d) = candidate;
    let (ay, am, ad) = anchor;
    let length = i64::from(tenor.length());
    match tenor.unit() {
        TimeUnit::Days | TimeUnit::Weeks => {
            let quantum = length
                * if tenor.unit() == TimeUnit::Weeks {
                    7
                } else {
                    1
                };
            let difference = ordinal(y, m, d) - ordinal(ay, am, ad);
            (difference % quantum == 0).then_some(difference / quantum)
        }
        TimeUnit::Months | TimeUnit::Years => {
            let difference = if tenor.unit() == TimeUnit::Years {
                if m != am {
                    return None;
                }
                i64::from(y - ay)
            } else {
                i64::from(y - ay) * 12 + i64::from(m) - i64::from(am)
            };
            if difference % length != 0 {
                return None;
            }
            let target_day = if eom == EndOfMonth::Preserve && ad == month_days(ay, am) {
                month_days(y, m)
            } else {
                ad.min(month_days(y, m))
            };
            (d == target_day).then_some(difference / length)
        }
    }
}

struct Expected {
    dates: Vec<Date>,
    flags: Vec<Regularity>,
    anchor: Date,
}

fn civil_span_oracle(
    start: Date,
    end: Date,
    first: Option<Date>,
    penultimate: Option<Date>,
    tenor: Period,
    rule: Rule,
    eom: EndOfMonth,
) -> Option<Expected> {
    let middle_start = first.unwrap_or(start);
    let middle_end = penultimate.unwrap_or(end);
    let anchor = if rule == Rule::Forward {
        middle_start
    } else {
        middle_end
    };
    let a = anchor.to_ymd();
    let endpoint = if rule == Rule::Forward {
        middle_end
    } else {
        middle_start
    };
    lattice_index(endpoint.to_ymd(), a, tenor, eom)?;
    let mut dates = Vec::new();
    let mut current = start.to_ymd();
    loop {
        let value = civil(current.0, current.1, current.2);
        if value == start
            || value == end
            || value >= middle_start
                && value <= middle_end
                && lattice_index(current, a, tenor, eom).is_some()
        {
            dates.push(value);
        }
        if value == end {
            break;
        }
        current = next_civil(current);
    }
    let flags = dates
        .windows(2)
        .map(|pair| {
            match (
                lattice_index(pair[0].to_ymd(), a, tenor, eom),
                lattice_index(pair[1].to_ymd(), a, tenor, eom),
            ) {
                (Some(left), Some(right)) if right - left == 1 => Regularity::Regular,
                _ => Regularity::Stub,
            }
        })
        .collect();
    Some(Expected {
        dates,
        flags,
        anchor,
    })
}

// Fixture input construction only: selects a convenient two-tenor endpoint.
// Expected complete vectors/flags still come exclusively from civil-span scan.
#[expect(
    clippy::expect_used,
    reason = "positive-cycle fixture offsets fit the civil component types"
)]
fn two_tenor_endpoint(anchor: Date, unit: TimeUnit, eom: EndOfMonth) -> Date {
    let (y, m, d) = anchor.to_ymd();
    match unit {
        TimeUnit::Days | TimeUnit::Weeks => {
            let mut cursor = (y, m, d);
            for _ in 0..if unit == TimeUnit::Weeks { 14 } else { 2 } {
                cursor = next_civil(cursor);
            }
            civil(cursor.0, cursor.1, cursor.2)
        }
        TimeUnit::Months => {
            let index = i64::from(y) * 12 + i64::from(m - 1) + 2;
            let year = i32::try_from(index / 12).expect("cycle year fits");
            let month = u32::try_from(index % 12 + 1).expect("cycle month fits");
            civil(
                year,
                month,
                if eom == EndOfMonth::Preserve {
                    month_days(year, month)
                } else {
                    d.min(month_days(year, month))
                },
            )
        }
        TimeUnit::Years => civil(
            y + 2,
            m,
            if eom == EndOfMonth::Preserve {
                month_days(y + 2, m)
            } else {
                d.min(month_days(y + 2, m))
            },
        ),
    }
}

#[test]
fn gregorian_cycle_civil_congruences_cover_every_unit_direction_and_role()
-> Result<(), Box<dyn Error>> {
    let mut checked = 0_usize;
    let mut successes = 0_usize;
    let mut irregular = 0_usize;
    let mut flags_checked = 0_usize;
    for year in 2000..2400 {
        for month in 1..=12 {
            let coupon = civil(year, month, month_days(year, month));
            for unit in TimeUnit::ALL {
                let tenor = Period::new(1, unit);
                let policies: &[EndOfMonth] = if matches!(unit, TimeUnit::Months | TimeUnit::Years)
                {
                    &[EndOfMonth::Ignore, EndOfMonth::Preserve]
                } else {
                    &[EndOfMonth::Ignore]
                };
                for &eom in policies {
                    let second = two_tenor_endpoint(coupon, unit, eom);
                    let p = previous_civil(coupon.to_ymd());
                    let before = civil(p.0, p.1, p.2);
                    let n = next_civil(second.to_ymd());
                    let after = civil(n.0, n.1, n.2);
                    let a = next_civil(coupon.to_ymd());
                    let after_coupon = civil(a.0, a.1, a.2);
                    for (start, end, first, penultimate) in [
                        (before, second, Some(coupon), None),
                        (coupon, after, None, Some(second)),
                        (before, after, Some(coupon), Some(second)),
                        (before, after_coupon, Some(coupon), Some(coupon)),
                    ] {
                        for rule in [Rule::Forward, Rule::Backward] {
                            let explicit = ExplicitStubDates::new(first, penultimate)?;
                            let model =
                                civil_span_oracle(start, end, first, penultimate, tenor, rule, eom);
                            let b = plain(start, end)?;
                            checked += 1;
                            match model {
                                Some(expected) => {
                                    let cap = u32::try_from(expected.dates.len())?;
                                    let schedule =
                                        b.build_explicit(tenor, rule, explicit, eom, cap)?;
                                    assert_eq!(
                                        schedule.contractual_dates(),
                                        expected.dates,
                                        "{year}-{month}, {unit:?}/{rule:?}/{eom:?}, {explicit:?}"
                                    );
                                    assert_eq!(schedule.adjusted_dates(), expected.dates);
                                    assert_eq!(schedule.regularity(), expected.flags);
                                    assert_eq!(schedule.explicit_stub_dates(), Some(explicit));
                                    assert_eq!(schedule.lattice_anchor(), Some(expected.anchor));
                                    assert_eq!(schedule.stub_policy(), None);
                                    assert_eq!(schedule.generation_rule(), rule);
                                    assert_eq!(schedule.end_of_month(), Some(eom));
                                    assert_eq!(
                                        schedule.tenor().map(|p| (p.length(), p.unit())),
                                        Some((1, unit))
                                    );
                                    assert!(
                                        matches!(b.build_explicit(tenor,rule,explicit,eom,cap-1),
                                        Err(ScheduleError::BoundaryLimitExceeded {maximum,..}) if maximum==cap-1)
                                    );
                                    successes += 1;
                                    flags_checked += expected.flags.len();
                                }
                                None => {
                                    let error = b
                                        .build_explicit(tenor, rule, explicit, eom, 10)
                                        .unwrap_err();
                                    assert!(
                                        matches!(error,ScheduleError::IrregularExplicitMiddle {rule: observed,tenor:period,..}
                                        if observed==rule && period==tenor)
                                    );
                                    irregular += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(checked, 230_400);
    assert_eq!(successes + irregular, checked);
    assert_eq!(successes, 227_709);
    assert_eq!(irregular, 2_691);
    assert_eq!(flags_checked, 682_230);
    Ok(())
}

#[test]
fn explicit_value_absence_order_and_equal_roles_are_checked() {
    let first = civil(2026, 2, 1);
    let second = civil(2026, 3, 1);
    assert_eq!(
        ExplicitStubDates::new(None, None).unwrap_err().kind(),
        "schedule.empty_explicit_stub_dates"
    );
    assert!(matches!(ExplicitStubDates::new(Some(second),Some(first)),
        Err(ScheduleError::ReversedExplicitStubDates {first:a,penultimate:b,..}) if a==second && b==first));
    for (a, b) in [
        (Some(first), None),
        (None, Some(first)),
        (Some(first), Some(first)),
        (Some(first), Some(second)),
    ] {
        let value = ExplicitStubDates::new(a, b).unwrap();
        assert_eq!(value.first(), a);
        assert_eq!(value.penultimate(), b);
        assert_eq!(value, value.clone());
        assert_eq!(hash(&value), hash(&value.clone()));
    }
}

#[test]
fn explicit_build_input_precedence_and_both_strict_interior_roles_are_contextual() {
    let start = civil(2026, 1, 1);
    let end = civil(2026, 4, 1);
    let b = plain(start, end).unwrap();
    let outside = ExplicitStubDates::new(Some(start), Some(end)).unwrap();
    assert_eq!(
        b.build_explicit(
            Period::new(0, TimeUnit::Days),
            Rule::Zero,
            outside,
            EndOfMonth::Preserve,
            1
        )
        .unwrap_err()
        .kind(),
        "schedule.invalid_boundary_limit"
    );
    assert_eq!(
        b.build_explicit(
            Period::new(0, TimeUnit::Days),
            Rule::Zero,
            outside,
            EndOfMonth::Preserve,
            2
        )
        .unwrap_err()
        .kind(),
        "schedule.invalid_rule"
    );
    assert_eq!(
        b.build_explicit(
            Period::new(0, TimeUnit::Days),
            Rule::Forward,
            outside,
            EndOfMonth::Preserve,
            2
        )
        .unwrap_err()
        .kind(),
        "schedule.invalid_tenor"
    );
    assert_eq!(
        b.build_explicit(
            Period::new(1, TimeUnit::Days),
            Rule::Forward,
            outside,
            EndOfMonth::Preserve,
            2
        )
        .unwrap_err()
        .kind(),
        "schedule.incompatible_end_of_month"
    );
    let error = b
        .build_explicit(
            Period::new(1, TimeUnit::Months),
            Rule::Forward,
            outside,
            EndOfMonth::Ignore,
            2,
        )
        .unwrap_err();
    assert!(
        matches!(error,ScheduleError::ExplicitStubOutsideRange {role:"first",date,..} if date==start)
    );
    for role in ["first", "penultimate"] {
        for value in [civil(2025, 12, 31), start, end, civil(2026, 4, 2)] {
            let dates = ExplicitStubDates::new(
                if role == "first" { Some(value) } else { None },
                if role == "penultimate" {
                    Some(value)
                } else {
                    None
                },
            )
            .unwrap();
            match b
                .build_explicit(
                    Period::new(1, TimeUnit::Months),
                    Rule::Backward,
                    dates,
                    EndOfMonth::Ignore,
                    10,
                )
                .unwrap_err()
            {
                ScheduleError::ExplicitStubOutsideRange {
                    role: observed,
                    date,
                    start: s,
                    end: e,
                    ..
                } => {
                    assert_eq!(observed, role);
                    assert_eq!(date, value);
                    assert_eq!((s, e), (start, end));
                }
                other => panic!("unexpected {other:?}"),
            }
        }
    }
    let proper = ExplicitStubDates::new(Some(civil(2026, 2, 1)), None).unwrap();
    for unit in TimeUnit::ALL {
        for length in [i32::MIN, -1, 0] {
            assert!(
                matches!(b.build_explicit(Period::new(length,unit),Rule::Forward,proper,EndOfMonth::Ignore,10),
                Err(ScheduleError::InvalidTenor {tenor,..}) if tenor.length()==length && tenor.unit()==unit)
            );
        }
    }
}

#[test]
fn clamp_and_eom_outer_flags_come_from_the_named_directional_seed() {
    let b = plain(civil(2023, 1, 31), civil(2023, 5, 31)).unwrap();
    let dates = ExplicitStubDates::new(Some(civil(2023, 2, 28)), Some(civil(2023, 4, 28))).unwrap();
    let schedule = b
        .build_explicit(
            Period::new(1, TimeUnit::Months),
            Rule::Forward,
            dates,
            EndOfMonth::Ignore,
            5,
        )
        .unwrap();
    assert_eq!(
        schedule.contractual_dates(),
        [
            civil(2023, 1, 31),
            civil(2023, 2, 28),
            civil(2023, 3, 28),
            civil(2023, 4, 28),
            civil(2023, 5, 31)
        ]
    );
    assert_eq!(
        schedule.regularity(),
        [
            Regularity::Stub,
            Regularity::Regular,
            Regularity::Regular,
            Regularity::Stub
        ]
    );
    assert_eq!(schedule.lattice_anchor(), Some(civil(2023, 2, 28)));
    let eom = ExplicitStubDates::new(Some(civil(2023, 2, 28)), Some(civil(2023, 4, 30))).unwrap();
    for rule in [Rule::Forward, Rule::Backward] {
        let result = b
            .build_explicit(
                Period::new(1, TimeUnit::Months),
                rule,
                eom,
                EndOfMonth::Preserve,
                5,
            )
            .unwrap();
        assert_eq!(
            result.contractual_dates(),
            [
                civil(2023, 1, 31),
                civil(2023, 2, 28),
                civil(2023, 3, 31),
                civil(2023, 4, 30),
                civil(2023, 5, 31)
            ]
        );
        assert!(
            result
                .regularity()
                .iter()
                .all(|&flag| flag == Regularity::Regular)
        );
    }
    // Friday Jan29 is a business month end, but not calendar EOM. It cannot
    // activate Preserve or rewrite an explicit coupon.
    let dates = ExplicitStubDates::new(Some(civil(2021, 1, 29)), Some(civil(2021, 3, 29))).unwrap();
    let result = ScheduleBuilder::new(
        civil(2021, 1, 1),
        civil(2021, 4, 29),
        ScheduleCalendar::Plain(Calendar::weekends_only()),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )
    .unwrap()
    .build_explicit(
        Period::new(1, TimeUnit::Months),
        Rule::Forward,
        dates,
        EndOfMonth::Preserve,
        5,
    )
    .unwrap();
    assert_eq!(
        result.contractual_dates(),
        [
            civil(2021, 1, 1),
            civil(2021, 1, 29),
            civil(2021, 2, 28),
            civil(2021, 3, 29),
            civil(2021, 4, 29)
        ]
    );
}

#[test]
fn explicit_middle_never_accepts_an_implicit_internal_stub() {
    let b = plain(civil(2026, 1, 1), civil(2026, 7, 1)).unwrap();
    for (rule, first, pen, anchor, endpoint, last) in [
        (
            Rule::Forward,
            civil(2026, 2, 1),
            civil(2026, 5, 2),
            civil(2026, 2, 1),
            civil(2026, 5, 2),
            civil(2026, 5, 1),
        ),
        (
            Rule::Backward,
            civil(2026, 2, 2),
            civil(2026, 5, 1),
            civil(2026, 5, 1),
            civil(2026, 2, 2),
            civil(2026, 3, 1),
        ),
    ] {
        let dates = ExplicitStubDates::new(Some(first), Some(pen)).unwrap();
        let error = b
            .build_explicit(
                Period::new(1, TimeUnit::Months),
                rule,
                dates,
                EndOfMonth::Ignore,
                10,
            )
            .unwrap_err();
        assert!(
            matches!(&error,ScheduleError::IrregularExplicitMiddle {rule:r,tenor,anchor:a,endpoint:e,regular_boundary:boundary,..}
            if *r==rule && tenor.length()==1 && *a==anchor && *e==endpoint && *boundary==last)
        );
        assert_eq!(error.kind(), "schedule.irregular_explicit_middle");
        assert!(error.source().is_none());
        assert!(error.to_string().contains(&endpoint.to_string()));
    }
}

#[test]
fn equal_roles_reuse_one_boundary_and_huge_tenors_only_classify_outer_stubs() {
    let b = plain(civil(2026, 1, 1), civil(2026, 3, 1)).unwrap();
    let mid = civil(2026, 2, 1);
    let dates = ExplicitStubDates::new(Some(mid), Some(mid)).unwrap();
    for rule in [Rule::Forward, Rule::Backward] {
        for unit in TimeUnit::ALL {
            let result = b
                .build_explicit(
                    Period::new(i32::MAX, unit),
                    rule,
                    dates,
                    EndOfMonth::Ignore,
                    3,
                )
                .unwrap();
            assert_eq!(
                result.contractual_dates(),
                [civil(2026, 1, 1), mid, civil(2026, 3, 1)]
            );
            assert_eq!(result.regularity(), [Regularity::Stub, Regularity::Stub]);
            assert_eq!(result.lattice_anchor(), Some(mid));
            assert!(matches!(
                b.build_explicit(
                    Period::new(i32::MAX, unit),
                    rule,
                    dates,
                    EndOfMonth::Ignore,
                    2
                ),
                Err(ScheduleError::BoundaryLimitExceeded {
                    maximum: 2,
                    required_at_least: 3,
                    ..
                })
            ));
        }
    }
    // A two-tenor outer span is Stub despite both endpoints lying on the seed.
    let dates = ExplicitStubDates::new(Some(civil(2026, 2, 1)), Some(civil(2026, 2, 1))).unwrap();
    let result = plain(civil(2025, 12, 1), civil(2026, 4, 1))
        .unwrap()
        .build_explicit(
            Period::new(1, TimeUnit::Months),
            Rule::Forward,
            dates,
            EndOfMonth::Ignore,
            3,
        )
        .unwrap();
    assert_eq!(result.regularity(), [Regularity::Stub, Regularity::Stub]);
}

#[test]
fn classification_only_quasi_dates_outside_the_supported_civil_range_are_stubs() {
    let first = civil(1, 1, 2);
    let forward = plain(Date::MIN, civil(1, 3, 2))
        .unwrap()
        .build_explicit(
            Period::new(1, TimeUnit::Months),
            Rule::Forward,
            ExplicitStubDates::new(Some(first), None).unwrap(),
            EndOfMonth::Ignore,
            4,
        )
        .unwrap();
    assert_eq!(
        forward.contractual_dates(),
        [Date::MIN, first, civil(1, 2, 2), civil(1, 3, 2)]
    );
    assert_eq!(
        forward.regularity(),
        [Regularity::Stub, Regularity::Regular, Regularity::Regular]
    );
    let pen = civil(9999, 12, 30);
    let backward = plain(civil(9999, 10, 30), Date::MAX)
        .unwrap()
        .build_explicit(
            Period::new(1, TimeUnit::Months),
            Rule::Backward,
            ExplicitStubDates::new(None, Some(pen)).unwrap(),
            EndOfMonth::Ignore,
            4,
        )
        .unwrap();
    assert_eq!(
        backward.contractual_dates(),
        [civil(9999, 10, 30), civil(9999, 11, 30), pen, Date::MAX]
    );
    assert_eq!(
        backward.regularity(),
        [Regularity::Regular, Regularity::Regular, Regularity::Stub]
    );
}

#[test]
fn all_adjustment_pairs_keep_role_correspondence_or_report_the_first_conflict() {
    let raw = [
        civil(2026, 1, 2),
        civil(2026, 1, 3),
        civil(2026, 1, 4),
        civil(2026, 1, 6),
    ];
    let dates = ExplicitStubDates::new(Some(raw[1]), Some(raw[2])).unwrap();
    let calendar = Calendar::weekends_only();
    for ordinary in Bdc::ALL {
        for termination in Bdc::ALL {
            let b = ScheduleBuilder::new(
                raw[0],
                raw[3],
                ScheduleCalendar::Plain(calendar.clone()),
                ordinary,
                termination,
            )
            .unwrap();
            let adjusted: Vec<_> = raw
                .iter()
                .enumerate()
                .map(|(index, &date)| {
                    calendar
                        .adjust(date, if index == 3 { termination } else { ordinary })
                        .unwrap()
                })
                .collect();
            let conflict = adjusted
                .windows(2)
                .position(|pair| pair[0] >= pair[1])
                .map(|index| index + 1);
            let actual = b.build_explicit(
                Period::new(1, TimeUnit::Days),
                Rule::Forward,
                dates,
                EndOfMonth::Ignore,
                4,
            );
            if let Some(index) = conflict {
                match actual.unwrap_err() {
                    ScheduleError::AdjustedOrder {
                        index: i,
                        previous_contractual,
                        contractual,
                        previous_adjusted,
                        adjusted: value,
                        ..
                    } => {
                        assert_eq!(i, index);
                        assert_eq!(
                            (previous_contractual, contractual),
                            (raw[index - 1], raw[index])
                        );
                        assert_eq!(
                            (previous_adjusted, value),
                            (adjusted[index - 1], adjusted[index])
                        );
                    }
                    other => panic!("unexpected {ordinary}/{termination}: {other:?}"),
                }
            } else {
                let result = actual.unwrap();
                assert_eq!(result.contractual_dates(), raw);
                assert_eq!(result.adjusted_dates(), adjusted);
                assert_eq!(
                    result.regularity(),
                    [Regularity::Regular, Regularity::Regular, Regularity::Stub]
                );
            }
        }
    }
}

#[test]
fn target_coverage_checks_the_entire_original_span_before_middle_generation() {
    let first = civil(2026, 1, 1);
    let explicit = ExplicitStubDates::new(Some(first), None).unwrap();
    for target in [
        TargetCalendar::new(),
        TargetCalendar::new()
            .joint_with(vec![Calendar::null()], JointRule::JoinBusinessDays)
            .unwrap(),
    ] {
        let b = ScheduleBuilder::new(
            civil(2002, 12, 1),
            civil(2026, 2, 1),
            ScheduleCalendar::Target(target),
            Bdc::Unadjusted,
            Bdc::Unadjusted,
        )
        .unwrap();
        let error = b
            .build_explicit(
                Period::new(1, TimeUnit::Months),
                Rule::Forward,
                explicit,
                EndOfMonth::Ignore,
                4,
            )
            .unwrap_err();
        assert!(
            matches!(&error,ScheduleError::Calendar {index:None,phase:"coverage",cause:CalendarError::UnverifiedCoverage {date,..},..}
            if *date==civil(2003,1,1))
        );
        let root = CrateError::from(error.clone());
        assert_eq!(root.kind(), "schedule.calendar");
        assert!(root.source().unwrap().is::<ScheduleError>());
        assert!(
            root.source()
                .unwrap()
                .source()
                .unwrap()
                .is::<CalendarError>()
        );
    }
    let projected = ScheduleBuilder::new(
        civil(2025, 12, 1),
        civil(2026, 2, 1),
        ScheduleCalendar::Target(TargetCalendar::with_policy(TargetPolicy::AllowProjection)),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )
    .unwrap()
    .build_explicit(
        Period::new(1, TimeUnit::Months),
        Rule::Forward,
        explicit,
        EndOfMonth::Ignore,
        3,
    )
    .unwrap();
    assert_eq!(
        projected.regularity(),
        [Regularity::Regular, Regularity::Regular]
    );
    let outer_gap = ScheduleBuilder::new(
        civil(2025, 12, 31),
        civil(2026, 2, 1),
        ScheduleCalendar::Target(TargetCalendar::new()),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )
    .unwrap();
    assert!(
        matches!(outer_gap.build_explicit(Period::new(1,TimeUnit::Months),Rule::Forward,explicit,EndOfMonth::Ignore,3),
        Err(ScheduleError::Calendar {cause:CalendarError::UnverifiedCoverage {date,..},..}) if date==civil(2025,12,31))
    );
}

#[test]
fn target_candidate_escape_and_easter_coupon_collision_retain_context() {
    let first = civil(2026, 1, 2);
    let b = ScheduleBuilder::new(
        civil(2026, 1, 1),
        civil(2026, 2, 2),
        ScheduleCalendar::Target(TargetCalendar::new()),
        Bdc::Preceding,
        Bdc::Unadjusted,
    )
    .unwrap();
    assert!(
        matches!(b.build_explicit(Period::new(1,TimeUnit::Months),Rule::Forward,
        ExplicitStubDates::new(Some(first),None).unwrap(),EndOfMonth::Ignore,3),
        Err(ScheduleError::Calendar {index:Some(0),phase:"adjustment",cause:CalendarError::UnverifiedCoverage {date,..},..}) if date==civil(2025,12,31))
    );
    let dates = ExplicitStubDates::new(Some(civil(2026, 4, 3)), Some(civil(2026, 4, 6))).unwrap();
    let b = ScheduleBuilder::new(
        civil(2026, 4, 1),
        civil(2026, 4, 10),
        ScheduleCalendar::Target(TargetCalendar::new()),
        Bdc::Following,
        Bdc::Unadjusted,
    )
    .unwrap();
    assert!(
        matches!(b.build_explicit(Period::new(3,TimeUnit::Days),Rule::Forward,dates,EndOfMonth::Ignore,4),
        Err(ScheduleError::AdjustedOrder {index:2,previous_contractual,contractual,previous_adjusted,adjusted,..})
        if previous_contractual==civil(2026,4,3) && contractual==civil(2026,4,6)
            && previous_adjusted==civil(2026,4,7) && adjusted==civil(2026,4,7))
    );
}

#[test]
fn adjusted_civil_range_failure_preserves_all_three_typed_source_layers() {
    let first = civil(1, 2, 1);
    let calendar = Calendar::custom([], [Date::MIN]);
    let b = ScheduleBuilder::new(
        Date::MIN,
        civil(1, 3, 1),
        ScheduleCalendar::Plain(calendar),
        Bdc::Preceding,
        Bdc::Unadjusted,
    )
    .unwrap();
    let error = b
        .build_explicit(
            Period::new(1, TimeUnit::Months),
            Rule::Forward,
            ExplicitStubDates::new(Some(first), None).unwrap(),
            EndOfMonth::Ignore,
            3,
        )
        .unwrap_err();
    assert!(matches!(
        &error,
        ScheduleError::Calendar {
            index: Some(0),
            cause: CalendarError::Date(DateError::OutOfRange { .. }),
            ..
        }
    ));
    let root = CrateError::from(error);
    assert!(root.source().unwrap().is::<ScheduleError>());
    assert!(
        root.source()
            .unwrap()
            .source()
            .unwrap()
            .is::<CalendarError>()
    );
    assert!(
        root.source()
            .unwrap()
            .source()
            .unwrap()
            .source()
            .unwrap()
            .is::<DateError>()
    );
}

#[test]
fn equal_date_vectors_keep_distinct_explicit_roles_and_stored_tenor_definitions() {
    fn traits<T: Clone + std::fmt::Debug + Eq + Hash + Send + Sync + 'static>() {}
    traits::<ExplicitStubDates>();
    traits::<Schedule>();
    traits::<ScheduleBuilder>();
    let b = plain(civil(2026, 1, 1), civil(2026, 3, 1)).unwrap();
    let mid = civil(2026, 2, 1);
    let schedules: Vec<_> = [(Some(mid), None), (None, Some(mid)), (Some(mid), Some(mid))]
        .into_iter()
        .map(|(first, pen)| {
            b.build_explicit(
                Period::new(1, TimeUnit::Months),
                Rule::Forward,
                ExplicitStubDates::new(first, pen).unwrap(),
                EndOfMonth::Ignore,
                3,
            )
            .unwrap()
        })
        .collect();
    for value in &schedules {
        assert_eq!(
            value.contractual_dates(),
            [civil(2026, 1, 1), mid, civil(2026, 3, 1)]
        );
        assert_eq!(
            value.regularity(),
            [Regularity::Regular, Regularity::Regular]
        );
        assert_eq!(hash(value), hash(&value.clone()));
    }
    assert_eq!(schedules.iter().cloned().collect::<HashSet<_>>().len(), 3);
    let b = plain(civil(2019, 1, 1), civil(2022, 1, 1)).unwrap();
    let explicit =
        ExplicitStubDates::new(Some(civil(2020, 1, 1)), Some(civil(2021, 1, 1))).unwrap();
    let years = b
        .build_explicit(
            Period::new(1, TimeUnit::Years),
            Rule::Forward,
            explicit,
            EndOfMonth::Ignore,
            4,
        )
        .unwrap();
    let months = b
        .build_explicit(
            Period::new(12, TimeUnit::Months),
            Rule::Forward,
            explicit,
            EndOfMonth::Ignore,
            4,
        )
        .unwrap();
    assert_eq!(years.contractual_dates(), months.contractual_dates());
    assert_ne!(years, months);
    let equal = b
        .build_explicit(
            Period::new(1, TimeUnit::Years),
            Rule::Forward,
            explicit,
            EndOfMonth::Ignore,
            100,
        )
        .unwrap();
    assert_eq!(equal, years);
    assert_eq!(hash(&equal), hash(&years));
    let automatic = b
        .build_recurrent(
            Period::new(1, TimeUnit::Years),
            Rule::Forward,
            StubPolicy::None,
            EndOfMonth::Ignore,
            4,
        )
        .unwrap();
    assert_eq!(automatic.explicit_stub_dates(), None);
    assert_eq!(automatic.lattice_anchor(), Some(civil(2019, 1, 1)));
    let zero = b.build_zero(2).unwrap();
    assert_eq!(zero.explicit_stub_dates(), None);
    assert_eq!(zero.lattice_anchor(), None);
}

#[test]
fn immutable_explicit_builders_rebuild_identically_across_threads() {
    let b = ScheduleBuilder::new(
        civil(2026, 1, 1),
        civil(2027, 1, 1),
        ScheduleCalendar::Plain(Calendar::custom(
            [Weekday::Saturday, Weekday::Sunday],
            [civil(2026, 7, 1)],
        )),
        Bdc::Following,
        Bdc::ModifiedFollowing,
    )
    .unwrap();
    let dates = ExplicitStubDates::new(Some(civil(2026, 2, 1)), Some(civil(2026, 11, 1))).unwrap();
    let baseline = b
        .build_explicit(
            Period::new(3, TimeUnit::Months),
            Rule::Forward,
            dates,
            EndOfMonth::Ignore,
            6,
        )
        .unwrap();
    std::thread::scope(|scope| {
        for _ in 0..8 {
            let builder = &b;
            let expected = &baseline;
            scope.spawn(move || {
                for _ in 0..32 {
                    let actual = builder
                        .build_explicit(
                            Period::new(3, TimeUnit::Months),
                            Rule::Forward,
                            dates,
                            EndOfMonth::Ignore,
                            6,
                        )
                        .unwrap();
                    assert_eq!(&actual, expected);
                    assert_eq!(hash(&actual), hash(expected));
                }
            });
        }
    });
}
