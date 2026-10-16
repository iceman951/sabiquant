//! F-DCC / TS-DCC: independent ISDA-clarified AFB dates and exact rationals.
//!
//! Neither this suite nor its fixture generator reads production AFB code.
//! Python actually removes years; this Rust oracle separately uses a manual
//! Gregorian month table, repeated annual steps and a civil-day membership
//! walk. Discrete oracle agreement is E0; public fractions use TOL-E2 (4 ULP).

#![expect(
    clippy::expect_used,
    reason = "independent oracle helpers fail immediately on invalid fixture inputs or broken civil invariants"
)]

mod support;

use sabiquant::daycount::DayCount;
use sabiquant::time::Date;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::error::Error;
use support::fixture::Fixture;
use support::float::ulp_distance;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
type Civil = (i32, u32, u32);
const AFB: DayCount = DayCount::ActualActualAfb;
const DURATIONS: [i32; 16] = [
    0, 1, 2, 28, 59, 60, 90, 180, 364, 365, 366, 367, 730, 1096, 1461, 2922,
];

fn date(text: &str) -> Date {
    text.parse().expect("valid independently specified date")
}

fn leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn months(year: i32) -> [u32; 12] {
    [
        31,
        if leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ]
}

#[derive(Debug, PartialEq, Eq)]
struct Decomposition {
    years: u32,
    remaining: Civil,
    days: u32,
    denominator: u32,
}

/// Prefixes are constructed year by year from our month table, without Date
/// serial arithmetic. AFB itself still performs every annual backward step.
struct CivilOracle {
    completed: Vec<i64>,
}

impl CivilOracle {
    fn new() -> Self {
        let mut completed = vec![0];
        let mut sum = 0;
        for year in 1..=9999 {
            sum += i64::from(months(year).iter().sum::<u32>());
            completed.push(sum);
        }
        Self { completed }
    }

    fn ordinal(&self, (year, month, day): Civil) -> i64 {
        let before_year = usize::try_from(year - 1).expect("civil year index");
        let before_month = usize::try_from(month - 1).expect("civil month index");
        self.completed[before_year]
            + i64::from(months(year)[..before_month].iter().sum::<u32>())
            + i64::from(day)
    }

    fn next_day((year, month, day): Civil) -> Civil {
        let limit = months(year)[usize::try_from(month - 1).expect("civil month index")];
        if day < limit {
            (year, month, day + 1)
        } else if month < 12 {
            (year, month + 1, 1)
        } else {
            assert!(year < 9999, "membership walk never passes Date::MAX");
            (year + 1, 1, 1)
        }
    }

    fn rollback(start: Civil, end: Civil) -> (u32, Civil) {
        assert!(start <= end);
        let mut remaining = end;
        let mut years = 0;
        while remaining.0 > 1 {
            let target_year = remaining.0 - 1;
            let (month, mut day) = (remaining.1, remaining.2);
            if month == 2 && day == 29 && !leap(target_year) {
                day = 28;
            }
            if month == 2 && day == 28 && leap(target_year) {
                day = 29;
            }
            let candidate = (target_year, month, day);
            if candidate < start {
                break;
            }
            assert!(candidate < remaining);
            remaining = candidate;
            years += 1;
        }
        (years, remaining)
    }

    fn decompose(&self, start: Civil, end: Civil) -> Decomposition {
        let (years, remaining) = Self::rollback(start, end);
        // Walk every actual residual day, including S and excluding E*.
        // This differs from the Python generator's per-year leap-date query.
        let mut cursor = start;
        let mut contains_feb29 = false;
        let mut days = 0;
        while cursor < remaining {
            contains_feb29 |= cursor.1 == 2 && cursor.2 == 29;
            cursor = Self::next_day(cursor);
            days += 1;
        }
        assert_eq!(
            i64::from(days),
            self.ordinal(remaining) - self.ordinal(start)
        );
        Decomposition {
            years,
            remaining,
            days,
            denominator: if contains_feb29 { 366 } else { 365 },
        }
    }

    /// The exhaustive multi-year pair matrix uses the same repeated annual
    /// moves but asks about each residual year's actual February 29 tuple.
    /// Avoiding a per-day residual walk here keeps the full Cartesian matrix
    /// practical; the fixture test independently retains that day walk.
    fn decompose_by_leap_dates(&self, start: Civil, end: Civil) -> Decomposition {
        let (years, remaining) = Self::rollback(start, end);
        let contains_feb29 = (start.0..=remaining.0)
            .filter(|year| leap(*year))
            .map(|year| (year, 2, 29))
            .any(|day| start <= day && day < remaining);
        Decomposition {
            years,
            remaining,
            days: u32::try_from(self.ordinal(remaining) - self.ordinal(start))
                .expect("nonnegative civil residual"),
            denominator: if contains_feb29 { 366 } else { 365 },
        }
    }
}

