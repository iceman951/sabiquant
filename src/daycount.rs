//! Source-verified core day-count conventions and year fractions.
//!
//! A convention maps a half-open calendar interval `[start, end)` to an
//! integer day count and a year fraction. The start date is included and the
//! end date is excluded. Dates carry no time zone or clock state.
//!
//! These definitions were verified in the original
//! [2006 ISDA Definitions, §§4.13 and 4.16](https://www.tdsecurities.com/tds/document/2006-ISDA-Definitions#page=23),
//! hosted by TD Securities. Printed pages 11–13 distinguish Actual/360,
//! Actual/365 (Fixed), Actual/Actual (ISDA), Bond Basis, and the two 30E/360
//! variants. [`ThirtyE360Isda`] requires an explicit termination date.
//! AFB follows ISDA's [1999 paper, page 1 §2(c)](https://www.isda.org/a/pIJEE/The-Actual-Actual-Day-Count-Fraction-1999.pdf),
//! including its February backward-year clarification. [`Thirty360Us`] uses
//! an explicit security EOM qualification and the dated ROC December 2025
//! A001 definition; its source limits are documented on that context type.
//! The source review and outstanding conventions are recorded in
//! [M3_DAYCOUNT_RESEARCH](https://github.com/iceman951/sabiquant/blob/main/docs/M3_DAYCOUNT_RESEARCH.md).
//!
//! # Numerical policy
//!
//! Actual/360, Actual/365 (Fixed), and the 30/360 conventions compute one division of exact
//! integers (TOL-E1, zero ULP from the correctly rounded quotient).
//! Actual/Actual (ISDA) divides the first and last calendar-year portions,
//! then adds the first quotient, the intervening whole-year count, and the
//! last quotient in that order. Its tolerance is TOL-E2: at most four ULP
//! from an independently rounded exact rational result. The operation takes
//! constant time even across the full supported date range. AFB first divides
//! the residual actual days by 365 or 366, then adds the backward whole-year
//! count. It uses the same TOL-E2 four-ULP bound and takes constant time;
//! the original ending date is retained when no whole year is removed.
//!
//! Reversed dates negate the forward fraction. Equal dates, and the distinct
//! dates that 30E/360 maps to a zero day count, return positive zero. Numeric
//! results are checked for finiteness before they are returned.
//!
//! # QuantLib correspondence
//!
//! The corresponding QuantLib 1.43 conventions are `Actual360`,
//! `Actual365Fixed(Standard)`, `ActualActual(ISDA)`, and
//! `ActualActual(AFB)`, `Thirty360(European)`, `Thirty360(BondBasis)`,
//! `Thirty360(ISDA)`, and `Thirty360(USA)` for US flag-on observations.
//! The explicit US flag-off arithmetic corresponds to Bond Basis; that
//! relationship does not merge their context definitions.
//! Bond Basis and ISDA 30E/360 deliberately normalize reversed intervals and
//! return zero for equal endpoints, including a February termination date.
//! No last-day inclusion, Canadian, NoLeap, or security EOM qualification is inferred. Bare convention names
//! that are known to be ambiguous are rejected, including when a candidate
//! convention is outside this currently implemented subset. Compatibility
//! claims remain limited to the scenario sets in the validation evidence.

use crate::calendar::{Calendar, CalendarError, EndpointInclusion, TargetCalendar};
use crate::error::{IdentifierError, ValueError};
use crate::identifier::{Entry, Identifiers};
use crate::time::Date;
use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;

mod afb;
mod icma;
mod us;
pub use icma::{ActualActualIcma, DeterminationPeriod, IcmaError};
pub use us::Thirty360Us;

