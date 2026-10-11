//! Immutable calendars and checked business-day operations.
//!
//! This module supplies user-defined calendar rules: null, weekends-only,
//! custom and joint calendars. [`TargetCalendar`] adds checked market rules
//! and explicit source coverage. Calendar
//! classifications depend only on the value and date, with no clock or I/O.
//! Holiday overrides create new values; removing a holiday explicitly makes
//! that date a business day, including a weekend. The weekend classification
//! itself still reflects the calendar's weekday rules.
//!
//! A custom calendar may close all seven weekdays. Queries and composition
//! remain valid; operations needing a business day return a typed failure
//! unless a reachable explicit business-day override exists. Searches stop
//! after 366 consecutive rejected candidates. Nested joint definitions are
//! limited to 64 levels to keep recursive operations and destruction bounded.
//!
//! # QuantLib correspondence
//!
//! Conventions, advancement and endpoint counting follow the FS-CAL/FS-BDC
//! specification. Deliberate differences include immutable holiday changes,
//! explicit bounded-search errors, strict modified-convention month bounds,
//! and the joint nesting limit. TARGET compatibility is limited to its
//! documented source coverage and separately tested projected intervals.

mod easter;
mod target;

pub use easter::western_easter;
pub use target::{TargetCalendar, TargetCoverage, TargetPolicy};

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;

use crate::error::IdentifierError;
use crate::identifier::{Entry, Identifiers};
use crate::time::{Date, DateError, Period, TimeUnit, Weekday};

const ALL_WEEKDAYS: u8 = 0x7f;
const WEEKEND: u8 = (1 << 5) | (1 << 6);

/// How member calendars determine the business days of a joint calendar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "FS-CAL defines exactly two joint rules"
)]
pub enum JointRule {
    /// A date is a business day only when every member considers it one.
    JoinHolidays,
    /// A date is a business day when at least one member considers it one.
    JoinBusinessDays,
}

impl JointRule {
    /// Both joint rules.
    pub const ALL: [Self; 2] = [Self::JoinHolidays, Self::JoinBusinessDays];

    /// Returns the exact identifier used by formatting and parsing.
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::JoinHolidays => "JoinHolidays",
            Self::JoinBusinessDays => "JoinBusinessDays",
        }
    }
}

impl fmt::Display for JointRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_identifier())
    }
}

impl FromStr for JointRule {
    type Err = IdentifierError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        JOINT_IDENTIFIERS.parse(text)
    }
}

/// A rule for adjusting a calendar date to a business day.
///
/// Exact variant names are the canonical text identifiers; parsing is
/// case-sensitive and accepts no aliases or surrounding whitespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "FS-BDC defines exactly these seven conventions"
)]
pub enum BusinessDayConvention {
    /// Earliest business day on or after the date.
    Following,
    /// Following within the same month, otherwise Preceding in that month.
    ModifiedFollowing,
    /// Latest business day on or before the date.
    Preceding,
    /// Preceding within the same month, otherwise Following in that month.
    ModifiedPreceding,
    /// Return the date unchanged, even when it is not a business day.
    Unadjusted,
    /// Modified Following that also avoids crossing the 15th from the first half.
    HalfMonthModifiedFollowing,
    /// Closest business day; equal distances favor Following.
    Nearest,
}

impl BusinessDayConvention {
    /// Every supported convention.
    pub const ALL: [Self; 7] = [
        Self::Following,
        Self::ModifiedFollowing,
        Self::Preceding,
        Self::ModifiedPreceding,
        Self::Unadjusted,
        Self::HalfMonthModifiedFollowing,
        Self::Nearest,
    ];

    /// Returns the canonical text identifier.
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::Following => "Following",
            Self::ModifiedFollowing => "ModifiedFollowing",
            Self::Preceding => "Preceding",
            Self::ModifiedPreceding => "ModifiedPreceding",
            Self::Unadjusted => "Unadjusted",
            Self::HalfMonthModifiedFollowing => "HalfMonthModifiedFollowing",
            Self::Nearest => "Nearest",
        }
    }
}

