//! Actual collection keys and the complete supported one-day schedule range.

use sabiquant::calendar::{
    BusinessDayConvention as Bdc, Calendar, CalendarError, EndOfMonth, TargetCalendar,
};
use sabiquant::schedule::{
    DateGenerationRule as Rule, PeriodRegularity, ScheduleBuilder, ScheduleCalendar, ScheduleError,
    StubPolicy,
};
use sabiquant::time::{Date, Period, TimeUnit};
use std::collections::HashMap;
use std::error::Error;

#[test]
fn equal_date_vectors_can_retain_distinct_hash_map_definitions() -> Result<(), Box<dyn Error>> {
    let start: Date = "2024-01-15".parse()?;
    let end: Date = "2026-01-15".parse()?;
    let builder = ScheduleBuilder::new(
        start,
        end,
        ScheduleCalendar::Plain(Calendar::null()),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )?;
    let tenor = Period::new(1, TimeUnit::Years);
    let baseline = builder.build_recurrent(
        tenor,
        Rule::Forward,
        StubPolicy::None,
        EndOfMonth::Ignore,
        3,
    )?;
    let same = builder.clone().build_recurrent(
        tenor,
        Rule::Forward,
        StubPolicy::None,
        EndOfMonth::Ignore,
        3,
    )?;
    let month_tenor = Period::new(12, TimeUnit::Months);
    assert_eq!(tenor, month_tenor);
    // On an exact lattice Long has no residual to merge. It cannot silently
    // remove a regular boundary just to fit the cap.
    assert!(matches!(
        builder.build_recurrent(
            tenor,
            Rule::Forward,
            StubPolicy::Long,
            EndOfMonth::Ignore,
            2
        ),
        Err(ScheduleError::BoundaryLimitExceeded {
            maximum: 2,
            required_at_least: 3,
            ..
        })
    ));
    let variants = [
        baseline.clone(),
        builder.build_recurrent(
            month_tenor,
            Rule::Forward,
            StubPolicy::None,
            EndOfMonth::Ignore,
            3,
        )?,
        builder.build_recurrent(
            tenor,
            Rule::Backward,
            StubPolicy::None,
            EndOfMonth::Ignore,
            3,
        )?,
        builder.build_recurrent(
            tenor,
            Rule::Forward,
            StubPolicy::Short,
            EndOfMonth::Ignore,
            3,
        )?,
        builder.build_recurrent(
            tenor,
            Rule::Forward,
            StubPolicy::Long,
            EndOfMonth::Ignore,
            3,
        )?,
        builder.build_recurrent(
            tenor,
            Rule::Forward,
            StubPolicy::None,
            EndOfMonth::Preserve,
            3,
        )?,
        ScheduleBuilder::new(
            start,
            end,
            ScheduleCalendar::Plain(Calendar::null()),
            Bdc::Following,
            Bdc::Unadjusted,
        )?
        .build_recurrent(
            tenor,
            Rule::Forward,
            StubPolicy::None,
            EndOfMonth::Ignore,
            3,
        )?,
        ScheduleBuilder::new(
            start,
            end,
            ScheduleCalendar::Plain(Calendar::null()),
            Bdc::Unadjusted,
            Bdc::Following,
        )?
        .build_recurrent(
            tenor,
            Rule::Forward,
            StubPolicy::None,
            EndOfMonth::Ignore,
            3,
        )?,
        ScheduleBuilder::new(
            start,
            end,
            ScheduleCalendar::Plain(Calendar::weekends_only()),
            Bdc::Unadjusted,
            Bdc::Unadjusted,
        )?
        .build_recurrent(
            tenor,
            Rule::Forward,
            StubPolicy::None,
            EndOfMonth::Ignore,
            3,
        )?,
    ];
    let mut keys = HashMap::new();
    for (index, schedule) in variants.iter().enumerate() {
        assert_eq!(schedule.contractual_dates(), baseline.contractual_dates());
        assert_eq!(schedule.adjusted_dates(), baseline.adjusted_dates());
        assert_eq!(schedule.regularity(), baseline.regularity());
        assert_eq!(keys.insert(schedule.clone(), index), None);
    }
    assert_eq!(keys.len(), 9);
    assert_eq!(keys.get(&same), Some(&0));
    for (index, schedule) in variants.iter().enumerate() {
        assert_eq!(keys.get(&schedule.clone()), Some(&index));
        for (other_index, other) in variants.iter().enumerate() {
            assert_eq!(schedule == other, index == other_index);
        }
    }
    // The cap is a work limit, not retained contractual metadata. Rebuilding
    // the same valid value with a larger cap must locate the same map key.
    let larger_cap = builder.build_recurrent(
        tenor,
        Rule::Forward,
        StubPolicy::None,
        EndOfMonth::Ignore,
        100,
    )?;
    assert_eq!(keys.get(&larger_cap), Some(&0));
    let zero = builder.build_zero(2)?;
    assert_ne!(zero, baseline);
    assert_eq!(keys.insert(zero.clone(), 9), None);
    let same_zero = builder.clone().build_zero(100)?;
    assert_eq!(same_zero, zero);
    assert_eq!(keys.get(&same_zero), Some(&9));
    assert_eq!(keys.len(), 10);
    Ok(())
}

