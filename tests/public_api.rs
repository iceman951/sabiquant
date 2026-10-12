//! Checks of the public API surface, seen from outside the crate.
//!
//! - NFR-05, INV-CORE-3: every public type can be sent and shared between
//!   threads, and is `'static`.
//! - ADR-0007 item 4: the error types implement the standard traits, so any
//!   SabiQuant error converts into a boxed standard error with `?`.
//!
//! The behavior of the error types is tested inside the crate
//! (TS-CORE-01), because only SabiQuant's own operations create errors.

use sabiquant::calendar::{BusinessDayConvention, Calendar, CalendarError, JointRule};
use sabiquant::daycount::{DayCount, DayCountError};
use sabiquant::error::{Error, IdentifierError, Interval, NonFinite, ValueError};
use sabiquant::rates::{
    AccrualConvention, Compounding, CompoundingMethod, InterestRate, RateError,
};
use sabiquant::time::{Date, DateError, Frequency, Month, Period, PeriodError, TimeUnit, Weekday};
use std::error::Error as StdError;
use std::fmt::Debug;
use std::hash::Hash;

fn assert_value_type<T: Clone + Debug + PartialEq + Send + Sync + 'static>() {}

fn assert_copy_type<T: Copy + Debug + PartialEq + Send + Sync + 'static>() {}

fn assert_error_type<T: StdError + Clone + PartialEq + Send + Sync + 'static>() {}

#[test]
fn public_types_are_thread_safe_values() {
    assert_value_type::<Error>();
    assert_value_type::<ValueError>();
    assert_value_type::<IdentifierError>();
    assert_copy_type::<NonFinite>();
    assert_copy_type::<Interval>();
    assert_copy_type::<Date>();
    assert_copy_type::<Period>();
    assert_copy_type::<TimeUnit>();
    assert_copy_type::<Frequency>();
    assert_copy_type::<Month>();
    assert_copy_type::<Weekday>();
    assert_value_type::<DateError>();
    assert_value_type::<PeriodError>();
    assert_value_type::<Calendar>();
    assert_value_type::<CalendarError>();
    assert_value_type::<DayCountError>();
    assert_copy_type::<BusinessDayConvention>();
    assert_copy_type::<JointRule>();
    assert_copy_type::<DayCount>();
    assert_copy_type::<CompoundingMethod>();
    assert_copy_type::<Compounding>();
    assert_value_type::<AccrualConvention>();
    assert_value_type::<InterestRate>();
    assert_value_type::<RateError>();
}

#[test]
fn error_types_implement_the_standard_traits() {
    assert_error_type::<Error>();
    assert_error_type::<ValueError>();
    assert_error_type::<IdentifierError>();
    assert_error_type::<DateError>();
    assert_error_type::<PeriodError>();
    assert_error_type::<CalendarError>();
    assert_error_type::<DayCountError>();
    assert_error_type::<RateError>();
}

#[test]
fn non_finite_classes_can_be_map_keys() {
    fn assert_key<T: Eq + Hash>() {}
    assert_key::<NonFinite>();
}

#[test]
fn errors_convert_with_the_question_mark_operator() {
    // Compile-time checks: each conversion that `?` performs exists.
    let _: fn(ValueError) -> Error = Error::from;
    let _: fn(IdentifierError) -> Error = Error::from;
    let _: fn(DateError) -> Error = Error::from;
    let _: fn(PeriodError) -> Error = Error::from;
    let _: fn(CalendarError) -> Error = Error::from;
    let _: fn(DayCountError) -> Error = Error::from;
    let _: fn(RateError) -> Error = Error::from;
    let _: fn(Error) -> Box<dyn StdError + Send + Sync> = Into::into;
}

#[test]
fn rate_context_errors_retain_every_typed_cause_through_the_root_error() {
    let calendar = sabiquant::calendar::TargetCalendar::new();
    let quote = InterestRate::new(
        0.0,
        sabiquant::daycount::Business252::with_target(calendar),
        Compounding::SIMPLE,
    )
    .unwrap();
    let date = "2025-12-31".parse().unwrap();
    let cause = quote.compound_factor_between(date, date).unwrap_err();
    let root = Error::from(cause.clone());
    assert_eq!(root.kind(), "rate.day_count");
    let rate = root.source().unwrap();
    assert_eq!(rate.downcast_ref::<RateError>(), Some(&cause));
    let daycount = rate.source().unwrap();
    assert!(daycount.is::<DayCountError>());
    let calendar = daycount.source().unwrap();
    assert!(calendar.is::<CalendarError>());
    assert!(calendar.source().is_none());
}

/// FR-CORE-02 and ADR-0007: a crate error preserves its typed domain cause.
#[test]
fn domain_errors_are_inspectable_through_source() {
    let date_error = Date::from_ymd(2023, 2, 29).unwrap_err();
    let error = Error::from(date_error.clone());
    assert_eq!(error.kind(), date_error.kind());
    assert_eq!(error.to_string(), date_error.to_string());
    assert_eq!(
        error.source().unwrap().downcast_ref::<DateError>(),
        Some(&date_error)
    );
    let period_error = Period::new(i32::MIN, TimeUnit::Days)
        .checked_neg()
        .unwrap_err();
    let error = Error::from(period_error.clone());
    assert_eq!(error.kind(), period_error.kind());
    assert_eq!(
        error.source().unwrap().downcast_ref::<PeriodError>(),
        Some(&period_error)
    );
}

/// Calendar errors retain both their domain and underlying date context.
#[test]
fn calendar_error_sources_preserve_the_date_cause() {
    let calendar = Calendar::custom([Weekday::Monday], []);
    let cause = calendar
        .adjust(Date::MIN, BusinessDayConvention::Preceding)
        .unwrap_err();
    let error = Error::from(cause.clone());
    assert_eq!(error.kind(), "calendar.date");
    assert_eq!(
        error.source().unwrap().downcast_ref::<CalendarError>(),
        Some(&cause)
    );
    assert!(error.source().unwrap().source().unwrap().is::<DateError>());
    assert!(error.source().unwrap().source().unwrap().source().is_none());
}

/// FR-DATE-03/04/05 and FR-PER-05: tenor dispatch has calendar-day semantics.
#[test]
fn date_period_dispatch_and_extreme_subtraction() {
    let date = Date::from_ymd(2024, 1, 31).unwrap();
    for unit in TimeUnit::ALL {
        assert_eq!(date.checked_add_period(Period::new(0, unit)).unwrap(), date);
        assert_eq!(date.checked_sub_period(Period::new(0, unit)).unwrap(), date);
        assert!(
            date.checked_add_period(Period::new(i32::MAX, unit))
                .is_err()
        );
        assert!(
            date.checked_sub_period(Period::new(i32::MIN, unit))
                .is_err()
        );
        let expected = match unit {
            TimeUnit::Days => date.checked_add_days(1),
            TimeUnit::Weeks => date.checked_add_weeks(1),
            TimeUnit::Months => date.checked_add_months(1),
            TimeUnit::Years => date.checked_add_years(1),
        };
        assert_eq!(date.checked_add_period(Period::new(1, unit)), expected);
        assert_eq!(date.checked_sub_period(Period::new(-1, unit)), expected);
    }
    assert_eq!(
        date.checked_add_period(Period::new(1, TimeUnit::Months))
            .unwrap()
            .to_string(),
        "2024-02-29"
    );
}
