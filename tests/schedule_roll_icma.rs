//! Independent civil-roll ICMA hulls, required extensions and coupon controls.

use sabiquant::calendar::{
    BusinessDayConvention as Bdc, Calendar, CalendarError, JointRule, TargetCalendar, TargetPolicy,
};
use sabiquant::cashflows::{CashFlow, FixedCouponRate, Leg, PaymentDateInclusion};
use sabiquant::daycount::{ActualActualIcma, DayCount, IcmaError};
use sabiquant::error::Error as RootError;
use sabiquant::rates::{AccrualConvention, Compounding, InterestRate};
use sabiquant::schedule::{
    PeriodRegularity, Schedule, ScheduleBuilder, ScheduleCalendar, ScheduleError,
    SpecialRollRule as Roll,
};
use sabiquant::time::{Date, DateError, Frequency, Period, TimeUnit, Weekday};
use std::collections::{HashSet, hash_map::DefaultHasher};
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[expect(clippy::expect_used, reason = "hand-checked valid literal civil dates")]
fn d(text: &str) -> Date {
    text.parse().expect("valid literal date")
}

fn months(length: i32) -> Period {
    Period::new(length, TimeUnit::Months)
}

fn builder(
    start: Date,
    end: Date,
    calendar: ScheduleCalendar,
) -> Result<ScheduleBuilder, ScheduleError> {
    ScheduleBuilder::new(start, end, calendar, Bdc::Unadjusted, Bdc::Unadjusted)
}

fn plain(start: Date, end: Date) -> Result<ScheduleBuilder, ScheduleError> {
    builder(start, end, ScheduleCalendar::Plain(Calendar::null()))
}

fn boundaries(context: &ActualActualIcma) -> Vec<Date> {
    let mut dates = vec![context.coverage_start()];
    for period in context.periods() {
        assert_eq!(dates.last(), Some(&period.start()));
        assert!(period.start() < period.end());
        dates.push(period.end());
    }
    assert_eq!(dates.last(), Some(&context.coverage_end()));
    dates
}

fn hash<T: Hash>(value: &T) -> u64 {
    let mut state = DefaultHasher::new();
    value.hash(&mut state);
    state.finish()
}

// These controls are positive, ordinary normal numbers. Comparing their bits
// therefore gives the exact representable-number distance, without an epsilon.
fn within_four_ulp(actual: f64, expected: f64) {
    assert!(actual.is_finite() && expected.is_finite());
    assert_eq!(actual.is_sign_negative(), expected.is_sign_negative());
    assert!(
        actual.to_bits().abs_diff(expected.to_bits()) <= 4,
        "{actual:.17e} versus {expected:.17e}"
    );
}

fn quarter_hull(rule: Roll) -> Vec<Date> {
    let text: &[&str] = match rule {
        Roll::ThirdWednesday => &[
            "2025-11-19",
            "2026-02-18",
            "2026-05-20",
            "2026-08-19",
            "2026-11-18",
        ],
        Roll::ThirdWednesdayInclusive => &["2026-02-18", "2026-05-20", "2026-08-19", "2026-11-18"],
        Roll::Twentieth => &[
            "2025-11-20",
            "2026-02-20",
            "2026-05-20",
            "2026-08-20",
            "2026-11-20",
        ],
        Roll::TwentiethIMM => &[
            "2025-12-20",
            "2026-03-20",
            "2026-06-20",
            "2026-09-20",
            "2026-12-20",
        ],
    };
    text.iter().map(|text| d(text)).collect()
}

