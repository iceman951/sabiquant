//! Guarded TARGET euro settlement calendar with explicit source coverage.

use std::fmt;

use super::{
    BusinessDayConvention, Calendar, CalendarError, EndOfMonth, EndpointInclusion,
    HolidayWeekendInclusion, JointRule, western_easter,
};
use crate::time::{Date, DateError, Period, Weekday};

/// Whether TARGET queries require verified source coverage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "the explicit coverage policy has exactly these two choices"
)]
pub enum TargetPolicy {
    /// Reject queries outside the verified date intervals.
    VerifiedOnly,
    /// Explicitly permit deterministic recurring-rule projection after inception.
    ///
    /// Projected dates remain labeled projected; this policy never admits
    /// dates before TARGET's production launch.
    AllowProjection,
}

impl fmt::Display for TargetPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::VerifiedOnly => "VerifiedOnly",
            Self::AllowProjection => "AllowProjection",
        })
    }
}

/// Source coverage of a TARGET date, independent of the selected query policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "dates are verified, projected, or outside market validity"
)]
pub enum TargetCoverage {
    /// The date belongs to an explicitly source-checked interval.
    Verified,
    /// The date uses recurring rules without individually verified coverage.
    Projected,
    /// The date precedes TARGET's production launch on 1999-01-04.
    OutsideValidity,
}

impl fmt::Display for TargetCoverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Verified => "Verified",
            Self::Projected => "Projected",
            Self::OutsideValidity => "OutsideValidity",
        })
    }
}

/// An immutable, coverage-checked TARGET euro settlement calendar.
///
/// [`new`](Self::new) requires verified source coverage. Projection requires an
/// explicit [`TargetPolicy::AllowProjection`] selection. Date queries check
/// market validity and coverage, including zero advancement, Unadjusted dates
/// and omitted counting endpoints. Holiday overrides are declarations: their
/// dates are validated when queried. They cannot extend source coverage.
/// Coverage checks survive joint-calendar composition.
///
/// The calendar models euro settlement/value dates, not ECB office holidays,
/// national domestic-system availability, Danish-krone settlement, or every
/// modern TARGET service. No method exposes the underlying plain calendar.
///
/// ```
/// use sabiquant::calendar::{EndpointInclusion, TargetCalendar, TargetCoverage};
/// use sabiquant::time::Date;
/// let target = TargetCalendar::new(); // explicit market, strict source policy
/// let start = Date::from_ymd(2026, 4, 2)?;
/// let end = Date::from_ymd(2026, 4, 7)?;
/// assert_eq!(target.coverage(start), TargetCoverage::Verified);
/// // Good Friday, Easter weekend and Easter Monday are closed.
/// assert_eq!(target.business_days_between(start, end, EndpointInclusion::Include, EndpointInclusion::Exclude)?, 1);
/// let unknown = Date::from_ymd(2025, 12, 31)?;
/// assert_eq!(target.is_business_day(unknown).unwrap_err().kind(), "calendar.unverified_coverage");
/// # Ok::<(), sabiquant::error::Error>(())
/// ```
///
/// A calendar has no implicit default market constructor:
///
/// ```compile_fail
/// use sabiquant::calendar::TargetCalendar;
/// let target: TargetCalendar = Default::default();
/// ```
#[doc(alias = "TARGET")]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TargetCalendar {
    calendar: Calendar,
}

impl TargetCalendar {
    /// TARGET production launch; earlier dates are outside market validity.
    pub const VALID_FROM: &'static str = "1999-01-04";

    /// Source-checked intervals, including their endpoints, in ISO date text.
    ///
    /// The gap between the intervals is deliberately not covered.
    pub const VERIFIED_INTERVALS: [(&'static str, &'static str); 2] =
        [("1999-01-04", "2002-12-31"), ("2026-01-01", "2028-12-31")];

    /// Date on which these primary sources and coverage were last reviewed.
    pub const LAST_REVIEWED: &'static str = "2026-10-11";

    /// The maintenance responsibility required before a calendar release.
    ///
    /// This role description is not a claim that a person has been appointed.
    pub const MAINTAINER_ROLE: &'static str =
        "SabiQuant calendar maintainer; appointment required before release";