/// A supported convention for calendar-day counts and accrual year fractions.
///
/// The variants have distinct meanings. Other day-count conventions
/// require further implementation or primary-source research; the enum may
/// grow in a future release.
///
/// # Identifiers
///
/// | Variant | Canonical identifier | Accepted aliases |
/// |---|---|---|
/// | [`Actual360`](Self::Actual360) | `Actual/360` | `Act/360`, `A/360` |
/// | [`Actual365Fixed`](Self::Actual365Fixed) | `Actual/365 (Fixed)` | `Act/365 (Fixed)`, `A/365 (Fixed)`, `A/365F` |
/// | [`ActualActualIsda`](Self::ActualActualIsda) | `Actual/Actual (ISDA)` | `Act/Act (ISDA)`, `Actual/Actual (Historical)` |
/// | [`ActualActualAfb`](Self::ActualActualAfb) | `Actual/Actual (AFB)` | `Actual/Actual (Euro)` |
/// | [`ThirtyE360`](Self::ThirtyE360) | `30E/360` | `30/360 (Eurobond Basis)` |
/// | [`Thirty360BondBasis`](Self::Thirty360BondBasis) | `30/360 (Bond Basis)` | `30/360 ISDA`, `360/360` |
///
/// Matching is exact, case-sensitive, and does not trim whitespace. The labels
/// `Actual/Actual`, `Act/Act`, `ACT/ACT`, `Actual/365`, and `30/360` are
/// ambiguous and are always rejected with their known canonical candidates.
/// Explicit identifiers for unsupported conventions return an unknown
/// identifier error; this subset never substitutes a similar convention.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DayCount {
    /// Actual calendar days divided by 360, including leap days without
    /// changing the denominator (ISDA §4.16(e)).
    Actual360,
    /// Actual calendar days divided by a fixed 365, including leap days
    /// without changing the denominator (ISDA §4.16(d)).
    Actual365Fixed,
    /// Actual days falling in common years divided by 365, plus actual days
    /// falling in leap years divided by 366 (ISDA §4.16(b)).
    ActualActualIsda,
    /// AFB with ISDA's published February backward-year clarification.
    ///
    /// Remove complete years backward from the end, promoting a resulting
    /// February 28 to 29 in a leap target year. Divide the remaining initial
    /// half-open stub by 366 only if it includes February 29, otherwise 365.
    /// This is distinct from distributing days over their ISDA calendar years.
    ActualActualAfb,
    /// European 30E/360, also called Eurobond Basis: cap both day-of-month
    /// values at 30, use twelve 30-day months per year, then divide by 360
    /// (ISDA §4.16(g)). February has no special month-end adjustment.
    ThirtyE360,
    /// Bond Basis: change a starting 31st to 30, then change an ending 31st
    /// to 30 only when the adjusted starting day is 30 (ISDA §4.16(f)).
    /// February has no special adjustment. Reversed intervals negate the
    /// forward result; they do not apply these directional rules backwards.
    Thirty360BondBasis,
}

impl DayCount {
    /// Every currently supported convention, in specification order.
    pub const ALL: [Self; 6] = [
        Self::Actual360,
        Self::Actual365Fixed,
        Self::ActualActualIsda,
        Self::ActualActualAfb,
        Self::ThirtyE360,
        Self::Thirty360BondBasis,
    ];