impl fmt::Display for BusinessDayConvention {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_identifier())
    }
}

impl FromStr for BusinessDayConvention {
    type Err = IdentifierError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        BDC_IDENTIFIERS.parse(text)
    }
}

/// An immutable calendar definition, including explicit per-date overrides.
///
/// Equality and hashing compare canonical definitions: input ordering and
/// duplicate holidays, weekends or joint members do not matter. Equal-rule
/// joints without overrides are flattened. Equality does not attempt general
/// Boolean equivalence between different joint definitions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Calendar {
    definition: Definition,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Definition {
    kind: Kind,
    // true means explicitly business, false means explicitly holiday.
    // Store only exceptions to this definition's baseline classification.
    overrides: BTreeMap<Date, bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Kind {
    Base {
        weekends: u8,
    },
    Target {
        allow_projection: bool,
    },
    Joint {
        members: Vec<Definition>,
        join_holidays: bool,
    },
}

impl Definition {
    fn base(weekends: u8) -> Self {
        Self {
            kind: Kind::Base { weekends },
            overrides: BTreeMap::new(),
        }
    }

    fn baseline(&self, date: Date) -> bool {
        match &self.kind {
            Kind::Base { weekends } => weekends & weekday_bit(date.weekday()) == 0,
            Kind::Target { .. } => target::target_business(date),
            Kind::Joint {
                members,
                join_holidays,
            } => {
                if *join_holidays {
                    members.iter().all(|member| member.business(date))
                } else {
                    members.iter().any(|member| member.business(date))
                }
            }
        }
    }

    fn business(&self, date: Date) -> bool {
        self.overrides
            .get(&date)
            .copied()
            .unwrap_or_else(|| self.baseline(date))
    }

    fn weekend_mask(&self) -> u8 {
        match &self.kind {
            Kind::Base { weekends } => *weekends,
            Kind::Target { .. } => WEEKEND,
            Kind::Joint {
                members,
                join_holidays,
            } => {
                if *join_holidays {
                    members
                        .iter()
                        .fold(0, |mask, member| mask | member.weekend_mask())
                } else {
                    members
                        .iter()
                        .fold(ALL_WEEKDAYS, |mask, member| mask & member.weekend_mask())
                }
            }
        }
    }

    fn has_business_override(&self) -> bool {
        self.overrides.values().any(|business| *business)
            || match &self.kind {
                Kind::Base { .. } | Kind::Target { .. } => false,
                Kind::Joint { members, .. } => members.iter().any(Self::has_business_override),
            }
    }

    fn depth(&self) -> u32 {
        match &self.kind {
            Kind::Base { .. } | Kind::Target { .. } => 0,
            Kind::Joint { members, .. } => 1 + members.iter().map(Self::depth).max().unwrap_or(0),
        }
    }

    fn override_date(&mut self, date: Date, business: bool) {
        if self.baseline(date) == business {
            self.overrides.remove(&date);
        } else {
            self.overrides.insert(date, business);
        }
    }

    fn validate_date(&self, date: Date) -> Result<(), CalendarError> {
        match &self.kind {
            Kind::Base { .. } => Ok(()),
            Kind::Target { allow_projection } => target::validate_date(date, *allow_projection),
            Kind::Joint { members, .. } => {
                // Validate every member before classification: a Boolean
                // short circuit must never conceal missing market coverage.
                for member in members {
                    member.validate_date(date)?;
                }
                Ok(())
            }
        }
    }

    fn collect_weekday_exceptions(&self, dates: &mut BTreeSet<Date>) -> bool {
        dates.extend(self.overrides.keys().copied());
        match &self.kind {
            Kind::Base { .. } => true,
            Kind::Target { .. } => false,
            Kind::Joint { members, .. } => members
                .iter()
                .all(|member| member.collect_weekday_exceptions(dates)),
        }
    }
}

impl Calendar {
    fn target_rules(allow_projection: bool) -> Self {
        Self {
            definition: Definition {
                kind: Kind::Target { allow_projection },
                overrides: BTreeMap::new(),
            },
        }
    }

    fn validate_date(&self, date: Date) -> Result<(), CalendarError> {
        self.definition.validate_date(date)
    }

    fn checked_business(&self, date: Date) -> Result<bool, CalendarError> {
        self.validate_date(date)?;
        Ok(self.is_business_day(date))
    }
    /// Maximum consecutive rejected candidates in one business-day search.
    ///
    /// A business candidate after 365 rejected dates is accepted. The 366th
    /// rejected date ends the search; candidate 367 is not inspected.
    pub const MAX_CONSECUTIVE_HOLIDAYS: u32 = 366;

    /// Maximum nested joint levels, after flattening equivalent joint rules.
    pub const MAX_JOINT_DEPTH: u32 = 64;

    /// Creates a calendar in which every date is a business day.
    #[must_use]
    pub fn null() -> Self {
        Self {
            definition: Definition::base(0),
        }
    }

    /// Creates a calendar whose only holidays are Saturdays and Sundays.
    #[must_use]
    pub fn weekends_only() -> Self {
        Self {
            definition: Definition::base(WEEKEND),
        }
    }

    /// Creates a calendar from weekend weekdays and explicit holiday dates.
    ///
    /// Ordering and duplication do not matter. Holidays already excluded by
    /// the weekend rules are redundant. All seven weekdays may be weekends:
    /// queries remain valid, while a business-day operation can fail.
    #[must_use]
    pub fn custom(
        weekends: impl IntoIterator<Item = Weekday>,
        holidays: impl IntoIterator<Item = Date>,
    ) -> Self {
        let mask = weekends
            .into_iter()
            .fold(0, |mask, weekday| mask | weekday_bit(weekday));
        let mut definition = Definition::base(mask);
        for date in holidays {
            definition.override_date(date, false);
        }
        Self { definition }
    }

    /// Combines calendars using the specified business-day rule.
    ///
    /// Members are canonicalized and deduplicated. Nested joints with the
    /// same rule and no per-date overrides are flattened. A joint's weekend
    /// weekdays are the union for JoinHolidays, intersection for JoinBusinessDays.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::EmptyJoint`] for no members, or
    /// [`CalendarError::NestingLimit`] if the normalized depth exceeds 64.
    pub fn joint(calendars: Vec<Self>, rule: JointRule) -> Result<Self, CalendarError> {
        if calendars.is_empty() {
            return Err(CalendarError::EmptyJoint);
        }
        let join_holidays = rule == JointRule::JoinHolidays;
        let mut members = Vec::new();
        for calendar in calendars {
            let definition = calendar.definition;
            match definition {
                Definition {
                    kind:
                        Kind::Joint {
                            members: nested,
                            join_holidays: nested_rule,
                        },
                    overrides,
                } if overrides.is_empty() && nested_rule == join_holidays => members.extend(nested),
                definition => members.push(definition),
            }
        }
        members.sort_unstable();
        members.dedup();
        let depth = 1 + members.iter().map(Definition::depth).max().unwrap_or(0);
        if depth > Self::MAX_JOINT_DEPTH {
            return Err(CalendarError::NestingLimit {
                depth,
                maximum: Self::MAX_JOINT_DEPTH,
            });
        }
        Ok(Self {
            definition: Definition {
                kind: Kind::Joint {
                    members,
                    join_holidays,
                },
                overrides: BTreeMap::new(),
            },
        })
    }

    /// Returns a new calendar where `date` is explicitly a holiday.
    ///
    /// This overrides the current definition, including a previous explicit
    /// business-day override. The original calendar is unaffected.
    #[must_use]
    pub fn with_holiday(&self, date: Date) -> Self {
        let mut result = self.clone();
        result.definition.override_date(date, false);
        result
    }

    /// Returns a new calendar where `date` is explicitly a business day.
    ///
    /// This can make a weekend business. [`is_weekend`](Self::is_weekend)
    /// continues to report the weekday rule; [`is_holiday`](Self::is_holiday)
    /// reports the resulting business classification. The original is unaffected.
    #[must_use]
    pub fn without_holiday(&self, date: Date) -> Self {
        let mut result = self.clone();
        result.definition.override_date(date, true);
        result
    }

    /// Returns whether the date's weekday belongs to the weekend definition.
    #[must_use]
    pub fn is_weekend(&self, date: Date) -> bool {
        self.is_weekend_weekday(date.weekday())
    }

    /// Returns whether a weekday belongs to this calendar's weekend rules.
    ///
    /// Per-date business overrides do not change the weekday rule.
    #[must_use]
    pub fn is_weekend_weekday(&self, weekday: Weekday) -> bool {
        self.definition.weekend_mask() & weekday_bit(weekday) != 0
    }

    /// Returns whether the date is a business day, including explicit overrides.
    #[must_use]
    pub fn is_business_day(&self, date: Date) -> bool {
        self.definition.business(date)
    }

    /// Returns whether the date is non-business, including weekends.
    #[must_use]
    pub fn is_holiday(&self, date: Date) -> bool {
        !self.is_business_day(date)
    }

    /// Adjusts a date using a business-day convention.
    ///
    /// Unadjusted is always the identity. Other conventions leave business
    /// dates unchanged. Modified conventions require a business day in the
    /// original month; Nearest favors Following on an equal-distance tie.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a date-range boundary, 366 rejected candidates,
    /// no possible business weekday, or no business day in a modified month.
    pub fn adjust(
        &self,
        date: Date,
        convention: BusinessDayConvention,
    ) -> Result<Date, CalendarError> {
        self.validate_date(date)?;
        if convention == BusinessDayConvention::Unadjusted || self.checked_business(date)? {
            return Ok(date);
        }
        self.require_possible_business_day()?;
        match convention {
            BusinessDayConvention::Unadjusted => Ok(date),
            BusinessDayConvention::Following => self.search(date, 1),
            BusinessDayConvention::Preceding => self.search(date, -1),
            BusinessDayConvention::ModifiedFollowing => self.modified(date, 1, None),
            BusinessDayConvention::ModifiedPreceding => self.modified(date, -1, None),
            BusinessDayConvention::HalfMonthModifiedFollowing => {
                self.modified(date, 1, if date.day() <= 15 { Some(15) } else { None })
            }
            BusinessDayConvention::Nearest => {
                let forward = self.search(date, 1);
                let backward = self.search(date, -1);
                match (forward, backward) {
                    (Ok(forward), Ok(backward)) => {
                        if forward.days_since(date) <= date.days_since(backward) {
                            Ok(forward)
                        } else {
                            Ok(backward)
                        }
                    }
                    (Ok(_), Err(error)) | (Err(error), Ok(_))
                        if matches!(
                            error,
                            CalendarError::OutsideValidity { .. }
                                | CalendarError::UnverifiedCoverage { .. }
                        ) =>
                    {
                        Err(error)
                    }
                    (Ok(result), Err(_)) | (Err(_), Ok(result)) => Ok(result),
                    (Err(forward), Err(backward)) => {
                        if matches!(
                            forward,
                            CalendarError::OutsideValidity { .. }
                                | CalendarError::UnverifiedCoverage { .. }
                        ) {
                            Err(forward)
                        } else if matches!(
                            backward,
                            CalendarError::Date(_)
                                | CalendarError::OutsideValidity { .. }
                                | CalendarError::UnverifiedCoverage { .. }
                        ) {
                            Err(backward)
                        } else {
                            Err(forward)
                        }
                    }
                }
            }
        }
    }

    /// Adds a signed number of business days, excluding the starting date.
    ///
    /// A zero count returns the original date even when it is a holiday.
    /// Positive counts move forward; negative counts move backward. The
    /// consecutive-holiday search counter resets after each business day.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::Date`] if the result leaves the date range,
    /// [`CalendarError::NoBusinessWeekday`] for a permanently closed definition,
    /// or [`CalendarError::SearchLimit`] after 366 consecutive rejected dates.
    pub fn add_business_days(&self, date: Date, count: i32) -> Result<Date, CalendarError> {
        self.validate_date(date)?;
        if count == 0 {
            return Ok(date);
        }
        self.require_possible_business_day()?;
        // At least |count| calendar days are required, so this rejects huge
        // counts before walking the calendar. Widen before taking abs(MIN).
        date.checked_add_days(i64::from(count))?;
        if matches!(self.definition.kind, Kind::Base { weekends: 0 })
            && self.definition.overrides.is_empty()
        {
            return Ok(date.checked_add_days(i64::from(count))?);
        }
        let direction = if count > 0 { 1 } else { -1 };
        let mut remaining = i64::from(count).abs();
        let mut current = date;
        let mut rejected = 0;
        let mut search_start = date;
        while remaining > 0 {
            current = current.checked_add_days(i64::from(direction))?;
            if self.checked_business(current)? {
                remaining -= 1;
                rejected = 0;
                search_start = current;
            } else {
                rejected += 1;
                if rejected == Self::MAX_CONSECUTIVE_HOLIDAYS {
                    return Err(CalendarError::SearchLimit {
                        start: search_start,
                        direction,
                        rejected,
                    });
                }
            }
        }
        Ok(current)
    }

    /// Advances by a tenor with business-day semantics for [`TimeUnit::Days`].
    ///
    /// A zero tenor adjusts the original date. Nonzero Days count business
    /// days and ignore the adjustment convention; Weeks add calendar weeks
    /// and adjust. Months/Years clamp first, then apply the explicit EOM rule
    /// from FS-CAL: Unadjusted preserves calendar EOM only; adjusted conventions
    /// move a business-EOM start to business EOM in the target month.
    ///
    /// # Errors
    ///
    /// Returns the errors of checked date arithmetic, adjustment, business-day
    /// addition or finding a business end of month.
    pub fn advance(
        &self,
        date: Date,
        period: Period,
        convention: BusinessDayConvention,
        end_of_month: bool,
    ) -> Result<Date, CalendarError> {
        self.validate_date(date)?;
        if period.length() == 0 {
            return self.adjust(date, convention);
        }
        match period.unit() {
            TimeUnit::Days => self.add_business_days(date, period.length()),
            TimeUnit::Weeks => self.adjust(
                date.checked_add_weeks(i64::from(period.length()))?,
                convention,
            ),
            TimeUnit::Months | TimeUnit::Years => {
                let target = date.checked_add_period(period)?;
                self.validate_date(target)?;
                if end_of_month {
                    if convention == BusinessDayConvention::Unadjusted {
                        if date.is_end_of_month() {
                            let result = target.last_day_of_month();
                            self.validate_date(result)?;
                            return Ok(result);
                        }
                    } else if self.is_end_of_month(date)? {
                        return self.end_of_month(target);
                    }
                }
                self.adjust(target, convention)
            }
        }
    }

    /// Returns the first business day of the date's month.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::NoBusinessDayInMonth`] for a fully closed month.
    pub fn start_of_month(&self, date: Date) -> Result<Date, CalendarError> {
        self.validate_date(date)?;
        self.search_in_month(date.first_day_of_month(), 1, None)
            .map_err(|error| match error {
                CalendarError::NoBusinessDayInMonth { .. } => {
                    CalendarError::NoBusinessDayInMonth { date }
                }
                error => error,
            })
    }

    /// Returns the last business day of the date's month.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::NoBusinessDayInMonth`] for a fully closed month.
    pub fn end_of_month(&self, date: Date) -> Result<Date, CalendarError> {
        self.validate_date(date)?;
        self.search_in_month(date.last_day_of_month(), -1, None)
            .map_err(|error| match error {
                CalendarError::NoBusinessDayInMonth { .. } => {
                    CalendarError::NoBusinessDayInMonth { date }
                }
                error => error,
            })
    }

    /// Returns whether the date is on or before its month's first business day.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::NoBusinessDayInMonth`] for a fully closed month.
    pub fn is_start_of_month(&self, date: Date) -> Result<bool, CalendarError> {
        Ok(date <= self.start_of_month(date)?)
    }

    /// Returns whether the date is on or after its month's last business day.
    ///
    /// A holiday after the final business day also counts as business EOM.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::NoBusinessDayInMonth`] for a fully closed month.
    pub fn is_end_of_month(&self, date: Date) -> Result<bool, CalendarError> {
        Ok(date >= self.end_of_month(date)?)
    }

    /// Counts business dates with explicit endpoint inclusion.
    ///
    /// Reversing endpoints negates the result and swaps inclusion flags. Equal
    /// dates count as one only when both flags are true and the date is business.
    /// The result always fits `i32` over the supported date range.
    /// Weekday-only definitions and their joints count complete weeks
    /// arithmetically, then correct distinct dated overrides. No count cache
    /// or mutable state is used.
    #[must_use]
    pub fn business_days_between(
        &self,
        start: Date,
        end: Date,
        include_start: bool,
        include_end: bool,
    ) -> i32 {
        if start == end {
            return i32::from(include_start && include_end && self.is_business_day(start));
        }
        if start > end {
            return -self.business_days_between(end, start, include_end, include_start);
        }
        let mut exceptions = BTreeSet::new();
        if self.definition.collect_weekday_exceptions(&mut exceptions) {
            let mask = self.definition.weekend_mask();
            let span = end.days_since(start) + 1;
            let whole_weeks = span / 7;
            let remainder = span % 7;
            let first_weekday = i64::from(start.weekday().number_from_monday());
            let mut count = 0;
            for weekday in Weekday::ALL {
                if mask & weekday_bit(weekday) == 0 {
                    let offset =
                        (i64::from(weekday.number_from_monday()) - first_weekday).rem_euclid(7);
                    count += whole_weeks + i32::from(offset < i64::from(remainder));
                }
            }
            let baseline_business = |date: Date| mask & weekday_bit(date.weekday()) == 0;
            count -= i32::from(!include_start && baseline_business(start));
            count -= i32::from(!include_end && baseline_business(end));
            for &date in exceptions.range(start..=end) {
                if (date != start || include_start) && (date != end || include_end) {
                    count +=
                        i32::from(self.is_business_day(date)) - i32::from(baseline_business(date));
                }
            }
            return count;
        }
        let span = end.days_since(start);
        let first = i32::from(!include_start);
        let last = span - i32::from(!include_end);
        let mut count = 0;
        for offset in first..=last {
            // Every offset lies between validated endpoints. The checked
            // operation therefore succeeds; retain non-panicking iteration.
            if let Ok(date) = start.checked_add_days(i64::from(offset)) {
                count += i32::from(self.is_business_day(date));
            }
        }
        count
    }

    /// Lists holidays in the inclusive interval in chronological order.
    ///
    /// When `include_weekends` is false, dates classified as weekend by the
    /// calendar's weekday rules are excluded, even if also explicitly holiday.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::InvalidRange`] if `start > end`.
    pub fn holiday_list(
        &self,
        start: Date,
        end: Date,
        include_weekends: bool,
    ) -> Result<Vec<Date>, CalendarError> {
        self.date_list(start, end, |date| {
            self.is_holiday(date) && (include_weekends || !self.is_weekend(date))
        })
    }

    /// Lists business days in the inclusive interval in chronological order.
    ///
    /// # Errors
    ///
    /// Returns [`CalendarError::InvalidRange`] if `start > end`.
    pub fn business_day_list(&self, start: Date, end: Date) -> Result<Vec<Date>, CalendarError> {
        self.date_list(start, end, |date| self.is_business_day(date))
    }

    fn date_list(
        &self,
        start: Date,
        end: Date,
        include: impl Fn(Date) -> bool,
    ) -> Result<Vec<Date>, CalendarError> {
        if start > end {
            return Err(CalendarError::InvalidRange { start, end });
        }
        let mut result = Vec::new();
        for offset in 0..=end.days_since(start) {
            let date = start.checked_add_days(i64::from(offset))?;
            if include(date) {
                result.push(date);
            }
        }
        Ok(result)
    }

    fn require_possible_business_day(&self) -> Result<(), CalendarError> {
        if self.definition.weekend_mask() == ALL_WEEKDAYS
            && !self.definition.has_business_override()
        {
            Err(CalendarError::NoBusinessWeekday)
        } else {
            Ok(())
        }
    }

    fn search(&self, date: Date, direction: i32) -> Result<Date, CalendarError> {
        self.require_possible_business_day()?;
        let mut current = date;
        for rejected in 0..Self::MAX_CONSECUTIVE_HOLIDAYS {
            if self.checked_business(current)? {
                return Ok(current);
            }
            if rejected + 1 == Self::MAX_CONSECUTIVE_HOLIDAYS {
                return Err(CalendarError::SearchLimit {
                    start: date,
                    direction,
                    rejected: rejected + 1,
                });
            }
            current = current.checked_add_days(i64::from(direction))?;
        }
        Err(CalendarError::SearchLimit {
            start: date,
            direction,
            rejected: Self::MAX_CONSECUTIVE_HOLIDAYS,
        })
    }

    fn search_in_month(
        &self,
        date: Date,
        direction: i32,
        last_forward_day: Option<u32>,
    ) -> Result<Date, CalendarError> {
        let mut current = date;
        loop {
            if self.checked_business(current)? {
                return Ok(current);
            }
            let next = current.checked_add_days(i64::from(direction));
            match next {
                Ok(next)
                    if next.year() == date.year()
                        && next.month() == date.month()
                        && last_forward_day.is_none_or(|limit| next.day() <= limit) =>
                {
                    current = next
                }
                _ => return Err(CalendarError::NoBusinessDayInMonth { date }),
            }
        }
    }

    fn modified(
        &self,
        date: Date,
        direction: i32,
        last_forward_day: Option<u32>,
    ) -> Result<Date, CalendarError> {
        match self.search_in_month(date, direction, last_forward_day) {
            Err(CalendarError::NoBusinessDayInMonth { .. }) => {
                self.search_in_month(date, -direction, None)
            }
            result => result,
        }
    }
}

/// A calendar construction or checked business-day operation failure.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CalendarError {
    /// Date arithmetic failed. Kind: `calendar.date`; source is the date error.
    Date(DateError),
    /// A market query precedes the modeled market's validity.
    /// Kind: `calendar.outside_validity`.
    #[non_exhaustive]
    OutsideValidity {
        /// The first rejected date.
        date: Date,
    },
    /// A strict market query lacks source-verified coverage.
    /// Kind: `calendar.unverified_coverage`.
    #[non_exhaustive]
    UnverifiedCoverage {
        /// The first rejected date.
        date: Date,
    },
    /// Weekday rules close every day and no explicit business override exists.
    /// Kind: `calendar.no_business_weekday`.
    NoBusinessWeekday,
    /// A joint calendar had no members. Kind: `calendar.empty_joint`.
    EmptyJoint,
    /// A directional search rejected 366 consecutive candidates.
    /// Kind: `calendar.search_limit`.
    #[non_exhaustive]
    SearchLimit {
        /// The date at the start of this search segment.
        start: Date,
        /// Search direction: 1 forward, -1 backward.
        direction: i32,
        /// Number of rejected consecutive candidates.
        rejected: u32,
    },
    /// A month has no eligible business day. Kind: `calendar.no_business_day_in_month`.
    #[non_exhaustive]
    NoBusinessDayInMonth {
        /// The original date whose month was searched.
        date: Date,
    },
    /// An inclusive list range was reversed. Kind: `calendar.invalid_range`.
    #[non_exhaustive]
    InvalidRange {
        /// The requested first date.
        start: Date,
        /// The requested last date.
        end: Date,
    },
    /// Normalized joint nesting exceeded its bound. Kind: `calendar.nesting_limit`.
    #[non_exhaustive]
    NestingLimit {
        /// The depth that construction would have produced.
        depth: u32,
        /// The maximum supported joint depth.
        maximum: u32,
    },
}

