//! Calendar dates and exact, checked Gregorian arithmetic.

use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;

use crate::error::IdentifierError;
use crate::identifier::{Entry, Identifiers};

/// A day in the proleptic Gregorian calendar, from 0001-01-01 to 9999-12-31.
///
/// Dates have no time of day or time zone. Gregorian leap-year rules apply
/// throughout the range, including before the Gregorian calendar was adopted.
/// Every value is valid; an absent date is represented by `Option<Date>`.
///
/// Arithmetic is checked and never reads a clock. Month and year addition
/// clamp the day to the target month's length: 2024-01-31 plus one month is
/// 2024-02-29. They do not implicitly preserve end of month: 2024-02-29 plus
/// one month is 2024-03-29. As a result, month addition is neither associative
/// nor invertible. The internal day number is not part of the public API.
///
/// # Examples
///
/// ```
/// use sabiquant::time::Date;
///
/// let date = Date::from_ymd(2024, 1, 31)?;
/// assert_eq!(date.checked_add_months(1)?.to_string(), "2024-02-29");
/// assert_eq!(date.checked_add_days(1)?.to_string(), "2024-02-01");
/// # Ok::<(), sabiquant::time::DateError>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    // Number of complete days since 0001-01-01. Its invariant is
    // 0 <= ordinal <= 3_652_058, so every date fits in four bytes.
    ordinal: i32,
}

impl Date {
    /// The earliest supported date, 0001-01-01.
    pub const MIN: Self = Self { ordinal: 0 };

    /// The latest supported date, 9999-12-31.
    pub const MAX: Self = Self { ordinal: 3_652_058 };

    /// Constructs a date from its Gregorian year, month and day.
    ///
    /// # Errors
    ///
    /// Returns [`DateError::YearOutOfRange`] unless `year` is 1 through 9999,
    /// or [`DateError::InvalidDate`] if `month` or `day` is invalid. The year
    /// is checked first, so rejection is deterministic for multiple errors.
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Result<Self, DateError> {
        validate_year(year)?;
        let Some(length) = month_length(year, month) else {
            return Err(DateError::InvalidDate { year, month, day });
        };
        if day == 0 || day > length {
            return Err(DateError::InvalidDate { year, month, day });
        }
        let completed_years = year - 1;
        let days_before_year = 365 * completed_years + completed_years / 4 - completed_years / 100
            + completed_years / 400;
        let before_month = days_before_month(month, Self::is_leap_year(year));
        Ok(Self {
            ordinal: days_before_year + bounded_u32_to_i32(before_month + day - 1),
        })
    }

    /// Returns the Gregorian year, in 1 through 9999.
    #[must_use]
    pub const fn year(self) -> i32 {
        self.year_and_ordinal_day().0
    }

    /// Returns the Gregorian month number, in 1 through 12.
    #[must_use]
    pub const fn month(self) -> u32 {
        self.to_ymd().1
    }

    /// Returns the day of the month, in 1 through 31.
    #[must_use]
    pub const fn day(self) -> u32 {
        self.to_ymd().2
    }

    /// Returns the Gregorian `(year, month, day)` components.
    ///
    /// Conversion takes constant time using Gregorian 400-, 100-, 4- and
    /// 1-year cycles. It does not loop over years or depend on the date's age.
    #[must_use]
    pub const fn to_ymd(self) -> (i32, u32, u32) {
        let (year, day) = self.year_and_ordinal_day();
        let leap = Self::is_leap_year(year);
        let mut month = 1;
        // At most eleven comparisons, independent of the year or date range.
        while month < 12 && day >= days_before_month(month + 1, leap) {
            month += 1;
        }
        (year, month, day - days_before_month(month, leap) + 1)
    }

    /// Returns the ISO weekday: Monday through Sunday.
    #[must_use]
    pub const fn weekday(self) -> Weekday {
        // 0001-01-01 is Monday in the proleptic Gregorian calendar.
        match self.ordinal % 7 {
            0 => Weekday::Monday,
            1 => Weekday::Tuesday,
            2 => Weekday::Wednesday,
            3 => Weekday::Thursday,
            4 => Weekday::Friday,
            5 => Weekday::Saturday,
            _ => Weekday::Sunday,
        }
    }

    /// Returns the day of the year, numbered from 1 through 365 or 366.
    #[must_use]
    pub const fn day_of_year(self) -> u32 {
        self.year_and_ordinal_day().1 + 1
    }

    /// Returns whether `year` is divisible by 4 except centuries not
    /// divisible by 400.
    ///
    /// This rule is defined for every integer year, including years outside
    /// the supported `Date` range. Constructing a date still requires 1–9999.
    #[must_use]
    pub const fn is_leap_year(year: i32) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    /// Returns the length of a supported Gregorian year: 365 or 366 days.
    ///
    /// # Errors
    ///
    /// Returns [`DateError::YearOutOfRange`] unless `year` is 1 through 9999.
    pub fn days_in_year(year: i32) -> Result<u32, DateError> {
        validate_year(year)?;
        Ok(if Self::is_leap_year(year) { 366 } else { 365 })
    }

    /// Returns the number of days in the specified Gregorian month.
    ///
    /// # Errors
    ///
    /// Returns [`DateError::YearOutOfRange`] for an unsupported year, or
    /// [`DateError::InvalidMonth`] unless `month` is 1 through 12.
    pub fn days_in_month(year: i32, month: u32) -> Result<u32, DateError> {
        validate_year(year)?;
        month_length(year, month).ok_or(DateError::InvalidMonth { month })
    }

    /// Adds a signed number of calendar days exactly.
    ///
    /// # Errors
    ///
    /// Returns [`DateError::OutOfRange`] if the result is outside the
    /// supported range, including overflow for extreme `days` values.
    pub fn checked_add_days(self, days: i64) -> Result<Self, DateError> {
        self.shift_days(days, "add days", false)
    }

    /// Subtracts a signed number of calendar days exactly.
    ///
    /// # Errors
    ///
    /// Returns [`DateError::OutOfRange`] if the result is outside the
    /// supported range, including `days == i64::MIN`.
    pub fn checked_sub_days(self, days: i64) -> Result<Self, DateError> {
        self.shift_days(days, "subtract days", true)
    }