    /// Returns the canonical identifier used by [`Display`](fmt::Display).
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::Actual360 => "Actual/360",
            Self::Actual365Fixed => "Actual/365 (Fixed)",
            Self::ActualActualIsda => "Actual/Actual (ISDA)",
            Self::ActualActualAfb => "Actual/Actual (AFB)",
            Self::ThirtyE360 => "30E/360",
            Self::Thirty360BondBasis => "30/360 (Bond Basis)",
        }
    }

    /// Returns the signed integer day count from `start` to `end`.
    ///
    /// Actual conventions return the exact calendar-day difference. For
    /// [`ThirtyE360`](Self::ThirtyE360), let `D′ = min(D,30)` for each date:
    ///
    /// `DC = 360 × (Y_end − Y_start) + 30 × (M_end − M_start) + D′_end − D′_start`.
    ///
    /// Reversing the dates negates the result; equal dates return zero.
    /// Every result fits `i32` within [`Date::MIN`] through [`Date::MAX`].
    /// The 30th and 31st of the same month have a zero European day count.
    /// Bond Basis uses the directional adjustments on the chronological
    /// forward interval, then negates for reversed endpoints. This preserves
    /// antisymmetry but differs from applying the formula directly backwards.
    #[must_use]
    pub fn day_count(self, start: Date, end: Date) -> i32 {
        match self {
            Self::Actual360
            | Self::Actual365Fixed
            | Self::ActualActualIsda
            | Self::ActualActualAfb => end.days_since(start),
            Self::ThirtyE360 => european_days(start, end),
            Self::Thirty360BondBasis => {
                if start > end {
                    -bond_basis_forward(end, start)
                } else {
                    bond_basis_forward(start, end)
                }
            }
        }
    }

    /// Returns the signed year fraction for `[start, end)`.
    ///
    /// Actual/360 and Actual/365 (Fixed) divide their integer day counts by
    /// 360 and 365 respectively. European 30E/360 divides its adjusted count
    /// by 360. Actual/Actual (ISDA) splits at calendar-year boundaries: each
    /// day's denominator is the length of its own calendar year. It does not
    /// divide the whole interval by the year length of either endpoint.
    /// AFB instead removes complete years backward, divides its remaining
    /// initial stub by 366 only when that stub includes February 29, then
    /// adds the whole-year count. These are distinct numerical orders.
    ///
    /// Reversed dates negate the forward fraction. Every zero day-count
    /// result is positive zero. No reference period, frequency, or calendar
    /// is inferred. See the module documentation for the fixed summation
    /// order and TOL-E1/TOL-E2 exactness contracts.
    ///
    /// # Errors
    ///
    /// [`DayCountError::Value`] wraps a [`ValueError::ResultNotFinite`] if a
    /// result fails numeric validation. The supported date range bounds all
    /// scalar formulas to finite values; this guard also protects future
    /// extensions of the enum.
    ///
    /// ```
    /// use sabiquant::daycount::DayCount;
    /// use sabiquant::time::Date;
    ///
    /// let start = Date::from_ymd(2024, 1, 1)?;
    /// let end = Date::from_ymd(2025, 1, 1)?;
    /// assert_eq!(DayCount::Actual360.day_count(start, end), 366);
    /// assert_eq!(DayCount::ActualActualIsda.year_fraction(start, end)?, 1.0);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn year_fraction(self, start: Date, end: Date) -> Result<f64, DayCountError> {
        let fraction = match self {
            Self::Actual360 => f64::from(end.days_since(start)) / 360.0,
            Self::Actual365Fixed => f64::from(end.days_since(start)) / 365.0,
            Self::ThirtyE360 => f64::from(european_days(start, end)) / 360.0,
            Self::Thirty360BondBasis => f64::from(self.day_count(start, end)) / 360.0,
            Self::ActualActualIsda => {
                if start > end {
                    -isda_forward(end, start)
                } else {
                    isda_forward(start, end)
                }
            }
            Self::ActualActualAfb => {
                if start > end {
                    -afb::forward(end, start)
                } else {
                    afb::forward(start, end)
                }
            }
        };
        crate::numeric::finite_result("year fraction", fraction).map_err(DayCountError::from)
    }
}

const IDENTIFIERS: Identifiers<DayCount> = Identifiers::new(
    "day-count convention",
    &[
        Entry {
            value: DayCount::Actual360,
            canonical: "Actual/360",
            aliases: &["Act/360", "A/360"],
        },
        Entry {
            value: DayCount::Actual365Fixed,
            canonical: "Actual/365 (Fixed)",
            aliases: &["Act/365 (Fixed)", "A/365 (Fixed)", "A/365F"],
        },
        Entry {
            value: DayCount::ActualActualIsda,
            canonical: "Actual/Actual (ISDA)",
            aliases: &["Act/Act (ISDA)", "Actual/Actual (Historical)"],
        },
        Entry {
            value: DayCount::ActualActualAfb,
            canonical: "Actual/Actual (AFB)",
            aliases: &["Actual/Actual (Euro)"],
        },
        Entry {
            value: DayCount::ThirtyE360,
            canonical: "30E/360",
            aliases: &["30/360 (Eurobond Basis)"],
        },
        Entry {
            value: DayCount::Thirty360BondBasis,
            canonical: "30/360 (Bond Basis)",
            aliases: &["30/360 ISDA", "360/360"],
        },
    ],
    &[],
);

