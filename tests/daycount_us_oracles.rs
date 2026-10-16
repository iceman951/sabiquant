//! Independent explicit US integer/rational oracle (FS-DCC §9.7; OQ-15).
//!
//! TS-DCC-01/04/07: rotating full-cycle coverage, every supported year,
//! original February predicates, exact signed integers and rounded rationals.
//! Financial scope remains the separately pinned local contract. Neither
//! production US source nor QuantLib output supplied expected results.

mod support;

use sabiquant::daycount::Thirty360Us;
use sabiquant::time::Date;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use support::fixture::Fixture;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn fnv64(bytes: &[u8]) -> String {
    let value = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    });
    format!("{value:016x}")
}

fn header_counts(fixture: &Fixture, key: &str) -> TestResult<BTreeMap<String, usize>> {
    fixture
        .header(key)
        .ok_or_else(|| format!("missing {key}"))?
        .split(';')
        .map(|item| {
            let (name, count) = item.split_once('=').ok_or("malformed count")?;
            Ok((name.to_owned(), count.parse()?))
        })
        .collect()
}

fn bump(map: &mut BTreeMap<String, usize>, key: &str) {
    *map.entry(key.to_owned()).or_default() += 1;
}

fn check_pin(fixture: &Fixture, key: &str, bytes: &[u8]) {
    assert_eq!(fixture.header(key), Some(fnv64(bytes).as_str()), "{key}");
}

fn fixture() -> TestResult<Fixture> {
    let path = Fixture::repository_path("validation/fixtures/daycount/us-30360-oracle.fixture");
    #[expect(
        clippy::disallowed_methods,
        reason = "test verifies the committed independent fixture's raw body fingerprint"
    )]
    let text = std::fs::read_to_string(path)?;
    let (_, body) = text.split_once("\n\n").ok_or("fixture separator")?;
    let fixture = Fixture::parse(&text)?;
    assert_eq!(
        fixture.header("body-fnv64"),
        Some(fnv64(body.as_bytes()).as_str())
    );
    assert_eq!(fixture.header("body-fnv64"), Some("2cfd06e603b8aba2"));
    assert_eq!(
        fixture.header("body-sha256"),
        Some("1098bbad2cacb39a60fec7689f017e6c995c5e7e7302179299e438bc741f2dfc")
    );
    check_pin(
        &fixture,
        "adopted-contract-fnv64",
        include_bytes!("../docs/M3_US_AFB_CONTRACT.md"),
    );
    check_pin(
        &fixture,
        "source-review-fnv64",
        include_bytes!("../docs/M3_THIRTY360_US_SOURCE_REVIEW.md"),
    );
    check_pin(
        &fixture,
        "generator-fnv64",
        include_bytes!("../validation/generator/daycount_us_oracles.py"),
    );
    assert_eq!(
        fixture.header("fixture-set"),
        Some("US-30360-INTEGER-RATIONAL-ORACLE")
    );
    assert!(
        fixture
            .header("scope")
            .ok_or("scope")?
            .contains("book unread")
    );
    Ok(fixture)
}

