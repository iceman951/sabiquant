//! Actual QuantLib and rational fixtures (F-DCC-01/02/03/07; TS-DCC-01/02/08).

mod support;

use sabiquant::daycount::DayCount;
use sabiquant::time::Date;
use support::fixture::Fixture;
use support::float::{Tolerance, assert_close, ulp_distance};

#[test]
fn four_core_day_counts_match_actual_quantlib_and_independent_exact_rationals() {
    let data = Fixture::load(Fixture::repository_path(
        "validation/fixtures/daycount/core-daycounts.fixture",
    ))
    .unwrap();
    assert_eq!(data.header("reference-version"), Some("1.43"));
    let mut counts = [0_u32; 4];
    let mut numerical = 0;
    let mut range = 0;
    for record in data.records() {
        let convention = record.parse::<DayCount>("convention").unwrap();
        let index = DayCount::ALL
            .iter()
            .position(|value| *value == convention)
            .unwrap();
        counts[index] += 1;
        let start = record.parse::<Date>("start").unwrap();
        let end = record.parse::<Date>("end").unwrap();
        let actual_days = convention.day_count(start, end);
        assert_eq!(actual_days, record.parse::<i32>("reference_days").unwrap());
        assert_eq!(actual_days, record.parse::<i32>("oracle_days").unwrap());
        let actual = convention.year_fraction(start, end).unwrap();
        let isda = convention == DayCount::ActualActualIsda;
        assert_eq!(
            record.text("tolerance").unwrap(),
            if isda { "TOL-E2" } else { "TOL-E1" }
        );
        assert_close(
            actual,
            record.real("oracle_fraction").unwrap(),
            Tolerance::Ulps(if isda { 4 } else { 0 }),
        );
        match record.text("discrepancy").unwrap() {
            "" => {
                assert_eq!(record.text("reference_behavior").unwrap(), "value");
                assert_eq!(record.text("reference_error").unwrap(), "");
                assert_close(
                    actual,
                    record.real("reference_fraction").unwrap(),
                    Tolerance::Ulps(if isda { 8 } else { 0 }),
                );
            }
            "D5-ISDA-SAME-YEAR-CANCELLATION" => {
                numerical += 1;
                assert!(isda);
                assert_eq!(start.year(), end.year());
                assert_eq!(record.text("reference_behavior").unwrap(), "value");
                assert_eq!(record.text("reference_error").unwrap(), "");
                let reference = record.real("reference_fraction").unwrap();
                let oracle = record.real("oracle_fraction").unwrap();
                assert!(ulp_distance(reference, oracle).unwrap() > 8);
                assert_close(actual, oracle, Tolerance::Ulps(0));
                assert_close(
                    reference,
                    oracle,
                    Tolerance::Mixed {
                        absolute: 3.0 * f64::EPSILON,
                        relative: 0.0,
                    },
                );
            }
            "D8-ISDA-MAX-YEAR-INTERMEDIATE" => {
                range += 1;
                assert!(isda);
                assert_ne!(start, end);
                assert_eq!(start.year(), 2199);
                assert_eq!(end.year(), 2199);
                assert_eq!(record.text("reference_behavior").unwrap(), "raises");
                assert_eq!(record.text("reference_fraction").unwrap(), "");
                assert!(record.text("reference_error").unwrap().contains("2200"));
            }
            label => panic!("unclassified reference discrepancy {label:?}"),
        }
        assert!(actual.is_finite());
        if actual_days == 0 {
            assert_eq!(actual.to_bits(), 0);
        }
    }
    assert!(counts.iter().all(|count| *count > 10_000));
    assert!(counts.iter().all(|count| *count == counts[0]));
    assert_eq!(numerical, 8372);
    assert_eq!(range, 452);
}
