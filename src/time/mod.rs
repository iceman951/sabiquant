//! Exact dates, tenors and periodic frequencies (milestone M2).
//!
//! [`Date`] covers the proleptic Gregorian calendar from 0001-01-01 through
//! 9999-12-31. Arithmetic is checked; adding months clamps the day to the
//! target month's length, and never silently preserves end-of-month status.
//! Dates have neither a time zone nor a time of day. There is no implicit today.
//!
//! [`Period`] has normalized equality and hashing: `12M == 1Y`, `14D == 2W`,
//! and all zero periods are equal. Ordering uses the normalized unit family:
//! months/years and days/weeks are separate, and canonical zero belongs to
//! days/weeks. A zero period and a nonzero month/year period are incomparable.
//! This keeps ordering consistent with equality and transitive.
//!
//! # QuantLib correspondence
//!
//! QuantLib 1.43 `Date`, `Period`, `TimeUnit` and `Frequency` are external
//! behavioral references. SabiQuant has a broader date range, ISO weekday
//! numbers, typed errors and deliberately narrower period comparisons.
//! Excel/QuantLib serial-number conversion is deferred. Compatibility claims
//! apply only to the tested scenario sets listed in the validation report.
//!
//! ```
//! use sabiquant::time::{Date, Period, TimeUnit};
//! let date = Date::from_ymd(2024, 1, 31)?;
//! let maturity = date.checked_add_period(Period::new(1, TimeUnit::Months))?;
//! assert_eq!(maturity.to_string(), "2024-02-29");
//! # Ok::<(), sabiquant::time::DateError>(())
//! ```

mod date;
mod period;

pub use date::{Date, DateError, Month, Weekday};
pub use period::{Frequency, Period, PeriodError, TimeUnit};

impl Date {
    /// Adds a tenor with calendar-day semantics for days and weeks.
    ///
    /// Months and years clamp the day to the target month's length. A `Days`
    /// tenor here counts calendar days; business-day advancement belongs to
    /// the calendar module in M3 (FR-PER-05).
    ///
    /// # Errors
    ///
    /// [`DateError::OutOfRange`] when the result is outside the supported range.
    pub fn checked_add_period(self, period: Period) -> Result<Self, DateError> {
        let length = i64::from(period.length());
        match period.unit() {
            TimeUnit::Days => self.checked_add_days(length),
            TimeUnit::Weeks => self.checked_add_weeks(length),
            TimeUnit::Months => self.checked_add_months(length),
            TimeUnit::Years => self.checked_add_years(length),
        }
    }

    /// Subtracts a tenor, counting days and weeks as calendar days.
    ///
    /// The length is widened before negation, so subtracting a tenor with
    /// length `i32::MIN` is checked without overflowing.
    ///
    /// # Errors
    ///
    /// [`DateError::OutOfRange`] when the result is outside the supported range.
    pub fn checked_sub_period(self, period: Period) -> Result<Self, DateError> {
        let length = -i64::from(period.length());
        match period.unit() {
            TimeUnit::Days => self.checked_add_days(length),
            TimeUnit::Weeks => self.checked_add_weeks(length),
            TimeUnit::Months => self.checked_add_months(length),
            TimeUnit::Years => self.checked_add_years(length),
        }
    }
}