fn gcd(mut left: u32, mut right: u32) -> u32 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}

fn fraction(parts: &Decomposition) -> (u32, u32, f64) {
    // These integers are below 2^53. One division therefore independently
    // rounds the exact rational, rather than copying N + rounded(stub / B).
    let numerator = parts.years * parts.denominator + parts.days;
    let common = gcd(numerator, parts.denominator);
    let (num, den) = (numerator / common, parts.denominator / common);
    (num, den, f64::from(num) / f64::from(den))
}

fn assert_fraction(start: Date, end: Date, expected: f64, label: &str) -> u64 {
    let actual = AFB.year_fraction(start, end).expect("valid AFB dates");
    let distance = ulp_distance(actual, expected).expect("finite values");
    assert!(
        actual.is_finite() && distance <= 4,
        "{label}: {start}->{end}: {actual:?}, expected {expected:?}, {distance} ULP"
    );
    if expected == 0.0 {
        assert_eq!(actual.to_bits(), 0, "{label}: canonical positive zero");
    }
    distance
}

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn counts(fixture: &Fixture, key: &str) -> TestResult<BTreeMap<String, usize>> {
    fixture
        .header(key)
        .ok_or_else(|| format!("missing {key}"))?
        .split(';')
        .map(|item| {
            let (name, count) = item.split_once('=').ok_or("malformed census entry")?;
            Ok((name.to_owned(), count.parse()?))
        })
        .collect()
}

fn increment(census: &mut BTreeMap<String, usize>, key: impl ToString) {
    *census.entry(key.to_string()).or_default() += 1;
}

#[expect(
    clippy::print_stderr,
    reason = "focused validation records the measured maximum ULP without relaxing its tolerance"
)]
fn report_measurement(rows: usize, year_steps: u64, maximum_ulp: u64) {
    eprintln!(
        "AFB: {rows} rows; {year_steps} actual accepted year steps; maximum {maximum_ulp} ULP"
    );
}

