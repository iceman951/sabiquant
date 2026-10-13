//! Generated contractual references, checked extensions and explicit frequency.

use sabiquant::calendar::{
    BusinessDayConvention as Bdc, Calendar, CalendarError, EndOfMonth, TargetCalendar, TargetPolicy,
};
use sabiquant::daycount::{ActualActualIcma, IcmaError};
use sabiquant::error::Error as CrateError;
use sabiquant::schedule::{
    DateGenerationRule as Rule, ExplicitStubDates, Schedule, ScheduleBuilder, ScheduleCalendar,
    ScheduleError, StubPolicy,
};
use sabiquant::time::{Date, DateError, Frequency, Period, TimeUnit, Weekday};
use std::collections::HashSet;
use std::error::Error;

#[expect(clippy::expect_used, reason = "hand-audited ISO test dates")]
fn date(text: &str) -> Date {
    text.parse().expect("valid date")
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
    dates.extend(context.periods().iter().map(|period| period.end()));
    assert!(
        context
            .periods()
            .windows(2)
            .all(|pair| pair[0].end() == pair[1].start())
    );
    dates
}

#[test]
fn named_frequency_and_stored_aliases_are_checked_before_generation() -> Result<(), Box<dyn Error>>
{
    let b = plain(date("2024-01-31"), date("2027-01-31"))?;
    for frequency in Frequency::ALL {
        let Some(tenor) = frequency.period() else {
            continue;
        };
        let eom = if matches!(tenor.unit(), TimeUnit::Months | TimeUnit::Years) {
            EndOfMonth::Preserve
        } else {
            EndOfMonth::Ignore
        };
        let schedule = b.build_recurrent(tenor, Rule::Forward, StubPolicy::Short, eom, 2_000)?;
        let context = schedule.build_icma_context(frequency, 2_000)?;
        assert_eq!(context.frequency(), frequency);
        assert_eq!(context.coverage_start(), b.start());
        assert!(context.coverage_end() >= b.end());
        let count = u32::try_from(context.periods().len())?;
        assert_eq!(context, schedule.build_icma_context(frequency, count)?);
        assert!(matches!(schedule.build_icma_context(frequency, count - 1),
            Err(ScheduleError::IcmaPeriodLimitExceeded { maximum, required_at_least, .. })
                if maximum == count - 1 && required_at_least == count));
        for other in Frequency::ALL {
            if other == frequency {
                continue;
            }
            let error = schedule.build_icma_context(other, 2_000).unwrap_err();
            assert_eq!(
                error.kind(),
                if other == Frequency::Once {
                    "schedule.invalid_icma_frequency"
                } else {
                    "schedule.icma_frequency_mismatch"
                }
            );
        }
        let alias = match tenor.unit() {
            TimeUnit::Years => Period::new(12 * tenor.length(), TimeUnit::Months),
            TimeUnit::Weeks => Period::new(7 * tenor.length(), TimeUnit::Days),
            _ => tenor,
        };
        let aliased = b.build_recurrent(alias, Rule::Forward, StubPolicy::Short, eom, 2_000)?;
        assert_eq!(context, aliased.build_icma_context(frequency, 2_000)?);
    }
    Ok(())
}

#[test]
fn invalid_limits_zero_once_and_mismatch_preserve_precedence() -> Result<(), Box<dyn Error>> {
    let b = plain(Date::MIN, Date::MIN.checked_add_days(1)?)?;
    let zero = b.build_zero(2)?;
    for cap in [0, Schedule::MAX_ICMA_PERIODS + 1, u32::MAX] {
        assert!(matches!(zero.build_icma_context(Frequency::Once, cap),
            Err(ScheduleError::InvalidIcmaPeriodLimit { requested, minimum: 1, maximum: 3_652_058, .. }) if requested == cap));
    }
    assert!(matches!(
        zero.build_icma_context(Frequency::Once, 1),
        Err(ScheduleError::NonPeriodicIcma {
            rule: Rule::Zero,
            ..
        })
    ));
    let recurring = b.build_recurrent(
        Period::new(2, TimeUnit::Days),
        Rule::Backward,
        StubPolicy::Short,
        EndOfMonth::Ignore,
        2,
    )?;
    assert!(matches!(
        recurring.build_icma_context(Frequency::Once, 1),
        Err(ScheduleError::InvalidIcmaFrequency {
            frequency: Frequency::Once,
            ..
        })
    ));
    assert!(matches!(recurring.build_icma_context(Frequency::Daily, 1),
        Err(ScheduleError::IcmaFrequencyMismatch { tenor, frequency: Frequency::Daily, expected_tenor, .. })
        if tenor.length() == 2 && expected_tenor == Period::new(1, TimeUnit::Days)));
    Ok(())
}

