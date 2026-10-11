//! Hostile text and integer inputs for NFR-02, TS-DATE-03/04/07 and TS-PER-03.
//! These deterministic campaigns supplement exhaustive valid-date tests and
//! external oracle fixtures. They are not a claim of exhaustive string coverage.

mod support;

use sabiquant::time::{Date, Frequency, Month, Period, TimeUnit, Weekday};
use support::prng::SplitMix64;

#[test]
fn every_single_ascii_byte_mutation_of_boundary_date_text_is_safe() {
    for text in ["0001-01-01", "1900-02-28", "2000-02-29", "9999-12-31"] {
        for position in 0..10 {
            for replacement in 0_u8..=127 {
                let mut bytes = text.as_bytes().to_vec();
                bytes[position] = replacement;
                let candidate = String::from_utf8(bytes).unwrap();
                if let Ok(date) = candidate.parse::<Date>() {
                    // Only an exact canonical ASCII date can be accepted.
                    assert_eq!(date.to_string(), candidate);
                }
            }
        }
    }
}

#[test]
fn unicode_insertion_and_whitespace_are_rejected_in_strict_parsers() {
    for text in ["0001-01-01", "2024-02-29", "9999-12-31"] {
        for position in 0..=10 {
            for addition in [
                " ", "\t", "\n", "\0", "\u{feff}", "é", "๒", "🙂", "\u{200b}", "\r",
            ] {
                let mut candidate = text.to_owned();
                candidate.insert_str(position, addition);
                assert!(candidate.parse::<Date>().is_err(), "{candidate:?}");
            }
        }
    }
    for text in ["1D", "-12M", "2147483647Y", "-2147483648W"] {
        for position in 0..=text.len() {
            for addition in [" ", "\t", "\n", "\0", "é", "๒", "🙂", "\u{200b}"] {
                let mut candidate = text.to_owned();
                candidate.insert_str(position, addition);
                assert!(candidate.parse::<Period>().is_err(), "{candidate:?}");
            }
        }
    }
}

#[test]
fn deterministic_hostile_ascii_strings_never_panic_or_create_invalid_values() {
    let mut rng = SplitMix64::new(0x5041_5253_4552_4D32);
    for _ in 0..50_000 {
        let length = usize::try_from(rng.below(65)).unwrap();
        let text: String = (0..length)
            .map(|_| char::from(u8::try_from(rng.below(128)).unwrap()))
            .collect();
        if let Ok(date) = text.parse::<Date>() {
            assert_eq!(date.to_string(), text);
            assert!((Date::MIN..=Date::MAX).contains(&date));
        }
        if let Ok(period) = text.parse::<Period>() {
            assert_eq!(period.to_string().parse::<Period>().unwrap(), period);
        }
        // Even the identifier parsers must be total on arbitrary input.
        let _ = text.parse::<Frequency>();
        let _ = text.parse::<TimeUnit>();
        let _ = text.parse::<Month>();
        let _ = text.parse::<Weekday>();
    }
}

#[test]
fn date_integer_edges_and_random_bit_patterns_match_widened_range_checks() {
    let maximum = i128::from(Date::MAX.days_since(Date::MIN));
    let mut rng = SplitMix64::new(0x494E_5445_4745_524D);
    for case in 0..25_000 {
        let ordinal = rng.range_i64(0, i64::try_from(maximum).unwrap());
        let date = Date::MIN.checked_add_days(ordinal).unwrap();
        let count = match case % 6 {
            0 => i64::MIN,
            1 => i64::MAX,
            2 => rng.range_i64(-4_000_000, 4_000_000),
            3 => rng.range_i64(-600_000, 600_000),
            _ => rng.range_i64(i64::MIN, i64::MAX),
        };
        for (actual, target) in [
            (
                date.checked_add_days(count),
                i128::from(ordinal) + i128::from(count),
            ),
            (
                date.checked_sub_days(count),
                i128::from(ordinal) - i128::from(count),
            ),
            (
                date.checked_add_weeks(count),
                i128::from(ordinal) + 7 * i128::from(count),
            ),
        ] {
            if (0..=maximum).contains(&target) {
                assert_eq!(i128::from(actual.unwrap().days_since(Date::MIN)), target);
            } else {
                assert!(actual.is_err(), "accepted target {target}, case {case}");
            }
        }
        let (year, month, day) = date.to_ymd();
        for (actual, target) in [
            (
                date.checked_add_months(count),
                i128::from(year - 1) * 12 + i128::from(month - 1) + i128::from(count),
            ),
            (
                date.checked_add_years(count),
                i128::from(year - 1) * 12 + i128::from(month - 1) + 12 * i128::from(count),
            ),
        ] {
            if (0..119_988).contains(&target) {
                let target_year = i32::try_from(target / 12 + 1).unwrap();
                let target_month = u32::try_from(target % 12 + 1).unwrap();
                let mut lengths = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
                if target_year % 4 == 0 && (target_year % 100 != 0 || target_year % 400 == 0) {
                    lengths[1] = 29;
                }
                let target_day = day.min(lengths[usize::try_from(target_month - 1).unwrap()]);
                assert_eq!(
                    actual.unwrap().to_ymd(),
                    (target_year, target_month, target_day)
                );
            } else {
                assert!(
                    actual.is_err(),
                    "accepted target month {target}, case {case}"
                );
            }
        }
    }
}

#[test]
fn extreme_component_and_long_text_inputs_are_rejected() {
    for year in [i32::MIN, -1, 0, 10_000, i32::MAX] {
        for month in [0, 1, 12, 13, u32::MAX] {
            for day in [0, 1, 31, 32, u32::MAX] {
                assert!(Date::from_ymd(year, month, day).is_err());
            }
        }
    }
    for (month, day) in [
        (0, 1),
        (13, 1),
        (u32::MAX, 1),
        (1, 0),
        (1, 32),
        (1, u32::MAX),
    ] {
        assert!(Date::from_ymd(2024, month, day).is_err());
    }
    for text in [
        "9".repeat(100_000),
        format!("{}D", "9".repeat(100_000)),
        "🙂".repeat(25_000),
    ] {
        assert!(text.parse::<Date>().is_err());
        assert!(text.parse::<Period>().is_err());
    }
}