#[test]
fn repeated_year_and_residual_day_oracles_match_all_frozen_rows() -> TestResult {
    #[expect(
        clippy::disallowed_methods,
        reason = "tests read committed oracle fixtures and validate their body fingerprint"
    )]
    let text = std::fs::read_to_string(Fixture::repository_path(
        "validation/fixtures/daycount/afb-oracle.fixture",
    ))?;
    let fixture = Fixture::parse(&text)?;
    assert_eq!(fixture.header("fixture-set"), Some("DAYCOUNT-AFB-ORACLE"));
    let (_, body) = text.split_once("\n\n").ok_or("missing fixture separator")?;
    for (key, bytes) in [
        ("body-fnv64", body.as_bytes()),
        (
            "adopted-contract-fnv64",
            &include_bytes!("../docs/M3_US_AFB_CONTRACT.md")[..],
        ),
        (
            "source-review-fnv64",
            &include_bytes!("../docs/M3_AFB_SOURCE_REVIEW.md")[..],
        ),
        (
            "generator-fnv64",
            &include_bytes!("../validation/generator/daycount_afb_oracles.py")[..],
        ),
    ] {
        let expected = u64::from_str_radix(fixture.header(key).ok_or("missing pin")?, 16)?;
        assert_eq!(fnv64(bytes), expected, "{key}");
    }
    assert_eq!(fixture.record_count(), 215_327);
    assert_eq!(fixture.header("record-count"), Some("215327"));
    let oracle = CivilOracle::new();
    let mut profiles = BTreeMap::new();
    let mut denominators = BTreeMap::new();
    let mut whole_years = BTreeMap::new();
    let mut inputs = BTreeSet::new();
    let mut cycle_seen = vec![false; 146_097];
    let mut all_years = BTreeSet::new();
    let mut anniversaries = BTreeSet::new();
    let mut year_steps = 0_u64;
    let mut maximum_stub = 0;
    let mut maximum_ulp = 0;
    let mut zero_rows = 0;
    let mut wide_state = 0xafb1_999c_1a41_f1ed_u64;
    let maximum_ordinal = u64::try_from(oracle.ordinal(Date::MAX.to_ymd()))?;
    for row in fixture.records() {
        let id = row.id();
        let profile = row.text("profile")?;
        let start: Date = row.parse("start")?;
        let end: Date = row.parse("end")?;
        let expected_days =
            i32::try_from(oracle.ordinal(end.to_ymd()) - oracle.ordinal(start.to_ymd()))?;
        let parts = oracle.decompose(start.to_ymd(), end.to_ymd());
        assert_eq!(row.parse::<i32>("actual_days")?, expected_days, "{id}");
        assert_eq!(row.parse::<u32>("whole_years")?, parts.years, "{id}");
        assert_eq!(
            row.parse::<Date>("residual_end")?.to_ymd(),
            parts.remaining,
            "{id}"
        );
        assert_eq!(row.parse::<u32>("stub_days")?, parts.days, "{id}");
        assert_eq!(row.parse::<u32>("denominator")?, parts.denominator, "{id}");
        let (num, den, exact) = fraction(&parts);
        assert_eq!(row.parse::<u32>("fraction_numerator")?, num, "{id}");
        assert_eq!(row.parse::<u32>("fraction_denominator")?, den, "{id}");
        assert_eq!(
            row.real_bits("oracle_fraction")?.to_bits(),
            exact.to_bits(),
            "{id}: rational rounding"
        );
        assert_eq!(AFB.day_count(start, end), expected_days, "{id}");
        assert_eq!(AFB.day_count(end, start), -expected_days, "{id}");
        maximum_ulp = maximum_ulp.max(assert_fraction(start, end, exact, id));
        maximum_ulp = maximum_ulp.max(assert_fraction(
            end,
            start,
            if exact == 0.0 { 0.0 } else { -exact },
            id,
        ));
        let forward = AFB.year_fraction(start, end)?;
        let reverse = AFB.year_fraction(end, start)?;
        assert_eq!(
            reverse.to_bits(),
            if forward == 0.0 {
                0
            } else {
                (-forward).to_bits()
            },
            "{id}: reversal"
        );
        assert_eq!(AFB.year_fraction(start, start)?.to_bits(), 0, "{id}");
        assert_eq!(AFB.day_count(start, start), 0, "{id}");
        zero_rows += usize::from(start == end);
        if let Some(duration) = profile.strip_prefix("cycle-plus") {
            let offset =
                usize::try_from(oracle.ordinal(start.to_ymd()) - oracle.ordinal((2000, 1, 1)))?;
            assert!(offset < cycle_seen.len() && !cycle_seen[offset], "{id}");
            cycle_seen[offset] = true;
            assert_eq!(
                duration.parse::<i32>()?,
                DURATIONS[offset % DURATIONS.len()],
                "{id}"
            );
            assert_eq!(expected_days, DURATIONS[offset % DURATIONS.len()], "{id}");
        } else if profile.starts_with("all-years-anniversary") {
            assert_eq!(end.year(), start.year() + 1, "{id}");
            assert!(anniversaries.insert(end.year()), "{id}");
        } else if profile.starts_with("all-years-") {
            assert_eq!(start.year(), end.year(), "{id}");
            assert!(all_years.insert((profile.to_owned(), start.year())), "{id}");
        } else if profile == "wide-deterministic" {
            let mut pair = [0_u64; 2];
            for ordinal in &mut pair {
                wide_state = wide_state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                *ordinal = wide_state % maximum_ordinal + 1;
            }
            pair.sort();
            assert_eq!(
                [
                    u64::try_from(oracle.ordinal(start.to_ymd()))?,
                    u64::try_from(oracle.ordinal(end.to_ymd()))?
                ],
                pair,
                "{id}"
            );
        }
        increment(&mut profiles, profile);
        increment(&mut denominators, parts.denominator);
        increment(
            &mut whole_years,
            if parts.years == 0 { "zero" } else { "positive" },
        );
        year_steps += u64::from(parts.years);
        maximum_stub = maximum_stub.max(parts.days);
        inputs.insert((start, end));
    }
    assert!(cycle_seen.into_iter().all(|seen| seen));
    assert_eq!(all_years.len(), 5 * 9999);
    assert_eq!(anniversaries, (2..=9999).collect());
    assert_eq!(profiles, counts(&fixture, "profile-counts")?);
    assert_eq!(denominators, counts(&fixture, "denominator-counts")?);
    assert_eq!(whole_years, counts(&fixture, "whole-year-counts")?);
    assert_eq!(inputs.len(), 205_673);
    assert_eq!(fixture.header("unique-input-count"), Some("205673"));
    assert_eq!(year_steps, 14_743_934);
    assert_eq!(fixture.header("accepted-year-steps"), Some("14743934"));
    assert_eq!(maximum_stub, 365);
    assert_eq!(fixture.header("maximum-stub-days"), Some("365"));
    assert_eq!(zero_rows, 19_621);
    report_measurement(fixture.record_count(), year_steps, maximum_ulp);
    Ok(())
}

