//! TARGET coverage failures remain visible in competing searches and deep joints.

use sabiquant::calendar::{
    BusinessDayConvention as Bdc, Calendar, CalendarError, JointRule, TargetCalendar,
};
use sabiquant::time::{Date, Weekday};

#[test]
fn nearest_keeps_coverage_errors_even_when_the_other_direction_succeeds() {
    let target = TargetCalendar::new();
    let boundary = Date::from_ymd(2028, 12, 31).unwrap();
    assert!(
        matches!(target.adjust(boundary, Bdc::Nearest), Err(CalendarError::UnverifiedCoverage { date, .. }) if date == Date::from_ymd(2029, 1, 1).unwrap())
    );
    let launch = Date::from_ymd(1999, 1, 4).unwrap();
    assert!(
        matches!(target.with_holiday(launch).adjust(launch, Bdc::Nearest), Err(CalendarError::OutsideValidity { date, .. }) if date == Date::from_ymd(1999, 1, 3).unwrap())
    );
    let mut closed = target;
    for offset in 0..366 {
        closed = closed.with_holiday(boundary.checked_sub_days(offset).unwrap());
    }
    // Forward is unverified; backward exhausts its finite search. Missing
    // coverage must remain visible rather than be replaced by SearchLimit.
    assert!(
        matches!(closed.adjust(boundary, Bdc::Nearest), Err(CalendarError::UnverifiedCoverage { date, .. }) if date == Date::from_ymd(2029, 1, 1).unwrap())
    );
}

#[test]
fn half_month_modified_fallback_preserves_the_inception_error() {
    let mut target = TargetCalendar::new();
    for day in 4..=15 {
        target = target.with_holiday(Date::from_ymd(1999, 1, day).unwrap());
    }
    let input = Date::from_ymd(1999, 1, 15).unwrap();
    assert!(
        matches!(target.adjust(input, Bdc::HalfMonthModifiedFollowing), Err(CalendarError::OutsideValidity { date, .. }) if date == Date::from_ymd(1999, 1, 3).unwrap())
    );
}

#[test]
fn guarded_joint_construction_keeps_the_same_nesting_limit_and_original_value() {
    let mut deep = Calendar::weekends_only();
    let mut last_rule = JointRule::JoinHolidays;
    for depth in 1..=Calendar::MAX_JOINT_DEPTH {
        last_rule = if depth % 2 == 0 {
            JointRule::JoinHolidays
        } else {
            JointRule::JoinBusinessDays
        };
        deep = Calendar::joint(
            vec![deep, Calendar::custom([Weekday::Tuesday], [])],
            last_rule,
        )
        .unwrap();
    }
    let outer = if last_rule == JointRule::JoinHolidays {
        JointRule::JoinBusinessDays
    } else {
        JointRule::JoinHolidays
    };
    let target = TargetCalendar::new();
    assert!(matches!(
        target.joint_with(vec![deep], outer),
        Err(CalendarError::NestingLimit {
            depth: 65,
            maximum: 64,
            ..
        })
    ));
    let date = Date::from_ymd(2026, 1, 2).unwrap();
    assert!(target.is_business_day(date).unwrap());
    assert_eq!(target, TargetCalendar::new());
}
