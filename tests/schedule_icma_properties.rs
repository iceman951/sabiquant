//! Contractual ICMA hulls checked by civil lattice membership and exact ratios.
//!
//! The independent oracle scans civil dates, not production recurrence calls.
//! Accrual expectations use reduced integer overlap fractions, not the library's
//! compensated floating-point summation or its generated fixture metadata.

use sabiquant::calendar::{
    BusinessDayConvention as Bdc, Calendar, CalendarError, EndOfMonth, JointRule, TargetCalendar,
    TargetPolicy,
};
use sabiquant::daycount::{ActualActualIcma, IcmaError};
use sabiquant::error::Error as CrateError;
use sabiquant::schedule::{
    DateGenerationRule as Rule, ExplicitStubDates, Schedule, ScheduleBuilder, ScheduleCalendar,
    ScheduleError, StubPolicy,
};
use sabiquant::time::{Date, Frequency, Period, TimeUnit, Weekday};
use std::collections::{HashSet, hash_map::DefaultHasher};
use std::error::Error;
use std::hash::{Hash, Hasher};

type Civil = (i32, u32, u32);

// FS-PER's declared contracts, independent of Frequency's production mapping.
const PERIODIC: [(Frequency, i32, TimeUnit, u32); 10] = [
    (Frequency::Annual, 1, TimeUnit::Years, 1),
    (Frequency::Semiannual, 6, TimeUnit::Months, 2),
    (Frequency::EveryFourthMonth, 4, TimeUnit::Months, 3),
    (Frequency::Quarterly, 3, TimeUnit::Months, 4),
    (Frequency::Bimonthly, 2, TimeUnit::Months, 6),
    (Frequency::Monthly, 1, TimeUnit::Months, 12),
    (Frequency::EveryFourthWeek, 4, TimeUnit::Weeks, 13),
    (Frequency::Biweekly, 2, TimeUnit::Weeks, 26),
    (Frequency::Weekly, 1, TimeUnit::Weeks, 52),
    (Frequency::Daily, 1, TimeUnit::Days, 365),
];

#[expect(
    clippy::expect_used,
    reason = "oracle constructs valid Gregorian dates"
)]
fn date((y, m, d): Civil) -> Date {
    Date::from_ymd(y, m, d).expect("valid independent civil date")
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

fn ordinal((y, m, d): Civil) -> i64 {
    let previous = i64::from(y - 1);
    365 * previous + previous / 4 - previous / 100
        + previous / 400
        + (1..m)
            .map(|month| i64::from(month_days(y, month)))
            .sum::<i64>()
        + i64::from(d - 1)
}

fn next((y, m, d): Civil) -> Civil {
    if d < month_days(y, m) {
        (y, m, d + 1)
    } else if m < 12 {
        (y, m + 1, 1)
    } else {
        (y + 1, 1, 1)
    }
}

fn previous((y, m, d): Civil) -> Civil {
    if d > 1 {
        (y, m, d - 1)
    } else if m > 1 {
        (y, m - 1, month_days(y, m - 1))
    } else {
        (y - 1, 12, 31)
    }
}

// Positive and negative index membership is an integer congruence. In
// particular, a clamped lattice neighbor is never made into another seed.
fn index(candidate: Civil, anchor: Civil, tenor: Period, eom: EndOfMonth) -> Option<i64> {
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
            let difference = ordinal(candidate) - ordinal(anchor);
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
            let expected_day = if eom == EndOfMonth::Preserve && ad == month_days(ay, am) {
                month_days(y, m)
            } else {
                ad.min(month_days(y, m))
            };
            (d == expected_day).then_some(difference / length)
        }
    }
}

fn hull(start: Civil, end: Civil, anchor: Civil, tenor: Period, eom: EndOfMonth) -> Vec<Date> {
    let mut lower = start;
    while index(lower, anchor, tenor, eom).is_none() {
        lower = previous(lower);
        assert!(ordinal(start) - ordinal(lower) <= 366);
    }
    let mut upper = end;
    while index(upper, anchor, tenor, eom).is_none() {
        upper = next(upper);
        assert!(ordinal(upper) - ordinal(end) <= 366);
    }
    let mut dates = Vec::new();
    let mut cursor = lower;
    loop {
        if index(cursor, anchor, tenor, eom).is_some() {
            dates.push(date(cursor));
        }
        if cursor == upper {
            break;
        }
        cursor = next(cursor);
    }
    assert!(dates.len() >= 2);
    assert!(dates.windows(2).all(|pair| {
        index(pair[1].to_ymd(), anchor, tenor, eom)
            .zip(index(pair[0].to_ymd(), anchor, tenor, eom))
            .is_some_and(|(right, left)| right - left == 1)
    }));
    dates
}

