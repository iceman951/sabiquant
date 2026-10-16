//! TS-DCC-05/06, OQ-14/15/16: unchanged attested public observations and explicit differences.

mod support;

use sabiquant::daycount::{DayCount, Thirty360Us};
use sabiquant::time::Date;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use support::fixture::Fixture;
use support::float::ulp_distance;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn fnv64(bytes: &[u8]) -> String {
    format!(
        "{:016x}",
        bytes
            .iter()
            .fold(0xcbf2_9ce4_8422_2325_u64, |h, b| (h ^ u64::from(*b))
                .wrapping_mul(0x0000_0100_0000_01b3))
    )
}

fn bump(map: &mut BTreeMap<String, usize>, key: &str) {
    *map.entry(key.to_owned()).or_default() += 1;
}

fn counts(fixture: &Fixture, name: &str) -> TestResult<BTreeMap<String, usize>> {
    fixture
        .header(name)
        .ok_or("missing census")?
        .split(';')
        .map(|part| {
            let (key, value) = part.split_once('=').ok_or("census delimiter")?;
            Ok((key.to_owned(), value.parse()?))
        })
        .collect()
}

#[expect(
    clippy::unwrap_used,
    reason = "validated Date month/day components always fit the exact i32 reference formula"
)]
fn direct_us(start: Date, end: Date, eom: bool) -> i32 {
    let mut first = start.day();
    let mut last = end.day();
    let feb_end = |date: Date| {
        date.month() == 2
            && date.day()
                == if Date::is_leap_year(date.year()) {
                    29
                } else {
                    28
                }
    };
    if eom && feb_end(start) {
        if feb_end(end) {
            last = 30;
        }
        first = 30;
    }
    if last == 31 && first >= 30 {
        last = 30;
    }
    if first == 31 {
        first = 30;
    }
    360 * (end.year() - start.year())
        + 30 * (i32::try_from(end.month()).unwrap() - i32::try_from(start.month()).unwrap())
        + i32::try_from(last).unwrap()
        - i32::try_from(first).unwrap()
}

fn afb_parts(start: Date, end: Date) -> TestResult<(u32, Date, u32, u32)> {
    let first = start.min(end);
    let mut remaining = start.max(end);
    let mut years = 0;
    while remaining.year() > 1 {
        let target = remaining.year() - 1;
        let day = if remaining.month() == 2 && remaining.day() >= 28 {
            if Date::is_leap_year(target) { 29 } else { 28 }
        } else {
            remaining.day()
        };
        let candidate = Date::from_ymd(target, remaining.month(), day)?;
        if candidate < first {
            break;
        }
        years += 1;
        remaining = candidate;
    }
    let days = u32::try_from(remaining.days_since(first))?;
    let mut denominator = 365;
    for year in first.year()..=remaining.year() {
        if Date::is_leap_year(year) {
            let leap = Date::from_ymd(year, 2, 29)?;
            if first <= leap && leap < remaining {
                denominator = 366;
            }
        }
    }
    Ok((years, remaining, days, denominator))
}