#[test]
fn same_month_wednesday_before_after_crossing_and_exact_endpoints_use_minimal_hulls()
-> Result<(), Box<dyn Error>> {
    // December 20 -> January 17 is 28 days; January 17 -> February 21
    // is 35 days. Fractions below are exact integer overlap / (12 * days).
    for (start, end, expected, numerator, denominator) in [
        (
            "2024-01-01",
            "2024-01-02",
            vec!["2023-12-20", "2024-01-17"],
            1.0,
            336.0,
        ),
        (
            "2024-01-25",
            "2024-01-26",
            vec!["2024-01-17", "2024-02-21"],
            1.0,
            420.0,
        ),
        (
            "2024-01-16",
            "2024-01-18",
            vec!["2023-12-20", "2024-01-17", "2024-02-21"],
            3.0,
            560.0,
        ),
        (
            "2024-01-16",
            "2024-01-17",
            vec!["2023-12-20", "2024-01-17"],
            1.0,
            336.0,
        ),
        (
            "2024-01-17",
            "2024-01-18",
            vec!["2024-01-17", "2024-02-21"],
            1.0,
            420.0,
        ),
        (
            "2024-01-17",
            "2024-02-21",
            vec!["2024-01-17", "2024-02-21"],
            1.0,
            12.0,
        ),
    ] {
        let (start, end) = (d(start), d(end));
        let schedule = plain(start, end)?.build_roll(months(1), Roll::ThirdWednesday, 3)?;
        let count = u32::try_from(expected.len() - 1)?;
        let context = schedule.build_icma_context(Frequency::Monthly, count)?;
        assert_eq!(
            boundaries(&context),
            expected.into_iter().map(d).collect::<Vec<_>>()
        );
        within_four_ulp(context.year_fraction(start, end)?, numerator / denominator);
        within_four_ulp(context.year_fraction(end, start)?, -numerator / denominator);
        for day in [start, end, context.coverage_start(), context.coverage_end()] {
            assert_eq!(context.year_fraction(day, day)?.to_bits(), 0);
        }
        assert_eq!(
            context,
            schedule.build_icma_context(Frequency::Monthly, count + 1)?
        );
        if count > 1 {
            assert!(
                matches!(schedule.build_icma_context(Frequency::Monthly,count-1),
                Err(ScheduleError::IcmaPeriodLimitExceeded { maximum, required_at_least, .. })
                if maximum == count-1 && required_at_least == count)
            );
        }
        let outside = context.coverage_start().checked_add_days(-1)?;
        assert!(matches!(
            context.year_fraction(outside, outside),
            Err(IcmaError::OutOfCoverage { .. })
        ));
    }
    Ok(())
}

#[test]
fn every_rule_uses_literal_same_roll_quasi_dates_and_exact_fraction_controls()
-> Result<(), Box<dyn Error>> {
    // Python Fraction on the listed civil boundaries gives 71/91, 3/4,
    // 145/184 and 13/15. This is separate from production floating summation.
    for (rule, numerator, denominator) in [
        (Roll::ThirdWednesday, 71.0, 91.0),
        (Roll::ThirdWednesdayInclusive, 3.0, 4.0),
        (Roll::Twentieth, 145.0, 184.0),
        (Roll::TwentiethIMM, 13.0, 15.0),
    ] {
        let schedule = plain(d("2026-02-06"), d("2026-11-17"))?.build_roll(months(3), rule, 5)?;
        let expected = quarter_hull(rule);
        let count = u32::try_from(expected.len() - 1)?;
        let context = schedule.build_icma_context(Frequency::Quarterly, count)?;
        assert_eq!(boundaries(&context), expected);
        let raw = schedule.contractual_dates();
        let (start, end) = (raw[0], raw[raw.len() - 1]);
        within_four_ulp(context.year_fraction(start, end)?, numerator / denominator);
        within_four_ulp(context.year_fraction(end, start)?, -numerator / denominator);
        assert_eq!(context.frequency(), Frequency::Quarterly);
        assert_eq!(
            context.year_fraction(context.coverage_start(), context.coverage_end())?,
            f64::from(count) / 4.0
        );
        for cap in 1..count {
            assert!(
                matches!(schedule.build_icma_context(Frequency::Quarterly,cap),
                Err(ScheduleError::IcmaPeriodLimitExceeded { maximum, required_at_least, .. })
                if maximum == cap && required_at_least == cap + 1)
            );
        }
    }
    Ok(())
}