// This chooses input endpoints only. Complete expected reference vectors
// independently come from the membership scan above.
#[expect(
    clippy::expect_used,
    reason = "two-tenor inputs lie inside years 1999..2402"
)]
fn two_tenors(anchor: Civil, tenor: Period, eom: EndOfMonth) -> Civil {
    let (y, m, d) = anchor;
    match tenor.unit() {
        TimeUnit::Days | TimeUnit::Weeks => {
            let quantum = tenor.length()
                * if tenor.unit() == TimeUnit::Weeks {
                    7
                } else {
                    1
                };
            let mut cursor = anchor;
            for _ in 0..2 * quantum {
                cursor = next(cursor);
            }
            cursor
        }
        TimeUnit::Months => {
            let coordinate = i64::from(y) * 12 + i64::from(m - 1) + 2 * i64::from(tenor.length());
            let year = i32::try_from(coordinate / 12).expect("bounded year");
            let month = u32::try_from(coordinate % 12 + 1).expect("bounded month");
            (
                year,
                month,
                if eom == EndOfMonth::Preserve {
                    month_days(year, month)
                } else {
                    d.min(month_days(year, month))
                },
            )
        }
        TimeUnit::Years => {
            let year = y + 2 * tenor.length();
            (
                year,
                m,
                if eom == EndOfMonth::Preserve {
                    month_days(year, m)
                } else {
                    d.min(month_days(year, m))
                },
            )
        }
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

fn boundaries(context: &ActualActualIcma) -> Vec<Date> {
    let mut values = vec![context.coverage_start()];
    for period in context.periods() {
        assert_eq!(
            period.start(),
            *values.last().unwrap_or(&context.coverage_start())
        );
        assert!(period.start() < period.end());
        values.push(period.end());
    }
    assert_eq!(values.last().copied(), Some(context.coverage_end()));
    values
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

#[expect(
    clippy::expect_used,
    reason = "checked arithmetic makes oracle bounds explicit"
)]
fn rational(references: &[Date], start: Date, end: Date, count: u32) -> (i128, i128) {
    assert!(start <= end);
    let mut numerator = 0_i128;
    let mut denominator = 1_i128;
    for pair in references.windows(2) {
        let left = start.max(pair[0]);
        let right = end.min(pair[1]);
        if left >= right {
            continue;
        }
        let mut n = i128::from(ordinal(right.to_ymd()) - ordinal(left.to_ymd()));
        let mut d = i128::from(count)
            .checked_mul(i128::from(
                ordinal(pair[1].to_ymd()) - ordinal(pair[0].to_ymd()),
            ))
            .expect("small exact reference denominator");
        let reduction = gcd(n, d);
        n /= reduction;
        d /= reduction;
        let shared = gcd(denominator, d);
        numerator = numerator
            .checked_mul(d / shared)
            .and_then(|a| {
                n.checked_mul(denominator / shared)
                    .and_then(|b| a.checked_add(b))
            })
            .expect("bounded exact overlap sum");
        denominator = denominator
            .checked_mul(d / shared)
            .expect("bounded exact common denominator");
        let reduction = gcd(numerator, denominator);
        numerator /= reduction;
        denominator /= reduction;
    }
    (numerator, denominator)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "explicit 2^53 bounds guarantee both oracle integers are exact binary64 inputs"
)]
fn rounded((numerator, denominator): (i128, i128)) -> f64 {
    assert!((0..=1_i128 << 53).contains(&numerator));
    assert!((1..=1_i128 << 53).contains(&denominator));
    numerator as f64 / denominator as f64
}