    /// Primary-source titles and URLs supporting the encoded rules.
    pub const SOURCES: [(&'static str, &'static str); 6] = [
        (
            "ECB TARGET launch, 1999-01-04",
            "https://www.ecb.europa.eu/press/pr/date/1999/html/pr990104_1.en.html",
        ),
        (
            "ECB exceptional 1999-12-31 closure, 1999-03-31",
            "https://www.ecb.europa.eu/press/pr/date/1999/html/pr990331.en.html",
        ),
        (
            "ECB 1999/2000 calendar, 1999-07-15",
            "https://www.ecb.europa.eu/press/pr/date/1999/html/pr990715_1.en.html",
        ),
        (
            "ECB 2001 calendar, 2000-05-25",
            "https://www.ecb.europa.eu/press/pr/date/2000/html/pr000525_2.en.html",
        ),
        (
            "ECB long-term calendar effective 2002, 2000-12-14",
            "https://www.ecb.europa.eu/press/pr/date/2000/html/pr001214_4.en.html",
        ),
        (
            "ECB TARGET-marked holidays for 2026-2028",
            "https://www.ecb.europa.eu/ecb/contacts/working-hours/html/index.en.html",
        ),
    ];

    /// Creates a TARGET calendar requiring verified coverage.
    #[must_use]
    #[expect(
        clippy::new_without_default,
        reason = "the adopted API policy requires an explicit market calendar; there is no financially neutral calendar default"
    )]
    pub fn new() -> Self {
        Self::with_policy(TargetPolicy::VerifiedOnly)
    }

    /// Creates a TARGET calendar using an explicit source-coverage policy.
    #[must_use]
    pub fn with_policy(policy: TargetPolicy) -> Self {
        Self {
            calendar: Calendar::target_rules(policy == TargetPolicy::AllowProjection),
        }
    }

    /// Returns source coverage even when this calendar would reject a query.
    #[must_use]
    pub fn coverage(&self, date: Date) -> TargetCoverage {
        coverage_of(date)
    }

    /// Returns whether a date is a TARGET business day.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::OutsideValidity`] before 1999-01-04 or
    /// [`CalendarError::UnverifiedCoverage`] in a strict-policy coverage gap.
    pub fn is_business_day(&self, date: Date) -> Result<bool, CalendarError> {
        self.calendar.checked_business(date)
    }

    /// Returns whether a date is a TARGET holiday, including weekends.
    ///
    /// # Errors
    ///
    /// Returns the validity or coverage error of [`is_business_day`](Self::is_business_day).
    pub fn is_holiday(&self, date: Date) -> Result<bool, CalendarError> {
        Ok(!self.is_business_day(date)?)
    }

    /// Returns whether a valid, covered date belongs to the weekend rules.
    ///
    /// Explicit business overrides do not alter weekday weekend classification.
    ///
    /// # Errors
    ///
    /// Returns a validity or strict coverage error for the given date.
    pub fn is_weekend(&self, date: Date) -> Result<bool, CalendarError> {
        self.calendar.validate_date(date)?;
        Ok(self.calendar.is_weekend(date))
    }

    /// Returns a new guarded value where `date` is explicitly a holiday.
    ///
    /// Overrides never extend coverage or inception validity.
    #[must_use]
    pub fn with_holiday(&self, date: Date) -> Self {
        Self {
            calendar: self.calendar.with_holiday(date),
        }
    }

    /// Returns a new guarded value where `date` is explicitly business.
    ///
    /// This can override a weekend, but cannot bypass validity or coverage.
    #[must_use]
    pub fn without_holiday(&self, date: Date) -> Self {
        Self {
            calendar: self.calendar.without_holiday(date),
        }
    }

    /// Combines this guarded calendar with plain user-defined calendars.
    ///
    /// TARGET checks apply even when another member would determine the
    /// Boolean result independently, for example a null JoinBusinessDays member.
    ///
    /// # Errors
    ///
    /// Returns the nesting error of [`Calendar::joint`]. An empty additional
    /// member list is valid because this TARGET calendar remains a member.
    pub fn joint_with(
        &self,
        mut calendars: Vec<Calendar>,
        rule: JointRule,
    ) -> Result<Self, CalendarError> {
        calendars.push(self.calendar.clone());
        Ok(Self {
            calendar: Calendar::joint(calendars, rule)?,
        })
    }

    /// Adjusts a date after checking TARGET validity and coverage.
    ///
    /// # Errors
    ///
    /// Returns the coverage, range or bounded-search errors of adjustment.
    pub fn adjust(
        &self,
        date: Date,
        convention: BusinessDayConvention,
    ) -> Result<Date, CalendarError> {
        self.calendar.validate_date(date)?;
        self.calendar.adjust(date, convention)
    }

    /// Adds signed business days without admitting uncovered input or candidates.
    ///
    /// # Errors
    ///
    /// Returns coverage, date-range or consecutive-holiday search errors.
    pub fn add_business_days(&self, date: Date, count: i32) -> Result<Date, CalendarError> {
        self.calendar.validate_date(date)?;
        self.calendar.add_business_days(date, count)
    }

    /// Advances by a tenor, convention and explicit end-of-month choice.
    ///
    /// # Errors
    ///
    /// Returns coverage, date-range or business-day operation errors. Zero
    /// and Unadjusted operations still validate the input date.
    pub fn advance(
        &self,
        date: Date,
        period: Period,
        convention: BusinessDayConvention,
        end_of_month: EndOfMonth,
    ) -> Result<Date, CalendarError> {
        self.calendar.validate_date(date)?;
        self.calendar
            .advance(date, period, convention, end_of_month)
    }

    /// Returns the first business day of the input date's month.
    ///
    /// # Errors
    ///
    /// Returns coverage or no-business-day-in-month errors.
    pub fn start_of_month(&self, date: Date) -> Result<Date, CalendarError> {
        self.calendar.validate_date(date)?;
        self.calendar.start_of_month(date)
    }

    /// Returns the last business day of the input date's month.
    ///
    /// # Errors
    ///
    /// Returns coverage or no-business-day-in-month errors.
    pub fn end_of_month(&self, date: Date) -> Result<Date, CalendarError> {
        self.calendar.validate_date(date)?;
        self.calendar.end_of_month(date)
    }

    /// Tests whether the date is on or before its month's first business day.
    ///
    /// # Errors
    ///
    /// Returns coverage or no-business-day-in-month errors.
    pub fn is_start_of_month(&self, date: Date) -> Result<bool, CalendarError> {
        self.calendar.validate_date(date)?;
        self.calendar.is_start_of_month(date)
    }

    /// Tests whether the date is on or after its month's last business day.
    ///
    /// # Errors
    ///
    /// Returns coverage or no-business-day-in-month errors.
    pub fn is_end_of_month(&self, date: Date) -> Result<bool, CalendarError> {
        self.calendar.validate_date(date)?;
        self.calendar.is_end_of_month(date)
    }

    /// Counts business dates with explicit endpoint inclusion and source checks.
    ///
    /// Both endpoints are checked even when omitted from counting. Every
    /// counted candidate is checked; verified endpoints cannot bridge an
    /// unverified gap. Reversed intervals swap flags and negate the count.
    /// Equal dates count only when both flags include a business date.
    ///
    /// # Errors
    ///
    /// Returns a validity or strict coverage error for an endpoint or candidate.
    pub fn business_days_between(
        &self,
        start: Date,
        end: Date,
        include_start: EndpointInclusion,
        include_end: EndpointInclusion,
    ) -> Result<i32, CalendarError> {
        self.calendar.validate_date(start)?;
        self.calendar.validate_date(end)?;
        if start == end {
            return Ok(i32::from(
                include_start == EndpointInclusion::Include
                    && include_end == EndpointInclusion::Include
                    && self.calendar.checked_business(start)?,
            ));
        }
        if start > end {
            return Ok(-self.business_days_between(end, start, include_end, include_start)?);
        }
        let include_start = include_start == EndpointInclusion::Include;
        let include_end = include_end == EndpointInclusion::Include;
        if let super::Kind::Target { allow_projection } = self.calendar.definition.kind {
            // With verified endpoints, crossing from the first source interval
            // into the second necessarily crosses this earliest uncovered date.
            // Explicit holiday overrides cannot turn a source gap into coverage.
            if !allow_projection && start.year() <= 2002 && end.year() >= 2026 {
                return Err(CalendarError::UnverifiedCoverage {
                    date: Date::from_ymd(2003, 1, 1)?,
                });
            }
            let first = start.checked_add_days(i64::from(!include_start))?;
            let last = end.checked_sub_days(i64::from(!include_end))?;
            if first > last {
                return Ok(0);
            }
            let mut count = target_count(first, last)?;
            for (&date, &business) in self.calendar.definition.overrides.range(first..=last) {
                count += i32::from(business) - i32::from(target_business(date));
            }
            return Ok(count);
        }
        let first = i32::from(!include_start);
        let last = end.days_since(start) - i32::from(!include_end);
        let mut count = 0;
        for offset in first..=last {
            count += i32::from(
                self.calendar
                    .checked_business(start.checked_add_days(i64::from(offset))?)?,
            );
        }
        Ok(count)
    }

    /// Lists business dates over an inclusive, coverage-checked interval.
    ///
    /// # Errors
    ///
    /// Returns reversed-range, validity or strict coverage errors.
    pub fn business_day_list(&self, start: Date, end: Date) -> Result<Vec<Date>, CalendarError> {
        self.date_list(start, end, false, true)
    }

    /// Lists holidays over an inclusive, coverage-checked interval.
    ///
    /// [`HolidayWeekendInclusion::Exclude`] omits weekend-classified dates.
    ///
    /// # Errors
    ///
    /// Returns reversed-range, validity or strict coverage errors.
    pub fn holiday_list(
        &self,
        start: Date,
        end: Date,
        include_weekends: HolidayWeekendInclusion,
    ) -> Result<Vec<Date>, CalendarError> {
        self.date_list(
            start,
            end,
            true,
            include_weekends == HolidayWeekendInclusion::Include,
        )
    }

    fn date_list(
        &self,
        start: Date,
        end: Date,
        holidays: bool,
        include_weekends: bool,
    ) -> Result<Vec<Date>, CalendarError> {
        self.calendar.validate_date(start)?;
        self.calendar.validate_date(end)?;
        if start > end {
            return Err(CalendarError::InvalidRange { start, end });
        }
        let mut result = Vec::new();
        for offset in 0..=end.days_since(start) {
            let date = start.checked_add_days(i64::from(offset))?;
            let business = self.calendar.checked_business(date)?;
            if if holidays {
                !business && (include_weekends || !self.calendar.is_weekend(date))
            } else {
                business
            } {
                result.push(date);
            }
        }
        Ok(result)
    }
}