#[test]
fn short_long_and_multiple_outer_stubs_use_every_original_lattice_period()
-> Result<(), Box<dyn Error>> {
    let b = plain(date("2024-01-31"), date("2024-05-15"))?;
    for stub in [StubPolicy::Short, StubPolicy::Long] {
        let schedule = b.build_recurrent(
            Period::new(1, TimeUnit::Months),
            Rule::Forward,
            stub,
            EndOfMonth::Preserve,
            6,
        )?;
        assert_eq!(
            boundaries(&schedule.build_icma_context(Frequency::Monthly, 4)?),
            [
                date("2024-01-31"),
                date("2024-02-29"),
                date("2024-03-31"),
                date("2024-04-30"),
                date("2024-05-31")
            ]
        );
    }
    let annual = plain(date("1999-02-01"), date("1999-07-01"))?.build_recurrent(
        Period::new(1, TimeUnit::Years),
        Rule::Backward,
        StubPolicy::Short,
        EndOfMonth::Ignore,
        2,
    )?;
    let context = annual.build_icma_context(Frequency::Annual, 1)?;
    assert_eq!(
        boundaries(&context),
        [date("1998-07-01"), date("1999-07-01")]
    );
    assert_eq!(
        context
            .year_fraction(date("1999-02-01"), date("1999-07-01"))?
            .to_bits(),
        (150.0_f64 / 365.0).to_bits()
    );
    let b = plain(date("2021-02-15"), date("2028-08-15"))?;
    let roles = ExplicitStubDates::new(Some(date("2024-02-29")), Some(date("2024-02-29")))?;
    let schedule = b.build_explicit(
        Period::new(1, TimeUnit::Years),
        Rule::Forward,
        roles,
        EndOfMonth::Preserve,
        3,
    )?;
    let context = schedule.build_icma_context(Frequency::Annual, 9)?;
    assert_eq!(
        boundaries(&context),
        [
            date("2020-02-29"),
            date("2021-02-28"),
            date("2022-02-28"),
            date("2023-02-28"),
            date("2024-02-29"),
            date("2025-02-28"),
            date("2026-02-28"),
            date("2027-02-28"),
            date("2028-02-29"),
            date("2029-02-28")
        ]
    );
    for cap in 1..9 {
        assert!(matches!(schedule.build_icma_context(Frequency::Annual,cap),
            Err(ScheduleError::IcmaPeriodLimitExceeded{maximum,required_at_least,..}) if maximum==cap && required_at_least==cap+1));
    }
    Ok(())
}

#[test]
fn exact_lattice_endpoints_need_no_additional_civil_or_market_dates() -> Result<(), Box<dyn Error>>
{
    for (start, end, rule) in [
        (Date::MIN, Date::MIN.checked_add_days(1)?, Rule::Backward),
        (Date::MAX.checked_add_days(-1)?, Date::MAX, Rule::Forward),
    ] {
        let schedule = plain(start, end)?.build_recurrent(
            Period::new(1, TimeUnit::Days),
            rule,
            StubPolicy::None,
            EndOfMonth::Ignore,
            2,
        )?;
        assert_eq!(
            boundaries(&schedule.build_icma_context(Frequency::Daily, 1)?),
            [start, end]
        );
    }
    for (start, end, rule) in [
        (date("2026-01-01"), date("2026-02-01"), Rule::Forward),
        (date("2028-11-30"), date("2028-12-31"), Rule::Forward),
    ] {
        let schedule = builder(start, end, ScheduleCalendar::Target(TargetCalendar::new()))?
            .build_recurrent(
                Period::new(1, TimeUnit::Months),
                rule,
                StubPolicy::None,
                EndOfMonth::Preserve,
                2,
            )?;
        assert_eq!(
            boundaries(&schedule.build_icma_context(Frequency::Monthly, 1)?),
            [start, end]
        );
    }
    Ok(())
}