#[test]
fn every_attested_observation_is_reclassified_against_the_local_operation() -> TestResult {
    let path = Fixture::repository_path("validation/fixtures/daycount/us-afb-probes.fixture");
    #[expect(
        clippy::disallowed_methods,
        reason = "read committed raw reference bytes for body fingerprint"
    )]
    let raw = std::fs::read_to_string(path)?;
    let fixture = Fixture::parse(&raw)?;
    assert_eq!(
        fixture.header("fixture-set"),
        Some("US-AFB-ATTESTED-REFERENCE-PROBES")
    );
    assert_eq!(fixture.record_count(), 741);
    assert_eq!(fixture.header("datepair-count"), Some("247"));
    assert_eq!(fixture.header("body-fnv64"), Some("76d2db8d5b1bdc8f"));
    assert_eq!(
        fixture.header("body-sha256"),
        Some("cce8244b329fcfa0b7c284e8e2b99dd35317e39cef17c5a6aa673b510f320be1")
    );
    assert_eq!(
        fixture.record_count(),
        fixture
            .header("record-count")
            .ok_or("record count")?
            .parse::<usize>()?
    );
    assert_eq!(
        fixture.header("quantlib-wheel-sha256"),
        Some("85d70b28778afca84b9b6ae74ef05a1069b2dca00e80fc74aa8ddf438259dc0c")
    );
    assert_eq!(
        fixture.header("quantlib-module-manifest-sha256"),
        Some("e0fa1b35187d100362a2f0feadc96ee118e411c152e35ed418f2c46dad767c2d")
    );
    assert_eq!(fixture.header("quantlib-module-files"), Some("3"));
    let (_, body) = raw.split_once("\n\n").ok_or("body delimiter")?;
    assert_eq!(
        fixture.header("body-fnv64"),
        Some(fnv64(body.as_bytes()).as_str())
    );
    for (key, bytes) in [
        (
            "contract-fnv64",
            &include_bytes!("../docs/M3_US_AFB_CONTRACT.md")[..],
        ),
        (
            "generator-fnv64",
            &include_bytes!("../validation/generator/daycount_us_afb_probes.py")[..],
        ),
        (
            "attestation-helper-fnv64",
            &include_bytes!("../validation/generator/schedule_probes.py")[..],
        ),
        (
            "us-oracle-helper-fnv64",
            &include_bytes!("../validation/generator/daycount_us_oracles.py")[..],
        ),
        (
            "us-source-fnv64",
            &include_bytes!("../docs/M3_THIRTY360_US_SOURCE_REVIEW.md")[..],
        ),
        (
            "afb-source-fnv64",
            &include_bytes!("../docs/M3_AFB_SOURCE_REVIEW.md")[..],
        ),
    ] {
        assert_eq!(fixture.header(key), Some(fnv64(bytes).as_str()), "{key}");
    }
    let mut classes = BTreeMap::new();
    let mut counters = BTreeMap::new();
    let mut groups = BTreeMap::new();
    let mut directions = BTreeMap::new();
    let mut maxima = BTreeMap::<String, usize>::new();
    let mut inputs = BTreeMap::<(Date, Date), BTreeSet<String>>::new();
    let mut ids = BTreeSet::new();
    for row in fixture.records() {
        assert!(ids.insert(row.id().to_owned()));
        let start: Date = row.parse("start")?;
        let end: Date = row.parse("end")?;
        let counter = row.text("counter")?;
        assert!(
            inputs
                .entry((start, end))
                .or_default()
                .insert(counter.to_owned())
        );
        let local_days: i32 = row.parse("local_days")?;
        let local_fraction = row.real_bits("local_fraction")?;
        let exact =
            f64::from(row.parse::<i32>("numerator")?) / f64::from(row.parse::<u32>("denominator")?);
        let reference_days: i32 = row.parse("reference_days")?;
        let status = row.text("reference_status")?;
        let class;
        if counter == "AFB" {
            let (years, remaining, days, denominator) = afb_parts(start, end)?;
            assert_eq!(row.parse::<u32>("whole_years")?, years);
            assert_eq!(row.parse::<Date>("residual_end")?, remaining);
            assert_eq!(row.parse::<u32>("residual_days")?, days);
            assert_eq!(row.parse::<u32>("residual_denominator")?, denominator);
            let sign = if start > end { -1.0 } else { 1.0 };
            let rational = sign * (f64::from(years * denominator + days) / f64::from(denominator));
            assert_eq!(rational.to_bits(), exact.to_bits());
            let ordered = f64::from(years) + f64::from(days) / f64::from(denominator);
            let ordered = if ordered == 0.0 { 0.0 } else { sign * ordered };
            assert_eq!(ordered.to_bits(), local_fraction.to_bits());
            assert_eq!(local_days, end.days_since(start));
            assert_eq!(reference_days, local_days);
            assert_eq!(DayCount::ActualActualAfb.day_count(start, end), local_days);
            assert!(
                ulp_distance(DayCount::ActualActualAfb.year_fraction(start, end)?, exact)
                    .ok_or("finite fraction")?
                    <= 4
            );
            if status == "raises" {
                assert_eq!(remaining.year(), 1901);
                assert_eq!(
                    row.text("reference_error")?,
                    "year 1900 out of bound. It must be in [1901,2199]"
                );
                assert_eq!(row.text("reference_fraction")?, "");
                assert_eq!(row.text("reference_ulp")?, "");
                class = "AFB-reference-lower-anchor";
            } else {
                assert_eq!(status, "value");
                let reference = row.real_bits("reference_fraction")?;
                let distance = ulp_distance(reference, local_fraction).ok_or("finite reference")?;
                assert_eq!(row.parse::<u64>("reference_ulp")?, distance);
                if reference == 0.0
                    && local_fraction == 0.0
                    && reference.to_bits() != local_fraction.to_bits()
                {
                    class = "reference-signed-zero";
                } else {
                    assert!(distance <= 8, "{}: AFB error {distance}", row.id());
                    class = "compatible-E2";
                    let maximum = maxima.entry(counter.to_owned()).or_default();
                    *maximum = (*maximum).max(usize::try_from(distance)?);
                }
            }
        } else {
            assert!(matches!(counter, "USA" | "BondBasis"));
            let eom = counter == "USA";
            let chronological = direct_us(start.min(end), start.max(end), eom);
            assert_eq!(
                local_days,
                if start > end {
                    -chronological
                } else {
                    chronological
                }
            );
            assert_eq!(status, "value");
            let actual = Thirty360Us::new(eom);
            assert_eq!(actual.day_count(start, end), local_days);
            assert_eq!(actual.year_fraction(start, end)?.to_bits(), exact.to_bits());
            assert_eq!(local_fraction.to_bits(), exact.to_bits());
            assert_eq!(reference_days, direct_us(start, end, eom));
            let reference = row.real_bits("reference_fraction")?;
            assert_eq!(
                reference.to_bits(),
                (f64::from(reference_days) / 360.0).to_bits()
            );
            let distance = ulp_distance(reference, local_fraction).ok_or("finite reference")?;
            assert_eq!(row.parse::<u64>("reference_ulp")?, distance);
            if reference_days != local_days {
                assert!(start > end);
                class = "US-direct-reverse-count";
            } else if local_days == 0 && reference.to_bits() != local_fraction.to_bits() {
                class = "reference-signed-zero";
            } else {
                assert_eq!(distance, 0);
                class = "compatible-E1";
                maxima.entry(counter.to_owned()).or_insert(0);
            }
        }
        assert_eq!(row.text("classification")?, class, "{}", row.id());
        if status == "value" {
            assert_eq!(row.text("reference_error")?, "");
        }
        bump(&mut classes, &format!("{counter}/{class}"));
        bump(&mut counters, counter);
        bump(&mut groups, row.text("group")?);
        bump(
            &mut directions,
            if start == end {
                "equal"
            } else if start < end {
                "forward"
            } else {
                "reverse"
            },
        );
    }
    for (name, actual) in [
        ("classification-counts", classes),
        ("counter-counts", counters),
        ("group-counts", groups),
        ("direction-counts", directions),
        ("compatible-max-ulp", maxima),
    ] {
        assert_eq!(counts(&fixture, name)?, actual, "{name}");
    }
    assert_eq!(
        inputs.len(),
        fixture
            .header("datepair-count")
            .ok_or("datepair count")?
            .parse::<usize>()?
    );
    for present in inputs.values() {
        assert_eq!(
            present,
            &BTreeSet::from(["USA".to_owned(), "BondBasis".to_owned(), "AFB".to_owned()])
        );
    }
    Ok(())
}