#[test]
fn all_eligible_named_frequencies_and_stored_annual_aliases_retain_their_definitions()
-> Result<(), Box<dyn Error>> {
    let mut cases = 0;
    for rule in Roll::ALL {
        for frequency in Frequency::ALL {
            let Some(tenor) = frequency.period() else {
                continue;
            };
            let stride = match tenor.unit() {
                TimeUnit::Months => tenor.length(),
                TimeUnit::Years => 12 * tenor.length(),
                TimeUnit::Days | TimeUnit::Weeks => continue,
            };
            if rule == Roll::TwentiethIMM && stride % 3 != 0 {
                continue;
            }
            let b = plain(d("2024-02-06"), d("2027-11-17"))?;
            let schedule = b.build_roll(tenor, rule, 50)?;
            let context = schedule.build_icma_context(frequency, 50)?;
            assert_eq!(context.frequency(), frequency);
            let mut regular_count = 0;
            for (pair, regularity) in schedule
                .contractual_dates()
                .windows(2)
                .zip(schedule.regularity())
            {
                if *regularity == PeriodRegularity::Regular {
                    assert!(
                        context
                            .periods()
                            .iter()
                            .any(|period| period.start() == pair[0] && period.end() == pair[1])
                    );
                    assert_eq!(
                        context.year_fraction(pair[0], pair[1])?,
                        1.0 / f64::from(frequency.events_per_year().ok_or("periodic event count")?)
                    );
                    regular_count += 1;
                }
            }
            assert!(regular_count > 0);
            let count = u32::try_from(context.periods().len())?;
            assert_eq!(context, schedule.build_icma_context(frequency, count)?);
            assert_eq!(context, schedule.build_icma_context(frequency, count + 1)?);
            for other in Frequency::ALL {
                if other == frequency {
                    continue;
                }
                let error = schedule.build_icma_context(other, 1).unwrap_err();
                assert_eq!(
                    error.kind(),
                    if other == Frequency::Once {
                        "schedule.invalid_icma_frequency"
                    } else {
                        "schedule.icma_frequency_mismatch"
                    }
                );
                assert!(error.source().is_none());
            }
            if frequency == Frequency::Annual {
                let alias = b.build_roll(months(12), rule, 50)?;
                assert_ne!(schedule, alias);
                let original = schedule
                    .roll_provenance()
                    .ok_or("missing provenance")?
                    .tenor();
                let aliased = alias
                    .roll_provenance()
                    .ok_or("missing alias provenance")?
                    .tenor();
                assert_eq!((original.length(), original.unit()), (1, TimeUnit::Years));
                assert_eq!((aliased.length(), aliased.unit()), (12, TimeUnit::Months));
                assert_eq!(context, alias.build_icma_context(frequency, count)?);
                assert_eq!(
                    hash(&context),
                    hash(&alias.build_icma_context(frequency, count)?)
                );
            }
            cases += 1;
        }
    }
    assert_eq!(cases, 21);
    Ok(())
}

#[test]
fn cap_once_and_mismatched_frequency_precede_required_civil_arithmetic()
-> Result<(), Box<dyn Error>> {
    for rule in Roll::ALL {
        let schedule = plain(d("0001-01-01"), d("0001-02-02"))?.build_roll(
            Period::new(i32::MAX, TimeUnit::Years),
            rule,
            3,
        )?;
        for cap in [0, Schedule::MAX_ICMA_PERIODS + 1, u32::MAX] {
            for frequency in Frequency::ALL {
                assert!(matches!(schedule.build_icma_context(frequency,cap),
                    Err(ScheduleError::InvalidIcmaPeriodLimit { requested, minimum: 1, maximum: 3_652_058, .. }) if requested == cap));
            }
        }
        for frequency in Frequency::ALL {
            let error = schedule.build_icma_context(frequency, 1).unwrap_err();
            if frequency == Frequency::Once {
                assert!(matches!(
                    error,
                    ScheduleError::InvalidIcmaFrequency {
                        frequency: Frequency::Once,
                        ..
                    }
                ));
            } else {
                assert!(
                    matches!(error, ScheduleError::IcmaFrequencyMismatch { tenor, frequency: actual, expected_tenor, .. }
                    if tenor.length() == i32::MAX && tenor.unit() == TimeUnit::Years && actual == frequency && Some(expected_tenor) == frequency.period())
                );
            }
        }
    }
    Ok(())
}