#[test]
fn required_out_of_range_quasi_dates_retain_actual_signed_date_causes() -> Result<(), Box<dyn Error>>
{
    for (start, end, rule, count) in [
        (
            Date::MIN,
            Date::MIN.checked_add_days(1)?,
            Rule::Backward,
            -1,
        ),
        (Date::MAX.checked_add_days(-1)?, Date::MAX, Rule::Forward, 1),
    ] {
        let schedule = plain(start, end)?.build_recurrent(
            Period::new(1, TimeUnit::Months),
            rule,
            StubPolicy::Short,
            EndOfMonth::Ignore,
            2,
        )?;
        let error = schedule
            .build_icma_context(Frequency::Monthly, 1)
            .unwrap_err();
        assert!(
            matches!(&error,ScheduleError::Date{index:1,phase:"ICMA reference extension",cause:DateError::OutOfRange{date:anchor,count:offset,..},..}
            if *anchor==schedule.lattice_anchor().unwrap() && *offset==count)
        );
        let source = error.source().unwrap();
        assert!(source.downcast_ref::<DateError>().is_some());
        let root = CrateError::from(error.clone());
        assert_eq!(root.kind(), "schedule.date");
        assert_eq!(
            root.source().unwrap().downcast_ref::<ScheduleError>(),
            Some(&error)
        );
    }
    let schedule = plain(Date::MIN, Date::MIN.checked_add_days(2)?)?.build_explicit(
        Period::new(1, TimeUnit::Months),
        Rule::Forward,
        ExplicitStubDates::new(
            Some(Date::MIN.checked_add_days(1)?),
            Some(Date::MIN.checked_add_days(1)?),
        )?,
        EndOfMonth::Ignore,
        3,
    )?;
    assert!(matches!(
        schedule.build_icma_context(Frequency::Monthly, 1),
        Err(ScheduleError::IcmaPeriodLimitExceeded {
            maximum: 1,
            required_at_least: 2,
            ..
        })
    ));
    assert!(matches!(
        schedule.build_icma_context(Frequency::Monthly, 2),
        Err(ScheduleError::Date { .. })
    ));
    Ok(())
}

#[test]
fn extended_target_span_is_guarded_before_returning_owned_references() -> Result<(), Box<dyn Error>>
{
    for policy in [TargetPolicy::VerifiedOnly, TargetPolicy::AllowProjection] {
        let schedule = builder(
            date("2026-01-02"),
            date("2026-02-10"),
            ScheduleCalendar::Target(TargetCalendar::with_policy(policy)),
        )?
        .build_explicit(
            Period::new(1, TimeUnit::Months),
            Rule::Forward,
            ExplicitStubDates::new(Some(date("2026-01-10")), None)?,
            EndOfMonth::Ignore,
            3,
        )?;
        let result = schedule.build_icma_context(Frequency::Monthly, 2);
        if policy == TargetPolicy::VerifiedOnly {
            let error = result.unwrap_err();
            assert!(
                matches!(&error,ScheduleError::Calendar{index:None,phase:"ICMA reference coverage",
                cause:CalendarError::UnverifiedCoverage{date:rejected,..},..} if *rejected==date("2025-12-10"))
            );
            assert!(
                error
                    .source()
                    .unwrap()
                    .downcast_ref::<CalendarError>()
                    .is_some()
            );
        } else {
            assert_eq!(
                boundaries(&result?),
                [date("2025-12-10"), date("2026-01-10"), date("2026-02-10")]
            );
        }
    }
    let schedule = builder(
        date("1999-01-04"),
        date("1999-02-06"),
        ScheduleCalendar::Target(TargetCalendar::with_policy(TargetPolicy::AllowProjection)),
    )?
    .build_explicit(
        Period::new(1, TimeUnit::Months),
        Rule::Forward,
        ExplicitStubDates::new(Some(date("1999-01-06")), None)?,
        EndOfMonth::Ignore,
        3,
    )?;
    assert!(matches!(schedule.build_icma_context(Frequency::Monthly,2),
        Err(ScheduleError::Calendar{phase:"ICMA reference coverage",cause:CalendarError::OutsideValidity{date:rejected,..},..})
            if rejected==date("1998-12-06")));
    Ok(())
}

