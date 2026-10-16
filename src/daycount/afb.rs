//! Constant-time evaluation of the ISDA-published AFB interpretation.

use crate::time::Date;

pub(super) fn forward(start: Date, end: Date) -> f64 {
    let (years, days, denominator) = decomposition(start, end);
    let residual_fraction = f64::from(days) / f64::from(denominator);
    f64::from(years) + residual_fraction
}

fn decomposition(start: Date, end: Date) -> (i32, u32, u32) {
    let start_year = start.year();
    let start_day = start.day_of_year();
    let mut years = end.year() - start_year;
    if years > 0 && rolled_day_of_year(end, start_year, years) < start_day {
        years -= 1;
    }
    // A positive rollback preserves month/day except that every rolled
    // February 28/29 is the target year's February end. Comparing the one
    // candidate in start.year therefore finds the maximum complete count.
    let residual_year = end.year() - years;
    let residual_day = rolled_day_of_year(end, residual_year, years);
    let next_year = residual_year > start_year;
    let days = if next_year {
        365 + u32::from(Date::is_leap_year(start_year)) - start_day + residual_day
    } else {
        residual_day - start_day
    };
    // The residual end is in start.year or start.year+1. Test the actual
    // half-open membership, including an initial Feb29 but excluding an
    // ending Feb29. Neither a Date nor a successor year is constructed.
    let start_year_leap_day =
        Date::is_leap_year(start_year) && start_day <= 60 && (next_year || residual_day > 60);
    let next_year_leap_day = next_year && Date::is_leap_year(residual_year) && residual_day > 60;
    let denominator = if start_year_leap_day || next_year_leap_day {
        366
    } else {
        365
    };
    (years, days, denominator)
}

fn rolled_day_of_year(end: Date, target_year: i32, years: i32) -> u32 {
    if years == 0 {
        // An original leap-year Feb28 must stay Feb28. Promotion applies
        // only to a positive backward move, not to the caller's endpoint.
        return end.day_of_year();
    }
    if end.month() == 2 && end.day() >= 28 {
        59 + u32::from(Date::is_leap_year(target_year))
    } else {
        let after_february = end.month() > 2;
        end.day_of_year() - u32::from(after_february && Date::is_leap_year(end.year()))
            + u32::from(after_february && Date::is_leap_year(target_year))
    }
}

#[cfg(test)]
mod tests {
    use super::decomposition;
    use crate::time::Date;

    #[test]
    fn afb_discrete_decomposition_preserves_the_half_open_february_boundary() {
        // FS-DCC §9.6; TS-DCC-02/03; OQ-16: hand-derived tuples distinguish whole-year
        // selection from the residual denominator; no floating-point oracle.
        for (start, end, expected) in [
            ("2024-02-28", "2025-02-28", (1, 1, 365)),
            ("2024-02-28", "2024-02-29", (0, 1, 365)),
            ("2024-02-29", "2024-03-01", (0, 1, 366)),
            ("2023-03-01", "2024-02-29", (0, 365, 365)),
            ("2023-12-31", "2024-02-29", (0, 60, 365)),
            ("2023-12-31", "2024-03-01", (0, 61, 366)),
            ("2023-03-01", "2024-02-28", (0, 364, 365)),
            ("2024-02-28", "2028-02-28", (4, 1, 365)),
            ("0001-01-01", "9999-12-31", (9998, 364, 365)),
            ("9999-12-31", "9999-12-31", (0, 0, 365)),
        ] {
            let start: Date = start.parse().unwrap();
            let end: Date = end.parse().unwrap();
            assert_eq!(decomposition(start, end), expected, "{start} to {end}");
        }
    }