fn check_fraction(
    context: &ActualActualIcma,
    references: &[Date],
    start: Date,
    end: Date,
    annual_count: u32,
) -> Result<u64, IcmaError> {
    let expected = rounded(rational(references, start, end, annual_count));
    let actual = context.year_fraction(start, end)?;
    let distance = actual.to_bits().abs_diff(expected.to_bits());
    assert!(
        distance <= 4,
        "{start}..{end}: {actual:?}, expected {expected:?}; {references:?}"
    );
    if start < end {
        assert_eq!(
            context.year_fraction(end, start)?.to_bits(),
            (-actual).to_bits()
        );
    }
    assert_eq!(
        i64::from(context.day_count(start, end)),
        ordinal(end.to_ymd()) - ordinal(start.to_ymd())
    );
    Ok(distance)
}

fn hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn gregorian_cycle_all_frequencies_directions_and_explicit_roles_match_civil_hulls_and_ratios()
-> Result<(), Box<dyn Error>> {
    let mut attempts = 0_u64;
    let mut success = 0_u64;
    let mut irregular = 0_u64;
    let mut periods = 0_u64;
    let mut fractions = 0_u64;
    let mut maximum_ulp = 0_u64;
    assert_eq!(PERIODIC.len() + 1, Frequency::ALL.len());
    for (frequency, length, unit, count) in PERIODIC {
        assert_eq!(frequency.period(), Some(Period::new(length, unit)));
        assert_eq!(frequency.events_per_year(), Some(count));
    }
    for year in 2000..2400 {
        for month in 1..=12 {
            let coupon = (year, month, month_days(year, month));
            for (frequency, length, unit, annual_count) in PERIODIC {
                let tenor = Period::new(length, unit);
                let policies: &[EndOfMonth] =
                    if matches!(tenor.unit(), TimeUnit::Months | TimeUnit::Years) {
                        &[EndOfMonth::Ignore, EndOfMonth::Preserve]
                    } else {
                        &[EndOfMonth::Ignore]
                    };
                for &eom in policies {
                    let second = two_tenors(coupon, tenor, eom);
                    for (start, end, first, penultimate) in [
                        (previous(coupon), second, Some(coupon), None),
                        (coupon, next(second), None, Some(second)),
                        (previous(coupon), next(second), Some(coupon), Some(second)),
                        (previous(coupon), next(coupon), Some(coupon), Some(coupon)),
                    ] {
                        for rule in [Rule::Forward, Rule::Backward] {
                            attempts += 1;
                            let anchor = if rule == Rule::Forward {
                                first.unwrap_or(start)
                            } else {
                                penultimate.unwrap_or(end)
                            };
                            let middle_endpoint = if rule == Rule::Forward {
                                penultimate.unwrap_or(end)
                            } else {
                                first.unwrap_or(start)
                            };
                            let b = plain(date(start), date(end))?;
                            let roles =
                                ExplicitStubDates::new(first.map(date), penultimate.map(date))?;
                            let built = b.build_explicit(tenor, rule, roles, eom, 8);
                            if index(middle_endpoint, anchor, tenor, eom).is_none() {
                                assert!(matches!(
                                    built,
                                    Err(ScheduleError::IrregularExplicitMiddle { .. })
                                ));
                                irregular += 1;
                                continue;
                            }
                            let schedule = built?;
                            let expected = hull(start, end, anchor, tenor, eom);
                            let count = u32::try_from(expected.len() - 1)?;
                            let context = schedule.build_icma_context(frequency, count)?;
                            assert_eq!(boundaries(&context), expected);
                            assert_eq!(context.frequency(), frequency);
                            assert_eq!(schedule.lattice_anchor(), Some(date(anchor)));
                            assert_eq!(
                                schedule.tenor().map(|p| (p.length(), p.unit())),
                                Some((tenor.length(), tenor.unit()))
                            );
                            assert!(matches!(schedule.build_icma_context(frequency,count-1),
                                Err(ScheduleError::IcmaPeriodLimitExceeded{maximum,required_at_least,..})
                                    if maximum==count-1 && required_at_least==count));
                            assert_eq!(context, schedule.build_icma_context(frequency, count + 1)?);
                            maximum_ulp = maximum_ulp.max(check_fraction(
                                &context,
                                &expected,
                                date(start),
                                date(end),
                                annual_count,
                            )?);
                            fractions += 1;
                            maximum_ulp = maximum_ulp.max(check_fraction(
                                &context,
                                &expected,
                                context.coverage_start(),
                                context.coverage_end(),
                                annual_count,
                            )?);
                            fractions += 1;
                            for pair in schedule.contractual_dates().windows(2) {
                                maximum_ulp = maximum_ulp.max(check_fraction(
                                    &context,
                                    &expected,
                                    pair[0],
                                    pair[1],
                                    annual_count,
                                )?);
                                fractions += 1;
                            }
                            for zero in [
                                context.coverage_start(),
                                date(start),
                                date(anchor),
                                date(end),
                                context.coverage_end(),
                            ] {
                                assert_eq!(
                                    context.year_fraction(zero, zero)?.to_bits(),
                                    0.0_f64.to_bits()
                                );
                            }
                            for period in context.periods() {
                                assert_eq!(
                                    context
                                        .year_fraction(period.start(), period.end())?
                                        .to_bits(),
                                    (1.0 / f64::from(annual_count)).to_bits()
                                );
                            }
                            success += 1;
                            periods += u64::from(count);
                        }
                    }
                }
            }
        }
    }
    assert_eq!(attempts, 614_400);
    assert_eq!(success + irregular, attempts);
    assert_eq!(success, 594_618);
    assert_eq!(irregular, 19_782);
    assert_eq!(periods, 1_777_260);
    assert_eq!(fractions, 2_966_496);
    assert!(maximum_ulp <= 4);
    Ok(())
}

