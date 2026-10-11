//! Independent M2 validation (TS-DATE-01..07, TS-PER-01..03).
//!
//! The exhaustive date oracle uses nested civil counters and a weekday
//! counter starting at the known Monday 0001-01-01; it does not reuse the
//! library's civil-to-serial algorithm. Committed Python datetime and actual
//! QuantLib 1.43 fixtures provide separately generated reference evidence.
//! Property seeds are fixed and failure reports identify each replay seed.

#![expect(
    clippy::expect_used,
    reason = "test helpers fail immediately when committed fixtures or valid test inputs are malformed"
)]

mod support;

use sabiquant::time::{Date, DateError, Frequency, Month, Period, PeriodError, TimeUnit, Weekday};
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use support::fixture::Fixture;
use support::prng::check;

fn fixture(name: &str) -> Fixture {
    let path = format!("validation/fixtures/time/{name}.fixture");
    Fixture::load(Fixture::repository_path(&path)).expect("committed M2 fixture is valid schema 1")
}

// Calendar-table oracle, independent of Date's civil/serial conversion.
fn month_days(year: i32, month: u32) -> u32 {
    match month {
        2 => 28 + u32::from(year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)),
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn shifted(date: Date, unit: TimeUnit, count: i64) -> Result<Date, DateError> {
    match unit {
        TimeUnit::Days => date.checked_add_days(count),
        TimeUnit::Weeks => date.checked_add_weeks(count),
        TimeUnit::Months => date.checked_add_months(count),
        TimeUnit::Years => date.checked_add_years(count),
    }
}

#[test]
fn ts_date_06_07_all_3652059_supported_dates_match_independent_civil_counters() {
    let mut ordinal = 0_i32;
    let mut previous = None;
    for year in 1..=9999 {
        let mut day_of_year = 0;
        for month in 1..=12 {
            let last = month_days(year, month);
            for day in 1..=last {
                let date = Date::from_ymd(year, month, day).expect("valid calendar-table date");
                assert_eq!(date.to_ymd(), (year, month, day));
                assert_eq!(date.days_since(Date::MIN), ordinal);
                assert_eq!(Date::MIN.days_since(date), -ordinal);
                assert_eq!(
                    date.weekday().number_from_monday(),
                    u32::try_from(ordinal % 7 + 1).unwrap()
                );
                day_of_year += 1;
                assert_eq!(date.day_of_year(), day_of_year);
                assert_eq!(date.is_end_of_month(), day == last);
                let text = format!("{year:04}-{month:02}-{day:02}");
                assert_eq!(date.to_string(), text);
                assert_eq!(text.parse::<Date>().unwrap(), date);
                if let Some(prior) = previous {
                    assert!(prior < date);
                    assert_eq!(date.checked_sub_days(1).unwrap(), prior);
                }
                previous = Some(date);
                ordinal += 1;
            }
        }
        assert_eq!(day_of_year, Date::days_in_year(year).unwrap());
    }
    assert_eq!(ordinal, 3_652_059);
    assert_eq!(previous, Some(Date::MAX));
    assert_eq!(
        Date::from_ymd(1970, 1, 1).unwrap().weekday(),
        Weekday::Thursday
    );
}

#[test]
fn ts_date_01_06_datetime_fixture_covers_every_years_leap_boundary() {
    let data = fixture("time-date-oracle");
    assert_eq!(
        data.header("reference"),
        Some("oracle:Python datetime/calendar")
    );
    assert_eq!(data.record_count(), 29_368);
    for record in data.records() {
        let date: Date = record.parse("date").unwrap();
        let year: i32 = record.parse("year").unwrap();
        let month: u32 = record.parse("month").unwrap();
        let day: u32 = record.parse("day").unwrap();
        assert_eq!(date.to_ymd(), (year, month, day), "{}", record.id());
        assert_eq!(date.year(), year, "{}", record.id());
        assert_eq!(date.month(), month, "{}", record.id());
        assert_eq!(date.day(), day, "{}", record.id());
        assert_eq!(
            date.weekday().number_from_monday(),
            record.parse::<u32>("weekday").unwrap(),
            "{}",
            record.id()
        );
        assert_eq!(
            date.day_of_year(),
            record.parse::<u32>("day_of_year").unwrap(),
            "{}",
            record.id()
        );
        assert_eq!(
            date.days_since(Date::MIN) + 1,
            record.parse::<i32>("ordinal").unwrap(),
            "{}",
            record.id()
        );
        assert_eq!(
            Date::days_in_month(year, month).unwrap(),
            record.parse::<u32>("month_length").unwrap(),
            "{}",
            record.id()
        );
        let year_length: u32 = record.parse("year_length").unwrap();
        assert_eq!(
            Date::days_in_year(year).unwrap(),
            year_length,
            "{}",
            record.id()
        );
        assert_eq!(
            Date::is_leap_year(year),
            year_length == 366,
            "{}",
            record.id()
        );
        assert_eq!(
            date.first_day_of_month(),
            record.parse::<Date>("first_of_month").unwrap(),
            "{}",
            record.id()
        );
        assert_eq!(
            date.last_day_of_month(),
            record.parse::<Date>("last_of_month").unwrap(),
            "{}",
            record.id()
        );
        assert_eq!(
            date.is_end_of_month(),
            record.parse::<u32>("is_end_of_month").unwrap() == 1,
            "{}",
            record.id()
        );
    }
}

#[test]
fn ts_date_04_05_independent_arithmetic_includes_i64_extremes() {
    validate_date_arithmetic("time-date-oracle-arithmetic", 11_100);
}

#[test]
fn ts_date_05_quantlib_143_date_arithmetic_with_declared_range_divergences() {
    validate_date_arithmetic("time-date-quantlib-arithmetic", 5_200);
}

fn validate_date_arithmetic(name: &str, count: usize) {
    let data = fixture(name);
    assert_eq!(data.record_count(), count);
    let quantlib = data.header("reference") == Some("QuantLib");
    let mut range_divergences = 0;
    for record in data.records() {
        let date = record.parse("date").expect("fixture date");
        let unit = record.parse("unit").expect("fixture time unit");
        let count = record.parse("count").expect("fixture signed shift");
        let result = shifted(date, unit, count);
        let expected = record
            .parse_optional::<Date>("expected_result")
            .expect("fixture expected date");
        if quantlib {
            let reference = record
                .parse_optional::<Date>("reference_result")
                .expect("fixture reference date");
            let behavior = record
                .text("reference_behavior")
                .expect("fixture reference behavior");
            let discrepancy = record
                .text("discrepancy")
                .expect("fixture discrepancy class");
            assert!(
                discrepancy.is_empty() || discrepancy == "D9-DATE-RANGE",
                "{}",
                record.id()
            );
            if discrepancy.is_empty() {
                assert_eq!(reference, expected, "{}", record.id());
                assert_eq!(
                    behavior,
                    if expected.is_some() {
                        "value"
                    } else {
                        "raises"
                    },
                    "{}",
                    record.id()
                );
            } else {
                let target =
                    expected.expect("D9 range divergence requires an independent valid date");
                assert!(!(1901..=2199).contains(&target.year()), "{}", record.id());
                assert_eq!(reference, None, "{}", record.id());
                assert_eq!(behavior, "raises", "{}", record.id());
                range_divergences += 1;
            }
        }
        if let Some(expected) = expected {
            assert!(
                record
                    .text("expected_error")
                    .expect("fixture error")
                    .is_empty(),
                "{}",
                record.id()
            );
            assert_eq!(
                result.expect("reference date is within SabiQuant range"),
                expected,
                "{}",
                record.id()
            );
        } else {
            assert_eq!(
                result
                    .expect_err("reference shift leaves SabiQuant range")
                    .kind(),
                record
                    .text("expected_error")
                    .expect("fixture expected error"),
                "{}",
                record.id()
            );
        }
    }
    assert_eq!(range_divergences, if quantlib { 721 } else { 0 });
}

#[test]
fn ts_date_06_quantlib_143_accessors_cover_all_109208_reference_dates() {
    let data = fixture("time-date-quantlib");
    assert_eq!(data.header("reference"), Some("QuantLib"));
    assert_eq!(data.record_count(), 109_208);
    for record in data.records() {
        let date: Date = record.parse("date").unwrap();
        assert_eq!(
            date.weekday().number_from_monday(),
            record.parse::<u32>("weekday").unwrap(),
            "{}",
            record.id()
        );
        assert_eq!(
            date.day_of_year(),
            record.parse::<u32>("day_of_year").unwrap(),
            "{}",
            record.id()
        );
        assert_eq!(
            date.last_day_of_month().day(),
            record.parse::<u32>("month_length").unwrap(),
            "{}",
            record.id()
        );
        assert_eq!(
            date.is_end_of_month(),
            record.parse::<u32>("is_end_of_month").unwrap() == 1,
            "{}",
            record.id()
        );
    }
}

#[test]
fn fr_date_04_05_all_119988_month_ends_clamp_across_adjacent_months() {
    for year in 1..=9999 {
        for month in 1..=12 {
            let end = Date::from_ymd(year, month, month_days(year, month)).unwrap();
            for shift in [-1_i64, 1] {
                let target_index = i64::from(year - 1) * 12 + i64::from(month - 1) + shift;
                let result = end.checked_add_months(shift);
                if (0..119_988).contains(&target_index) {
                    let target_year = i32::try_from(target_index / 12 + 1).unwrap();
                    let target_month = u32::try_from(target_index % 12 + 1).unwrap();
                    let expected_day = end.day().min(month_days(target_year, target_month));
                    assert_eq!(
                        result.unwrap().to_ymd(),
                        (target_year, target_month, expected_day)
                    );
                } else {
                    assert_eq!(result.unwrap_err().kind(), "date.out_of_range");
                }
            }
        }
    }
}

#[test]
fn inv_date_03_04_05_seeded_wide_range_arithmetic_properties() {
    check("date arithmetic", 0x4441_5445_4D32_2026, 20_000, |rng| {
        let start_offset = rng.range_i64(0, 3_652_058);
        let end_offset = rng.range_i64(0, 3_652_058);
        let start = Date::MIN.checked_add_days(start_offset).unwrap();
        let end = Date::MIN.checked_add_days(end_offset).unwrap();
        let difference = i64::from(end.days_since(start));
        if difference != end_offset - start_offset
            || start.checked_add_days(difference).unwrap() != end
        {
            return Err(format!(
                "day inverse: start={start}, end={end}, difference={difference}"
            ));
        }
        let weeks = rng.range_i64(-100_000, 100_000);
        let week_result = start.checked_add_weeks(weeks);
        if week_result.as_ref().ok() != start.checked_add_days(7 * weeks).as_ref().ok() {
            return Err(format!("week equivalence: start={start}, weeks={weeks}"));
        }
        if let Ok(shifted) = week_result
            && shifted.weekday() != start.weekday()
        {
            return Err(format!(
                "weekday invariant: start={start}, shifted={shifted}"
            ));
        }
        let years = rng.range_i64(-10_001, 10_001);
        if start.checked_add_years(years).ok() != start.checked_add_months(12 * years).ok() {
            return Err(format!("year equivalence: start={start}, years={years}"));
        }
        Ok(())
    });
}

#[test]
fn fr_date_10_nth_weekdays_match_scanned_complete_400_year_cycle() {
    // Gregorian leap and weekday patterns repeat every 400 years. Enumerate
    // days from a known Saturday, independently of Date::weekday().
    let mut weekday_index = 5_usize; // 2000-01-01 is Saturday.
    for year in 2000..2400 {
        for month in 1..=12 {
            let mut occurrences: [Vec<Date>; 7] = std::array::from_fn(|_| Vec::new());
            for day in 1..=month_days(year, month) {
                occurrences[weekday_index].push(Date::from_ymd(year, month, day).unwrap());
                weekday_index = (weekday_index + 1) % 7;
            }
            for (index, weekday) in Weekday::ALL.into_iter().enumerate() {
                for nth in 1..=5 {
                    let result = Date::nth_weekday(year, month, nth, weekday);
                    let index = usize::try_from(nth - 1).unwrap();
                    match occurrences[usize::try_from(weekday.number_from_monday() - 1).unwrap()]
                        .get(index)
                    {
                        Some(expected) => assert_eq!(result.unwrap(), *expected),
                        None => assert_eq!(result.unwrap_err().kind(), "date.invalid_occurrence"),
                    }
                }
                // Preserve an independent check of ALL's numeric order.
                assert_eq!(
                    usize::try_from(weekday.number_from_monday()).unwrap(),
                    index + 1
                );
            }
        }
    }
    assert_eq!(weekday_index, 5);
}

#[test]
fn fr_date_10_inclusive_weekday_searches_cover_cycle_and_range_edges() {
    for year in [1, 4, 100, 400, 1582, 1900, 2000, 2100, 2400, 9996, 9999] {
        for month in 1..=12 {
            for day in 1..=month_days(year, month) {
                let date = Date::from_ymd(year, month, day).unwrap();
                for weekday in Weekday::ALL {
                    let current = i64::from(date.weekday().number_from_monday());
                    let target = i64::from(weekday.number_from_monday());
                    let forward = (target - current).rem_euclid(7);
                    let backward = (current - target).rem_euclid(7);
                    assert_eq!(
                        date.next_weekday(weekday),
                        date.checked_add_days(forward),
                        "{date} {weekday}"
                    );
                    assert_eq!(
                        date.previous_weekday(weekday),
                        date.checked_sub_days(backward),
                        "{date} {weekday}"
                    );
                }
            }
        }
    }
}

#[test]
fn ts_date_03_07_invalid_components_and_strict_iso_text_are_rejected() {
    for year in [1, 4, 99, 100, 400, 1582, 1900, 2000, 2100, 9999] {
        for month in 1..=12 {
            assert!(Date::from_ymd(year, month, 0).is_err());
            for day in month_days(year, month) + 1..=35 {
                assert_eq!(
                    Date::from_ymd(year, month, day).unwrap_err().kind(),
                    "date.invalid_date"
                );
            }
        }
    }
    for year in [i32::MIN, -1, 0, 10_000, i32::MAX] {
        assert_eq!(
            Date::from_ymd(year, 1, 1).unwrap_err().kind(),
            "date.year_out_of_range"
        );
    }
    for month in [0, 13, u32::MAX] {
        assert!(Date::from_ymd(2000, month, 1).is_err());
    }
    for text in [
        "",
        "2024-2-01",
        "2024-02-1",
        "24-02-01",
        "+2024-02-01",
        " 2024-02-01",
        "2024-02-01 ",
        "2024-02-01T00:00:00",
        "2024/02/01",
        "2024−02−01",
        "２０２４-02-01",
        "2024-02-01\0",
        "2024-02-01\n",
        "abcd-02-01",
    ] {
        assert_eq!(
            text.parse::<Date>().unwrap_err().kind(),
            "date.parse",
            "{text:?}"
        );
    }
    for text in [
        "0000-01-01",
        "2023-02-29",
        "2024-02-30",
        "2024-00-01",
        "2024-13-01",
        "2024-01-00",
        "2024-04-31",
    ] {
        assert!(text.parse::<Date>().is_err(), "{text:?}");
    }
}

fn canonical(period: Period) -> (bool, i64) {
    if period.length() == 0 {
        return (false, 0);
    }
    match period.unit() {
        TimeUnit::Days => (false, i64::from(period.length())),
        TimeUnit::Weeks => (false, 7 * i64::from(period.length())),
        TimeUnit::Months => (true, i64::from(period.length())),
        TimeUnit::Years => (true, 12 * i64::from(period.length())),
    }
}

fn hash(period: Period) -> u64 {
    let mut hasher = DefaultHasher::new();
    period.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn inv_per_01_02_exhaustive_small_lengths_match_independent_canonical_model() {
    let periods: Vec<_> = (-120..=120)
        .flat_map(|length| TimeUnit::ALL.map(|unit| Period::new(length, unit)))
        .collect();
    for &left in &periods {
        assert_eq!(left.normalized().normalized(), left.normalized());
        assert_eq!(left.normalized(), left);
        assert_eq!(left.to_string().parse::<Period>().unwrap(), left);
        let left_key = canonical(left);
        for &right in &periods {
            let right_key = canonical(right);
            let equal = left_key == right_key;
            assert_eq!(left == right, equal, "{left} {right}");
            if equal {
                assert_eq!(hash(left), hash(right), "{left} {right}");
            }
            let comparison = (left_key.0 == right_key.0).then(|| left_key.1.cmp(&right_key.1));
            assert_eq!(left.partial_cmp(&right), comparison, "{left} {right}");
            assert_eq!(
                right.partial_cmp(&left),
                comparison.map(Ordering::reverse),
                "{left} {right}"
            );
        }
    }
}

#[test]
fn inv_per_02_partial_order_is_substitutive_and_transitive_including_every_zero_unit() {
    let periods: Vec<_> = (-2..=2)
        .flat_map(|length| TimeUnit::ALL.map(|unit| Period::new(length, unit)))
        .collect();
    for &left in &periods {
        for &middle in &periods {
            for &right in &periods {
                if left == middle {
                    assert_eq!(left.partial_cmp(&right), middle.partial_cmp(&right));
                    assert_eq!(right.partial_cmp(&left), right.partial_cmp(&middle));
                }
                if left <= middle && middle <= right {
                    assert!(left <= right, "{left} <= {middle} <= {right}");
                }
                if left == middle && middle == right {
                    assert_eq!(left, right);
                }
            }
        }
    }
}

fn expected_sum(left: Period, right: Period, subtract: bool) -> Option<(i32, TimeUnit)> {
    let sign = if subtract { -1_i64 } else { 1 };
    if right.length() == 0 {
        return Some((left.length(), left.unit()));
    }
    if left.length() == 0 {
        return i32::try_from(sign * i64::from(right.length()))
            .ok()
            .map(|n| (n, right.unit()));
    }
    if left.unit() == right.unit() {
        return i32::try_from(i64::from(left.length()) + sign * i64::from(right.length()))
            .ok()
            .map(|n| (n, left.unit()));
    }
    let (left_family, left_value) = canonical(left);
    let (right_family, right_value) = canonical(right);
    if left_family != right_family {
        return None;
    }
    i32::try_from(left_value + sign * right_value)
        .ok()
        .map(|n| {
            (
                n,
                if left_family {
                    TimeUnit::Months
                } else {
                    TimeUnit::Days
                },
            )
        })
}

#[test]
fn ts_per_03_arithmetic_grid_checks_wide_intermediates_cancellation_and_overflow() {
    let mut lengths: Vec<i32> = (-32..=32).collect();
    lengths.extend([
        i32::MIN,
        i32::MIN + 1,
        -i32::MAX,
        -178_956_970,
        178_956_970,
        i32::MAX - 1,
        i32::MAX,
    ]);
    let periods: Vec<_> = lengths
        .into_iter()
        .flat_map(|length| TimeUnit::ALL.map(|unit| Period::new(length, unit)))
        .collect();
    for &left in &periods {
        assert_eq!(
            left.checked_neg().ok(),
            left.length()
                .checked_neg()
                .map(|n| Period::new(n, left.unit()))
        );
        assert_eq!(left.checked_mul(0).unwrap(), Period::new(0, TimeUnit::Days));
        if let Ok(negative) = left.checked_neg() {
            assert_eq!(negative.checked_neg().unwrap(), left);
        }
        for factor in [i32::MIN, -13, -1, 0, 1, 7, 12, i32::MAX] {
            let expected = left
                .length()
                .checked_mul(factor)
                .map(|n| Period::new(n, left.unit()));
            assert_eq!(left.checked_mul(factor).ok(), expected, "{left} * {factor}");
        }
        for &right in &periods {
            for subtract in [false, true] {
                let actual = if subtract {
                    left.checked_sub(right)
                } else {
                    left.checked_add(right)
                };
                let expected = expected_sum(left, right, subtract);
                match expected {
                    Some((length, unit)) => {
                        let actual = actual.unwrap();
                        assert_eq!(
                            (actual.length(), actual.unit()),
                            (length, unit),
                            "{left} +/- {right}; subtract={subtract}"
                        );
                    }
                    None => assert!(actual.is_err(), "{left} +/- {right}; subtract={subtract}"),
                }
            }
        }
    }
}

#[test]
fn ts_per_01_02_quantlib_143_normalization_and_frequency_fixtures() {
    let data = fixture("time-period-quantlib");
    assert_eq!(data.record_count(), 964);
    let mut differences = Vec::new();
    for record in data.records() {
        let period = Period::new(
            record.parse("length").unwrap(),
            record.parse("unit").unwrap(),
        );
        let normal = period.normalized();
        assert_eq!(
            normal.length(),
            record.parse::<i32>("normalized_length").unwrap(),
            "{}",
            record.id()
        );
        assert_eq!(
            normal.unit(),
            record.parse::<TimeUnit>("normalized_unit").unwrap(),
            "{}",
            record.id()
        );
        assert_eq!(
            period.frequency(),
            record.parse_optional::<Frequency>("frequency").unwrap(),
            "{}",
            record.id()
        );
        // No normalization divergence exists, including the 15 rows whose
        // frequency behavior differs. Always compare the actual reference.
        assert_eq!(
            normal.length(),
            record.parse::<i32>("reference_normalized_length").unwrap(),
            "{}",
            record.id()
        );
        assert_eq!(
            normal.unit(),
            record
                .parse::<TimeUnit>("reference_normalized_unit")
                .unwrap(),
            "{}",
            record.id()
        );
        let reference_frequency: Option<Frequency> =
            record.parse_optional("reference_frequency").unwrap();
        let reference_code: i32 = record.parse("reference_frequency_code").unwrap();
        if let Some(reference) = reference_frequency {
            assert_eq!(
                reference_code,
                reference
                    .events_per_year()
                    .map_or(0, |n| i32::try_from(n).unwrap()),
                "{}",
                record.id()
            );
        } else {
            assert!([-1, 999].contains(&reference_code), "{}", record.id());
        }
        if record.text("discrepancy").unwrap().is_empty() {
            assert_eq!(period.frequency(), reference_frequency, "{}", record.id());
        } else {
            assert_eq!(record.text("discrepancy").unwrap(), "D9-PERIOD-FREQUENCY");
            differences.push((period.length(), period.unit().to_string()));
            if period.length() < 0 {
                assert_eq!(
                    reference_frequency,
                    period.checked_neg().unwrap().frequency()
                );
                assert!(reference_frequency.is_some());
                assert_eq!(period.frequency(), None);
            } else if period.length() == 0 {
                assert_eq!(period.unit(), TimeUnit::Years);
                assert_eq!(reference_frequency, Some(Frequency::Once));
                assert_eq!(reference_code, 0);
                assert_eq!(period.frequency(), None);
            } else {
                assert_eq!(period.unit(), TimeUnit::Days);
                assert_eq!(reference_frequency, None);
                assert_eq!(reference_code, 999);
                assert_eq!(
                    period.frequency(),
                    match period.length() {
                        7 => Some(Frequency::Weekly),
                        14 => Some(Frequency::Biweekly),
                        28 => Some(Frequency::EveryFourthWeek),
                        _ => None,
                    }
                );
            }
        }
    }
    let mut expected = vec![
        (-12, TimeUnit::Months),
        (-6, TimeUnit::Months),
        (-4, TimeUnit::Weeks),
        (-4, TimeUnit::Months),
        (-3, TimeUnit::Months),
        (-2, TimeUnit::Weeks),
        (-2, TimeUnit::Months),
        (-1, TimeUnit::Days),
        (-1, TimeUnit::Weeks),
        (-1, TimeUnit::Months),
        (-1, TimeUnit::Years),
        (0, TimeUnit::Years),
        (7, TimeUnit::Days),
        (14, TimeUnit::Days),
        (28, TimeUnit::Days),
    ]
    .into_iter()
    .map(|(n, unit)| (n, unit.to_string()))
    .collect::<Vec<_>>();
    differences.sort();
    expected.sort();
    assert_eq!(differences, expected);
}

#[test]
fn ts_per_01_quantlib_143_comparison_differences_are_explicitly_classified() {
    let data = fixture("time-period-quantlib-comparisons");
    assert_eq!(data.record_count(), 4096);
    let mut divergences = 0;
    for record in data.records() {
        let left = Period::new(
            record.parse("left_length").unwrap(),
            record.parse("left_unit").unwrap(),
        );
        let right = Period::new(
            record.parse("right_length").unwrap(),
            record.parse("right_unit").unwrap(),
        );
        let actual = match left.partial_cmp(&right) {
            Some(Ordering::Less) => "less",
            Some(Ordering::Equal) => "equal",
            Some(Ordering::Greater) => "greater",
            None => "incomparable",
        };
        assert_eq!(
            actual,
            record.text("comparison").unwrap(),
            "{}",
            record.id()
        );
        if record.text("discrepancy").unwrap().is_empty() {
            assert_eq!(
                actual,
                record.text("reference_comparison").unwrap(),
                "{}",
                record.id()
            );
        } else {
            assert_eq!(record.text("discrepancy").unwrap(), "D9-PERIOD-ORDER");
            assert_ne!(canonical(left).0, canonical(right).0);
            assert_eq!(actual, "incomparable");
            assert!(
                ["less", "greater", "raises"]
                    .contains(&record.text("reference_comparison").unwrap())
            );
            divergences += 1;
        }
    }
    assert_eq!(divergences, 2040);
}

#[test]
fn ts_per_02_all_named_frequencies_match_quantlib_table_and_round_trip() {
    let data = fixture("time-period-quantlib-frequencies");
    assert_eq!(data.record_count(), Frequency::ALL.len());
    for record in data.records() {
        let frequency: Frequency = record.parse("frequency").unwrap();
        let period = record
            .parse_optional::<i32>("length")
            .unwrap()
            .map(|n| Period::new(n, record.parse("unit").unwrap()));
        assert_eq!(frequency.period(), period, "{}", record.id());
        if let Some(period) = period {
            assert_eq!(
                (period.length(), period.unit()),
                (
                    record.parse::<i32>("reference_length").unwrap(),
                    record.parse::<TimeUnit>("reference_unit").unwrap(),
                ),
                "{}",
                record.id()
            );
            assert_eq!(period.frequency(), Some(frequency));
            assert_eq!(
                frequency.events_per_year(),
                Some(record.parse::<u32>("reference_events_per_year").unwrap())
            );
        } else {
            assert_eq!(frequency, Frequency::Once);
            assert_eq!(frequency.events_per_year(), None);
            assert_eq!(record.text("discrepancy").unwrap(), "D9-ONCE-PERIOD");
            assert_eq!(record.parse::<i32>("reference_length").unwrap(), 0);
            assert_eq!(
                record.parse::<TimeUnit>("reference_unit").unwrap(),
                TimeUnit::Years
            );
            assert_eq!(record.parse::<u32>("reference_events_per_year").unwrap(), 0);
        }
    }
}

#[test]
fn public_time_values_and_errors_have_standard_traits_and_are_thread_safe() {
    fn value<T: Copy + Debug + Eq + Hash + Send + Sync + 'static>() {}
    fn error<T: std::error::Error + Clone + Debug + PartialEq + Send + Sync + 'static>() {}
    value::<Date>();
    value::<Period>();
    value::<TimeUnit>();
    value::<Frequency>();
    value::<Weekday>();
    value::<Month>();
    error::<DateError>();
    error::<PeriodError>();
}