#[test]
fn complete_supported_daily_span_reaches_the_exact_public_boundary_limit()
-> Result<(), Box<dyn Error>> {
    let builder = ScheduleBuilder::new(
        Date::MIN,
        Date::MAX,
        ScheduleCalendar::Plain(Calendar::null()),
        Bdc::Unadjusted,
        Bdc::Unadjusted,
    )?;
    let maximum = ScheduleBuilder::MAX_BOUNDARIES;
    let full = builder.build_recurrent(
        Period::new(1, TimeUnit::Days),
        Rule::Forward,
        StubPolicy::None,
        EndOfMonth::Ignore,
        maximum,
    )?;
    assert_eq!(full.contractual_dates().len(), usize::try_from(maximum)?);
    assert_eq!(full.contractual_dates().first(), Some(&Date::MIN));
    assert_eq!(full.contractual_dates().last(), Some(&Date::MAX));
    assert_eq!(full.adjusted_dates(), full.contractual_dates());
    assert_eq!(full.regularity().len() + 1, full.contractual_dates().len());
    assert!(
        full.regularity()
            .iter()
            .all(|flag| *flag == PeriodRegularity::Regular)
    );
    for pair in full.contractual_dates().windows(2) {
        assert_eq!(pair[1].days_since(pair[0]), 1);
    }
    drop(full);
    let error = builder
        .build_recurrent(
            Period::new(1, TimeUnit::Days),
            Rule::Forward,
            StubPolicy::None,
            EndOfMonth::Ignore,
            maximum - 1,
        )
        .unwrap_err();
    assert!(matches!(error, ScheduleError::BoundaryLimitExceeded {
        maximum: actual, required_at_least, ..
    } if actual == maximum - 1 && required_at_least == maximum));
    Ok(())
}

#[test]
fn verified_raw_spans_preserve_failures_from_adjustment_candidates_outside_coverage()
-> Result<(), Box<dyn Error>> {
    for (start, end, ordinary, terminal, index, missing_date) in [
        (
            "2026-01-01",
            "2026-01-31",
            Bdc::Nearest,
            Bdc::Unadjusted,
            0,
            "2025-12-31",
        ),
        (
            "2028-12-30",
            "2028-12-31",
            Bdc::Unadjusted,
            Bdc::Following,
            1,
            "2029-01-01",
        ),
    ] {
        let error = ScheduleBuilder::new(
            start.parse()?,
            end.parse()?,
            ScheduleCalendar::Target(TargetCalendar::new()),
            ordinary,
            terminal,
        )?
        .build_zero(2)
        .unwrap_err();
        let ScheduleError::Calendar {
            index: observed,
            phase,
            cause,
            ..
        } = &error
        else {
            panic!("expected a checked calendar failure, got {error}");
        };
        assert_eq!(*observed, Some(index));
        assert_eq!(*phase, "adjustment");
        let expected_missing: Date = missing_date.parse()?;
        assert!(
            matches!(cause, CalendarError::UnverifiedCoverage { date, .. } if *date == expected_missing)
        );
        let root = sabiquant::error::Error::from(error.clone());
        assert_eq!(root.kind(), "schedule.calendar");
        let schedule = root
            .source()
            .unwrap()
            .downcast_ref::<ScheduleError>()
            .unwrap();
        assert_eq!(schedule, &error);
        assert_eq!(
            schedule.source().unwrap().downcast_ref::<CalendarError>(),
            Some(cause)
        );
    }
    Ok(())
}