impl CalendarError {
    /// Returns the stable `calendar.<variant>` kind name.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Date(_) => "calendar.date",
            Self::OutsideValidity { .. } => "calendar.outside_validity",
            Self::UnverifiedCoverage { .. } => "calendar.unverified_coverage",
            Self::NoBusinessWeekday => "calendar.no_business_weekday",
            Self::EmptyJoint => "calendar.empty_joint",
            Self::SearchLimit { .. } => "calendar.search_limit",
            Self::NoBusinessDayInMonth { .. } => "calendar.no_business_day_in_month",
            Self::InvalidRange { .. } => "calendar.invalid_range",
            Self::NestingLimit { .. } => "calendar.nesting_limit",
        }
    }
}

impl fmt::Display for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Date(error) => fmt::Display::fmt(error, f),
            Self::OutsideValidity { date } => {
                write!(f, "TARGET market rules are not valid on {date}")
            }
            Self::UnverifiedCoverage { date } => write!(
                f,
                "TARGET source coverage is not verified on {date}; projection requires an explicit policy"
            ),
            Self::NoBusinessWeekday => {
                f.write_str("calendar has no business weekday or explicit business-day override")
            }
            Self::EmptyJoint => f.write_str("a joint calendar must have at least one member"),
            Self::SearchLimit {
                start,
                direction,
                rejected,
            } => write!(
                f,
                "business-day search from {start} in direction {direction} rejected {rejected} consecutive dates"
            ),
            Self::NoBusinessDayInMonth { date } => {
                write!(f, "no eligible business day in the month of {date}")
            }
            Self::InvalidRange { start, end } => {
                write!(f, "list range starts at {start}, after its end {end}")
            }
            Self::NestingLimit { depth, maximum } => {
                write!(f, "joint calendar depth {depth} exceeds maximum {maximum}")
            }
        }
    }
}