// Count weekday blocks arithmetically, then remove unique source closures
// that land on weekdays. Complexity is O(years + overrides), not O(days).
// The caller validates endpoints and the entire applicable coverage interval.
fn target_count(first: Date, last: Date) -> Result<i32, DateError> {
    let days = last.days_since(first) + 1;
    let mut count = days / 7 * 5;
    let first_weekday = i64::from(first.weekday().number_from_monday()) - 1;
    for offset in 0..days % 7 {
        count += i32::from((first_weekday + i64::from(offset)) % 7 < 5);
    }
    for year in first.year()..=last.year() {
        let mut holidays = vec![Date::from_ymd(year, 1, 1)?, Date::from_ymd(year, 12, 25)?];
        if year >= 2000 {
            let easter = western_easter(year)?;
            holidays.extend([
                easter.checked_sub_days(2)?,
                easter.checked_add_days(1)?,
                Date::from_ymd(year, 5, 1)?,
                Date::from_ymd(year, 12, 26)?,
            ]);
        }
        if matches!(year, 1999 | 2001) {
            holidays.push(Date::from_ymd(year, 12, 31)?);
        }
        holidays.sort_unstable();
        holidays.dedup();
        for holiday in holidays {
            if holiday >= first
                && holiday <= last
                && !matches!(holiday.weekday(), Weekday::Saturday | Weekday::Sunday)
            {
                count -= 1;
            }
        }
    }
    Ok(count)
}