#[test]
fn explicit_us_flags_match_every_frozen_integer_and_rounded_rational() -> TestResult {
    let fixture = fixture()?;
    assert_eq!(fixture.record_count(), 393_158);
    assert_eq!(fixture.header("record-count"), Some("393158"));
    let mut profiles = BTreeMap::new();
    let mut directions = BTreeMap::new();
    let mut zeros = BTreeMap::new();
    let mut flags = BTreeMap::new();
    let mut inputs = BTreeSet::new();
    let mut cycle = [vec![false; 146_097], vec![false; 146_097]];
    let mut years = BTreeMap::<String, Vec<[bool; 2]>>::new();
    let mut previous = None;
    let mut different_pairs = 0;
    let cycle_start: Date = "2000-01-01".parse()?;
    let cycle_end: Date = "2399-12-31".parse()?;

    for (index, row) in fixture.records().enumerate() {
        let id = row.id();
        let profile = row.text("profile")?;
        let anchor: Date = row.parse("anchor")?;
        let start: Date = row.parse("start")?;
        let end: Date = row.parse("end")?;
        let eom: usize = row.parse("eom")?;
        assert_eq!(eom, index % 2, "{id}: alternating explicit flags");
        let context = Thirty360Us::new(eom == 1);
        assert_eq!(context.end_of_month_security(), eom == 1, "{id}");
        assert_eq!(context.canonical_identifier(), "30/360 (US)", "{id}");
        let expected_days: i32 = row.parse("days")?;
        let numerator: i64 = row.parse("numerator")?;
        let denominator: i64 = row.parse("denominator")?;
        assert!(denominator > 0 && 360 % denominator == 0, "{id}");
        assert_eq!(
            numerator * 360,
            i64::from(expected_days) * denominator,
            "{id}"
        );
        let expected_fraction = row.real_bits("fraction")?;
        let actual_days = context.day_count(start, end);
        assert_eq!(actual_days, expected_days, "{id}");
        let actual_fraction = context.year_fraction(start, end)?;
        assert!(actual_fraction.is_finite(), "{id}");
        assert_eq!(
            actual_fraction.to_bits(),
            expected_fraction.to_bits(),
            "{id}: TOL-E1"
        );
        if expected_days == 0 {
            assert_eq!(actual_fraction.to_bits(), 0, "{id}: every zero is positive");
        }

        bump(&mut profiles, profile);
        bump(&mut flags, row.text("eom")?);
        let direction = if start == end {
            "equal"
        } else if start < end {
            "forward"
        } else {
            "reverse"
        };
        bump(&mut directions, direction);
        let zero_class = if start == end {
            "equal"
        } else if expected_days != 0 {
            "nonzero"
        } else if start < end {
            "synthetic-forward"
        } else {
            "synthetic-reverse"
        };
        bump(&mut zeros, zero_class);
        inputs.insert((start, end, eom));

        if let Some(code) = profile.strip_prefix('C') {
            assert!((cycle_start..=cycle_end).contains(&anchor), "{id}");
            let offset = usize::try_from(anchor.days_since(cycle_start))?;
            assert_eq!(code.parse::<usize>()?, offset % 12, "{id}");
            assert!(!cycle[eom][offset], "{id}: duplicate cycle anchor/flag");
            cycle[eom][offset] = true;
        } else if profile.starts_with("Y-") {
            let year = usize::try_from(anchor.year())?;
            let seen = years
                .entry(profile.to_owned())
                .or_insert_with(|| vec![[false; 2]; 10_000]);
            assert!(!seen[year][eom], "{id}: duplicate year class/flag");
            seen[year][eom] = true;
            // The generator deliberately alternates direction across years.
            assert_eq!(start > end, year % 2 == 1, "{id}");
        }

        if eom == 0 {
            previous = Some((profile.to_owned(), anchor, start, end, expected_days));
        } else {
            let (old_profile, old_anchor, old_start, old_end, old_days) =
                previous.take().ok_or("unpaired EOM row")?;
            assert_eq!(
                (old_profile.as_str(), old_anchor, old_start, old_end),
                (profile, anchor, start, end),
                "{id}"
            );
            different_pairs += usize::from(old_days != expected_days);
        }
    }
    assert!(previous.is_none());
    assert_eq!(inputs.len(), 392_772);
    assert_eq!(fixture.header("unique-input-count"), Some("392772"));
    assert_eq!(profiles, header_counts(&fixture, "profile-counts")?);
    assert_eq!(directions, header_counts(&fixture, "direction-counts")?);
    assert_eq!(zeros, header_counts(&fixture, "zero-class-counts")?);
    assert_eq!(flags, header_counts(&fixture, "eom-counts")?);
    assert_eq!(different_pairs, 26_164);
    assert_eq!(fixture.header("eom-different-pair-count"), Some("26164"));
    assert_eq!(directions["equal"], 26_128);
    assert_eq!(directions["forward"], 205_578);
    assert_eq!(directions["reverse"], 161_452);
    assert_eq!(zeros["synthetic-forward"], 996);
    assert_eq!(zeros["synthetic-reverse"], 472);
    assert!(cycle.iter().all(|seen| seen.iter().all(|value| *value)));
    assert_eq!(years.len(), 5);
    for seen in years.values() {
        assert_eq!(seen[0], [false, false]);
        assert!(seen[1..].iter().all(|flags| *flags == [true, true]));
    }
    Ok(())
}