    /// Adds a signed number of weeks, each exactly seven calendar days.
    ///
    /// # Errors
    ///
    /// Returns [`DateError::OutOfRange`] if the result is outside the
    /// supported range or multiplying `weeks` by seven overflows.
    pub fn checked_add_weeks(self, weeks: i64) -> Result<Self, DateError> {
        let error = || self.range_error("add weeks", weeks);
        let days = weeks.checked_mul(7).ok_or_else(error)?;
        self.checked_add_days(days).map_err(|_| error())
    }

    /// Adds signed calendar months and clamps to the target month's length.
    ///
    /// For example, 2023-01-31 plus one month is 2023-02-28; 2023-02-28
    /// plus one month is 2023-03-28. Repeated additions can differ from one
    /// combined addition because each operation clamps independently.
    ///
    /// # Errors
    ///
    /// Returns [`DateError::OutOfRange`] if the target year is outside
    /// 1 through 9999 or adding `months` overflows.
    pub fn checked_add_months(self, months: i64) -> Result<Self, DateError> {
        let (year, month, day) = self.to_ymd();
        let index = i64::from(year - 1) * 12 + i64::from(month - 1);
        let error = || self.range_error("add months", months);
        let target = index.checked_add(months).ok_or_else(error)?;
        if !(0..9999 * 12).contains(&target) {
            return Err(error());
        }
        let target_year = bounded_i64_to_i32(target / 12 + 1);
        let target_month = bounded_i64_to_u32(target % 12 + 1);
        // The target index establishes that these components are valid.
        let length = Self::days_in_month(target_year, target_month)?;
        Self::from_ymd(target_year, target_month, day.min(length))
    }

    /// Adds signed calendar years, with the same clamping as adding twelve
    /// months per year.
    ///
    /// For example, 2024-02-29 plus one year is 2025-02-28.
    ///
    /// # Errors
    ///
    /// Returns [`DateError::OutOfRange`] if the target year is outside
    /// 1 through 9999 or multiplying `years` by twelve overflows.
    pub fn checked_add_years(self, years: i64) -> Result<Self, DateError> {
        let error = || self.range_error("add years", years);
        let months = years.checked_mul(12).ok_or_else(error)?;
        self.checked_add_months(months).map_err(|_| error())
    }

    /// Returns the signed number of calendar days from `other` to `self`.
    ///
    /// The result is positive when `self` is later. The full supported
    /// range fits in an `i32`.
    #[must_use]
    pub const fn days_since(self, other: Self) -> i32 {
        self.ordinal - other.ordinal
    }

    /// Returns the first calendar day of this date's month.
    #[must_use]
    pub const fn first_day_of_month(self) -> Self {
        Self {
            ordinal: self.ordinal - bounded_u32_to_i32(self.day() - 1),
        }
    }

    /// Returns the last calendar day of this date's month.
    #[must_use]
    pub const fn last_day_of_month(self) -> Self {
        let (year, month, day) = self.to_ymd();
        let leap = Self::is_leap_year(year);
        let end_of_month = if month == 12 {
            if leap { 366 } else { 365 }
        } else {
            days_before_month(month + 1, leap)
        };
        let length = end_of_month - days_before_month(month, leap);
        Self {
            ordinal: self.ordinal + bounded_u32_to_i32(length - day),
        }
    }

    /// Returns whether this is the last calendar day of its month.
    #[must_use]
    pub const fn is_end_of_month(self) -> bool {
        self.ordinal == self.last_day_of_month().ordinal
    }

    /// Finds the `nth` occurrence of `weekday` in the specified month.
    ///
    /// Occurrences are numbered from one; a fifth occurrence is accepted
    /// only when that weekday actually occurs five times in the month.
    ///
    /// # Errors
    ///
    /// Returns the constructor error for an unsupported year or month, or
    /// [`DateError::InvalidOccurrence`] if `nth` is outside 1 through 5 or
    /// the requested occurrence does not exist.
    pub fn nth_weekday(
        year: i32,
        month: u32,
        nth: u32,
        weekday: Weekday,
    ) -> Result<Self, DateError> {
        let first = Self::from_ymd(year, month, 1)?;
        let error = || DateError::InvalidOccurrence {
            year,
            month,
            nth,
            weekday,
        };
        if !(1..=5).contains(&nth) {
            return Err(error());
        }
        let first_number = first.weekday().number_from_monday();
        let offset = (weekday.number_from_monday() + 7 - first_number) % 7;
        let day = 1 + offset + 7 * (nth - 1);
        if day > Self::days_in_month(year, month)? {
            return Err(error());
        }
        Self::from_ymd(year, month, day)
    }

    /// Finds the given weekday on or after this date, including this date
    /// when it already has that weekday.
    ///
    /// # Errors
    ///
    /// Returns [`DateError::OutOfRange`] if the requested weekday would be
    /// after [`Date::MAX`].
    pub fn next_weekday(self, weekday: Weekday) -> Result<Self, DateError> {
        let offset = (weekday.number_from_monday() + 7 - self.weekday().number_from_monday()) % 7;
        self.checked_add_days(i64::from(offset))
    }

    /// Finds the given weekday on or before this date, including this date
    /// when it already has that weekday.
    ///
    /// # Errors
    ///
    /// Returns [`DateError::OutOfRange`] if the requested weekday would be
    /// before [`Date::MIN`].
    pub fn previous_weekday(self, weekday: Weekday) -> Result<Self, DateError> {
        let offset = (self.weekday().number_from_monday() + 7 - weekday.number_from_monday()) % 7;
        self.checked_sub_days(i64::from(offset))
    }

    const fn year_and_ordinal_day(self) -> (i32, u32) {
        // Each complete 400-year block contains 146097 days. The first
        // three centuries each contain 36524; the fourth contains 36525.
        // Each four-year block contains 1461 days, with its leap day last.
        // Capping the century/year quotient assigns the final leap day to
        // the final year instead of a nonexistent next block.
        let era = self.ordinal / 146_097;
        let remaining = self.ordinal % 146_097;
        let century = if remaining / 36_524 < 3 {
            remaining / 36_524
        } else {
            3
        };
        let remaining = remaining - century * 36_524;
        let quadrennium = remaining / 1461;
        let remaining = remaining % 1461;
        let year = if remaining / 365 < 3 {
            remaining / 365
        } else {
            3
        };
        (
            era * 400 + century * 100 + quadrennium * 4 + year + 1,
            bounded_i32_to_u32(remaining - year * 365),
        )
    }