impl StdError for CalendarError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Date(error) => Some(error),
            _ => None,
        }
    }
}

impl From<DateError> for CalendarError {
    fn from(error: DateError) -> Self {
        Self::Date(error)
    }
}

const fn weekday_bit(weekday: Weekday) -> u8 {
    1 << (weekday.number_from_monday() - 1)
}

const JOINT_IDENTIFIERS: Identifiers<JointRule> = Identifiers::new(
    "joint rule",
    &[
        Entry {
            value: JointRule::JoinHolidays,
            canonical: "JoinHolidays",
            aliases: &[],
        },
        Entry {
            value: JointRule::JoinBusinessDays,
            canonical: "JoinBusinessDays",
            aliases: &[],
        },
    ],
    &[],
);

const BDC_IDENTIFIERS: Identifiers<BusinessDayConvention> = Identifiers::new(
    "business-day convention",
    &[
        Entry {
            value: BusinessDayConvention::Following,
            canonical: "Following",
            aliases: &[],
        },
        Entry {
            value: BusinessDayConvention::ModifiedFollowing,
            canonical: "ModifiedFollowing",
            aliases: &[],
        },
        Entry {
            value: BusinessDayConvention::Preceding,
            canonical: "Preceding",
            aliases: &[],
        },
        Entry {
            value: BusinessDayConvention::ModifiedPreceding,
            canonical: "ModifiedPreceding",
            aliases: &[],
        },
        Entry {
            value: BusinessDayConvention::Unadjusted,
            canonical: "Unadjusted",
            aliases: &[],
        },
        Entry {
            value: BusinessDayConvention::HalfMonthModifiedFollowing,
            canonical: "HalfMonthModifiedFollowing",
            aliases: &[],
        },
        Entry {
            value: BusinessDayConvention::Nearest,
            canonical: "Nearest",
            aliases: &[],
        },
    ],
    &[],
);