// The supported identifier table's self-check requires all ambiguity
// candidates to be implemented values. These known labels also name planned
// conventions, so a separate exact precheck preserves useful ambiguity
// context without pretending those variants are implemented.
const ACTUAL_ACTUAL_CANDIDATES: &[&str] = &[
    "Actual/Actual (ISDA)",
    "Actual/Actual (ICMA)",
    "Actual/Actual (AFB)",
];
const ACTUAL_365_CANDIDATES: &[&str] = &["Actual/365 (Fixed)", "Actual/Actual (ISDA)"];
const THIRTY_360_CANDIDATES: &[&str] = &[
    "30/360 (Bond Basis)",
    "30/360 (US)",
    "30E/360",
    "30E/360 (ISDA)",
];

impl fmt::Display for DayCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_identifier())
    }
}

impl FromStr for DayCount {
    type Err = IdentifierError;

    /// Parses an exact canonical identifier or a documented alias.
    ///
    /// # Errors
    ///
    /// - [`IdentifierError::Ambiguous`] for a documented ambiguous generic
    ///   label, with all known canonical candidates;
    /// - [`IdentifierError::Unknown`] for every other unsupported spelling.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let candidates = match text {
            "Actual/Actual" | "Act/Act" | "ACT/ACT" => Some(ACTUAL_ACTUAL_CANDIDATES),
            "Actual/365" => Some(ACTUAL_365_CANDIDATES),
            "30/360" => Some(THIRTY_360_CANDIDATES),
            _ => None,
        };
        if let Some(candidates) = candidates {
            Err(IdentifierError::Ambiguous {
                concept: "day-count convention",
                text: text.to_owned(),
                candidates,
            })
        } else {
            IDENTIFIERS.parse(text)
        }
    }
}

fn isda_forward(start: Date, end: Date) -> f64 {
    let start_year = start.year();
    let end_year = end.year();
    let start_length: u32 = if Date::is_leap_year(start_year) {
        366
    } else {
        365
    };
    if start_year == end_year {
        return f64::from(end.days_since(start)) / f64::from(start_length);
    }
    let end_length: u32 = if Date::is_leap_year(end_year) {
        366
    } else {
        365
    };
    let first_days = start_length - start.day_of_year() + 1;
    let last_days = end.day_of_year() - 1;
    let first_fraction = f64::from(first_days) / f64::from(start_length);
    let full_years = f64::from(end_year - start_year - 1);
    let last_fraction = f64::from(last_days) / f64::from(end_length);
    (first_fraction + full_years) + last_fraction
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "valid Date components bound the absolute European day count to 3,599,639, which fits i32"
)]
fn european_days(start: Date, end: Date) -> i32 {
    let (start_year, start_month, start_day) = start.to_ymd();
    let (end_year, end_month, end_day) = end.to_ymd();
    let count = 360 * (i64::from(end_year) - i64::from(start_year))
        + 30 * (i64::from(end_month) - i64::from(start_month))
        + i64::from(end_day.min(30))
        - i64::from(start_day.min(30));
    count as i32
}

fn bond_basis_forward(start: Date, end: Date) -> i32 {
    let first_day = start.day().min(30);
    let last_day = if end.day() == 31 && first_day == 30 {
        30
    } else {
        end.day()
    };
    adjusted_days(start, end, first_day, last_day)
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "valid Date components and adjusted days 1..31 bound the absolute result to 3,599,640, which fits i32"
)]
fn adjusted_days(start: Date, end: Date, first_day: u32, last_day: u32) -> i32 {
    let count = 360 * (i64::from(end.year()) - i64::from(start.year()))
        + 30 * (i64::from(end.month()) - i64::from(start.month()))
        + i64::from(last_day)
        - i64::from(first_day);
    count as i32
}