#[test]
fn long_first_and_equal_role_examples_have_independent_reduced_rational_results()
-> Result<(), Box<dyn Error>> {
    let long = plain(date((2002, 8, 15)), date((2003, 7, 15)))?.build_recurrent(
        Period::new(6, TimeUnit::Months),
        Rule::Backward,
        StubPolicy::Long,
        EndOfMonth::Ignore,
        2,
    )?;
    let context = long.build_icma_context(Frequency::Semiannual, 2)?;
    let references = [
        date((2002, 7, 15)),
        date((2003, 1, 15)),
        date((2003, 7, 15)),
    ];
    assert_eq!(boundaries(&context), references);
    assert_eq!(
        rational(&references, date((2002, 8, 15)), date((2003, 7, 15)), 2),
        (337, 368)
    );
    check_fraction(
        &context,
        &references,
        date((2002, 8, 15)),
        date((2003, 7, 15)),
        2,
    )?;

    let shared = date((2024, 2, 1));
    for rule in [Rule::Forward, Rule::Backward] {
        let schedule = plain(date((2023, 12, 15)), date((2024, 3, 15)))?.build_explicit(
            Period::new(1, TimeUnit::Months),
            rule,
            ExplicitStubDates::new(Some(shared), Some(shared))?,
            EndOfMonth::Ignore,
            3,
        )?;
        let context = schedule.build_icma_context(Frequency::Monthly, 4)?;
        let references = [
            date((2023, 12, 1)),
            date((2024, 1, 1)),
            shared,
            date((2024, 3, 1)),
            date((2024, 4, 1)),
        ];
        assert_eq!(boundaries(&context), references);
        assert_eq!(
            rational(&references, date((2023, 12, 15)), date((2024, 3, 15)), 12),
            (1, 4)
        );
        assert_eq!(
            context
                .year_fraction(date((2023, 12, 15)), date((2024, 3, 15)))?
                .to_bits(),
            0.25_f64.to_bits()
        );
    }
    Ok(())
}

#[test]
fn eom_activation_stays_with_the_original_anchor_across_long_outer_spans()
-> Result<(), Box<dyn Error>> {
    for anchor in [(2024, 4, 29), (2024, 4, 30)] {
        for eom in [EndOfMonth::Ignore, EndOfMonth::Preserve] {
            let expected = hull(
                (2024, 2, 15),
                (2024, 7, 15),
                anchor,
                Period::new(1, TimeUnit::Months),
                eom,
            );
            for rule in [Rule::Forward, Rule::Backward] {
                let seed = date(anchor);
                let schedule = plain(date((2024, 2, 15)), date((2024, 7, 15)))?.build_explicit(
                    Period::new(1, TimeUnit::Months),
                    rule,
                    ExplicitStubDates::new(Some(seed), Some(seed))?,
                    eom,
                    3,
                )?;
                let context = schedule.build_icma_context(Frequency::Monthly, 6)?;
                assert_eq!(boundaries(&context), expected);
                check_fraction(
                    &context,
                    &expected,
                    date((2024, 2, 15)),
                    date((2024, 7, 15)),
                    12,
                )?;
            }
        }
    }
    Ok(())
}

