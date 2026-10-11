//! Exact validation of independently implemented Western Gregorian computus.
//!
//! Oudin production arithmetic is checked against the distinct final Gauss
//! formula and its two exceptional corrections: Bien (2004), Table 1,
//! <https://www.staff.science.uu.nl/~gent0113/hovo/downloads/text1_08b.pdf>.
//! Every supported year is enumerated; Gregorian leap cycles alone do not
//! imply an Easter cycle. Published Census dates supply external anchors.

use sabiquant::calendar::western_easter;
use sabiquant::time::{Date, DateError, Weekday};

// The formula is independently expressed in i64 and returns a civil-day
// offset from 22 March rather than production's epact-to-month conversion.
// correction 1 replaces 26 April; correction 2 replaces exceptional 25 April.
fn gauss(year: i32) -> Result<(Date, u8), DateError> {
    let year_wide = i64::from(year);
    let golden = year_wide.rem_euclid(19);
    let leap_phase = year_wide.rem_euclid(4);
    let week_phase = year_wide.rem_euclid(7);
    let century = year_wide.div_euclid(100);
    let lunar_correction = (13 + 8 * century).div_euclid(25);
    let leap_correction = century.div_euclid(4);
    let lunar_anchor = (15 - lunar_correction + century - leap_correction).rem_euclid(30);
    let week_anchor = (4 + century - leap_correction).rem_euclid(7);
    let lunar_offset = (19 * golden + lunar_anchor).rem_euclid(30);
    let sunday_offset =
        (2 * leap_phase + 4 * week_phase + 6 * lunar_offset + week_anchor).rem_euclid(7);
    let correction = if lunar_offset == 29 && sunday_offset == 6 {
        1
    } else if lunar_offset == 28
        && sunday_offset == 6
        && (11 * lunar_anchor + 11).rem_euclid(30) < 19
    {
        2
    } else {
        0
    };
    let days = lunar_offset + sunday_offset - if correction == 0 { 0 } else { 7 };
    Ok((
        Date::from_ymd(year, 3, 22)?.checked_add_days(days)?,
        correction,
    ))
}

#[test]
fn every_supported_year_matches_independent_gauss_and_sunday_bounds() -> Result<(), DateError> {
    let mut correction_counts = [0_u32; 3];
    let mut earliest = false;
    let mut latest = false;
    for year in 1..=9999 {
        let actual = western_easter(year)?;
        let (expected, correction) = gauss(year)?;
        correction_counts[usize::from(correction)] += 1;
        assert_eq!(actual, expected, "year {year}");
        assert_eq!(actual.year(), year);
        assert_eq!(actual.weekday(), Weekday::Sunday, "year {year}");
        let first = Date::from_ymd(year, 3, 22)?;
        let last = Date::from_ymd(year, 4, 25)?;
        assert!(actual >= first && actual <= last, "year {year}");
        assert_eq!(actual.checked_sub_days(2)?.weekday(), Weekday::Friday);
        assert_eq!(actual.checked_add_days(1)?.weekday(), Weekday::Monday);
        earliest |= actual == first;
        latest |= actual == last;
    }
    assert_eq!(correction_counts.iter().sum::<u32>(), 9999);
    assert!(correction_counts.into_iter().all(|count| count > 0));
    assert!(earliest && latest);
    Ok(())
}

#[test]
fn primary_census_anchors_include_both_exception_classes_and_extreme_dates() -> Result<(), DateError>
{
    // Independently checked on 2026-10-11 against the Census Bureau table:
    // https://www.census.gov/data/software/x13as/genhol/easter-dates.html
    for (year, month, day) in [
        (1818, 3, 22),
        (1943, 4, 25),
        (1954, 4, 18),
        (1962, 4, 22),
        (1981, 4, 19),
        (2000, 4, 23),
        (2010, 4, 4),
        (2024, 3, 31),
        (2025, 4, 20),
        (2026, 4, 5),
        (2049, 4, 18),
        (2076, 4, 19),
    ] {
        assert_eq!(
            western_easter(year)?,
            Date::from_ymd(year, month, day)?,
            "year {year}"
        );
    }
    for (year, correction) in [(1954, 2), (1981, 1), (2049, 2), (2076, 1)] {
        let (corrected, actual_correction) = gauss(year)?;
        assert_eq!(actual_correction, correction, "year {year}");
        let uncorrected = corrected.checked_add_days(7)?;
        assert_eq!(
            (uncorrected.month(), uncorrected.day()),
            (4, if correction == 1 { 26 } else { 25 })
        );
    }
    Ok(())
}

#[test]
fn century_and_proleptic_boundaries_are_mathematical_not_historical_observances()
-> Result<(), DateError> {
    for year in [
        1, 2, 99, 100, 199, 200, 399, 400, 401, 1582, 1583, 1599, 1600, 1699, 1700, 1899, 1900,
        1999, 2000, 2099, 2100, 2399, 2400, 9998, 9999,
    ] {
        assert_eq!(western_easter(year)?, gauss(year)?.0, "year {year}");
    }
    // Century 2 gives a negative intermediate in Oudin's correction. The
    // year 216 catches truncation toward zero: that mistake yields March 24.
    assert_eq!(western_easter(216)?, Date::from_ymd(216, 4, 21)?);
    Ok(())
}

#[test]
fn unsupported_years_are_rejected_with_original_year_before_arithmetic() {
    for year in [i32::MIN, -9999, -400, -1, 0, 10_000, i32::MAX] {
        assert!(
            matches!(western_easter(year), Err(DateError::YearOutOfRange { year: rejected, .. }) if rejected == year)
        );
    }
}
