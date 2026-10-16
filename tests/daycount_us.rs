//! F-DCC-09, FS-DCC §9.7 and TS-DCC-01..08: explicit local US 30/360.
//!
//! The frozen rule is docs/M3_US_AFB_CONTRACT.md, adopted from ROC Revised
//! CDE Technical Guidance v4 (December 2025), Table 4, A001's FIX column.
//! Full SIA/SIFMA financial acceptance remains separate. Hand counts are
//! independently calculated consequences, not claimed source worked examples.
//! The integer oracle projects dates onto a 30-day-month axis and adds
//! February corrections; it does not call production adjustment/day-count
//! helpers or duplicate the production sequence of four mutations.

#![expect(
    clippy::expect_used,
    reason = "independent test civil arithmetic and bounded date inputs fail immediately if a test invariant is broken"
)]

mod support;

use sabiquant::daycount::{DayCount, Thirty360Us};
use sabiquant::error::IdentifierError;
use sabiquant::time::Date;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use support::prng;

fn date(year: i32, month: u32, day: u32) -> Date {
    Date::from_ymd(year, month, day).expect("valid hand or enumerated civil date")
}

fn leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn month_lengths(year: i32) -> [u32; 12] {
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

#[derive(Clone, Copy)]
struct Civil {
    year: i32,
    month: i32,
    day: i32,
    last_february: bool,
}

impl Civil {
    fn new(value: Date) -> Self {
        let (year, month, day) = value.to_ymd();
        Self {
            year,
            month: i32::try_from(month).expect("civil month fits"),
            day: i32::try_from(day).expect("civil day fits"),
            last_february: month == 2 && day == month_lengths(year)[1],
        }
    }

    fn axis(self) -> i32 {
        self.year * 360 + self.month * 30 + self.day
    }
}

fn reference_count(start: Date, end: Date, eom: bool) -> i32 {
    if start > end {
        return -reference_count(end, start, eom);
    }
    let first = Civil::new(start);
    let last = Civil::new(end);
    // Start with the actual day numbers on a synthetic 30-day-month axis.
    // Each Bond 31st correction is a scalar delta to that projected distance.
    let mut count = last.axis() - first.axis();
    count += i32::from(first.day == 31);
    count -= i32::from(last.day == 31 && first.day >= 30);
    if eom && first.last_february {
        count -= 30 - first.day;
        // February's two extra corrections are disjoint. A last-Feb end
        // moves upwards to 30; an end 31st moves down by one because its
        // adjusted starting day has become 30.
        count += if last.last_february {
            30 - last.day
        } else {
            -i32::from(last.day == 31)
        };
    }
    count
}

fn check_pair(start: Date, end: Date) -> Result<(), String> {
    for flag in [false, true] {
        let context = Thirty360Us::new(flag);
        let expected = reference_count(start, end, flag);
        let actual = context.day_count(start, end);
        if actual != expected {
            return Err(format!(
                "US eom={flag} {start}->{end}: count={actual}, expected={expected}"
            ));
        }
        let expected_fraction = f64::from(expected) / 360.0;
        let fraction = context
            .year_fraction(start, end)
            .map_err(|error| error.to_string())?;
        if fraction.to_bits() != expected_fraction.to_bits() {
            return Err(format!(
                "US eom={flag} {start}->{end}: TOL-E1 bits {:#018x}, expected {:#018x}",
                fraction.to_bits(),
                expected_fraction.to_bits()
            ));
        }
        let reverse = context.day_count(end, start);
        if reverse != -expected {
            return Err(format!(
                "US eom={flag} {start}<->{end}: reverse count={reverse}, expected {}",
                -expected
            ));
        }
        let reverse_fraction = context
            .year_fraction(end, start)
            .map_err(|error| error.to_string())?;
        let expected_reverse = if expected == 0 {
            0.0
        } else {
            -expected_fraction
        };
        if reverse_fraction.to_bits() != expected_reverse.to_bits() {
            return Err(format!(
                "US eom={flag} {start}<->{end}: reverse fraction bits differ"
            ));
        }
        if !flag {
            let bond = DayCount::Thirty360BondBasis;
            if actual != bond.day_count(start, end)
                || fraction.to_bits()
                    != bond
                        .year_fraction(start, end)
                        .map_err(|error| error.to_string())?
                        .to_bits()
            {
                return Err(format!(
                    "US flag-off arithmetic differs from Bond Basis for {start}->{end}"
                ));
            }
        }
    }
    Ok(())
}

#[test]
fn hand_counts_distinguish_both_february_rules_and_31st_ordering() {
    // (start, end, enabled count, disabled count). The two settings remain
    // explicit even on dates for which their numerical results agree.
    let rows = [
        ((2023, 2, 28), (2023, 3, 31), 30, 33),
        ((2024, 2, 29), (2024, 3, 31), 30, 32),
        ((2023, 2, 28), (2024, 2, 29), 360, 361),
        ((2024, 2, 29), (2025, 2, 28), 360, 359),
        ((2023, 1, 31), (2023, 2, 28), 28, 28),
        ((2024, 2, 28), (2024, 3, 31), 33, 33),
        ((2023, 8, 31), (2024, 2, 29), 179, 179),
        ((2023, 1, 29), (2023, 1, 31), 2, 2),
        ((2023, 1, 30), (2023, 1, 31), 0, 0),
        ((2023, 2, 28), (2023, 3, 1), 1, 3),
        ((2024, 2, 29), (2024, 3, 1), 1, 2),
        ((2024, 2, 28), (2024, 3, 1), 3, 3),
        ((2023, 2, 27), (2023, 3, 31), 34, 34),
        ((2023, 2, 28), (2023, 3, 30), 30, 32),
        ((2024, 1, 31), (2024, 2, 29), 29, 29),
        ((2024, 1, 30), (2024, 2, 29), 29, 29),
        ((2023, 2, 28), (2024, 2, 28), 358, 360),
        ((2024, 2, 28), (2025, 2, 28), 360, 360),
        ((2024, 2, 29), (2028, 2, 29), 1440, 1440),
        ((2023, 4, 30), (2023, 5, 31), 30, 30),
        ((2023, 4, 29), (2023, 5, 31), 32, 32),
        ((2023, 1, 31), (2023, 3, 31), 60, 60),
        ((2023, 1, 31), (2023, 3, 30), 60, 60),
        ((2023, 1, 31), (2024, 2, 29), 389, 389),
        ((1900, 2, 28), (1900, 3, 31), 30, 33),
        ((2000, 2, 28), (2000, 3, 31), 33, 33),
        ((2000, 2, 29), (2000, 3, 31), 30, 32),
        ((2100, 2, 28), (2100, 3, 31), 30, 33),
        ((2400, 2, 29), (2400, 3, 31), 30, 32),
    ];
    for (first, last, enabled, disabled) in rows {
        let start = date(first.0, first.1, first.2);
        let end = date(last.0, last.1, last.2);
        for (flag, expected) in [(true, enabled), (false, disabled)] {
            let context = Thirty360Us::new(flag);
            assert_eq!(
                context.day_count(start, end),
                expected,
                "{flag} {start}->{end}"
            );
            assert_eq!(context.day_count(end, start), -expected);
            assert_eq!(reference_count(start, end, flag), expected);
        }
        check_pair(start, end).expect("hand oracle and exact fraction");
    }
}

#[test]
fn every_supported_years_same_and_next_year_month_end_pairs_match_integer_oracle() {
    let mut pairs = 0_u64;
    for year in 1..=9999 {
        for (start_month, start_day) in month_lengths(year).into_iter().enumerate() {
            let start = date(year, u32::try_from(start_month + 1).unwrap(), start_day);
            for end_year in year..=(year + 1).min(9999) {
                for (end_month, end_day) in month_lengths(end_year).into_iter().enumerate() {
                    let end = date(end_year, u32::try_from(end_month + 1).unwrap(), end_day);
                    check_pair(start, end).expect("all-year month-end oracle");
                    pairs += 1;
                }
            }
        }
    }
    assert_eq!(pairs, 2_879_568);
}

#[test]
fn complete_gregorian_cycle_daily_anchors_use_rotating_short_and_long_spans() {
    let first = date(2000, 1, 1);
    let offsets = [0, 1, 27, 28, 29, 30, 31, 59, 60, 365, 366, 146_097];
    for index in 0..146_097_i64 {
        let start = first.checked_add_days(index).unwrap();
        let profile = usize::try_from(index).unwrap() % offsets.len();
        let end = start.checked_add_days(offsets[profile]).unwrap();
        check_pair(start, end).expect("complete daily anchor cycle");
    }
    assert_eq!(first.checked_add_days(146_097).unwrap(), date(2400, 1, 1));
}

#[test]
fn all_ordered_pairs_in_leap_and_non_leap_century_windows_match_rationals() {
    let mut pairs = 0_u64;
    for (first, last) in [
        (date(1999, 1, 1), date(2001, 12, 31)),
        (date(2100, 1, 1), date(2100, 4, 1)),
    ] {
        let dates: Vec<_> = (0..=last.days_since(first))
            .map(|offset| first.checked_add_days(i64::from(offset)).unwrap())
            .collect();
        for &start in &dates {
            for &end in &dates {
                check_pair(start, end).expect("exhaustive leap/century window");
                pairs += 1;
            }
        }
    }
    assert_eq!(pairs, 1_209_497);
}

#[test]
fn deterministic_full_range_pairs_and_extreme_endpoints_are_exact() {
    prng::check(
        "US explicit context full-range integer/rational oracle",
        0x5AB1_3036_0A11_2026,
        40_000,
        |rng| {
            let start = Date::MIN
                .checked_add_days(rng.range_i64(0, 3_652_058))
                .map_err(|error| error.to_string())?;
            let end = Date::MIN
                .checked_add_days(rng.range_i64(0, 3_652_058))
                .map_err(|error| error.to_string())?;
            check_pair(start, end)
        },
    );
    let edges = [
        Date::MIN,
        date(1, 1, 30),
        date(1, 1, 31),
        date(1, 2, 28),
        date(4, 2, 29),
        date(9996, 2, 29),
        date(9999, 2, 28),
        date(9999, 12, 30),
        Date::MAX,
    ];
    for start in edges {
        for end in edges {
            check_pair(start, end).expect("MIN/MAX and edge years");
        }
    }
    for flag in [false, true] {
        let context = Thirty360Us::new(flag);
        assert_eq!(context.day_count(Date::MIN, Date::MAX), 3_599_640);
        assert_eq!(context.day_count(Date::MAX, Date::MIN), -3_599_640);
    }
}

#[test]
fn all_equal_dates_and_all_monthly_30th_31st_collapses_produce_positive_zero() {
    let mut current = Date::MIN;
    loop {
        for flag in [false, true] {
            let context = Thirty360Us::new(flag);
            assert_eq!(context.day_count(current, current), 0);
            assert_eq!(
                context.year_fraction(current, current).unwrap().to_bits(),
                0
            );
        }
        if current == Date::MAX {
            break;
        }
        current = current.checked_add_days(1).unwrap();
    }
    for year in 1..=9999 {
        for month in [1, 3, 5, 7, 8, 10, 12] {
            let start = date(year, month, 30);
            let end = date(year, month, 31);
            for flag in [false, true] {
                let context = Thirty360Us::new(flag);
                assert_eq!(context.day_count(start, end), 0);
                assert_eq!(context.day_count(end, start), 0);
                assert_eq!(context.year_fraction(start, end).unwrap().to_bits(), 0);
                assert_eq!(context.year_fraction(end, start).unwrap().to_bits(), 0);
            }
        }
    }
}

#[test]
fn chronological_reversal_and_nonadditivity_are_explicit_api_policy() {
    let start = date(2023, 2, 28);
    let end = date(2023, 3, 31);
    let enabled = Thirty360Us::new(true);
    let disabled = Thirty360Us::new(false);
    assert_eq!(enabled.day_count(end, start), -30);
    assert_eq!(disabled.day_count(end, start), -33);
    // Directly applying the directional 31st rule to Mar31 -> Feb28 would
    // give -30 + (28-30) = -32. SabiQuant retains chronological reversal.
    let direct_directional_reverse = -30 + (28 - 30);
    assert_eq!(direct_directional_reverse, -32);
    assert_ne!(enabled.day_count(end, start), direct_directional_reverse);
    assert_ne!(disabled.day_count(end, start), direct_directional_reverse);
    for context in [enabled, disabled] {
        let first = date(2023, 1, 15);
        let split = date(2023, 1, 31);
        let last = date(2023, 2, 15);
        assert_eq!(context.day_count(first, split), 16);
        assert_eq!(context.day_count(split, last), 15);
        assert_eq!(context.day_count(first, last), 30);
    }
}

fn hash(value: Thirty360Us) -> u64 {
    let mut state = DefaultHasher::new();
    value.hash(&mut state);
    state.finish()
}

#[test]
fn context_is_a_copy_key_and_name_does_not_supply_missing_qualification() {
    fn traits<T: Copy + Eq + Hash + Debug + Send + Sync + 'static>() {}
    traits::<Thirty360Us>();
    const ENABLED: Thirty360Us = Thirty360Us::new(true);
    const DISABLED: Thirty360Us = Thirty360Us::new(false);
    const QUALIFIED: bool = ENABLED.end_of_month_security();
    const NAME: &str = DISABLED.canonical_identifier();
    assert!(std::hint::black_box(QUALIFIED));
    assert_eq!(NAME, "30/360 (US)");
    assert_eq!(ENABLED.canonical_identifier(), NAME);
    assert!(!DISABLED.end_of_month_security());
    assert_ne!(ENABLED, DISABLED);
    let mut definitions = HashMap::new();
    definitions.insert(ENABLED, "enabled");
    definitions.insert(DISABLED, "disabled");
    assert_eq!(definitions.get(&Thirty360Us::new(true)), Some(&"enabled"));
    assert_eq!(definitions.get(&Thirty360Us::new(false)), Some(&"disabled"));
    assert_eq!(hash(ENABLED), hash(Thirty360Us::new(true)));
    assert_eq!(hash(DISABLED), hash(Thirty360Us::new(false)));
    assert_eq!(HashSet::from([ENABLED, DISABLED, ENABLED]).len(), 2);
    for text in ["30/360 (US)", "30/360 (US) ", "30/360 (us)", "30/360 US"] {
        assert!(matches!(
            text.parse::<DayCount>(),
            Err(IdentifierError::Unknown { .. })
        ));
    }
    assert!(matches!(
        "30/360".parse::<DayCount>(),
        Err(IdentifierError::Ambiguous { .. })
    ));
    // Definition identity survives even on intervals with equal outputs.
    let start = date(2023, 1, 1);
    let end = date(2023, 2, 1);
    assert_eq!(
        ENABLED.day_count(start, end),
        DISABLED.day_count(start, end)
    );
    assert_eq!(definitions.len(), 2);
}

#[test]
fn immutable_contexts_are_repeatable_across_threads_and_call_orders() {
    let dates = [
        Date::MIN,
        date(1900, 2, 28),
        date(2000, 2, 29),
        date(2023, 2, 28),
        date(2024, 2, 28),
        date(2024, 2, 29),
        date(2025, 2, 28),
        Date::MAX,
    ];
    let mut expected = Vec::new();
    for flag in [false, true] {
        let context = Thirty360Us::new(flag);
        for start in dates {
            for end in dates {
                expected.push((
                    flag,
                    start,
                    end,
                    context.day_count(start, end),
                    context.year_fraction(start, end).unwrap().to_bits(),
                ));
            }
        }
    }
    let mut workers = Vec::new();
    for thread in 0..8 {
        let mut cases = expected.clone();
        if thread % 2 == 0 {
            cases.reverse();
        }
        #[expect(
            clippy::disallowed_methods,
            reason = "NFR-03/05 test shares only immutable Copy contexts and checks independence of execution order"
        )]
        let worker = std::thread::spawn(move || {
            for _ in 0..16 {
                for &(flag, start, end, count, bits) in &cases {
                    let context = Thirty360Us::new(flag);
                    assert_eq!(context.day_count(start, end), count);
                    assert_eq!(context.year_fraction(start, end).unwrap().to_bits(), bits);
                }
            }
        });
        workers.push(worker);
    }
    for worker in workers {
        worker.join().unwrap();
    }
    assert_eq!(expected.len(), 128);
}