#[test]
fn required_predecessors_and_successors_keep_exact_anchor_offset_index_and_source_chain()
-> Result<(), Box<dyn Error>> {
    for (rule, stride, frequency, start, end, anchor, offset, index, count) in [
        (
            Roll::ThirdWednesday,
            1,
            Frequency::Monthly,
            "0001-01-01",
            "0001-01-02",
            "0001-01-17",
            -1,
            1,
            1,
        ),
        (
            Roll::ThirdWednesday,
            1,
            Frequency::Monthly,
            "0001-01-16",
            "0001-01-18",
            "0001-01-17",
            -1,
            1,
            2,
        ),
        (
            Roll::ThirdWednesday,
            1,
            Frequency::Monthly,
            "9999-12-16",
            "9999-12-17",
            "9999-12-15",
            1,
            1,
            1,
        ),
        (
            Roll::ThirdWednesday,
            1,
            Frequency::Monthly,
            "9999-12-14",
            "9999-12-16",
            "9999-12-15",
            1,
            2,
            2,
        ),
        (
            Roll::ThirdWednesdayInclusive,
            12,
            Frequency::Annual,
            "9999-11-01",
            "9999-12-30",
            "9999-11-17",
            12,
            1,
            1,
        ),
        (
            Roll::Twentieth,
            1,
            Frequency::Monthly,
            "0001-01-01",
            "0001-01-02",
            "0001-01-20",
            -1,
            1,
            1,
        ),
        (
            Roll::TwentiethIMM,
            3,
            Frequency::Quarterly,
            "0001-01-01",
            "0001-01-02",
            "0001-03-20",
            -3,
            1,
            1,
        ),
        (
            Roll::Twentieth,
            6,
            Frequency::Semiannual,
            "9999-09-01",
            "9999-12-20",
            "9999-09-20",
            6,
            2,
            2,
        ),
        (
            Roll::TwentiethIMM,
            6,
            Frequency::Semiannual,
            "9999-09-01",
            "9999-12-20",
            "9999-09-20",
            6,
            2,
            2,
        ),
    ] {
        let schedule = plain(d(start), d(end))?.build_roll(months(stride), rule, 3)?;
        if count > 1 {
            assert!(matches!(
                schedule.build_icma_context(frequency, count - 1),
                Err(ScheduleError::IcmaPeriodLimitExceeded {
                    maximum: 1,
                    required_at_least: 2,
                    ..
                })
            ));
        }
        let error = schedule.build_icma_context(frequency, count).unwrap_err();
        assert!(
            matches!(&error, ScheduleError::Date { index: actual_index, phase: "ICMA reference extension",
            cause: DateError::OutOfRange { date, operation: "add months", count: actual_offset, .. }, .. }
            if *date == d(anchor) && *actual_offset == offset && *actual_index == index),
            "{error:?}"
        );
        let cause = error.source().ok_or("missing Date cause")?;
        assert!(cause.downcast_ref::<DateError>().is_some());
        assert_eq!(error.kind(), "schedule.date");
        let root = RootError::from(error.clone());
        assert_eq!(
            root.source()
                .and_then(|source| source.downcast_ref::<ScheduleError>()),
            Some(&error)
        );
        assert_eq!(root.kind(), "schedule.date");
        assert!(error.to_string().contains("ICMA reference extension"));
    }
    Ok(())
}

#[test]
fn exact_civil_extreme_roll_endpoints_need_no_unused_predecessor_or_successor()
-> Result<(), Box<dyn Error>> {
    for (rule, stride, frequency, start, end) in [
        (
            Roll::ThirdWednesday,
            1,
            Frequency::Monthly,
            "0001-01-17",
            "0001-02-21",
        ),
        (
            Roll::ThirdWednesdayInclusive,
            1,
            Frequency::Monthly,
            "0001-01-17",
            "0001-02-21",
        ),
        (
            Roll::ThirdWednesday,
            1,
            Frequency::Monthly,
            "9999-11-17",
            "9999-12-15",
        ),
        (
            Roll::ThirdWednesdayInclusive,
            1,
            Frequency::Monthly,
            "9999-11-17",
            "9999-12-15",
        ),
        (
            Roll::Twentieth,
            1,
            Frequency::Monthly,
            "0001-01-20",
            "0001-02-20",
        ),
        (
            Roll::Twentieth,
            1,
            Frequency::Monthly,
            "9999-11-20",
            "9999-12-20",
        ),
        (
            Roll::TwentiethIMM,
            3,
            Frequency::Quarterly,
            "0001-03-20",
            "0001-06-20",
        ),
        (
            Roll::TwentiethIMM,
            3,
            Frequency::Quarterly,
            "9999-09-20",
            "9999-12-20",
        ),
    ] {
        let (start, end) = (d(start), d(end));
        let schedule = plain(start, end)?.build_roll(months(stride), rule, 2)?;
        let context = schedule.build_icma_context(frequency, 1)?;
        assert_eq!(boundaries(&context), [start, end]);
        assert_eq!(context.year_fraction(start, end)?, f64::from(stride) / 12.0);
    }
    Ok(())
}

