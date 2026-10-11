//! Source-verified core day-count conventions and year fractions.
//!
//! A convention maps a half-open calendar interval `[start, end)` to an
//! integer day count and a year fraction. The start date is included and the
//! end date is excluded. Dates carry no time zone or clock state.
//!
//! These four definitions were verified in the original
//! [2006 ISDA Definitions, §§4.13 and 4.16](https://www.tdsecurities.com/tds/document/2006-ISDA-Definitions#page=23),
//! hosted by TD Securities. Printed pages 11–13 distinguish Actual/360,
//! Actual/365 (Fixed), Actual/Actual (ISDA), and 30E/360 (Eurobond Basis).
//! The source review and outstanding conventions are recorded in
//! [M3_DAYCOUNT_RESEARCH](https://github.com/iceman951/sabiquant/blob/main/docs/M3_DAYCOUNT_RESEARCH.md).
//!
//! # Numerical policy
//!
//! Actual/360, Actual/365 (Fixed), and 30E/360 compute one division of exact
//! integers (TOL-E1, zero ULP from the correctly rounded quotient).
//! Actual/Actual (ISDA) divides the first and last calendar-year portions,
//! then adds the first quotient, the intervening whole-year count, and the
//! last quotient in that order. Its tolerance is TOL-E2: at most four ULP
//! from an independently rounded exact rational result. The operation takes
//! constant time even across the full supported date range.
//!
//! Reversed dates negate the forward fraction. Equal dates, and the distinct
//! dates that 30E/360 maps to a zero day count, return positive zero. Numeric
//! results are checked for finiteness before they are returned.
//!
//! # QuantLib correspondence
//!
//! The corresponding QuantLib 1.43 conventions are `Actual360`,
//! `Actual365Fixed(Standard)`, `ActualActual(ISDA)`, and
//! `Thirty360(European)`. No last-day inclusion, Canadian, NoLeap, US February,
//! or ISDA 30E/360 termination-date rules are inferred. Bare convention names
//! that are known to be ambiguous are rejected, including when a candidate
//! convention is outside this currently implemented subset. Compatibility
//! claims remain limited to the scenario sets in the validation evidence.

use crate::error::{IdentifierError, ValueError};
use crate::identifier::{Entry, Identifiers};
use crate::time::Date;
use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;

/// A supported convention for calendar-day counts and accrual year fractions.
///
/// The four variants have distinct meanings. Other day-count conventions
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
/// | [`ThirtyE360`](Self::ThirtyE360) | `30E/360` | `30/360 (Eurobond Basis)` |
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
    /// European 30E/360, also called Eurobond Basis: cap both day-of-month
    /// values at 30, use twelve 30-day months per year, then divide by 360
    /// (ISDA §4.16(g)). February has no special month-end adjustment.
    ThirtyE360,
}

impl DayCount {
    /// Every currently supported convention, in specification order.
    pub const ALL: [Self; 4] = [
        Self::Actual360,
        Self::Actual365Fixed,
        Self::ActualActualIsda,
        Self::ThirtyE360,
    ];

    /// Returns the canonical identifier used by [`Display`](fmt::Display).
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::Actual360 => "Actual/360",
            Self::Actual365Fixed => "Actual/365 (Fixed)",
            Self::ActualActualIsda => "Actual/Actual (ISDA)",
            Self::ThirtyE360 => "30E/360",
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
    #[must_use]
    pub fn day_count(self, start: Date, end: Date) -> i32 {
        match self {
            Self::Actual360 | Self::Actual365Fixed | Self::ActualActualIsda => {
                end.days_since(start)
            }
            Self::ThirtyE360 => european_days(start, end),
        }
    }

    /// Returns the signed year fraction for `[start, end)`.
    ///
    /// Actual/360 and Actual/365 (Fixed) divide their integer day counts by
    /// 360 and 365 respectively. European 30E/360 divides its adjusted count
    /// by 360. Actual/Actual (ISDA) splits at calendar-year boundaries: each
    /// day's denominator is the length of its own calendar year. It does not
    /// divide the whole interval by the year length of either endpoint.
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
    /// four formulas to finite values; this guard also protects future
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
            Self::ActualActualIsda => {
                if start > end {
                    -isda_forward(end, start)
                } else {
                    isda_forward(start, end)
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
            value: DayCount::ThirtyE360,
            canonical: "30E/360",
            aliases: &["30/360 (Eurobond Basis)"],
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

/// A numeric validation failure while computing a year fraction.
///
/// The domain kind name `daycount.value` is stable. The underlying
/// [`ValueError`] retains its numeric kind and structured context and is
/// available through [`source`](StdError::source). Message wording may change.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum DayCountError {
    /// Numeric validation rejected a year-fraction result.
    ///
    /// Kind name: `daycount.value`. The source error identifies the precise
    /// numeric failure, such as `value.result_not_finite`.
    Value(ValueError),
}

impl DayCountError {
    /// Returns the stable domain kind name, currently `daycount.value`.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Value(_) => "daycount.value",
        }
    }
}

impl fmt::Display for DayCountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl StdError for DayCountError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Value(error) => Some(error),
        }
    }
}

impl From<ValueError> for DayCountError {
    fn from(error: ValueError) -> Self {
        Self::Value(error)
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