    fn shift_days(
        self,
        count: i64,
        operation: &'static str,
        subtract: bool,
    ) -> Result<Self, DateError> {
        let ordinal = i64::from(self.ordinal);
        let target = if subtract {
            ordinal.checked_sub(count)
        } else {
            ordinal.checked_add(count)
        };
        match target {
            Some(target) if (0..=i64::from(Self::MAX.ordinal)).contains(&target) => Ok(Self {
                ordinal: bounded_i64_to_i32(target),
            }),
            _ => Err(self.range_error(operation, count)),
        }
    }

    const fn range_error(self, operation: &'static str, count: i64) -> DateError {
        DateError::OutOfRange {
            date: self,
            operation,
            count,
        }
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (year, month, day) = self.to_ymd();
        write!(f, "{year:04}-{month:02}-{day:02}")
    }
}

impl FromStr for Date {
    type Err = DateError;

    /// Parses exactly ten ASCII bytes in the form `YYYY-MM-DD`.
    ///
    /// No whitespace, sign, alternate separators or Unicode digits are
    /// accepted. Syntactically valid text is validated by [`Date::from_ymd`].
    ///
    /// # Errors
    ///
    /// Returns [`DateError::Parse`] with the original input for malformed
    /// text, or the constructor error for an unsupported year or invalid date.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let bytes = text.as_bytes();
        if bytes.len() != 10
            || bytes[4] != b'-'
            || bytes[7] != b'-'
            || !bytes
                .iter()
                .enumerate()
                .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
        {
            return Err(DateError::Parse {
                text: text.to_owned(),
            });
        }
        let digit = |i: usize| u32::from(bytes[i] - b'0');
        let year = bounded_u32_to_i32(1000 * digit(0) + 100 * digit(1) + 10 * digit(2) + digit(3));
        let month = 10 * digit(5) + digit(6);
        let day = 10 * digit(8) + digit(9);
        Self::from_ymd(year, month, day)
    }
}

/// A failed date construction, arithmetic operation or parse.
///
/// Kind names returned by [`kind`](Self::kind) are stable; display messages
/// may change. Errors retain rejected components, text or arithmetic inputs.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DateError {
    /// A year is outside 1 through 9999. Kind: `date.year_out_of_range`.
    #[non_exhaustive]
    YearOutOfRange {
        /// The rejected Gregorian year.
        year: i32,
    },
    /// Components do not form a date. Kind: `date.invalid_date`.
    #[non_exhaustive]
    InvalidDate {
        /// The given Gregorian year, already within the supported range.
        year: i32,
        /// The given month number.
        month: u32,
        /// The given day of month.
        day: u32,
    },
    /// Arithmetic would leave the supported range. Kind: `date.out_of_range`.
    #[non_exhaustive]
    OutOfRange {
        /// The date on which arithmetic was attempted.
        date: Date,
        /// The operation, for example `"add months"`.
        operation: &'static str,
        /// The signed count given to that operation.
        count: i64,
    },
    /// A month number is outside 1 through 12. Kind: `date.invalid_month`.
    #[non_exhaustive]
    InvalidMonth {
        /// The rejected month number.
        month: u32,
    },
    /// A weekday number is outside 1 through 7. Kind: `date.invalid_weekday`.
    #[non_exhaustive]
    InvalidWeekday {
        /// The rejected ISO weekday number.
        weekday: u32,
    },
    /// An n-th weekday does not exist. Kind: `date.invalid_occurrence`.
    #[non_exhaustive]
    InvalidOccurrence {
        /// The given Gregorian year.
        year: i32,
        /// The given month number.
        month: u32,
        /// The requested occurrence, counted from one.
        nth: u32,
        /// The requested weekday.
        weekday: Weekday,
    },
    /// Text is not exactly ASCII `YYYY-MM-DD`. Kind: `date.parse`.
    #[non_exhaustive]
    Parse {
        /// The rejected input without normalization.
        text: String,
    },
}

impl DateError {
    /// Returns the stable `date.<variant>` kind name.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::YearOutOfRange { .. } => "date.year_out_of_range",
            Self::InvalidDate { .. } => "date.invalid_date",
            Self::OutOfRange { .. } => "date.out_of_range",
            Self::InvalidMonth { .. } => "date.invalid_month",
            Self::InvalidWeekday { .. } => "date.invalid_weekday",
            Self::InvalidOccurrence { .. } => "date.invalid_occurrence",
            Self::Parse { .. } => "date.parse",
        }
    }
}

impl fmt::Display for DateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::YearOutOfRange { year } => write!(f, "Gregorian year {year} is outside 1..=9999"),
            Self::InvalidDate { year, month, day } => write!(
                f,
                "invalid Gregorian date: year {year}, month {month}, day {day}"
            ),
            Self::OutOfRange {
                date,
                operation,
                count,
            } => write!(
                f,
                "cannot {operation} ({count}) to {date}: result is outside 0001-01-01..=9999-12-31"
            ),
            Self::InvalidMonth { month } => write!(f, "month {month} is outside 1..=12"),
            Self::InvalidWeekday { weekday } => write!(f, "ISO weekday {weekday} is outside 1..=7"),
            Self::InvalidOccurrence {
                year,
                month,
                nth,
                weekday,
            } => write!(
                f,
                "occurrence {nth} of {weekday} does not exist in {year:04}-{month:02}"
            ),
            Self::Parse { text } => write!(f, "date {text:?} must be exactly ASCII YYYY-MM-DD"),
        }
    }
}

impl StdError for DateError {}