/// The ISDA 30E/360 convention with an explicit contractual termination date.
///
/// This is distinct from European [`DayCount::ThirtyE360`]. The starting
/// day becomes 30 when it is the 31st or the last day of February. The ending
/// day becomes 30 when it is the 31st or the last day of February, except that
/// a February termination date keeps its actual day. The definition is in
/// [ISDA 2006 §4.16(h), printed p.13](https://www.tdsecurities.com/tds/document/2006-ISDA-Definitions#page=25).
///
/// The termination date is a required value, not inferred from a calculation's
/// endpoint. It remains the same when an interval is reversed. Equal
/// endpoints return zero even when both are the February termination date;
/// this intentional divergence preserves SabiQuant's zero-accrual invariant.
/// Reversed intervals negate the chronological forward result. No default
/// constructor or context-free text parser is provided.
///
/// ```
/// use sabiquant::daycount::ThirtyE360Isda;
/// use sabiquant::time::Date;
/// let start = Date::from_ymd(2023, 1, 31)?;
/// let end = Date::from_ymd(2023, 2, 28)?;
/// assert_eq!(ThirtyE360Isda::new(end).day_count(start, end), 28);
/// let intermediate = ThirtyE360Isda::new(Date::from_ymd(2023, 3, 31)?);
/// assert_eq!(intermediate.day_count(start, end), 30);
/// # Ok::<(), sabiquant::error::Error>(())
/// ```
#[doc(alias = "30E/360 (ISDA)")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ThirtyE360Isda {
    termination: Date,
}

impl ThirtyE360Isda {
    /// Constructs the convention with its explicit contractual termination date.
    ///
    /// Every valid [`Date`] is accepted. Calculations may use intervals before
    /// or after it; the February exception applies only when the chronological
    /// forward interval's end equals this date.
    #[must_use]
    pub const fn new(termination_date: Date) -> Self {
        Self {
            termination: termination_date,
        }
    }

    /// Returns the contractual termination date used by this value.
    #[must_use]
    pub const fn termination_date(self) -> Date {
        self.termination
    }

    /// Returns the convention's canonical name, `30E/360 (ISDA)`.
    ///
    /// The name alone does not supply its required termination-date context.
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        "30E/360 (ISDA)"
    }

    /// Returns the signed 30E/360 ISDA integer day count.
    ///
    /// The forward formula is `360ΔY + 30ΔM + D′_end − D′_start` with the
    /// adjustments in the type documentation. Reversed dates negate that
    /// forward count using the same termination date. Equal dates give zero.
    /// The full supported date range fits `i32`.
    #[must_use]
    pub fn day_count(self, start: Date, end: Date) -> i32 {
        if start == end {
            return 0;
        }
        if start > end {
            return -self.forward(end, start);
        }
        self.forward(start, end)
    }

    /// Returns the signed year fraction as one correctly rounded division
    /// of the adjusted integer day count by 360 (TOL-E1).
    ///
    /// Every zero count returns positive zero.
    ///
    /// # Errors
    ///
    /// [`DayCountError::Value`] if numeric validation rejects the computed
    /// result. Valid supported dates bound this formula to finite values.
    pub fn year_fraction(self, start: Date, end: Date) -> Result<f64, DayCountError> {
        crate::numeric::finite_result(
            "year fraction",
            f64::from(self.day_count(start, end)) / 360.0,
        )
        .map_err(DayCountError::from)
    }

    fn forward(self, start: Date, end: Date) -> i32 {
        let last_february = |date: Date| date.month() == 2 && date.is_end_of_month();
        let first_day = if start.day() == 31 || last_february(start) {
            30
        } else {
            start.day()
        };
        let last_day = if end.day() == 31 || (last_february(end) && end != self.termination) {
            30
        } else {
            end.day()
        };
        adjusted_days(start, end, first_day, last_day)
    }
}

