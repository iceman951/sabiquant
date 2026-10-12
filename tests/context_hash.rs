//! Definition-based hashing of discrete accrual contexts (API-04).

use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};

use sabiquant::calendar::{Calendar, JointRule, TargetCalendar, TargetPolicy};
use sabiquant::daycount::{
    ActualActualIcma, Business252, DayCount, DeterminationPeriod, ThirtyE360Isda,
};
use sabiquant::rates::AccrualConvention;
use sabiquant::time::{Date, Frequency};

fn hash<T: Hash>(value: &T) -> u64 {
    let mut state = DefaultHasher::new();
    value.hash(&mut state);
    state.finish()
}

#[test]
fn independently_built_equal_icma_contexts_have_equal_hashes_and_retrieve_cached_definitions() {
    let first = Date::from_ymd(2024, 1, 1).unwrap();
    let middle = Date::from_ymd(2024, 7, 1).unwrap();
    let last = Date::from_ymd(2025, 1, 1).unwrap();
    let mut unique = HashSet::new();
    for frequency in Frequency::ALL {
        if frequency.events_per_year().is_none() {
            continue;
        }
        let make = || {
            ActualActualIcma::new(
                vec![
                    DeterminationPeriod::new(first, middle).unwrap(),
                    DeterminationPeriod::new(middle, last).unwrap(),
                ],
                frequency,
            )
            .unwrap()
        };
        let a = make();
        let b = make();
        assert_eq!(a, b);
        assert_eq!(hash(&a), hash(&b));
        let mut cache = HashMap::new();
        cache.insert(a.clone(), (a.coverage_start(), a.coverage_end()));
        assert_eq!(cache.get(&b), Some(&(b.coverage_start(), b.coverage_end())));
        assert!(unique.insert(a));
        assert!(!unique.insert(b));
    }
    assert_eq!(unique.len(), 10);
}

#[test]
fn accrual_hash_keys_preserve_context_and_canonical_calendar_definition_equality() {
    let holiday = Date::from_ymd(2026, 2, 2).unwrap();
    let a = Calendar::joint(
        vec![Calendar::weekends_only(), Calendar::null()],
        JointRule::JoinHolidays,
    )
    .unwrap()
    .with_holiday(holiday);
    let b = Calendar::joint(
        vec![
            Calendar::null(),
            Calendar::weekends_only(),
            Calendar::null(),
        ],
        JointRule::JoinHolidays,
    )
    .unwrap()
    .with_holiday(holiday);
    let a = AccrualConvention::from(Business252::new(a));
    let b = AccrualConvention::from(Business252::new(b));
    assert_eq!(a, b);
    assert_eq!(hash(&a), hash(&b));
    let mut contexts = HashSet::from([a.clone(), b]);
    assert_eq!(contexts.len(), 1);
    assert!(contexts.contains(&a));
    for value in DayCount::ALL {
        let context = AccrualConvention::from(value);
        assert_eq!(hash(&context), hash(&context.clone()));
        assert!(contexts.insert(context));
    }
    for termination in [holiday, holiday.checked_add_days(1).unwrap()] {
        assert!(contexts.insert(ThirtyE360Isda::new(termination).into()));
    }
    assert!(contexts.insert(Business252::with_target(TargetCalendar::new()).into()));
    assert!(contexts.insert(
        Business252::with_target(TargetCalendar::with_policy(TargetPolicy::AllowProjection)).into()
    ));
    let period = DeterminationPeriod::new(holiday, holiday.checked_add_days(1).unwrap()).unwrap();
    assert!(
        contexts.insert(
            ActualActualIcma::new(vec![period], Frequency::Annual)
                .unwrap()
                .into()
        )
    );
    assert_eq!(contexts.len(), 11);
}