#[test]
fn adjustments_and_calendar_closures_do_not_change_contractual_references()
-> Result<(), Box<dyn Error>> {
    let start = date("2024-01-31");
    let end = date("2024-04-30");
    let mut values = HashSet::new();
    for convention in Bdc::ALL {
        let schedule = ScheduleBuilder::new(
            start,
            end,
            ScheduleCalendar::Plain(Calendar::weekends_only()),
            convention,
            convention,
        )?
        .build_recurrent(
            Period::new(1, TimeUnit::Months),
            Rule::Forward,
            StubPolicy::None,
            EndOfMonth::Preserve,
            4,
        )?;
        values.insert(schedule.build_icma_context(Frequency::Monthly, 3)?);
    }
    let closed = builder(
        start,
        end,
        ScheduleCalendar::Plain(Calendar::custom(Weekday::ALL, [])),
    )?
    .build_recurrent(
        Period::new(1, TimeUnit::Months),
        Rule::Forward,
        StubPolicy::None,
        EndOfMonth::Preserve,
        4,
    )?;
    values.insert(closed.build_icma_context(Frequency::Monthly, 3)?);
    assert_eq!(values.len(), 1);
    let context = values.into_iter().next().unwrap();
    assert_eq!(
        context
            .year_fraction(context.coverage_start(), context.coverage_end())?
            .to_bits(),
        0.25_f64.to_bits()
    );
    assert_eq!(
        context
            .year_fraction(context.coverage_end(), context.coverage_start())?
            .to_bits(),
        (-0.25_f64).to_bits()
    );
    assert!(matches!(
        context.year_fraction(Date::MIN, Date::MIN),
        Err(IcmaError::OutOfCoverage { .. })
    ));
    Ok(())
}

#[test]
fn maximum_daily_references_cover_every_civil_day_with_one_shared_cap() -> Result<(), Box<dyn Error>>
{
    let anchor = date("5000-01-01");
    let schedule = plain(Date::MIN, Date::MAX)?.build_explicit(
        Period::new(1, TimeUnit::Days),
        Rule::Forward,
        ExplicitStubDates::new(Some(anchor), Some(anchor))?,
        EndOfMonth::Ignore,
        3,
    )?;
    let context = schedule.build_icma_context(Frequency::Daily, Schedule::MAX_ICMA_PERIODS)?;
    assert_eq!(
        context.periods().len(),
        usize::try_from(Schedule::MAX_ICMA_PERIODS)?
    );
    assert_eq!(
        (context.coverage_start(), context.coverage_end()),
        (Date::MIN, Date::MAX)
    );
    for (index, period) in context.periods().iter().enumerate() {
        assert_eq!(
            i64::from(period.start().days_since(Date::MIN)),
            i64::try_from(index)?
        );
        assert_eq!(period.end().days_since(period.start()), 1);
    }
    assert_eq!(
        context.year_fraction(Date::MIN, Date::MAX)?.to_bits(),
        (f64::from(Date::MAX.days_since(Date::MIN)) / 365.0).to_bits()
    );
    drop(context);
    assert!(
        matches!(schedule.build_icma_context(Frequency::Daily,Schedule::MAX_ICMA_PERIODS-1),
        Err(ScheduleError::IcmaPeriodLimitExceeded{maximum,required_at_least,..})
            if maximum==Schedule::MAX_ICMA_PERIODS-1 && required_at_least==Schedule::MAX_ICMA_PERIODS)
    );
    Ok(())
}