#[test]
fn actual_whole_civil_roll_hulls_keep_every_period_and_stop_at_exact_endpoints()
-> Result<(), Box<dyn Error>> {
    for (rule, start, end, stride, frequency, count) in [
        (
            Roll::ThirdWednesday,
            "0001-01-17",
            "9999-12-15",
            1,
            Frequency::Monthly,
            119_987_u32,
        ),
        (
            Roll::ThirdWednesdayInclusive,
            "0001-01-17",
            "9999-12-15",
            1,
            Frequency::Monthly,
            119_987,
        ),
        (
            Roll::Twentieth,
            "0001-01-20",
            "9999-12-20",
            1,
            Frequency::Monthly,
            119_987,
        ),
        (
            Roll::TwentiethIMM,
            "0001-03-20",
            "9999-12-20",
            3,
            Frequency::Quarterly,
            39_995,
        ),
    ] {
        let (start, end) = (d(start), d(end));
        let schedule = plain(start, end)?.build_roll(months(stride), rule, count + 1)?;
        let context = schedule.build_icma_context(frequency, count)?;
        let dates = boundaries(&context);
        assert_eq!(context.periods().len(), usize::try_from(count)?);
        assert_eq!(dates, schedule.contractual_dates());
        assert_eq!(
            (context.coverage_start(), context.coverage_end()),
            (start, end)
        );
        assert!(
            schedule
                .regularity()
                .iter()
                .all(|r| *r == PeriodRegularity::Regular)
        );
        for (index, date) in dates.iter().enumerate() {
            let coordinate = 12 * i64::from(date.year() - 1) + i64::from(date.month() - 1);
            let first_coordinate = i64::from(start.month() - 1);
            assert_eq!(
                coordinate,
                first_coordinate + i64::try_from(index)? * i64::from(stride)
            );
            match rule {
                Roll::ThirdWednesday | Roll::ThirdWednesdayInclusive => {
                    assert!((15..=21).contains(&date.day()));
                    // Independent Gregorian ordinal: civil 0001-01-01 is Monday.
                    let prior = i64::from(date.year() - 1);
                    let offsets = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
                    let leap =
                        date.year() % 4 == 0 && (date.year() % 100 != 0 || date.year() % 400 == 0);
                    let ordinal = 365 * prior + prior / 4 - prior / 100
                        + prior / 400
                        + offsets[usize::try_from(date.month() - 1)?]
                        + i64::from(date.month() > 2 && leap)
                        + i64::from(date.day() - 1);
                    assert_eq!(ordinal % 7, 2);
                }
                Roll::Twentieth => assert_eq!(date.day(), 20),
                Roll::TwentiethIMM => {
                    assert_eq!(date.day(), 20);
                    assert!(date.month().is_multiple_of(3));
                }
            }
        }
        assert!(matches!(
            schedule.build_icma_context(frequency, count - 1),
            Err(ScheduleError::IcmaPeriodLimitExceeded { maximum, required_at_least, .. })
                if maximum == count - 1 && required_at_least == count
        ));
        within_four_ulp(
            context.year_fraction(start, end)?,
            f64::from(count) / f64::from(frequency.events_per_year().ok_or("periodic frequency")?),
        );
    }
    Ok(())
}

#[test]
fn stored_year_aliases_keep_roll_month_arithmetic_in_required_date_causes()
-> Result<(), Box<dyn Error>> {
    for (rule, start, end, anchor, offset, index, cap) in [
        (
            Roll::ThirdWednesday,
            "0001-01-01",
            "0001-01-02",
            "0001-01-17",
            -12,
            1,
            1,
        ),
        (
            Roll::ThirdWednesdayInclusive,
            "9999-11-01",
            "9999-12-30",
            "9999-11-17",
            12,
            1,
            1,
        ),
        (
            Roll::Twentieth,
            "0001-01-01",
            "0001-01-02",
            "0001-01-20",
            -12,
            1,
            1,
        ),
        (
            Roll::TwentiethIMM,
            "0001-01-01",
            "0001-01-02",
            "0001-03-20",
            -12,
            1,
            1,
        ),
        (
            Roll::Twentieth,
            "9999-09-01",
            "9999-12-20",
            "9999-09-20",
            12,
            2,
            2,
        ),
        (
            Roll::TwentiethIMM,
            "9999-09-01",
            "9999-12-20",
            "9999-09-20",
            12,
            2,
            2,
        ),
    ] {
        let b = plain(d(start), d(end))?;
        let year = b.build_roll(Period::new(1, TimeUnit::Years), rule, 3)?;
        let month = b.build_roll(months(12), rule, 3)?;
        assert_ne!(year, month);
        let error = year.build_icma_context(Frequency::Annual, cap).unwrap_err();
        assert_eq!(
            error,
            month
                .build_icma_context(Frequency::Annual, cap)
                .unwrap_err()
        );
        assert!(
            matches!(error, ScheduleError::Date { index: actual_index, phase: "ICMA reference extension",
            cause: DateError::OutOfRange { date, operation: "add months", count: actual_offset, .. }, .. }
            if date == d(anchor) && actual_offset == offset && actual_index == index)
        );
    }
    Ok(())
}

