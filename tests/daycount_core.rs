//! Independent validation of the four source-verified core day counts.
//!
//! Definitions: ISDA 2006 §§4.13/4.16(b,d,e,g), independently reviewed in
//! docs/M3_DAYCOUNT_RESEARCH.md. Expected values are rational formulas, not
//! copied software outputs or rounded decimal samples. The oracle accumulates
//! calendar days falling in common and leap years using its own month table.
//! It evaluates ISDA as one exact i128 rational, independently of production's
//! three-term floating-point summation. TOL-E1 = 0 ULP; TOL-E2 = 4 ULP.

#![expect(
    clippy::expect_used,
    reason = "test and independent-oracle helpers fail immediately on invalid test inputs or a broken oracle invariant"
)]

mod support;

use sabiquant::daycount::{DayCount, DayCountError};
use sabiquant::error::IdentifierError;
use sabiquant::time::Date;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use support::float::{Tolerance, assert_close};
use support::prng;

const TOL_E1: Tolerance = Tolerance::Ulps(0);
const TOL_E2: Tolerance = Tolerance::Ulps(4);
const ISDA_DENOMINATOR: i128 = 365 * 366;

fn date(year: i32, month: u32, day: u32) -> Date {
    Date::from_ymd(year, month, day).expect("valid test date")
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

/// Prefixes count actual civil days in leap/common years, separately.
/// The index is the number of completed years, not a Date serial number.
struct RationalOracle {
    common: Vec<i128>,
    leap: Vec<i128>,
}

impl RationalOracle {
    fn new() -> Self {
        let mut common = vec![0];
        let mut leap_days = vec![0];
        let mut common_count = 0;
        let mut leap_count = 0;
        for year in 1..=9999 {
            let length = i128::from(months(year).into_iter().sum::<u32>());
            if leap(year) {
                leap_count += length;
            } else {
                common_count += length;
            }
            common.push(common_count);
            leap_days.push(leap_count);
        }
        Self {
            common,
            leap: leap_days,
        }
    }

    fn before(&self, date: Date) -> (i128, i128) {
        let (year, month, day) = date.to_ymd();
        let completed_years = usize::try_from(year - 1).expect("valid year index");
        let completed_months = usize::try_from(month - 1).expect("valid month index");
        let ordinal_day =
            i128::from(months(year)[..completed_months].iter().sum::<u32>()) + i128::from(day - 1);
        let common = self.common[completed_years];
        let leap_days = self.leap[completed_years];
        if leap(year) {
            (common, leap_days + ordinal_day)
        } else {
            (common + ordinal_day, leap_days)
        }
    }

    fn counts(&self, start: Date, end: Date) -> (i32, i32) {
        let (start_common, start_leap) = self.before(start);
        let (end_common, end_leap) = self.before(end);
        let common_days = end_common - start_common;
        let leap_days = end_leap - start_leap;
        let actual =
            i32::try_from(common_days + leap_days).expect("supported actual range fits i32");
        let weighted = i32::try_from(common_days * 366 + leap_days * 365)
            .expect("supported weighted rational numerator fits i32");
        (actual, weighted)
    }

    fn fraction(&self, start: Date, end: Date) -> f64 {
        let numerator = self.counts(start, end).1;
        f64::from(numerator)
            / f64::from(i32::try_from(ISDA_DENOMINATOR).expect("small exact denominator"))
    }
}

fn european_ordinal(date: Date) -> i32 {
    let (year, month, day) = date.to_ymd();
    // Each civil date is independently projected onto a 30-day-month axis.
    // The 30th and 31st occupy the same point; February keeps its actual day.
    year * 360
        + i32::try_from(month - 1).expect("valid month") * 30
        + i32::try_from(day.min(30) - 1).expect("valid day")
}

fn check_pair(oracle: &RationalOracle, start: Date, end: Date) -> Result<(), String> {
    let (actual_days, _) = oracle.counts(start, end);
    for (convention, numerator, denominator) in [
        (DayCount::Actual360, actual_days, 360),
        (DayCount::Actual365Fixed, actual_days, 365),
        (
            DayCount::ThirtyE360,
            european_ordinal(end) - european_ordinal(start),
            360,
        ),
    ] {
        let actual_count = convention.day_count(start, end);
        if actual_count != numerator {
            return Err(format!(
                "{convention} {start}->{end}: count={actual_count}, expected={numerator}"
            ));
        }
        let fraction = convention
            .year_fraction(start, end)
            .map_err(|error| error.to_string())?;
        let expected = f64::from(numerator) / f64::from(denominator);
        TOL_E1
            .check(fraction, expected)
            .map_err(|report| format!("TOL-E1 {convention} {start}->{end}: {report}"))?;
        if numerator == 0 && fraction.to_bits() != 0 {
            return Err(format!(
                "{convention} {start}->{end}: expected canonical positive zero"
            ));
        }
    }
    if DayCount::ActualActualIsda.day_count(start, end) != actual_days {
        return Err(format!("ISDA {start}->{end}: actual day count differs"));
    }
    let fraction = DayCount::ActualActualIsda
        .year_fraction(start, end)
        .map_err(|error| error.to_string())?;
    TOL_E2
        .check(fraction, oracle.fraction(start, end))
        .map_err(|report| format!("TOL-E2 ISDA {start}->{end}: {report}"))?;
    if !fraction.is_finite() {
        return Err(format!("ISDA {start}->{end}: non-finite fraction"));
    }
    if start == end && fraction.to_bits() != 0 {
        return Err(format!(
            "ISDA {start}->{end}: expected canonical positive zero"
        ));
    }
    Ok(())
}

#[test]
fn source_defined_identifiers_aliases_and_rejections_are_exact() {
    let rows = [
        (DayCount::Actual360, "Actual/360", &["Act/360", "A/360"][..]),
        (
            DayCount::Actual365Fixed,
            "Actual/365 (Fixed)",
            &["Act/365 (Fixed)", "A/365 (Fixed)", "A/365F"][..],
        ),
        (
            DayCount::ActualActualIsda,
            "Actual/Actual (ISDA)",
            &["Act/Act (ISDA)", "Actual/Actual (Historical)"][..],
        ),
        (
            DayCount::ThirtyE360,
            "30E/360",
            &["30/360 (Eurobond Basis)"][..],
        ),
    ];
    assert_eq!(DayCount::ALL.len(), rows.len());
    assert_eq!(
        DayCount::ALL.into_iter().collect::<HashSet<_>>().len(),
        rows.len()
    );
    for (value, canonical, aliases) in rows {
        assert_eq!(value.canonical_identifier(), canonical);
        assert_eq!(value.to_string(), canonical);
        for accepted in std::iter::once(canonical).chain(aliases.iter().copied()) {
            assert_eq!(accepted.parse::<DayCount>(), Ok(value));
            for text in [
                format!(" {accepted}"),
                format!("{accepted} "),
                format!("{accepted}\n"),
                accepted.to_ascii_lowercase(),
            ] {
                assert!(
                    matches!(
                        text.parse::<DayCount>(),
                        Err(IdentifierError::Unknown { .. })
                    ),
                    "{text:?}"
                );
            }
        }
    }
    for text in [
        "",
        "ACT/360",
        "Actual/Actual (ICMA)",
        "Actual/Actual (AFB)",
        "30/360 (US)",
        "30/360 (Bond Basis)",
        "30E/360 (ISDA)",
        "Business/252",
        "Actual/365 (No Leap)",
        "Actual/360\0",
        "Ａctual/360",
        "Act/365",
        "A/365",
        "act/act",
    ] {
        let error = text
            .parse::<DayCount>()
            .expect_err("unlisted or unsupported identifier");
        assert_eq!(error.kind(), "identifier.unknown");
        match error {
            IdentifierError::Unknown {
                concept,
                text: rejected,
                ..
            } => {
                assert_eq!(concept, "day-count convention");
                assert_eq!(rejected, text);
            }
            _ => panic!("expected an unknown identifier"),
        }
    }
}

#[test]
fn generic_labels_remain_ambiguous_instead_of_selecting_a_supported_default() {
    for (text, candidates) in [
        (
            "Actual/Actual",
            &[
                "Actual/Actual (ISDA)",
                "Actual/Actual (ICMA)",
                "Actual/Actual (AFB)",
            ][..],
        ),
        (
            "Act/Act",
            &[
                "Actual/Actual (ISDA)",
                "Actual/Actual (ICMA)",
                "Actual/Actual (AFB)",
            ][..],
        ),
        (
            "ACT/ACT",
            &[
                "Actual/Actual (ISDA)",
                "Actual/Actual (ICMA)",
                "Actual/Actual (AFB)",
            ][..],
        ),
        (
            "Actual/365",
            &["Actual/365 (Fixed)", "Actual/Actual (ISDA)"][..],
        ),
        (
            "30/360",
            &[
                "30/360 (Bond Basis)",
                "30/360 (US)",
                "30E/360",
                "30E/360 (ISDA)",
            ][..],
        ),
    ] {
        let error = text
            .parse::<DayCount>()
            .expect_err("ambiguous generic label");
        assert_eq!(error.kind(), "identifier.ambiguous");
        match error {
            IdentifierError::Ambiguous {
                concept,
                text: rejected,
                candidates: actual,
                ..
            } => {
                assert_eq!(concept, "day-count convention");
                assert_eq!(rejected, text);
                assert_eq!(actual, candidates);
            }
            _ => panic!("expected ambiguity with candidates"),
        }
    }
}

#[test]
fn primary_definition_examples_have_exact_or_tolerance_qualified_expectations() {
    // Each expectation below is independently derived from the reviewed
    // primary formula, not a decimal copied from a secondary worked example.
    let oracle = RationalOracle::new();
    for (start, end, actual_days, european_days, isda_numerator) in [
        (date(2024, 1, 1), date(2025, 1, 1), 366, 360, 365 * 366),
        (date(2023, 1, 1), date(2024, 1, 1), 365, 360, 365 * 366),
        (date(2024, 1, 1), date(2024, 7, 1), 182, 180, 182 * 365),
        (date(2024, 2, 29), date(2024, 3, 31), 31, 31, 31 * 365),
        (date(2023, 1, 31), date(2023, 2, 28), 28, 28, 28 * 366),
        (date(2023, 1, 15), date(2023, 3, 31), 75, 75, 75 * 366),
        (
            date(1999, 7, 1),
            date(2000, 7, 1),
            366,
            360,
            184 * 366 + 182 * 365,
        ),
        // QR-01: this is a formula-derived expectation. The discrepant
        // older ISDA memo's worked example remains an open research item.
        (
            date(2003, 7, 15),
            date(2004, 1, 15),
            184,
            180,
            170 * 366 + 14 * 365,
        ),
    ] {
        assert_eq!(oracle.counts(start, end), (actual_days, isda_numerator));
        assert_eq!(DayCount::ThirtyE360.day_count(start, end), european_days);
        check_pair(&oracle, start, end).unwrap();
        check_pair(&oracle, end, start).unwrap();
    }
}

#[test]
fn full_range_endpoints_and_year_9999_never_require_a_year_10000_date() {
    let oracle = RationalOracle::new();
    for start in [
        Date::MIN,
        date(1, 12, 31),
        date(4, 2, 29),
        date(400, 12, 31),
        date(9999, 1, 1),
        date(9999, 12, 30),
        Date::MAX,
    ] {
        for end in [
            Date::MIN,
            date(1, 1, 2),
            date(9996, 2, 29),
            date(9999, 1, 1),
            date(9999, 12, 30),
            Date::MAX,
        ] {
            check_pair(&oracle, start, end).unwrap();
        }
    }
    assert_eq!(
        DayCount::Actual360.day_count(Date::MIN, Date::MAX),
        3_652_058
    );
    assert_eq!(
        DayCount::ThirtyE360.day_count(Date::MIN, Date::MAX),
        3_599_639
    );
    assert_close(
        DayCount::ActualActualIsda
            .year_fraction(date(9999, 1, 1), Date::MAX)
            .unwrap(),
        364.0 / 365.0,
        TOL_E2,
    );
    assert_close(
        DayCount::ActualActualIsda
            .year_fraction(Date::MIN, Date::MAX)
            .unwrap(),
        9998.0 + 364.0 / 365.0,
        TOL_E2,
    );
}

#[test]
fn equal_dates_and_collapsed_european_30th_31st_intervals_are_positive_zero() {
    for current in [
        Date::MIN,
        date(1900, 2, 28),
        date(2000, 2, 29),
        date(2024, 5, 30),
        Date::MAX,
    ] {
        for convention in DayCount::ALL {
            assert_eq!(convention.day_count(current, current), 0);
            assert_eq!(
                convention
                    .year_fraction(current, current)
                    .unwrap()
                    .to_bits(),
                0
            );
        }
    }
    for year in [1, 100, 400, 1900, 2000, 2100, 2400, 9999] {
        for month in [1, 3, 5, 7, 8, 10, 12] {
            let start = date(year, month, 30);
            let end = date(year, month, 31);
            for (start, end) in [(start, end), (end, start)] {
                assert_eq!(DayCount::ThirtyE360.day_count(start, end), 0);
                assert_eq!(
                    DayCount::ThirtyE360
                        .year_fraction(start, end)
                        .unwrap()
                        .to_bits(),
                    0
                );
                assert_eq!(DayCount::Actual360.day_count(start, end).abs(), 1);
            }
        }
    }
}

#[test]
fn every_adjacent_date_in_a_complete_gregorian_cycle_matches_the_rational_oracle() {
    let oracle = RationalOracle::new();
    let mut current = date(2000, 1, 1);
    let end = date(2400, 1, 1);
    let mut cases = 0;
    while current < end {
        let next = current.checked_add_days(1).unwrap();
        check_pair(&oracle, current, next).unwrap();
        check_pair(&oracle, next, current).unwrap();
        current = next;
        cases += 1;
    }
    assert_eq!(cases, 146_097);
}

#[test]
fn complete_pair_windows_expose_february_century_and_year_boundary_rules() {
    let oracle = RationalOracle::new();
    for year in [
        4, 100, 400, 1582, 1600, 1700, 1800, 1900, 1999, 2000, 2001, 2099, 2100, 2400, 9996,
    ] {
        for anchor in [date(year, 2, 1), date(year, 12, 15)] {
            let dates: Vec<_> = (0..=60)
                .map(|offset| anchor.checked_add_days(offset).unwrap())
                .collect();
            for &start in &dates {
                for &end in &dates {
                    check_pair(&oracle, start, end).unwrap();
                }
            }
        }
    }
}

#[test]
fn all_supported_month_ends_and_adjacent_year_anniversaries_match_independent_counts() {
    let oracle = RationalOracle::new();
    let mut previous = Date::MIN;
    for year in 1..=9999 {
        for (month_index, length) in months(year).into_iter().enumerate() {
            let month = u32::try_from(month_index + 1).unwrap();
            let first = date(year, month, 1);
            let end = date(year, month, length);
            for (start, end) in [(first, end), (end, first), (previous, end), (end, previous)] {
                check_pair(&oracle, start, end).unwrap();
            }
            if year < 9999 {
                let anniversary = date(year + 1, month, length.min(months(year + 1)[month_index]));
                check_pair(&oracle, end, anniversary).unwrap();
                check_pair(&oracle, anniversary, end).unwrap();
            }
            previous = end;
        }
    }
    assert_eq!(previous, Date::MAX);
}

#[test]
fn deterministic_full_range_pairs_match_exact_integer_rational_oracles() {
    let oracle = RationalOracle::new();
    prng::check(
        "core day counts match i128 rational oracle",
        0x4443_435F_4D33_2026,
        24_000,
        |rng| {
            let start = Date::MIN
                .checked_add_days(rng.range_i64(0, 3_652_058))
                .unwrap();
            let end = Date::MIN
                .checked_add_days(rng.range_i64(0, 3_652_058))
                .unwrap();
            check_pair(&oracle, start, end)?;
            check_pair(&oracle, end, start)?;
            for convention in DayCount::ALL {
                let forward = convention.year_fraction(start, end).unwrap();
                let reverse = convention.year_fraction(end, start).unwrap();
                if convention.day_count(start, end) == 0 {
                    if forward.to_bits() != 0 || reverse.to_bits() != 0 {
                        return Err(format!("{convention} {start}<->{end}: zero sign differs"));
                    }
                } else if forward.to_bits() != (-reverse).to_bits() {
                    return Err(format!(
                        "{convention} {start}<->{end}: reversed fraction is not exact negation"
                    ));
                }
                let repeated = convention.year_fraction(start, end).unwrap();
                if forward.to_bits() != repeated.to_bits() {
                    return Err(format!(
                        "{convention} {start}->{end}: repeated call differs"
                    ));
                }
            }
            Ok(())
        },
    );
}

#[test]
fn integer_counts_are_additive_and_isda_composites_meet_the_combined_ulp_budget() {
    let oracle = RationalOracle::new();
    prng::check(
        "ordered day-count triples telescope",
        0x4443_435F_4144_4449,
        16_000,
        |rng| {
            let mut ordinals = [
                rng.range_i64(0, 3_652_058),
                rng.range_i64(0, 3_652_058),
                rng.range_i64(0, 3_652_058),
            ];
            ordinals.sort_unstable();
            let dates = ordinals.map(|ordinal| Date::MIN.checked_add_days(ordinal).unwrap());
            let [a, b, c] = dates;
            for convention in DayCount::ALL {
                let direct = convention.day_count(a, c);
                let parts = convention.day_count(a, b) + convention.day_count(b, c);
                if direct != parts {
                    return Err(format!(
                        "{convention} {a}->{b}->{c}: integer counts fail additivity"
                    ));
                }
            }
            let direct = DayCount::ActualActualIsda.year_fraction(a, c).unwrap();
            let parts = DayCount::ActualActualIsda.year_fraction(a, b).unwrap()
                + DayCount::ActualActualIsda.year_fraction(b, c).unwrap();
            // TOL-E2's documented eight-ULP combined-implementation budget is
            // used only for comparing two separately rounded composite results.
            Tolerance::Ulps(8)
                .check(parts, direct)
                .map_err(|report| format!("ISDA combined TOL-E2 {a}->{b}->{c}: {report}"))?;
            TOL_E2.check(direct, oracle.fraction(a, c))?;
            Ok(())
        },
    );
}

#[test]
fn full_years_and_year_split_endpoints_follow_their_own_calendar_denominator() {
    for year in 1..9999 {
        let start = date(year, 1, 1);
        let end = date(year + 1, 1, 1);
        assert_eq!(
            DayCount::ActualActualIsda
                .year_fraction(start, end)
                .unwrap()
                .to_bits(),
            1.0_f64.to_bits()
        );
        let last = date(year, 12, 31);
        let expected = 1.0 / if leap(year) { 366.0 } else { 365.0 };
        assert_close(
            DayCount::ActualActualIsda.year_fraction(last, end).unwrap(),
            expected,
            TOL_E2,
        );
    }
}

#[test]
fn public_day_count_values_and_errors_have_required_standard_traits() {
    fn value<T: Copy + Debug + Eq + Hash + Send + Sync + 'static>() {}
    fn error<T: std::error::Error + Clone + Debug + PartialEq + Send + Sync + 'static>() {}
    value::<DayCount>();
    error::<DayCountError>();
}

/// TS-DCC-07: every ordered pair in 1999–2002 and 2099–2101, for every
/// currently implemented convention. Other conventions remain later scope.
#[test]
fn mandatory_exhaustive_pair_windows_match_independent_exact_rational_oracles() {
    let oracle = RationalOracle::new();
    let mut count = 0_u64;
    for (first, last) in [(1999, 2002), (2099, 2101)] {
        let mut dates = Vec::new();
        for year in first..=last {
            for (index, length) in months(year).into_iter().enumerate() {
                for day in 1..=length {
                    dates.push(date(year, u32::try_from(index + 1).unwrap(), day));
                }
            }
        }
        for &start in &dates {
            for &end in &dates {
                check_pair(&oracle, start, end).unwrap();
                count += 1;
            }
        }
    }
    assert_eq!(count, 3_333_546);
}
