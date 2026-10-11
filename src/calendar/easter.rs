//! Western Easter in the proleptic Gregorian calendar.

use crate::time::{Date, DateError};

/// Computes Western Easter Sunday for a Gregorian year from 1 through 9999.
///
/// This is ecclesiastical computus, using tabular lunar corrections, rather
/// than an astronomical full-moon calculation. Dates before the Gregorian
/// reform are mathematical extensions of its rules, not historical observances.
/// Orthodox Easter is outside this function's scope.
///
/// The constant-time integer algorithm independently implements Oudin's
/// calculation published by the [U.S. Naval Observatory](https://aa.usno.navy.mil/faq/easter).
/// Its century correction uses mathematical floor division for the proleptic
/// range. Division denominators are positive and arithmetic is bounded after
/// year validation. Every result is a Sunday from 22 March through 25 April.
///
/// # Errors
///
/// Returns [`DateError::YearOutOfRange`] unless `year` is 1 through 9999,
/// preserving the rejected input year.
///
/// # Examples
///
/// ```
/// use sabiquant::calendar::western_easter;
/// assert_eq!(western_easter(2026)?.to_string(), "2026-04-05");
/// # Ok::<(), sabiquant::time::DateError>(())
/// ```
#[expect(
    clippy::cast_sign_loss,
    reason = "after validated-year computus, month is 3 or 4 and day is 1 through 31"
)]
pub fn western_easter(year: i32) -> Result<Date, DateError> {
    // Reject invalid years before any century, cycle or arithmetic operation.
    Date::days_in_year(year)?;
    let century = year / 100;
    let lunar_cycle = year % 19;
    // For centuries before 17 this numerator is negative: Rust's ordinary
    // division truncates toward zero, while computus requires its floor.
    let century_correction = (century - 17).div_euclid(25);
    let epact =
        (century - century / 4 - (century - century_correction) / 3 + 19 * lunar_cycle + 15)
            .rem_euclid(30);
    let corrected_epact =
        epact - (epact / 28) * (1 - (epact / 28) * (29 / (epact + 1)) * ((21 - lunar_cycle) / 11));
    let weekday = (year + year / 4 + corrected_epact + 2 - century + century / 4) % 7;
    let offset = corrected_epact - weekday;
    let month = 3 + (offset + 40) / 44;
    let day = offset + 28 - 31 * (month / 4);
    Date::from_ymd(year, month as u32, day as u32)
}