#[test]
fn resolved_inclusive_endpoints_and_off_stride_twentieth_endpoints_define_coverage()
-> Result<(), Box<dyn Error>> {
    let inclusive = plain(d("2024-01-01"), d("2024-02-28"))?.build_roll(
        months(1),
        Roll::ThirdWednesdayInclusive,
        2,
    )?;
    let context = inclusive.build_icma_context(Frequency::Monthly, 1)?;
    assert_eq!(boundaries(&context), [d("2024-01-17"), d("2024-02-21")]);
    assert!(matches!(
        context.year_fraction(d("2024-01-01"), d("2024-02-28")),
        Err(IcmaError::OutOfCoverage { .. })
    ));
    let equivalent = plain(d("2024-01-17"), d("2024-02-21"))?.build_roll(
        months(1),
        Roll::ThirdWednesdayInclusive,
        2,
    )?;
    assert_ne!(inclusive, equivalent);
    assert_eq!(
        context,
        equivalent.build_icma_context(Frequency::Monthly, 1)?
    );
    assert_eq!(
        hash(&context),
        hash(&equivalent.build_icma_context(Frequency::Monthly, 1)?)
    );

    for (rule, start, end, stride, frequency, expected, numerator, denominator) in [
        (
            Roll::Twentieth,
            "2024-01-21",
            "2024-06-05",
            3,
            Frequency::Quarterly,
            vec!["2023-11-20", "2024-02-20", "2024-05-20", "2024-08-20"],
            153.0,
            368.0,
        ),
        (
            Roll::TwentiethIMM,
            "2024-03-21",
            "2024-09-05",
            6,
            Frequency::Semiannual,
            vec!["2023-12-20", "2024-06-20", "2024-12-20"],
            183.0,
            366.0,
        ),
    ] {
        let schedule = plain(d(start), d(end))?.build_roll(months(stride), rule, 4)?;
        let expected: Vec<_> = expected.into_iter().map(d).collect();
        let count = u32::try_from(expected.len() - 1)?;
        let context = schedule.build_icma_context(frequency, count)?;
        assert_eq!(boundaries(&context), expected);
        let raw = schedule.contractual_dates();
        within_four_ulp(
            context.year_fraction(raw[0], raw[raw.len() - 1])?,
            numerator / denominator,
        );
        assert!(!boundaries(&context).contains(&raw[raw.len() - 1]));
    }
    Ok(())
}

#[test]
fn strict_target_extension_guards_survive_overrides_and_both_joint_rules()
-> Result<(), Box<dyn Error>> {
    for (rule, start, end, stride, frequency, lower, upper, rejected, count) in [
        (
            Roll::ThirdWednesday,
            "2026-01-02",
            "2026-02-10",
            1,
            Frequency::Monthly,
            "2025-12-17",
            "2026-02-18",
            "2025-12-17",
            2,
        ),
        (
            Roll::Twentieth,
            "2026-01-02",
            "2026-02-10",
            1,
            Frequency::Monthly,
            "2025-12-20",
            "2026-02-20",
            "2025-12-20",
            2,
        ),
        (
            Roll::TwentiethIMM,
            "2026-01-02",
            "2026-02-10",
            3,
            Frequency::Quarterly,
            "2025-12-20",
            "2026-03-20",
            "2025-12-20",
            1,
        ),
        (
            Roll::ThirdWednesdayInclusive,
            "2028-11-21",
            "2028-12-30",
            3,
            Frequency::Quarterly,
            "2028-11-15",
            "2029-02-21",
            "2029-02-21",
            1,
        ),
        (
            Roll::Twentieth,
            "2028-11-01",
            "2028-12-20",
            3,
            Frequency::Quarterly,
            "2028-08-20",
            "2029-02-20",
            "2029-02-20",
            2,
        ),
        (
            Roll::TwentiethIMM,
            "2028-09-01",
            "2028-12-20",
            6,
            Frequency::Semiannual,
            "2028-03-20",
            "2029-03-20",
            "2029-03-20",
            2,
        ),
        (
            Roll::ThirdWednesday,
            "2002-12-20",
            "2002-12-31",
            1,
            Frequency::Monthly,
            "2002-12-18",
            "2003-01-15",
            "2003-01-15",
            1,
        ),
    ] {
        for policy in [TargetPolicy::VerifiedOnly, TargetPolicy::AllowProjection] {
            let target = TargetCalendar::with_policy(policy);
            let removed = target.without_holiday(d(rejected));
            let calendars = [
                target,
                removed.clone(),
                removed.joint_with(vec![Calendar::null()], JointRule::JoinBusinessDays)?,
                removed.joint_with(vec![Calendar::null()], JointRule::JoinHolidays)?,
            ];
            for calendar in calendars {
                let schedule = builder(d(start), d(end), ScheduleCalendar::Target(calendar))?
                    .build_roll(months(stride), rule, 5)?;
                if count > 1 {
                    assert!(matches!(
                        schedule.build_icma_context(frequency, count - 1),
                        Err(ScheduleError::IcmaPeriodLimitExceeded {
                            maximum: 1,
                            required_at_least: 2,
                            ..
                        })
                    ));
                }
                let result = schedule.build_icma_context(frequency, count);
                if policy == TargetPolicy::VerifiedOnly {
                    let error = result.unwrap_err();
                    assert!(
                        matches!(&error,ScheduleError::Calendar { index: None, phase: "ICMA reference coverage",
                        cause: CalendarError::UnverifiedCoverage { date, .. }, .. } if *date == d(rejected)),
                        "{error:?}"
                    );
                    assert!(
                        error
                            .source()
                            .and_then(|source| source.downcast_ref::<CalendarError>())
                            .is_some()
                    );
                } else {
                    let context = result?;
                    assert_eq!(
                        (context.coverage_start(), context.coverage_end()),
                        (d(lower), d(upper))
                    );
                    assert_eq!(context.periods().len(), count as usize);
                }
            }
        }
    }
    Ok(())
}