fn coverage_of(date: Date) -> TargetCoverage {
    let civil = date.to_ymd();
    if civil < (1999, 1, 4) {
        TargetCoverage::OutsideValidity
    } else if civil <= (2002, 12, 31) || ((2026, 1, 1)..=(2028, 12, 31)).contains(&civil) {
        TargetCoverage::Verified
    } else {
        TargetCoverage::Projected
    }
}

pub(super) fn validate_date(date: Date, allow_projection: bool) -> Result<(), CalendarError> {
    match coverage_of(date) {
        TargetCoverage::OutsideValidity => Err(CalendarError::OutsideValidity { date }),
        TargetCoverage::Projected if !allow_projection => {
            Err(CalendarError::UnverifiedCoverage { date })
        }
        TargetCoverage::Verified | TargetCoverage::Projected => Ok(()),
    }
}

pub(super) fn target_business(date: Date) -> bool {
    if coverage_of(date) == TargetCoverage::OutsideValidity
        || matches!(date.weekday(), Weekday::Saturday | Weekday::Sunday)
    {
        return false;
    }
    let (year, month, day) = date.to_ymd();
    if (month == 1 && day == 1)
        || (month == 12 && day == 25)
        || (month == 12 && day == 31 && matches!(year, 1999 | 2001))
    {
        return false;
    }
    if year >= 2000 {
        if (month == 5 && day == 1) || (month == 12 && day == 26) {
            return false;
        }
        // Date guarantees a supported year; exhaustive Easter tests cover
        // every such year. An impossible internal computus error fails closed
        // without panicking. Public queries check coverage before this helper.
        let easter = match western_easter(year) {
            Ok(easter) => easter,
            Err(_) => return false,
        };
        if easter.checked_sub_days(2) == Ok(date) || easter.checked_add_days(1) == Ok(date) {
            return false;
        }
    }
    true
}