#[test]
fn all_adjustment_pairs_and_distinct_calendar_definitions_keep_the_same_context()
-> Result<(), Box<dyn Error>> {
    let start = date((2026, 1, 15));
    let end = date((2026, 7, 15));
    let seed = date((2026, 4, 30));
    let roles = ExplicitStubDates::new(Some(seed), Some(seed))?;
    let mut contexts = HashSet::new();
    let mut schedules = HashSet::new();
    for convention in Bdc::ALL {
        for termination in Bdc::ALL {
            let schedule = ScheduleBuilder::new(
                start,
                end,
                ScheduleCalendar::Plain(Calendar::weekends_only()),
                convention,
                termination,
            )?
            .build_explicit(
                Period::new(1, TimeUnit::Months),
                Rule::Forward,
                roles,
                EndOfMonth::Preserve,
                3,
            )?;
            contexts.insert(schedule.build_icma_context(Frequency::Monthly, 7)?);
            schedules.insert(schedule);
        }
    }
    for calendar in [Calendar::null(), Calendar::custom(Weekday::ALL, [])] {
        let schedule = ScheduleBuilder::new(
            start,
            end,
            ScheduleCalendar::Plain(calendar),
            Bdc::Unadjusted,
            Bdc::Unadjusted,
        )?
        .build_explicit(
            Period::new(1, TimeUnit::Months),
            Rule::Forward,
            roles,
            EndOfMonth::Preserve,
            3,
        )?;
        contexts.insert(schedule.build_icma_context(Frequency::Monthly, 7)?);
        schedules.insert(schedule);
    }
    assert_eq!(schedules.len(), 51);
    assert_eq!(contexts.len(), 1);
    let context = contexts.into_iter().next().unwrap();
    let clone = context.clone();
    assert_eq!(context, clone);
    assert_eq!(hash(&context), hash(&clone));
    let different_divisor =
        ActualActualIcma::new(context.periods().to_vec(), Frequency::Quarterly)?;
    assert_ne!(context, different_divisor);
    assert_eq!(HashSet::from([context, different_divisor]).len(), 2);
    Ok(())
}

