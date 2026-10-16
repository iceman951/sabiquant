//! US 30/360 with an explicit security end-of-month qualification.

use super::{DayCountError, adjusted_days};
use crate::time::Date;

/// US 30/360 with the security's end-of-month qualification supplied explicitly.
///
/// This local definition follows the four ordered adjustments in the FIX
/// column of [ROC Revised CDE Technical Guidance v4 (December 2025), Table 4,
/// A001, printed p.166](https://www.leiroc.org/publications/gls/Final_ROC202531-Revised-CDE-Technical-Guidance-Version-4.pdf#page=157).
/// It pins that reporting definition; the complete SIA/SIFMA book definition
/// remains uninspected, and other columns or instruments may use different
/// February treatment. See the [adopted local contract](https://github.com/iceman951/sabiquant/blob/main/docs/M3_US_AFB_CONTRACT.md).
///
/// On an increasing interval, the explicit flag enables two adjustments:
/// change the ending day to 30 when both original dates are February month
/// ends, then change a February-month-end starting day to 30. Next, change an
/// ending 31st to 30 when the current starting day is 30 or 31; finally change
/// a starting 31st to 30. Divide `360ΔY + 30ΔM + D′end − D′start` by 360.
/// Disabling the flag gives the existing Bond Basis arithmetic.
///
/// The flag is a security property, never inferred from dates, schedules or
/// calendar month-end advancement. There is no default constructor or
/// context-free text parser. Reversed dates negate the chronological forward
/// result while retaining the same flag; every zero result is positive zero.
/// All operations take constant time across [`Date::MIN`] to [`Date::MAX`].
///
/// QuantLib 1.43's `Thirty360(USA)` has no corresponding explicit flag: its
/// forward formula matches the enabled setting. SabiQuant's chronological
/// normalization can differ from applying a directional formula backwards.
/// Agreement with a reference does not determine a security's qualification.
///
/// ```
/// use sabiquant::daycount::Thirty360Us;
/// use sabiquant::time::Date;
///
/// let start = Date::from_ymd(2023, 2, 28)?;
/// let end = Date::from_ymd(2023, 3, 31)?;
/// let end_of_month = Thirty360Us::new(true);
/// let other = Thirty360Us::new(false);
/// assert!(end_of_month.end_of_month_security());
/// assert!(!other.end_of_month_security());
/// assert_eq!(end_of_month.day_count(start, end), 30);
/// assert_eq!(other.day_count(start, end), 33);
/// assert_eq!(end_of_month.year_fraction(start, end)?.to_bits(), (30.0_f64 / 360.0).to_bits());
/// assert_eq!(other.year_fraction(start, end)?.to_bits(), (33.0_f64 / 360.0).to_bits());
/// # Ok::<(), sabiquant::error::Error>(())
/// ```
#[doc(alias = "30/360 (US)")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Thirty360Us {
    end_of_month_security: bool,
}

impl Thirty360Us {
    /// Constructs the convention with the security's explicit EOM qualification.
    ///
    /// Both values are valid. `true` enables the February-month-end rules;
    /// `false` retains Bond Basis arithmetic. No context is inferred.
    #[must_use]
    pub const fn new(end_of_month_security: bool) -> Self {
        Self {
            end_of_month_security,
        }
    }

    /// Returns the security's supplied end-of-month qualification.
    #[must_use]
    pub const fn end_of_month_security(self) -> bool {
        self.end_of_month_security
    }

    /// Returns the canonical name, `30/360 (US)`.
    ///
    /// This name does not encode the required security qualification.
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        "30/360 (US)"
    }

    /// Returns the signed integer count using the type's ordered adjustments.
    ///
    /// Reversed endpoints negate the forward count using the same explicit
    /// qualification; equal endpoints return zero. Synthetic zero counts can
    /// also occur between distinct dates. Every supported result fits `i32`.
    #[must_use]
    pub fn day_count(self, start: Date, end: Date) -> i32 {
        if start > end {
            -self.forward(end, start)
        } else {
            self.forward(start, end)
        }
    }

    /// Returns the signed fraction as one division of the exact day count by 360.
    ///
    /// This is TOL-E1: zero ULP from the correctly rounded rational quotient.
    /// Every zero count gives positive zero, including reversed zero intervals.
    ///
    /// # Errors
    ///
    /// [`DayCountError::Value`] if finite-result validation rejects the
    /// result. The supported date range bounds this formula to finite values.
    pub fn year_fraction(self, start: Date, end: Date) -> Result<f64, DayCountError> {
        crate::numeric::finite_result(
            "year fraction",
            f64::from(self.day_count(start, end)) / 360.0,
        )
        .map_err(DayCountError::from)
    }

    fn forward(self, start: Date, end: Date) -> i32 {
        let start_february_end = start.month() == 2 && start.is_end_of_month();
        let end_february_end = end.month() == 2 && end.is_end_of_month();
        let mut first_day = start.day();
        let mut last_day = end.day();
        if self.end_of_month_security && start_february_end && end_february_end {
            last_day = 30;
        }
        if self.end_of_month_security && start_february_end {
            first_day = 30;
        }
        if last_day == 31 && first_day >= 30 {
            last_day = 30;
        }
        if first_day == 31 {
            first_day = 30;
        }
        adjusted_days(start, end, first_day, last_day)
    }
}