#[test]
fn projected_target_still_rejects_required_pre_inception_hulls() -> Result<(), Box<dyn Error>> {
    for (rule, stride, frequency, rejected) in [
        (Roll::ThirdWednesday, 1, Frequency::Monthly, "1998-12-16"),
        (Roll::Twentieth, 1, Frequency::Monthly, "1998-12-20"),
        (Roll::TwentiethIMM, 3, Frequency::Quarterly, "1998-12-20"),
    ] {
        for policy in [TargetPolicy::VerifiedOnly, TargetPolicy::AllowProjection] {
            let target = TargetCalendar::with_policy(policy).without_holiday(d(rejected));
            for calendar in [
                target.clone(),
                target.joint_with(vec![Calendar::null()], JointRule::JoinBusinessDays)?,
            ] {
                let schedule = builder(
                    d("1999-01-04"),
                    d("1999-01-19"),
                    ScheduleCalendar::Target(calendar),
                )?
                .build_roll(months(stride), rule, 3)?;
                let error = schedule.build_icma_context(frequency, 1).unwrap_err();
                assert!(
                    matches!(&error, ScheduleError::Calendar { index: None, phase: "ICMA reference coverage",
                    cause: CalendarError::OutsideValidity { date, .. }, .. } if *date == d(rejected)),
                    "{error:?}"
                );
            }
        }
    }
    // Projection changes the actual accrual endpoints here, so the original
    // January 4 request is not a reason to invent a December reference.
    let schedule = builder(
        d("1999-01-04"),
        d("1999-02-01"),
        ScheduleCalendar::Target(TargetCalendar::new()),
    )?
    .build_roll(months(1), Roll::ThirdWednesdayInclusive, 2)?;
    assert_eq!(
        boundaries(&schedule.build_icma_context(Frequency::Monthly, 1)?),
        [d("1999-01-20"), d("1999-02-17")]
    );
    Ok(())
}

#[test]
fn all_ordinary_and_terminal_adjustment_pairs_share_identical_raw_reference_hulls()
-> Result<(), Box<dyn Error>> {
    let calendar = Calendar::custom(
        [Weekday::Saturday, Weekday::Sunday],
        [
            d("2026-02-06"),
            d("2026-05-20"),
            d("2026-11-17"),
            d("2026-11-20"),
        ],
    );
    let mut cases = 0;
    for rule in Roll::ALL {
        let mut contexts = HashSet::new();
        let expected = quarter_hull(rule);
        let count = u32::try_from(expected.len() - 1)?;
        for ordinary in Bdc::ALL {
            for terminal in Bdc::ALL {
                let schedule = ScheduleBuilder::new(
                    d("2026-02-06"),
                    d("2026-11-17"),
                    ScheduleCalendar::Plain(calendar.clone()),
                    ordinary,
                    terminal,
                )?
                .build_roll(months(3), rule, 5)?;
                let context = schedule.build_icma_context(Frequency::Quarterly, count)?;
                assert_eq!(boundaries(&context), expected);
                contexts.insert(context);
                cases += 1;
            }
        }
        let closed = builder(
            d("2026-02-06"),
            d("2026-11-17"),
            ScheduleCalendar::Plain(Calendar::custom(Weekday::ALL, [])),
        )?
        .build_roll(months(3), rule, 5)?;
        contexts.insert(closed.build_icma_context(Frequency::Quarterly, count)?);
        assert_eq!(contexts.len(), 1);
    }
    assert_eq!(cases, 196);
    Ok(())
}