/// Independent endpoint record: no US production helpers or adjustments.
#[derive(Clone, Copy)]
struct Endpoint {
    date: Date,
    month_coordinate: i32,
    day: i32,
    february_end: bool,
}

impl Endpoint {
    fn new(date: Date) -> TestResult<Self> {
        let year = date.year();
        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        Ok(Self {
            date,
            month_coordinate: year * 360 + (i32::try_from(date.month())? - 1) * 30,
            day: i32::try_from(date.day())?,
            february_end: date.month() == 2 && date.day() == if leap { 29 } else { 28 },
        })
    }
}

/// Select endpoint representations from original date categories; no ordered
/// mutation of dates/day values. Ascending inputs only. This cross-language
/// oracle is independently written from the frozen mathematical contract.
fn endpoint_count(first: Endpoint, last: Endpoint, eom: bool) -> i32 {
    let first_day = match (eom, first.february_end, first.day) {
        (true, true, _) | (_, _, 31) => 30,
        (_, _, day) => day,
    };
    let last_day = match (
        eom && first.february_end && last.february_end,
        last.day,
        first_day,
    ) {
        (true, _, _) | (_, 31, 30) => 30,
        (_, day, _) => day,
    };
    last.month_coordinate + last_day - first.month_coordinate - first_day
}

fn check_query(context: Thirty360Us, start: Date, end: Date, count: i32) -> TestResult {
    assert_eq!(
        context.day_count(start, end),
        count,
        "{context:?} {start}..{end}"
    );
    let expected = f64::from(count) / 360.0;
    assert_eq!(
        context.year_fraction(start, end)?.to_bits(),
        expected.to_bits(),
        "TOL-E1 {context:?} {start}..{end}"
    );
    Ok(())
}

#[test]
fn us_both_flags_exhaust_all_pairs_in_required_leap_and_century_windows() -> TestResult {
    // TS-DCC-07: every ORIENTED pair, equal dates included, in each named
    // closed civil window. Dates are enumerated first, not sampled. No
    // outside-year endpoints enter this census, and no fixture is expanded.
    let mut query_count = 0_u64;
    for (begin, finish, expected_length) in [
        ("1999-01-01", "2002-12-31", 1461_usize),
        ("2099-01-01", "2101-12-31", 1095_usize),
    ] {
        let begin: Date = begin.parse()?;
        let finish: Date = finish.parse()?;
        let dates: Vec<Endpoint> = (0..=finish.days_since(begin))
            .map(|offset| Endpoint::new(begin.checked_add_days(i64::from(offset))?))
            .collect::<TestResult<_>>()?;
        assert_eq!(dates.len(), expected_length);
        let before = query_count;
        for (index, first) in dates.iter().copied().enumerate() {
            for last in dates[index..].iter().copied() {
                for eom in [false, true] {
                    let context = Thirty360Us::new(eom);
                    let expected = endpoint_count(first, last, eom);
                    check_query(context, first.date, last.date, expected)?;
                    query_count += 1;
                    if first.date != last.date {
                        check_query(context, last.date, first.date, -expected)?;
                        query_count += 1;
                    }
                }
            }
        }
        let length = u64::try_from(dates.len())?;
        assert_eq!(query_count - before, 2 * length * length);
    }
    assert_eq!(query_count, 6_667_092);
    Ok(())
}