#[test]
fn target_overrides_and_null_joint_shortcuts_neither_change_references_nor_waive_guards()
-> Result<(), Box<dyn Error>> {
    let start = date((2026, 2, 15));
    let end = date((2026, 4, 15));
    let seed = date((2026, 3, 31));
    let mut contexts = HashSet::new();
    for policy in [TargetPolicy::VerifiedOnly, TargetPolicy::AllowProjection] {
        let base = TargetCalendar::with_policy(policy);
        for calendar in [
            base.clone(),
            base.with_holiday(date((2026, 2, 28))),
            base.without_holiday(date((2026, 2, 28))),
            base.joint_with(vec![Calendar::null()], JointRule::JoinBusinessDays)?,
            base.joint_with(
                vec![Calendar::custom(Weekday::ALL, [])],
                JointRule::JoinHolidays,
            )?,
        ] {
            let schedule = ScheduleBuilder::new(
                start,
                end,
                ScheduleCalendar::Target(calendar),
                Bdc::Unadjusted,
                Bdc::Unadjusted,
            )?
            .build_explicit(
                Period::new(1, TimeUnit::Months),
                Rule::Forward,
                ExplicitStubDates::new(Some(seed), Some(seed))?,
                EndOfMonth::Preserve,
                3,
            )?;
            contexts.insert(schedule.build_icma_context(Frequency::Monthly, 3)?);
        }
    }
    assert_eq!(contexts.len(), 1);
    assert_eq!(
        boundaries(contexts.iter().next().unwrap()),
        [
            date((2026, 1, 31)),
            date((2026, 2, 28)),
            seed,
            date((2026, 4, 30))
        ]
    );

    let q_upper = date((2029, 1, 29));
    for calendar in [
        TargetCalendar::new(),
        TargetCalendar::new()
            .without_holiday(q_upper)
            .without_holiday(date((2029, 1, 1)))
            .joint_with(vec![Calendar::null()], JointRule::JoinBusinessDays)?,
    ] {
        let seed = date((2028, 11, 29));
        let schedule = ScheduleBuilder::new(
            date((2028, 11, 1)),
            date((2028, 12, 31)),
            ScheduleCalendar::Target(calendar),
            Bdc::Unadjusted,
            Bdc::Unadjusted,
        )?
        .build_explicit(
            Period::new(1, TimeUnit::Months),
            Rule::Backward,
            ExplicitStubDates::new(Some(seed), Some(seed))?,
            EndOfMonth::Ignore,
            3,
        )?;
        assert!(matches!(
            schedule.build_icma_context(Frequency::Monthly, 2),
            Err(ScheduleError::IcmaPeriodLimitExceeded {
                maximum: 2,
                required_at_least: 3,
                ..
            })
        ));
        let error = schedule
            .build_icma_context(Frequency::Monthly, 3)
            .unwrap_err();
        assert!(
            matches!(&error,ScheduleError::Calendar{index:None,phase:"ICMA reference coverage",cause:CalendarError::UnverifiedCoverage{date:rejected,..},..} if *rejected==q_upper),
            "{error:?}"
        );
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<CalendarError>()
                .is_some()
        );
        let root = CrateError::from(error.clone());
        assert_eq!(root.kind(), "schedule.calendar");
        assert_eq!(
            root.source().unwrap().downcast_ref::<ScheduleError>(),
            Some(&error)
        );
    }

    let outside = date((1998, 12, 13));
    let target = TargetCalendar::with_policy(TargetPolicy::AllowProjection)
        .without_holiday(outside)
        .joint_with(vec![Calendar::null()], JointRule::JoinBusinessDays)?;
    let seed = date((1999, 1, 10));
    let schedule = ScheduleBuilder::new(
        date((1999, 1, 4)),
        date((1999, 1, 20)),
        ScheduleCalendar::Target(target),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )?
    .build_explicit(
        Period::new(28, TimeUnit::Days),
        Rule::Forward,
        ExplicitStubDates::new(Some(seed), Some(seed))?,
        EndOfMonth::Ignore,
        3,
    )?;
    assert!(
        matches!(schedule.build_icma_context(Frequency::EveryFourthWeek,2),
        Err(ScheduleError::Calendar{index:None,phase:"ICMA reference coverage",cause:CalendarError::OutsideValidity{date:rejected,..},..}) if rejected==outside)
    );
    Ok(())
}

#[test]
fn immutable_contexts_and_schedule_definitions_are_reusable_across_threads()
-> Result<(), Box<dyn Error>> {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<ActualActualIcma>();
    send_sync::<Schedule>();
    let mut cases = Vec::new();
    for frequency in Frequency::ALL {
        let Some(tenor) = frequency.period() else {
            continue;
        };
        let seed = date((2024, 2, 29));
        let eom = if matches!(tenor.unit(), TimeUnit::Months | TimeUnit::Years) {
            EndOfMonth::Preserve
        } else {
            EndOfMonth::Ignore
        };
        let schedule = plain(date((2023, 12, 15)), date((2025, 4, 15)))?.build_explicit(
            tenor,
            Rule::Backward,
            ExplicitStubDates::new(Some(seed), Some(seed))?,
            eom,
            3,
        )?;
        let context = schedule.build_icma_context(frequency, 600)?;
        cases.push((schedule, frequency, context));
    }
    std::thread::scope(|scope| {
        for _ in 0..8 {
            let cases = &cases;
            scope.spawn(move || {
                for _ in 0..16 {
                    for (schedule, frequency, baseline) in cases {
                        let rebuilt = schedule.build_icma_context(*frequency, 600).unwrap();
                        assert_eq!(&rebuilt, baseline);
                        assert_eq!(hash(&rebuilt), hash(baseline));
                        assert_eq!(
                            rebuilt
                                .year_fraction(rebuilt.coverage_start(), rebuilt.coverage_end())
                                .unwrap()
                                .to_bits(),
                            baseline
                                .year_fraction(baseline.coverage_start(), baseline.coverage_end())
                                .unwrap()
                                .to_bits()
                        );
                        assert_eq!(schedule.generation_rule(), Rule::Backward);
                    }
                }
            });
        }
    });
    Ok(())
}