/// An ISO weekday, numbered Monday = 1 through Sunday = 7.
///
/// Text identifiers are the exact English names, such as `"Monday"`.
/// Parsing is case-sensitive and accepts no abbreviations or whitespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[expect(
    clippy::exhaustive_enums,
    reason = "the Gregorian week has exactly seven weekdays"
)]
pub enum Weekday {
    /// Monday, ISO number 1.
    Monday = 1,
    /// Tuesday, ISO number 2.
    Tuesday = 2,
    /// Wednesday, ISO number 3.
    Wednesday = 3,
    /// Thursday, ISO number 4.
    Thursday = 4,
    /// Friday, ISO number 5.
    Friday = 5,
    /// Saturday, ISO number 6.
    Saturday = 6,
    /// Sunday, ISO number 7.
    Sunday = 7,
}

impl Weekday {
    /// All weekdays in ISO order, Monday through Sunday.
    pub const ALL: [Self; 7] = [
        Self::Monday,
        Self::Tuesday,
        Self::Wednesday,
        Self::Thursday,
        Self::Friday,
        Self::Saturday,
        Self::Sunday,
    ];

    /// Returns the ISO number, Monday = 1 through Sunday = 7.
    #[must_use]
    pub const fn number_from_monday(self) -> u32 {
        self as u32
    }
}

impl TryFrom<u32> for Weekday {
    type Error = DateError;

    /// Converts an ISO weekday number.
    ///
    /// # Errors
    ///
    /// Returns [`DateError::InvalidWeekday`] unless the number is 1–7.
    fn try_from(weekday: u32) -> Result<Self, Self::Error> {
        match weekday {
            1 => Ok(Self::Monday),
            2 => Ok(Self::Tuesday),
            3 => Ok(Self::Wednesday),
            4 => Ok(Self::Thursday),
            5 => Ok(Self::Friday),
            6 => Ok(Self::Saturday),
            7 => Ok(Self::Sunday),
            _ => Err(DateError::InvalidWeekday { weekday }),
        }
    }
}

impl fmt::Display for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Monday => "Monday",
            Self::Tuesday => "Tuesday",
            Self::Wednesday => "Wednesday",
            Self::Thursday => "Thursday",
            Self::Friday => "Friday",
            Self::Saturday => "Saturday",
            Self::Sunday => "Sunday",
        })
    }
}

impl FromStr for Weekday {
    type Err = IdentifierError;

    /// Parses an exact canonical English weekday name.
    ///
    /// # Errors
    ///
    /// Returns [`IdentifierError::Unknown`] for any other spelling.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        WEEKDAY_IDENTIFIERS.parse(text)
    }
}

/// A Gregorian month, numbered January = 1 through December = 12.
///
/// Text identifiers are the exact English names, such as `"January"`.
/// Parsing is case-sensitive and accepts no abbreviations or whitespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[expect(
    clippy::exhaustive_enums,
    reason = "the Gregorian calendar has exactly twelve months"
)]
pub enum Month {
    /// January, month 1.
    January = 1,
    /// February, month 2.
    February = 2,
    /// March, month 3.
    March = 3,
    /// April, month 4.
    April = 4,
    /// May, month 5.
    May = 5,
    /// June, month 6.
    June = 6,
    /// July, month 7.
    July = 7,
    /// August, month 8.
    August = 8,
    /// September, month 9.
    September = 9,
    /// October, month 10.
    October = 10,
    /// November, month 11.
    November = 11,
    /// December, month 12.
    December = 12,
}

impl Month {
    /// All months in calendar order.
    pub const ALL: [Self; 12] = [
        Self::January,
        Self::February,
        Self::March,
        Self::April,
        Self::May,
        Self::June,
        Self::July,
        Self::August,
        Self::September,
        Self::October,
        Self::November,
        Self::December,
    ];

    /// Returns the Gregorian month number, January = 1 through December = 12.
    #[must_use]
    pub const fn number(self) -> u32 {
        self as u32
    }
}

impl TryFrom<u32> for Month {
    type Error = DateError;

    /// Converts a Gregorian month number.
    ///
    /// # Errors
    ///
    /// Returns [`DateError::InvalidMonth`] unless the number is 1–12.
    fn try_from(month: u32) -> Result<Self, Self::Error> {
        match month {
            1 => Ok(Self::January),
            2 => Ok(Self::February),
            3 => Ok(Self::March),
            4 => Ok(Self::April),
            5 => Ok(Self::May),
            6 => Ok(Self::June),
            7 => Ok(Self::July),
            8 => Ok(Self::August),
            9 => Ok(Self::September),
            10 => Ok(Self::October),
            11 => Ok(Self::November),
            12 => Ok(Self::December),
            _ => Err(DateError::InvalidMonth { month }),
        }
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::January => "January",
            Self::February => "February",
            Self::March => "March",
            Self::April => "April",
            Self::May => "May",
            Self::June => "June",
            Self::July => "July",
            Self::August => "August",
            Self::September => "September",
            Self::October => "October",
            Self::November => "November",
            Self::December => "December",
        })
    }
}

impl FromStr for Month {
    type Err = IdentifierError;

    /// Parses an exact canonical English month name.
    ///
    /// # Errors
    ///
    /// Returns [`IdentifierError::Unknown`] for any other spelling.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        MONTH_IDENTIFIERS.parse(text)
    }
}

const WEEKDAY_IDENTIFIERS: Identifiers<Weekday> = Identifiers::new(
    "weekday",
    &[
        Entry {
            value: Weekday::Monday,
            canonical: "Monday",
            aliases: &[],
        },
        Entry {
            value: Weekday::Tuesday,
            canonical: "Tuesday",
            aliases: &[],
        },
        Entry {
            value: Weekday::Wednesday,
            canonical: "Wednesday",
            aliases: &[],
        },
        Entry {
            value: Weekday::Thursday,
            canonical: "Thursday",
            aliases: &[],
        },
        Entry {
            value: Weekday::Friday,
            canonical: "Friday",
            aliases: &[],
        },
        Entry {
            value: Weekday::Saturday,
            canonical: "Saturday",
            aliases: &[],
        },
        Entry {
            value: Weekday::Sunday,
            canonical: "Sunday",
            aliases: &[],
        },
    ],
    &[],
);