/// Business-day counts divided by 252 under an explicit immutable calendar.
///
/// The Brazilian business-day/252 basis is documented by
/// [B3](https://www.b3.com.br/en_us/products-and-services/trading/interest-rates/di-training-program/02-interest-rates.htm).
/// The calendar is a required owned value: no Brazil or other market calendar
/// is silently selected, and there is no global count cache.
///
/// A forward interval includes its start and excludes its end. Reversed
/// intervals negate that same chronological forward count, preserving
/// antisymmetry. This deliberately differs from passing reversed endpoints
/// directly to [`Calendar::business_days_between`] with fixed inclusion flags,
/// which swaps those flags. With a weekends-only calendar, Saturday
/// 2024-01-06 to Monday 2024-01-08 counts zero here in both directions;
/// direct reversed calendar/QuantLib counting instead gives minus one.
/// Equal dates give zero after any guarded source validates coverage.
///
/// No default constructor or context-free text parser is supplied. Choose
/// user-defined rules through [`new`](Self::new), or an explicit guarded
/// TARGET value through [`with_target`](Self::with_target). Accessors return
/// `Some` only for the selected source; a TARGET value is never exposed as a
/// plain calendar.
///
/// ```
/// use sabiquant::calendar::Calendar;
/// use sabiquant::daycount::Business252;
/// use sabiquant::rates::{Compounding, InterestRate, decimal_from_percent};
/// use sabiquant::time::Date;
/// let accrual = Business252::new(Calendar::weekends_only());
/// let start = Date::from_ymd(2026, 1, 2)?;
/// let end = Date::from_ymd(2026, 1, 9)?;
/// assert_eq!(accrual.day_count(start, end)?, 5);
/// let quote = InterestRate::new(decimal_from_percent(5.0)?, accrual, Compounding::SIMPLE)?;
/// let present = quote.present_value_between(-1_000.0, start, end)?;
/// assert!(present > -1_000.0 && present < 0.0);
/// # Ok::<(), sabiquant::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Business252 {
    source: CalendarSource,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum CalendarSource {
    Rules(Calendar),
    Target(TargetCalendar),
}

impl Business252 {
    /// Constructs a Business/252 convention with its explicit calendar.
    ///
    /// Every calendar definition is accepted, including a fully closed
    /// calendar whose business-day counts are zero.
    #[must_use]
    pub fn new(calendar: Calendar) -> Self {
        Self {
            source: CalendarSource::Rules(calendar),
        }
    }

    /// Constructs a convention with an explicit guarded TARGET value,
    /// preserving its validity and source-coverage policy.
    #[must_use]
    pub fn with_target(calendar: TargetCalendar) -> Self {
        Self {
            source: CalendarSource::Target(calendar),
        }
    }

    /// Returns the user-defined rules when constructed with [`new`](Self::new),
    /// or `None` for a guarded TARGET source.
    #[must_use]
    pub const fn calendar(&self) -> Option<&Calendar> {
        match &self.source {
            CalendarSource::Rules(calendar) => Some(calendar),
            CalendarSource::Target(_) => None,
        }
    }

    /// Returns the guarded TARGET value when constructed with
    /// [`with_target`](Self::with_target), or `None` for user-defined rules.
    #[must_use]
    pub const fn target_calendar(&self) -> Option<&TargetCalendar> {
        match &self.source {
            CalendarSource::Rules(_) => None,
            CalendarSource::Target(calendar) => Some(calendar),
        }
    }

    /// Counts business days in the chronological forward half-open interval,
    /// then negates for reversed dates.
    ///
    /// Equal dates return zero after any guarded source validates their
    /// coverage. The result fits `i32` across the supported range. Counting
    /// performs no business-day search and does not apply its search limit.
    ///
    /// # Errors
    ///
    /// [`DayCountError::Calendar`] preserves guarded TARGET validity or
    /// coverage failures, including equal dates and excluded endpoints.
    pub fn day_count(&self, start: Date, end: Date) -> Result<i32, DayCountError> {
        let reversed = start > end;
        let (start, end) = if reversed { (end, start) } else { (start, end) };
        let count = match &self.source {
            CalendarSource::Rules(calendar) => calendar.business_days_between(
                start,
                end,
                EndpointInclusion::Include,
                EndpointInclusion::Exclude,
            ),
            CalendarSource::Target(calendar) => calendar.business_days_between(
                start,
                end,
                EndpointInclusion::Include,
                EndpointInclusion::Exclude,
            )?,
        };
        if reversed { Ok(-count) } else { Ok(count) }
    }

    /// Returns the signed fraction as one correctly rounded division of the
    /// business-day count by 252 (TOL-E1).
    ///
    /// Every zero count returns positive zero, including a reversed interval
    /// containing no business days.
    ///
    /// # Errors
    ///
    /// - [`DayCountError::Calendar`] for guarded validity or coverage failures,
    ///   as in [`day_count`](Self::day_count);
    /// - [`DayCountError::Value`] if numeric validation rejects the result.
    ///   Valid supported dates bound this formula to finite values.
    pub fn year_fraction(&self, start: Date, end: Date) -> Result<f64, DayCountError> {
        crate::numeric::finite_result(
            "year fraction",
            f64::from(self.day_count(start, end)?) / 252.0,
        )
        .map_err(DayCountError::from)
    }
}