#[test]
fn primary_1999_printed_afb_fraction_anchors() {
    // ISDA 1999 pp.2–6, independently visually inspected; rational amounts,
    // not rounded currency samples. Requirements §9.6 / OQ-16 / QR-04.
    for (start, end, numerator, denominator) in [
        ("2003-11-01", "2004-05-01", 182, 366),
        ("1999-02-01", "1999-07-01", 150, 365),
        ("1999-07-01", "2000-07-01", 366, 366),
        ("2002-08-15", "2003-07-15", 334, 365),
        ("2000-01-30", "2000-06-30", 152, 366),
    ] {
        let (start, end) = (date(start), date(end));
        let expected = f64::from(numerator) / f64::from(denominator);
        assert_eq!(AFB.day_count(start, end), numerator);
        assert_eq!(
            AFB.year_fraction(start, end)
                .expect("valid primary pair")
                .to_bits(),
            expected.to_bits()
        );
        assert_eq!(
            AFB.year_fraction(end, start)
                .expect("valid reverse")
                .to_bits(),
            (-expected).to_bits()
        );
    }
}

#[test]
fn ts_dcc_07_every_pair_in_the_two_required_multiyear_windows() {
    let oracle = CivilOracle::new();
    let mut total = 0_u64;
    let mut year_steps = 0_u64;
    let mut maximum_ulp = 0;
    for (first, after_last, expected_days, expected_pairs) in [
        ((1999, 1, 1), (2003, 1, 1), 1461, 1_067_991),
        ((2099, 1, 1), (2102, 1, 1), 1095, 600_060),
    ] {
        let mut dates = Vec::new();
        let mut cursor = first;
        while cursor < after_last {
            dates.push((
                cursor,
                Date::from_ymd(cursor.0, cursor.1, cursor.2).expect("manual civil date"),
            ));
            cursor = CivilOracle::next_day(cursor);
        }
        assert_eq!(dates.len(), expected_days);
        let mut pairs = 0;
        for (index, &(start_civil, start)) in dates.iter().enumerate() {
            for &(end_civil, end) in &dates[index..] {
                let parts = oracle.decompose_by_leap_dates(start_civil, end_civil);
                year_steps += u64::from(parts.years);
                let expected = fraction(&parts).2;
                let actual_days =
                    i32::try_from(oracle.ordinal(end_civil) - oracle.ordinal(start_civil))
                        .expect("small exact window count");
                assert_eq!(AFB.day_count(start, end), actual_days);
                assert_eq!(AFB.day_count(end, start), -actual_days);
                maximum_ulp = maximum_ulp.max(assert_fraction(start, end, expected, "TS-DCC-07"));
                maximum_ulp = maximum_ulp.max(assert_fraction(
                    end,
                    start,
                    if expected == 0.0 { 0.0 } else { -expected },
                    "TS-DCC-07 reverse",
                ));
                pairs += 1;
            }
        }
        assert_eq!(pairs, expected_pairs);
        total += pairs;
    }
    assert_eq!(total, 1_668_051);
    report_measurement(
        usize::try_from(total).expect("bounded matrix count"),
        year_steps,
        maximum_ulp,
    );
}

