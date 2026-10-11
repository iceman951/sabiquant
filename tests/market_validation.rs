//! Attested public QuantLib comparisons with explicitly classified divergences.

mod support;

use sabiquant::calendar::{Calendar, TargetCalendar, TargetCoverage, TargetPolicy, western_easter};
use sabiquant::daycount::{Business252, DayCount, ThirtyE360Isda};
use sabiquant::time::Date;
use support::fixture::Fixture;
use support::float::{Tolerance, assert_close};

#[test]
fn western_easter_matches_unique_public_uk_good_friday_in_all_299_reference_years() {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/market/easter.fixture",
    ))
    .unwrap();
    assert_eq!(fixture.header("reference-version"), Some("1.43"));
    assert_eq!(fixture.record_count(), 299);
    for row in fixture.records() {
        let year = row.parse::<i32>("year").unwrap();
        assert!((1901..=2199).contains(&year));
        let expected = row.parse::<Date>("oracle_date").unwrap();
        assert_eq!(expected, row.parse::<Date>("reference_date").unwrap());
        assert_eq!(western_easter(year).unwrap(), expected, "{}", row.id());
    }
}

#[test]
fn every_modeled_target_reference_date_matches_without_promoting_projection_coverage() {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/market/target-membership.fixture",
    ))
    .unwrap();
    assert_eq!(fixture.header("reference-version"), Some("1.43"));
    assert_eq!(fixture.record_count(), 73_411);
    let strict = TargetCalendar::new();
    let projected = TargetCalendar::with_policy(TargetPolicy::AllowProjection);
    let mut verified = 0;
    for row in fixture.records() {
        let date = row.parse::<Date>("date").unwrap();
        let expected = row.parse::<u32>("oracle_business").unwrap() == 1;
        assert_eq!(
            expected,
            row.parse::<u32>("reference_business").unwrap() == 1
        );
        assert_eq!(
            projected.is_business_day(date).unwrap(),
            expected,
            "{}",
            row.id()
        );
        match row.text("coverage").unwrap() {
            "Verified" => {
                verified += 1;
                assert_eq!(strict.coverage(date), TargetCoverage::Verified);
                assert_eq!(strict.is_business_day(date).unwrap(), expected);
            }
            "Projected" => {
                assert_eq!(strict.coverage(date), TargetCoverage::Projected);
                assert_eq!(
                    strict.is_business_day(date).unwrap_err().kind(),
                    "calendar.unverified_coverage"
                );
            }
            label => panic!("unrecognized coverage {label:?}"),
        }
        assert_eq!(projected.coverage(date), strict.coverage(date));
    }
    assert_eq!(verified, 2554);
}

#[test]
fn contextual_daycounts_match_exact_oracles_and_narrow_reference_differences() {
    let fixture = Fixture::load(Fixture::repository_path(
        "validation/fixtures/market/extended-daycounts.fixture",
    ))
    .unwrap();
    assert_eq!(fixture.header("reference-version"), Some("1.43"));
    assert_eq!(fixture.record_count(), 62_138);
    let weekends = Business252::new(Calendar::weekends_only());
    let target =
        Business252::with_target(TargetCalendar::with_policy(TargetPolicy::AllowProjection));
    let mut differences = [0; 4];
    let mut categories = [0; 4];
    for row in fixture.records() {
        let start = row.parse::<Date>("start").unwrap();
        let end = row.parse::<Date>("end").unwrap();
        let name = row.text("convention").unwrap();
        let (days, fraction, category) = match name {
            "30/360 (Bond Basis)" => (
                DayCount::Thirty360BondBasis.day_count(start, end),
                DayCount::Thirty360BondBasis
                    .year_fraction(start, end)
                    .unwrap(),
                0,
            ),
            "30E/360 (ISDA)" => {
                let convention = ThirtyE360Isda::new(row.parse("termination").unwrap());
                (
                    convention.day_count(start, end),
                    convention.year_fraction(start, end).unwrap(),
                    1,
                )
            }
            "Business/252" => {
                let (convention, category) = match row.text("calendar").unwrap() {
                    "WeekendsOnly" => (&weekends, 2),
                    "TARGET" => (&target, 3),
                    value => panic!("unrecognized calendar {value:?}"),
                };
                (
                    convention.day_count(start, end).unwrap(),
                    convention.year_fraction(start, end).unwrap(),
                    category,
                )
            }
            value => panic!("unrecognized convention {value:?}"),
        };
        categories[category] += 1;
        let oracle_days = row.parse::<i32>("oracle_days").unwrap();
        let reference_days = row.parse::<i32>("reference_days").unwrap();
        assert_eq!(days, oracle_days, "{}", row.id());
        assert_close(
            fraction,
            row.real("oracle_fraction").unwrap(),
            Tolerance::Ulps(0),
        );
        if days == 0 {
            assert_eq!(fraction.to_bits(), 0);
        }
        match row.text("discrepancy").unwrap() {
            "" => {
                assert_eq!(days, reference_days);
                assert_close(
                    fraction,
                    row.real("reference_fraction").unwrap(),
                    Tolerance::Ulps(0),
                );
            }
            "D9-BOND-SIGNED" => {
                differences[0] += 1;
                assert_eq!(category, 0);
                assert!(start > end);
                assert_ne!(days, reference_days);
            }
            "D9-30E-ISDA-SIGNED" => {
                differences[1] += 1;
                assert_eq!(category, 1);
                assert!(start > end);
                assert_ne!(days, reference_days);
            }
            "D9-30E-ISDA-ZERO" => {
                differences[2] += 1;
                assert_eq!(category, 1);
                assert_eq!(start, end);
                assert_eq!(end, row.parse::<Date>("termination").unwrap());
                assert_eq!(end.month(), 2);
                assert!(end.is_end_of_month());
                assert_eq!(days, 0);
                assert_eq!(reference_days, i32::try_from(end.day()).unwrap() - 30);
            }
            "D9-B252-SIGNED" => {
                differences[3] += 1;
                assert!(category >= 2);
                assert!(start > end);
                assert_eq!((days - reference_days).abs(), 1);
            }
            value => panic!("unclassified difference {value:?}"),
        }
    }
    assert_eq!(differences, [241, 103, 10, 3375]);
    assert_eq!(categories[0], categories[1]);
    assert_eq!(categories[2], categories[3]);
    assert!(categories.iter().all(|&count| count > 10_000));
}