#[test]
fn generic_fixed_coupons_keep_complete_roll_context_and_independent_fraction_values()
-> Result<(), Box<dyn Error>> {
    // Exact Fraction controls from the literal quarter_hull dates. N=8 and
    // r=1/8 make the final three-factor product equal to the supplied alpha.
    // The Twentieth rounded-coupon sum differs from the single exact full-span
    // fraction by one ULP; the two quantities are deliberately distinguished.
    for (rule, fractions, partial, total_bits) in [
        (
            Roll::ThirdWednesday,
            vec![(103.0, 364.0), (1.0, 4.0), (45.0, 182.0)],
            (1.0, 28.0),
            0x3fe8_f78f_78f7_8f79_u64,
        ),
        (
            Roll::ThirdWednesdayInclusive,
            vec![(1.0, 4.0); 3],
            (1.0, 364.0),
            0x3fe8_0000_0000_0000,
        ),
        (
            Roll::Twentieth,
            vec![(7.0, 184.0), (1.0, 4.0), (1.0, 4.0), (1.0, 4.0)],
            (13.0, 368.0),
            0x3fe9_37a6_f4de_9bd4,
        ),
        (
            Roll::TwentiethIMM,
            vec![(7.0, 60.0), (1.0, 4.0), (1.0, 4.0), (1.0, 4.0)],
            (13.0, 360.0),
            0x3feb_bbbb_bbbb_bbbc,
        ),
    ] {
        let schedule = ScheduleBuilder::new(
            d("2026-02-06"),
            d("2026-11-17"),
            ScheduleCalendar::Plain(Calendar::weekends_only()),
            Bdc::Following,
            Bdc::ModifiedFollowing,
        )?
        .build_roll(months(3), rule, 5)?;
        let context = schedule.build_icma_context(Frequency::Quarterly, 4)?;
        let quote = FixedCouponRate::new(0.125, context.clone())?;
        assert_eq!(quote.accrual(), &AccrualConvention::Icma(context));
        let count = u32::try_from(fractions.len())?;
        let leg = Leg::fixed_rate(&schedule, 8.0, &quote, count)?;
        for (index, (flow, (numerator, denominator))) in
            leg.flows().iter().zip(fractions).enumerate()
        {
            let CashFlow::FixedRateCoupon(coupon) = flow else {
                return Err("unexpected simple flow".into());
            };
            assert_eq!(coupon.start(), schedule.contractual_dates()[index]);
            assert_eq!(coupon.end(), schedule.contractual_dates()[index + 1]);
            assert_eq!(coupon.payment_date(), schedule.adjusted_dates()[index + 1]);
            assert_eq!(coupon.coupon_rate().accrual(), quote.accrual());
            assert!(std::ptr::eq(
                coupon.coupon_rate().accrual(),
                quote.accrual()
            ));
            within_four_ulp(coupon.accrual_fraction(), numerator / denominator);
            assert_eq!(
                coupon.amount().to_bits(),
                coupon.accrual_fraction().to_bits()
            );
        }
        within_four_ulp(
            leg.accrued_amount(d("2026-02-19"), PaymentDateInclusion::Exclude, 4)?,
            partial.0 / partial.1,
        );
        let discount = InterestRate::new(0.0, DayCount::Actual365Fixed, Compounding::SIMPLE)?;
        within_four_ulp(
            leg.net_present_value(
                d("2026-02-06"),
                &discount,
                PaymentDateInclusion::Exclude,
                u64::from(count) + 1,
            )?,
            f64::from_bits(total_bits),
        );
    }
    Ok(())
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "integration test of immutable sharing; production starts no threads"
)]
fn shared_contexts_rebuild_with_equal_values_hashes_and_queries_concurrently()
-> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for rule in Roll::ALL {
        let schedule = plain(d("2026-02-06"), d("2026-11-17"))?.build_roll(months(3), rule, 5)?;
        let context = schedule.build_icma_context(Frequency::Quarterly, 4)?;
        cases.push((schedule, context));
    }
    let cases = Arc::new(cases);
    let mut threads = Vec::new();
    for _ in 0..4 {
        let cases = Arc::clone(&cases);
        threads.push(std::thread::spawn(move || {
            let mut count = 0;
            for _ in 0..16 {
                for (schedule, expected) in cases.iter() {
                    let rebuilt = schedule
                        .build_icma_context(Frequency::Quarterly, 4)
                        .expect("valid immutable rebuild");
                    assert_eq!(&rebuilt, expected);
                    assert_eq!(hash(&rebuilt), hash(expected));
                    let start = expected.coverage_start();
                    let end = expected.coverage_end();
                    assert_eq!(
                        rebuilt
                            .year_fraction(start, end)
                            .expect("covered query")
                            .to_bits(),
                        expected
                            .year_fraction(start, end)
                            .expect("covered query")
                            .to_bits()
                    );
                    count += 1;
                }
            }
            count
        }));
    }
    let completed: usize = threads
        .into_iter()
        .map(|thread| thread.join().expect("test worker succeeds"))
        .sum();
    assert_eq!(completed, 256);
    Ok(())
}