/// A numeric or calendar failure while computing a count or year fraction.
///
/// The domain kind names `daycount.value` and `daycount.calendar` are stable.
/// The underlying [`ValueError`] or [`CalendarError`] retains its own kind and context and is
/// available through [`source`](StdError::source). Message wording may change.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum DayCountError {
    /// Numeric validation rejected a year-fraction result.
    ///
    /// Kind name: `daycount.value`. The source error identifies the precise
    /// numeric failure, such as `value.result_not_finite`.
    Value(ValueError),
    /// Guarded calendar validation rejected a business-day calculation.
    ///
    /// Kind name: `daycount.calendar`. The source identifies the precise
    /// calendar failure and preserves any nested date error.
    Calendar(CalendarError),
}

impl DayCountError {
    /// Returns the stable domain kind name: `daycount.value` or
    /// `daycount.calendar`.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Value(_) => "daycount.value",
            Self::Calendar(_) => "daycount.calendar",
        }
    }
}

impl fmt::Display for DayCountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(error) => fmt::Display::fmt(error, f),
            Self::Calendar(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl StdError for DayCountError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Value(error) => Some(error),
            Self::Calendar(error) => Some(error),
        }
    }
}

impl From<ValueError> for DayCountError {
    fn from(error: ValueError) -> Self {
        Self::Value(error)
    }
}

impl From<CalendarError> for DayCountError {
    fn from(error: CalendarError) -> Self {
        Self::Calendar(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::NonFinite;
    use std::collections::HashSet;

    #[test]
    fn supported_identifier_table_is_exhaustive_and_well_formed() {
        assert_eq!(IDENTIFIERS.check(), Ok(()));
        assert_eq!(
            IDENTIFIERS.check_round_trip(&DayCount::ALL, |value| value.to_string()),
            Ok(())
        );
    }

    #[test]
    fn ambiguities_list_distinct_named_candidates_without_accepting_unimplemented_values() {
        for candidates in [
            ACTUAL_ACTUAL_CANDIDATES,
            ACTUAL_365_CANDIDATES,
            THIRTY_360_CANDIDATES,
        ] {
            assert!(candidates.len() >= 2);
            let unique: HashSet<_> = candidates.iter().copied().collect();
            assert_eq!(unique.len(), candidates.len());
            for &candidate in candidates {
                assert!(!candidate.is_empty());
                assert_eq!(candidate.trim(), candidate);
                assert!(!candidate.chars().any(char::is_control));
            }
        }
        for (text, candidates) in [
            ("Actual/Actual", ACTUAL_ACTUAL_CANDIDATES),
            ("Act/Act", ACTUAL_ACTUAL_CANDIDATES),
            ("ACT/ACT", ACTUAL_ACTUAL_CANDIDATES),
            ("Actual/365", ACTUAL_365_CANDIDATES),
            ("30/360", THIRTY_360_CANDIDATES),
        ] {
            assert_eq!(
                text.parse::<DayCount>(),
                Err(IdentifierError::Ambiguous {
                    concept: "day-count convention",
                    text: text.to_owned(),
                    candidates,
                })
            );
        }
    }

    #[test]
    fn numeric_error_conversion_preserves_context_and_source() {
        let cause = crate::numeric::finite_result("year fraction", f64::INFINITY)
            .expect_err("non-finite result");
        assert_eq!(
            cause,
            ValueError::ResultNotFinite {
                quantity: "year fraction",
                value: NonFinite::PositiveInfinity
            }
        );
        let error = DayCountError::from(cause.clone());
        assert_eq!(error.kind(), "daycount.value");
        assert_eq!(error.clone(), error);
        assert_eq!(error.to_string(), cause.to_string());
        assert_eq!(
            error
                .source()
                .expect("wrapped cause")
                .downcast_ref::<ValueError>(),
            Some(&cause)
        );
        assert!(error.source().expect("wrapped cause").source().is_none());
    }
}