const MONTH_IDENTIFIERS: Identifiers<Month> = Identifiers::new(
    "month",
    &[
        Entry {
            value: Month::January,
            canonical: "January",
            aliases: &[],
        },
        Entry {
            value: Month::February,
            canonical: "February",
            aliases: &[],
        },
        Entry {
            value: Month::March,
            canonical: "March",
            aliases: &[],
        },
        Entry {
            value: Month::April,
            canonical: "April",
            aliases: &[],
        },
        Entry {
            value: Month::May,
            canonical: "May",
            aliases: &[],
        },
        Entry {
            value: Month::June,
            canonical: "June",
            aliases: &[],
        },
        Entry {
            value: Month::July,
            canonical: "July",
            aliases: &[],
        },
        Entry {
            value: Month::August,
            canonical: "August",
            aliases: &[],
        },
        Entry {
            value: Month::September,
            canonical: "September",
            aliases: &[],
        },
        Entry {
            value: Month::October,
            canonical: "October",
            aliases: &[],
        },
        Entry {
            value: Month::November,
            canonical: "November",
            aliases: &[],
        },
        Entry {
            value: Month::December,
            canonical: "December",
            aliases: &[],
        },
    ],
    &[],
);

fn validate_year(year: i32) -> Result<(), DateError> {
    if (1..=9999).contains(&year) {
        Ok(())
    } else {
        Err(DateError::YearOutOfRange { year })
    }
}

const fn month_length(year: i32, month: u32) -> Option<u32> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 => Some(if Date::is_leap_year(year) { 29 } else { 28 }),
        _ => None,
    }
}

const fn days_before_month(month: u32, leap: bool) -> u32 {
    let common = match month {
        1 => 0,
        2 => 31,
        3 => 59,
        4 => 90,
        5 => 120,
        6 => 151,
        7 => 181,
        8 => 212,
        9 => 243,
        10 => 273,
        11 => 304,
        _ => 334,
    };
    common + if leap && month > 2 { 1 } else { 0 }
}

#[expect(
    clippy::cast_possible_wrap,
    reason = "only validated day, month and four-digit-year values <= 9999 reach this helper"
)]
const fn bounded_u32_to_i32(value: u32) -> i32 {
    value as i32
}