    #[test]
    fn constant_time_parts_match_independent_repeated_year_and_civil_day_oracle() {
        // FS-DCC §9.6, NFR-11, TS-DCC-02/03/07; the adopted executable rule
        // and source limits are docs/M3_US_AFB_CONTRACT.md and
        // docs/M3_AFB_SOURCE_REVIEW.md. Expected tuples actually remove each
        // year and walk each residual day; they do not use the production
        // target-year selector or its two-year leap-membership expressions.
        type Civil = (i32, u32, u32);

        fn leap(year: i32) -> bool {
            year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
        }

        fn lengths(year: i32) -> [u32; 12] {
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

        fn ordinal((year, month, day): Civil) -> i64 {
            let completed = i64::from(year - 1);
            completed * 365 + completed / 4 - completed / 100
                + completed / 400
                + i64::from(
                    lengths(year)[..usize::try_from(month - 1).unwrap()]
                        .iter()
                        .sum::<u32>(),
                )
                + i64::from(day - 1)
        }

        fn next_day((year, month, day): Civil) -> Civil {
            if day < lengths(year)[usize::try_from(month - 1).unwrap()] {
                (year, month, day + 1)
            } else if month < 12 {
                (year, month + 1, 1)
            } else {
                assert!(year < 9999, "residual membership walk stays within MAX");
                (year + 1, 1, 1)
            }
        }

        fn oracle(start: Civil, end: Civil) -> ((i32, u32, u32), u64) {
            let mut remaining = end;
            let mut years = 0;
            let mut attempts = 0;
            while remaining.0 > 1 {
                let target_year = remaining.0 - 1;
                let mut day = remaining
                    .2
                    .min(lengths(target_year)[usize::try_from(remaining.1 - 1).unwrap()]);
                if remaining.1 == 2 && day == 28 && leap(target_year) {
                    day = 29;
                }
                let candidate = (target_year, remaining.1, day);
                attempts += 1;
                if candidate < start {
                    break;
                }
                remaining = candidate;
                years += 1;
            }
            let mut cursor = start;
            let mut days = 0;
            let mut contains_february_29 = false;
            while cursor < remaining {
                contains_february_29 |= cursor.1 == 2 && cursor.2 == 29;
                days += 1;
                cursor = next_day(cursor);
            }
            assert_eq!(i64::from(days), ordinal(remaining) - ordinal(start));
            (
                (years, days, if contains_february_29 { 366 } else { 365 }),
                attempts,
            )
        }

        let mut checked = 0_u64;
        let mut annual_attempts = 0_u64;
        let mut maximum_residual_days = 0;
        let mut check = |left: Date, right: Date| {
            let start = left.min(right);
            let end = left.max(right);
            let (expected, attempts) = oracle(start.to_ymd(), end.to_ymd());
            // Compare the complete discrete tuple before any floating-point
            // reduction can hide distinct decompositions with equal totals.
            assert_eq!(decomposition(start, end), expected, "{start} to {end}");
            checked += 1;
            annual_attempts += attempts;
            maximum_residual_days = maximum_residual_days.max(expected.1);
        };

        let cycle_start = Date::from_ymd(2000, 1, 1).unwrap();
        let durations = [
            0, 1, 2, 28, 59, 60, 180, 364, 365, 366, 367, 730, 1096, 1461, 2922, 146_097,
        ];
        for index in 0..146_097_i64 {
            let start = cycle_start.checked_add_days(index).unwrap();
            let profile = usize::try_from(index).unwrap() % durations.len();
            check(start, start.checked_add_days(durations[profile]).unwrap());
        }
        // Twenty profiles for each supported start year. Common-year Feb28
        // and February end coincide; these are profile counts, not a claim
        // that all input pairs are distinct. Reversal is tested publicly.
        for year in 1..=9999 {
            for (start_month, start_day) in
                [(2, 27), (2, 28), (2, lengths(year)[1]), (3, 1), (12, 31)]
            {
                let start = Date::from_ymd(year, start_month, start_day).unwrap();
                let next_year = (year + 1).min(9999);
                let fourth_year = (year + 4).min(9999);
                for (end_year, end_month, end_day) in [
                    (year, 2, lengths(year)[1]),
                    (next_year, 2, 28),
                    (next_year, 3, 1),
                    (fourth_year, 2, lengths(fourth_year)[1]),
                ] {
                    check(start, Date::from_ymd(end_year, end_month, end_day).unwrap());
                }
            }
        }
        let edges = [
            Date::MIN,
            Date::from_ymd(1, 2, 28).unwrap(),
            Date::from_ymd(4, 2, 29).unwrap(),
            Date::from_ymd(1900, 2, 28).unwrap(),
            Date::from_ymd(2000, 2, 29).unwrap(),
            Date::from_ymd(2023, 3, 1).unwrap(),
            Date::from_ymd(2024, 2, 28).unwrap(),
            Date::from_ymd(2024, 2, 29).unwrap(),
            Date::from_ymd(9996, 2, 29).unwrap(),
            Date::from_ymd(9999, 2, 28).unwrap(),
            Date::MAX,
        ];
        for left in edges {
            for right in edges {
                check(left, right);
            }
        }
        assert_eq!(checked, 146_097 + 20 * 9999 + 11 * 11);
        assert_eq!(maximum_residual_days, 365);
        assert!(annual_attempts > checked);
    }
}