#[test]
fn february_roll_and_half_open_residuals_are_distinguished() {
    let oracle = CivilOracle::new();
    for (start, end, years, remaining, days, denominator) in [
        ("2024-02-28", "2025-02-28", 1, "2024-02-29", 1, 365),
        ("2023-02-28", "2024-02-28", 1, "2023-02-28", 0, 365),
        ("2024-02-29", "2025-02-28", 1, "2024-02-29", 0, 365),
        ("2023-02-28", "2024-02-29", 1, "2023-02-28", 0, 365),
        ("2024-02-28", "2024-02-29", 0, "2024-02-29", 1, 365),
        ("2024-02-27", "2024-02-28", 0, "2024-02-28", 1, 365),
        ("2024-01-01", "2024-02-28", 0, "2024-02-28", 58, 365),
        ("2024-02-28", "2024-03-01", 0, "2024-03-01", 2, 366),
        ("2024-02-29", "2024-03-01", 0, "2024-03-01", 1, 366),
        ("2023-03-01", "2024-02-29", 0, "2024-02-29", 365, 365),
        ("2024-02-28", "2028-02-28", 4, "2024-02-29", 1, 365),
        ("1994-02-10", "1997-06-30", 3, "1994-06-30", 140, 365),
        ("2023-12-31", "2024-02-29", 0, "2024-02-29", 60, 365),
        ("2023-12-31", "2024-03-01", 0, "2024-03-01", 61, 366),
        ("0001-01-01", "9999-12-31", 9998, "0001-12-31", 364, 365),
    ] {
        let (start, end) = (date(start), date(end));
        let expected = Decomposition {
            years,
            remaining: date(remaining).to_ymd(),
            days,
            denominator,
        };
        assert_eq!(oracle.decompose(start.to_ymd(), end.to_ymd()), expected);
        assert_fraction(start, end, fraction(&expected).2, "hand boundary");
    }
    assert_eq!(
        AFB.year_fraction(date("2023-03-01"), date("2024-02-29"))
            .expect("residual one"),
        1.0
    );
}

#[test]
fn source_clarified_afb_identifiers_have_no_generic_default() {
    assert_eq!(AFB.canonical_identifier(), "Actual/Actual (AFB)");
    assert_eq!(AFB.to_string(), "Actual/Actual (AFB)");
    assert_eq!(DayCount::ALL.len(), 6);
    assert_eq!(DayCount::ALL.into_iter().collect::<HashSet<_>>().len(), 6);
    assert!(DayCount::ALL.contains(&AFB));
    for text in ["Actual/Actual (AFB)", "Actual/Actual (Euro)"] {
        assert_eq!(text.parse::<DayCount>(), Ok(AFB));
    }
    for text in [
        "Actual/Actual",
        "Act/Act",
        "actual/actual (afb)",
        "Actual/Actual (afb)",
        "Actual/Actual AFB",
        " Actual/Actual (AFB)",
        "Actual/Actual (AFB) ",
        "Actual/Actual (Euro)\n",
        "30/360 (US)",
    ] {
        assert!(text.parse::<DayCount>().is_err(), "{text:?}");
    }
}

#[test]
fn anniversary_and_split_additivity_are_not_assumed() {
    let (start, cut, end) = (date("2024-02-28"), date("2024-02-29"), date("2025-02-28"));
    let whole = AFB.year_fraction(start, end).expect("whole");
    assert_fraction(start, end, 366.0 / 365.0, "promoted anniversary");
    assert_ne!(whole, 1.0);
    let left = AFB.year_fraction(start, cut).expect("left");
    let right = AFB.year_fraction(cut, end).expect("right");
    assert_eq!(left, 1.0 / 365.0);
    assert_eq!(right, 1.0);
    // This cut happens to compose. A different cut exposes non-additivity.
    assert_eq!(whole, left + right);
    let cut = date("2024-03-01");
    let split =
        AFB.year_fraction(start, cut).expect("left") + AFB.year_fraction(cut, end).expect("right");
    assert!(ulp_distance(whole, split).expect("finite") > 4);
}

#[test]
fn stateless_copy_context_is_reproducible_across_threads() {
    let pairs = [
        (date("2024-02-28"), date("2025-02-28"), 366.0 / 365.0),
        (date("2024-02-29"), date("2024-03-01"), 1.0 / 366.0),
        (Date::MIN, Date::MAX, 3_649_634.0 / 365.0),
        (Date::MAX, Date::MIN, -3_649_634.0 / 365.0),
        (Date::MAX, Date::MAX, 0.0),
    ];
    let baseline = pairs.map(|(start, end, _)| {
        AFB.year_fraction(start, end)
            .expect("sequential deterministic baseline")
            .to_bits()
    });
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..16)
            .map(|_| {
                scope.spawn(|| {
                    for _ in 0..32 {
                        for ((start, end, expected), bits) in pairs.into_iter().zip(baseline) {
                            assert_fraction(start, end, expected, "thread");
                            assert_eq!(
                                AFB.year_fraction(start, end)
                                    .expect("threaded deterministic result")
                                    .to_bits(),
                                bits
                            );
                        }
                    }
                })
            })
            .collect();
        for handle in handles {
            handle.join().expect("deterministic AFB worker");
        }
    });
}