#[expect(
    clippy::cast_sign_loss,
    reason = "cycle decomposition only produces nonnegative ordinal days <= 365"
)]
const fn bounded_i32_to_u32(value: i32) -> u32 {
    value as u32
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "callers first bound targets to the supported date or month range, both fitting i32"
)]
const fn bounded_i64_to_i32(value: i64) -> i32 {
    value as i32
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a validated target month index produces only month numbers 1 through 12"
)]
const fn bounded_i64_to_u32(value: i64) -> u32 {
    value as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeSet, HashSet};
    use std::mem::size_of;

    fn date(year: i32, month: u32, day: u32) -> Date {
        Date::from_ymd(year, month, day).expect("valid test date")
    }

    #[test]
    fn range_endpoints_and_compact_representation() {
        assert_eq!(Date::MIN, date(1, 1, 1));
        assert_eq!(Date::MAX, date(9999, 12, 31));
        assert_eq!(Date::MAX.days_since(Date::MIN), 3_652_058);
        assert_eq!(Date::MIN.days_since(Date::MAX), -3_652_058);
        assert_eq!(Date::MIN.weekday(), Weekday::Monday);
        assert_eq!(Date::MAX.weekday(), Weekday::Friday);
        // The representation is deliberately private; this is an internal
        // performance regression check rather than a public layout promise.
        assert_eq!(size_of::<Date>(), 4);
    }

    #[test]
    fn rejects_every_unsupported_year_without_overflow() {
        for year in [i32::MIN, -9999, -400, -1, 0, 10_000, i32::MAX] {
            assert_eq!(
                Date::from_ymd(year, 1, 1),
                Err(DateError::YearOutOfRange { year })
            );
            assert_eq!(
                Date::days_in_month(year, 1),
                Err(DateError::YearOutOfRange { year })
            );
            assert_eq!(
                Date::days_in_year(year),
                Err(DateError::YearOutOfRange { year })
            );
            assert_eq!(
                Date::from_ymd(year, 0, 0),
                Err(DateError::YearOutOfRange { year })
            );
        }
    }

    #[test]
    fn invalid_components_retain_all_inputs() {
        for (year, month, day) in [
            (2024, 0, 1),
            (2024, 13, 1),
            (2024, u32::MAX, 1),
            (2024, 1, 0),
            (2024, 1, 32),
            (2024, 1, u32::MAX),
            (2023, 2, 29),
            (2024, 2, 30),
            (2024, 4, 31),
            (1900, 2, 29),
            (2100, 2, 29),
            (1, 0, 0),
            (9999, 13, u32::MAX),
        ] {
            assert_eq!(
                Date::from_ymd(year, month, day),
                Err(DateError::InvalidDate { year, month, day })
            );
        }
        for month in [0, 13, u32::MAX] {
            assert_eq!(
                Date::days_in_month(2024, month),
                Err(DateError::InvalidMonth { month })
            );
        }
    }

    #[test]
    fn every_year_and_month_rejects_the_day_after_month_end() -> Result<(), DateError> {
        for year in 1..=9999 {
            let mut length_of_year = 0;
            for month in 1..=12 {
                let length = Date::days_in_month(year, month)?;
                length_of_year += length;
                for day in [0, length + 1, u32::MAX] {
                    assert_eq!(
                        Date::from_ymd(year, month, day),
                        Err(DateError::InvalidDate { year, month, day })
                    );
                }
                assert_eq!(
                    Date::from_ymd(year, month, length)?
                        .last_day_of_month()
                        .day(),
                    length
                );
            }
            assert_eq!(Date::days_in_year(year)?, length_of_year);
        }
        Ok(())
    }

    #[test]
    fn gregorian_leap_rules_cover_centuries_and_signed_years() -> Result<(), DateError> {
        for year in [i32::MIN, -800, -400, -4, 0, 4, 1600, 2000, 2024, 2400, 9600] {
            assert!(Date::is_leap_year(year), "year {year}");
        }
        for year in [i32::MAX, -100, -1, 1, 1900, 2100, 2200, 9999] {
            assert!(!Date::is_leap_year(year), "year {year}");
        }
        let mut leap_count = 0;
        for year in 1..=400 {
            let expected = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
            assert_eq!(Date::is_leap_year(year), expected);
            assert_eq!(
                Date::days_in_month(year, 2)?,
                if expected { 29 } else { 28 }
            );
            leap_count += u32::from(expected);
        }
        assert_eq!(leap_count, 97);
        Ok(())
    }

    #[test]
    fn conversion_cycle_boundaries_and_historical_dates() {
        for (year, month, day, weekday) in [
            (1, 1, 1, Weekday::Monday),
            (4, 12, 31, Weekday::Friday),
            (100, 12, 31, Weekday::Friday),
            (400, 12, 31, Weekday::Sunday),
            (401, 1, 1, Weekday::Monday),
            (1582, 10, 4, Weekday::Monday),
            (1582, 10, 15, Weekday::Friday),
            (1900, 1, 1, Weekday::Monday),
            (1970, 1, 1, Weekday::Thursday),
            (2000, 1, 1, Weekday::Saturday),
            (2024, 2, 29, Weekday::Thursday),
        ] {
            let d = date(year, month, day);
            assert_eq!(d.to_ymd(), (year, month, day));
            assert_eq!((d.year(), d.month(), d.day()), (year, month, day));
            assert_eq!(d.weekday(), weekday, "{d}");
        }
        // The proleptic calendar does not skip the historical reform days.
        assert_eq!(date(1582, 10, 15).days_since(date(1582, 10, 4)), 11);
    }

    #[test]
    fn ordinal_day_and_calendar_month_queries() -> Result<(), DateError> {
        for year in [1, 4, 100, 400, 1900, 2000, 2023, 2024, 2100, 2400, 9999] {
            let mut ordinal = 1;
            for month in 1..=12 {
                let last = Date::days_in_month(year, month)?;
                let first = date(year, month, 1);
                let end = date(year, month, last);
                for day in 1..=last {
                    let current = date(year, month, day);
                    assert_eq!(current.day_of_year(), ordinal);
                    assert_eq!(current.first_day_of_month(), first);
                    assert_eq!(current.last_day_of_month(), end);
                    assert_eq!(current.is_end_of_month(), day == last);
                    ordinal += 1;
                }
            }
            assert_eq!(ordinal - 1, Date::days_in_year(year)?);
        }
        Ok(())
    }

    #[test]
    fn exact_day_and_week_arithmetic_across_months_and_years() -> Result<(), DateError> {
        for (from, to) in [
            (date(2023, 2, 28), date(2023, 3, 1)),
            (date(2024, 2, 28), date(2024, 2, 29)),
            (date(2024, 2, 29), date(2024, 3, 1)),
            (date(2023, 12, 31), date(2024, 1, 1)),
            (date(1900, 2, 28), date(1900, 3, 1)),
            (date(2000, 2, 28), date(2000, 2, 29)),
            (date(2100, 2, 28), date(2100, 3, 1)),
            (date(2400, 2, 28), date(2400, 2, 29)),
        ] {
            assert_eq!(from.checked_add_days(1)?, to);
            assert_eq!(to.checked_add_days(-1)?, from);
            assert_eq!(to.checked_sub_days(1)?, from);
            assert_eq!(from.checked_sub_days(-1)?, to);
            assert_eq!(to.days_since(from), 1);
        }
        let base = date(2000, 1, 1);
        for count in [
            -100_000, -1461, -366, -365, -7, -1, 0, 1, 7, 365, 366, 1461, 100_000,
        ] {
            let shifted = base.checked_add_days(count)?;
            assert_eq!(i64::from(shifted.days_since(base)), count);
            assert_eq!(shifted.checked_sub_days(count)?, base);
        }
        for weeks in [-1000, -53, -1, 0, 1, 53, 1000] {
            let shifted = base.checked_add_weeks(weeks)?;
            assert_eq!(shifted, base.checked_add_days(weeks * 7)?);
            assert_eq!(shifted.weekday(), base.weekday());
        }
        assert_eq!(Date::MIN.checked_add_days(3_652_058)?, Date::MAX);
        assert_eq!(Date::MAX.checked_sub_days(3_652_058)?, Date::MIN);
        Ok(())
    }

    #[test]
    fn arithmetic_errors_keep_original_operation_and_count() {
        for (base, count) in [
            (Date::MIN, -1),
            (Date::MAX, 1),
            (Date::MIN, i64::MIN),
            (Date::MAX, i64::MAX),
        ] {
            assert_eq!(
                base.checked_add_days(count),
                Err(DateError::OutOfRange {
                    date: base,
                    operation: "add days",
                    count
                })
            );
        }
        for base in [Date::MIN, date(2024, 1, 1), Date::MAX] {
            for count in [i64::MIN, i64::MAX, i64::MIN / 7, i64::MAX / 7] {
                assert_eq!(
                    base.checked_add_weeks(count),
                    Err(DateError::OutOfRange {
                        date: base,
                        operation: "add weeks",
                        count
                    })
                );
            }
            for count in [i64::MIN, i64::MAX, -119_988, 119_988] {
                assert_eq!(
                    base.checked_add_months(count),
                    Err(DateError::OutOfRange {
                        date: base,
                        operation: "add months",
                        count
                    })
                );
            }
            for count in [i64::MIN, i64::MAX, -9999, 9999] {
                assert_eq!(
                    base.checked_add_years(count),
                    Err(DateError::OutOfRange {
                        date: base,
                        operation: "add years",
                        count
                    })
                );
            }
            for count in [i64::MIN, i64::MAX] {
                assert_eq!(
                    base.checked_sub_days(count),
                    Err(DateError::OutOfRange {
                        date: base,
                        operation: "subtract days",
                        count
                    })
                );
            }
        }
        assert!(Date::MIN.checked_sub_days(1).is_err());
        assert!(Date::MAX.checked_sub_days(-1).is_err());
    }

    #[test]
    fn clamping_and_month_addition_anti_invariants() -> Result<(), DateError> {
        for (from, count, to) in [
            (date(2023, 1, 31), 1, date(2023, 2, 28)),
            (date(2024, 1, 31), 1, date(2024, 2, 29)),
            (date(2024, 3, 31), -1, date(2024, 2, 29)),
            (date(2023, 2, 28), 1, date(2023, 3, 28)),
            (date(2024, 2, 29), 1, date(2024, 3, 29)),
            (date(2024, 12, 31), 2, date(2025, 2, 28)),
            (date(2024, 1, 31), -2, date(2023, 11, 30)),
        ] {
            assert_eq!(from.checked_add_months(count)?, to);
        }
        let january = date(2023, 1, 31);
        assert_eq!(
            january.checked_add_months(1)?.checked_add_months(-1)?,
            date(2023, 1, 28)
        );
        assert_eq!(
            january.checked_add_months(1)?.checked_add_months(1)?,
            date(2023, 3, 28)
        );
        assert_eq!(january.checked_add_months(2)?, date(2023, 3, 31));
        assert_eq!(date(2024, 2, 29).checked_add_years(1)?, date(2025, 2, 28));
        assert_eq!(date(2024, 2, 29).checked_add_years(-1)?, date(2023, 2, 28));
        assert_eq!(date(2024, 2, 29).checked_add_years(4)?, date(2028, 2, 29));
        Ok(())
    }

    #[test]
    fn clamping_and_year_equivalence_for_every_supported_month_end() -> Result<(), DateError> {
        for year in 1..=9999 {
            for month in 1..=12 {
                let current = date(year, month, Date::days_in_month(year, month)?);
                for offset in [-1, 1] {
                    let target_year = if offset == -1 && month == 1 {
                        year - 1
                    } else if offset == 1 && month == 12 {
                        year + 1
                    } else {
                        year
                    };
                    let target_month = if offset == -1 {
                        if month == 1 { 12 } else { month - 1 }
                    } else if month == 12 {
                        1
                    } else {
                        month + 1
                    };
                    if (1..=9999).contains(&target_year) {
                        let target_day = current
                            .day()
                            .min(Date::days_in_month(target_year, target_month)?);
                        assert_eq!(
                            current.checked_add_months(offset)?,
                            date(target_year, target_month, target_day)
                        );
                    } else {
                        assert!(current.checked_add_months(offset).is_err());
                    }
                }
                for years in [-1, 0, 1] {
                    assert_eq!(
                        current.checked_add_years(years).map(|d| d.to_ymd()),
                        current
                            .checked_add_months(12 * years)
                            .map(|d| d.to_ymd())
                            .map_err(|_| current.range_error("add years", years))
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn exact_iso_format_and_semantic_parse_errors() {
        for (text, expected) in [
            ("0001-01-01", Date::MIN),
            ("0099-02-03", date(99, 2, 3)),
            ("2024-02-29", date(2024, 2, 29)),
            ("9999-12-31", Date::MAX),
        ] {
            assert_eq!(text.parse::<Date>(), Ok(expected));
            assert_eq!(expected.to_string(), text);
        }
        assert_eq!(
            "0000-01-01".parse::<Date>(),
            Err(DateError::YearOutOfRange { year: 0 })
        );
        for (text, year, month, day) in [
            ("2023-02-29", 2023, 2, 29),
            ("2024-00-01", 2024, 0, 1),
            ("2024-13-01", 2024, 13, 1),
            ("2024-01-00", 2024, 1, 0),
            ("2024-01-99", 2024, 1, 99),
        ] {
            assert_eq!(
                text.parse::<Date>(),
                Err(DateError::InvalidDate { year, month, day })
            );
        }
    }

    #[test]
    fn hostile_text_never_slices_unicode_or_normalizes_input() {
        for text in [
            "",
            "2024",
            "2024-02-2",
            "2024-2-29",
            "024-02-29",
            "02024-02-29",
            "10000-01-01",
            " 2024-02-29",
            "2024-02-29 ",
            "2024-02-29\n",
            "2024-02-29\0",
            "2024/02/29",
            "2024.02.29",
            "2024--2-29",
            "+024-02-29",
            "-024-02-29",
            "2e24-02-29",
            "2024-02-2x",
            "2024-02-２",
            "２０２４-02-29",
            "2024−02−29",
            "💥💥💥",
            "ééééé",
            "\0\0\0\0-00-00",
            "2024-02-29T00:00:00Z",
        ] {
            assert_eq!(
                text.parse::<Date>(),
                Err(DateError::Parse {
                    text: text.to_owned()
                })
            );
        }
        let ascii = b"2024-02-29";
        for position in 0..ascii.len() {
            for replacement in 0_u8..=127 {
                let mut changed = *ascii;
                changed[position] = replacement;
                if let Ok(text) = std::str::from_utf8(&changed) {
                    let result = text.parse::<Date>();
                    let syntactic = if position == 4 || position == 7 {
                        replacement == b'-'
                    } else {
                        replacement.is_ascii_digit()
                    };
                    if !syntactic {
                        assert_eq!(
                            result,
                            Err(DateError::Parse {
                                text: text.to_owned()
                            })
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn nth_weekday_covers_a_complete_gregorian_cycle() -> Result<(), DateError> {
        for year in 1..=400 {
            for month in 1..=12 {
                let length = Date::days_in_month(year, month)?;
                for weekday in Weekday::ALL {
                    let occurrences: Vec<Date> = (1..=length)
                        .map(|day| date(year, month, day))
                        .filter(|d| d.weekday() == weekday)
                        .collect();
                    for nth in 1..=5 {
                        let expected = occurrences
                            .get(usize::try_from(nth - 1).map_err(|_| {
                                DateError::InvalidOccurrence {
                                    year,
                                    month,
                                    nth,
                                    weekday,
                                }
                            })?)
                            .copied();
                        let actual = Date::nth_weekday(year, month, nth, weekday);
                        match expected {
                            Some(expected) => assert_eq!(actual, Ok(expected)),
                            None => assert_eq!(
                                actual,
                                Err(DateError::InvalidOccurrence {
                                    year,
                                    month,
                                    nth,
                                    weekday
                                })
                            ),
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn nth_weekday_invalid_occurrences_and_endpoint_years() {
        for nth in [0, 6, u32::MAX] {
            assert_eq!(
                Date::nth_weekday(2024, 2, nth, Weekday::Monday),
                Err(DateError::InvalidOccurrence {
                    year: 2024,
                    month: 2,
                    nth,
                    weekday: Weekday::Monday
                })
            );
        }
        assert_eq!(
            Date::nth_weekday(2024, 2, 5, Weekday::Thursday),
            Ok(date(2024, 2, 29))
        );
        assert_eq!(
            Date::nth_weekday(2024, 2, 5, Weekday::Monday),
            Err(DateError::InvalidOccurrence {
                year: 2024,
                month: 2,
                nth: 5,
                weekday: Weekday::Monday
            })
        );
        assert_eq!(Date::nth_weekday(1, 1, 1, Weekday::Monday), Ok(Date::MIN));
        assert_eq!(
            Date::nth_weekday(9999, 12, 5, Weekday::Friday),
            Ok(Date::MAX)
        );
        assert!(Date::nth_weekday(0, 1, 1, Weekday::Monday).is_err());
        assert!(Date::nth_weekday(2024, 0, 1, Weekday::Monday).is_err());
    }

    #[test]
    fn adjacent_weekday_queries_are_inclusive_and_minimal() -> Result<(), DateError> {
        let monday = date(2024, 4, 1);
        for day in 0..14 {
            let current = monday.checked_add_days(day)?;
            for weekday in Weekday::ALL {
                let next = current.next_weekday(weekday)?;
                let previous = current.previous_weekday(weekday)?;
                assert_eq!(next.weekday(), weekday);
                assert_eq!(previous.weekday(), weekday);
                assert!((0..=6).contains(&next.days_since(current)));
                assert!((0..=6).contains(&current.days_since(previous)));
                assert_eq!(next == current, current.weekday() == weekday);
                assert_eq!(previous == current, current.weekday() == weekday);
            }
        }
        for weekday in Weekday::ALL {
            assert_eq!(
                Date::MIN.previous_weekday(weekday).is_ok(),
                weekday == Date::MIN.weekday()
            );
            assert_eq!(
                Date::MAX.next_weekday(weekday).is_ok(),
                weekday == Date::MAX.weekday()
            );
            assert!(Date::MIN.next_weekday(weekday).is_ok());
            assert!(Date::MAX.previous_weekday(weekday).is_ok());
        }
        Ok(())
    }

    #[test]
    fn closed_enum_numbers_names_and_identifier_tables() {
        assert_eq!(WEEKDAY_IDENTIFIERS.check(), Ok(()));
        assert_eq!(MONTH_IDENTIFIERS.check(), Ok(()));
        assert_eq!(
            WEEKDAY_IDENTIFIERS.check_round_trip(&Weekday::ALL, |d| d.to_string()),
            Ok(())
        );
        assert_eq!(
            MONTH_IDENTIFIERS.check_round_trip(&Month::ALL, |m| m.to_string()),
            Ok(())
        );
        for weekday in Weekday::ALL {
            assert_eq!(Weekday::try_from(weekday.number_from_monday()), Ok(weekday));
            assert_eq!(weekday.to_string().parse::<Weekday>(), Ok(weekday));
        }
        for month in Month::ALL {
            assert_eq!(Month::try_from(month.number()), Ok(month));
            assert_eq!(month.to_string().parse::<Month>(), Ok(month));
        }
        for weekday in [0, 8, u32::MAX] {
            assert_eq!(
                Weekday::try_from(weekday),
                Err(DateError::InvalidWeekday { weekday })
            );
        }
        for month in [0, 13, u32::MAX] {
            assert_eq!(
                Month::try_from(month),
                Err(DateError::InvalidMonth { month })
            );
        }
        for text in ["", "monday", "Mon", "Monday ", "1", "星期一", "Monday\n"] {
            assert!(text.parse::<Weekday>().is_err());
        }
        for text in ["", "january", "Jan", "January ", "1", "มกราคม", "January\n"] {
            assert!(text.parse::<Month>().is_err());
        }
    }

    #[test]
    fn dates_are_ordered_hashed_and_shareable() {
        let ordered = [
            Date::MIN,
            date(1, 1, 2),
            date(1900, 1, 1),
            date(2000, 1, 1),
            date(2000, 2, 29),
            Date::MAX,
        ];
        for pair in ordered.windows(2) {
            assert!(pair[0] < pair[1]);
            assert!(pair[0].days_since(pair[1]) < 0);
        }
        assert_eq!(
            ordered.into_iter().collect::<HashSet<_>>().len(),
            ordered.len()
        );
        assert_eq!(
            ordered
                .into_iter()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>(),
            ordered
        );
        fn send_sync_static<T: Send + Sync + 'static>() {}
        send_sync_static::<Date>();
        send_sync_static::<DateError>();
        send_sync_static::<Month>();
        send_sync_static::<Weekday>();
    }

    #[test]
    fn every_date_error_has_unique_stable_kind_context_and_no_source() {
        let errors = [
            (
                DateError::YearOutOfRange { year: 0 },
                "date.year_out_of_range",
                "0",
            ),
            (
                DateError::InvalidDate {
                    year: 2023,
                    month: 2,
                    day: 29,
                },
                "date.invalid_date",
                "2023",
            ),
            (
                DateError::OutOfRange {
                    date: Date::MIN,
                    operation: "add days",
                    count: -1,
                },
                "date.out_of_range",
                "-1",
            ),
            (
                DateError::InvalidMonth { month: 13 },
                "date.invalid_month",
                "13",
            ),
            (
                DateError::InvalidWeekday { weekday: 8 },
                "date.invalid_weekday",
                "8",
            ),
            (
                DateError::InvalidOccurrence {
                    year: 2024,
                    month: 2,
                    nth: 5,
                    weekday: Weekday::Monday,
                },
                "date.invalid_occurrence",
                "Monday",
            ),
            (
                DateError::Parse {
                    text: "bad date".to_owned(),
                },
                "date.parse",
                "bad date",
            ),
        ];
        let kinds: HashSet<_> = errors.iter().map(|(error, _, _)| error.kind()).collect();
        assert_eq!(kinds.len(), errors.len());
        for (error, kind, context) in errors {
            assert_eq!(error.kind(), kind);
            assert!(error.to_string().contains(context));
            assert_eq!(error.clone(), error);
            assert!(!format!("{error:?}").is_empty());
            assert!(error.source().is_none());
        }
    }
}
