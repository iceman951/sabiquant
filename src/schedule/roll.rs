//! Month-coordinate civil rolls with explicit requested/resolved provenance.

use super::{
    DateGenerationRule, PeriodRegularity, Rolling, Schedule, ScheduleBuilder, ScheduleError,
    validate_limit,
};
use crate::error::IdentifierError;
use crate::identifier::{Entry, Identifiers};
use crate::time::{Date, Month, Period, TimeUnit, Weekday};
use std::fmt;
use std::str::FromStr;

/// The four explicitly supported civil month-roll definitions.
///
/// Exact canonical names are the variant names. No calendar, frequency,
/// CDS product definition or implicit EOM policy follows from this label.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "closed four-rule analytical civil roll contract"
)]
pub enum SpecialRollRule {
    /// Keep requested endpoints; roll interior stride months to third Wednesday.
    ThirdWednesday,
    /// Also replace both endpoints by third Wednesday of their requested months.
    ThirdWednesdayInclusive,
    /// Monthly twentieth endpoint/anchor projection with the supplied month stride.
    Twentieth,
    /// Quarter-twentieth projection; the month stride must be divisible by three.
    TwentiethIMM,
}

impl SpecialRollRule {
    /// Every supported civil roll in canonical order.
    pub const ALL: [Self; 4] = [
        Self::ThirdWednesday,
        Self::ThirdWednesdayInclusive,
        Self::Twentieth,
        Self::TwentiethIMM,
    ];

    /// Returns the exact canonical identifier.
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::ThirdWednesday => "ThirdWednesday",
            Self::ThirdWednesdayInclusive => "ThirdWednesdayInclusive",
            Self::Twentieth => "Twentieth",
            Self::TwentiethIMM => "TwentiethIMM",
        }
    }

    fn is_wednesday(self) -> bool {
        matches!(self, Self::ThirdWednesday | Self::ThirdWednesdayInclusive)
    }
}

const ROLL_IDS: Identifiers<SpecialRollRule> = Identifiers::new(
    "special roll rule",
    &[
        Entry {
            value: SpecialRollRule::ThirdWednesday,
            canonical: "ThirdWednesday",
            aliases: &[],
        },
        Entry {
            value: SpecialRollRule::ThirdWednesdayInclusive,
            canonical: "ThirdWednesdayInclusive",
            aliases: &[],
        },
        Entry {
            value: SpecialRollRule::Twentieth,
            canonical: "Twentieth",
            aliases: &[],
        },
        Entry {
            value: SpecialRollRule::TwentiethIMM,
            canonical: "TwentiethIMM",
            aliases: &[],
        },
    ],
    &[],
);

impl fmt::Display for SpecialRollRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_identifier())
    }
}

impl FromStr for SpecialRollRule {
    type Err = IdentifierError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        ROLL_IDS.parse(text)
    }
}

/// Immutable requested range, resolved range and month-lattice definition.
///
/// Equality/hash retain the tenor's STORED length/unit: 1Y and 12M define
/// different provenance even though their Period values normalize equally.
/// The anchor year/month is a roll coordinate, not a clamped Date seed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RollProvenance {
    requested_start: Date,
    requested_end: Date,
    resolved_start: Date,
    resolved_end: Date,
    rule: SpecialRollRule,
    tenor_length: i32,
    tenor_unit: TimeUnit,
    anchor_year: i32,
    anchor_month: Month,
}

impl RollProvenance {
    /// Returns the caller's original start, before endpoint projection.
    #[must_use]
    pub const fn requested_start(&self) -> Date {
        self.requested_start
    }

    /// Returns the caller's original end, before endpoint projection.
    #[must_use]
    pub const fn requested_end(&self) -> Date {
        self.requested_end
    }

    /// Returns the first raw contractual date, before business-day adjustment.
    #[must_use]
    pub const fn resolved_start(&self) -> Date {
        self.resolved_start
    }

    /// Returns the last raw contractual date, before business-day adjustment.
    #[must_use]
    pub const fn resolved_end(&self) -> Date {
        self.resolved_end
    }

    /// Returns the explicit civil roll definition.
    #[must_use]
    pub const fn rule(&self) -> SpecialRollRule {
        self.rule
    }

    /// Returns the original stored tenor, without normalizing its unit.
    #[must_use]
    pub const fn tenor(&self) -> Period {
        Period::new(self.tenor_length, self.tenor_unit)
    }

    /// Returns the civil year of the retained roll anchor month.
    #[must_use]
    pub const fn anchor_year(&self) -> i32 {
        self.anchor_year
    }

    /// Returns the civil month of the retained roll anchor.
    #[must_use]
    pub const fn anchor_month(&self) -> Month {
        self.anchor_month
    }
}

impl ScheduleBuilder {
    /// Builds an explicit civil roll on a retained month-coordinate lattice.
    ///
    /// Positive stored Months/Years are required; TwentiethIMM additionally
    /// requires a stride divisible by three. Wednesday interiors start one
    /// stride after the requested start month. Inclusive replaces both
    /// endpoints by their month's Wednesday. Twentieth rules retain start,
    /// anchor on the first eligible twentieth on/after start, and project end
    /// to the first eligible twentieth on/after end, independently of stride.
    /// Only consecutive actual points of the retained lattice are Regular.
    ///
    /// No EOM, automatic stub policy or explicit coupon roles are inferred.
    /// Original and projected inputs are available through roll provenance.
    /// Existing adjustment, strict ordering and full TARGET guards apply.
    /// Generated roll ICMA contexts remain explicitly unsupported.
    ///
    /// Work/storage is O(final boundaries), plus existing calendar costs.
    /// `max_boundaries` counts every retained boundary including endpoints.
    ///
    /// ```
    /// use sabiquant::calendar::{BusinessDayConvention as Bdc, Calendar};
    /// use sabiquant::schedule::{ScheduleBuilder, ScheduleCalendar, SpecialRollRule};
    /// use sabiquant::time::{Date, Period, TimeUnit};
    /// let schedule = ScheduleBuilder::new(Date::from_ymd(2026, 2, 6)?,
    ///     Date::from_ymd(2026, 11, 17)?, ScheduleCalendar::Plain(Calendar::null()),
    ///     Bdc::Unadjusted, Bdc::Unadjusted)?
    ///     .build_roll(Period::new(3, TimeUnit::Months), SpecialRollRule::TwentiethIMM, 5)?;
    /// assert_eq!(schedule.contractual_dates()[1], Date::from_ymd(2026, 3, 20)?);
    /// assert_eq!(schedule.contractual_dates()[4], Date::from_ymd(2026, 12, 20)?);
    /// # Ok::<(), sabiquant::error::Error>(())
    /// ```
    ///
    /// # Errors
    /// Checks cap, stored tenor domain, original TARGET span, required roll
    /// dates/range, union coverage, boundary count and adjustment in order.
    /// Required roll-date failures preserve the input, role and Date cause.
    /// A future coordinate beyond resolved end needs no Date construction.
    /// No partial value is returned for coverage, allocation or order errors.
    pub fn build_roll(
        &self,
        tenor: Period,
        rule: SpecialRollRule,
        max_boundaries: u32,
    ) -> Result<Schedule, ScheduleError> {
        validate_limit(max_boundaries)?;
        let stride = match tenor.unit() {
            TimeUnit::Months => i64::from(tenor.length()),
            TimeUnit::Years => 12 * i64::from(tenor.length()),
            TimeUnit::Days | TimeUnit::Weeks => 0,
        };
        if stride <= 0 || (rule == SpecialRollRule::TwentiethIMM && stride % 3 != 0) {
            return Err(ScheduleError::InvalidRollTenor { rule, tenor });
        }
        self.calendar.validate_span(self.start, self.end)?;
        let (start, end, anchor) = match rule {
            SpecialRollRule::ThirdWednesday => (
                self.start,
                self.end,
                roll_date(self.start, 0, rule, tenor, "anchor")?,
            ),
            SpecialRollRule::ThirdWednesdayInclusive => {
                let start = roll_date(self.start, 0, rule, tenor, "start")?;
                let end = roll_date(self.end, 0, rule, tenor, "end")?;
                (start, end, start)
            }
            SpecialRollRule::Twentieth | SpecialRollRule::TwentiethIMM => {
                let anchor = twentieth_on_or_after(self.start, rule, tenor, "anchor")?;
                let end = twentieth_on_or_after(self.end, rule, tenor, "end")?;
                (self.start, end, anchor)
            }
        };
        if start >= end {
            return Err(ScheduleError::ResolvedRollRange {
                rule,
                tenor,
                requested_start: self.start,
                requested_end: self.end,
                resolved_start: start,
                resolved_end: end,
            });
        }
        self.calendar.validate_span_for(
            self.start.min(start),
            self.end.max(end),
            "roll coverage",
        )?;
        // Every valid roll date has a month in 1..=12. Keep a typed date cause
        // instead of assuming a fallible conversion can never be extended.
        let anchor_month =
            Month::try_from(anchor.month()).map_err(|cause| ScheduleError::RollDate {
                rule,
                tenor,
                role: "anchor",
                input: self.start,
                cause,
            })?;
        let provenance = RollProvenance {
            requested_start: self.start,
            requested_end: self.end,
            resolved_start: start,
            resolved_end: end,
            rule,
            tenor_length: tenor.length(),
            tenor_unit: tenor.unit(),
            anchor_year: anchor.year(),
            anchor_month,
        };
        let anchor_coordinate = month_coordinate(anchor);
        let end_coordinate = month_coordinate(end);
        let mut coordinate = anchor_coordinate + if rule.is_wednesday() { stride } else { 0 };
        let mut rolling = Rolling::new(start)?;
        let mut previous = start;
        // Inspect civil coordinates first. No out-of-range Date is needed for
        // an unused candidate, including a maximal stored i32 year stride.
        while coordinate <= end_coordinate {
            let candidate = roll_date(
                anchor,
                coordinate - anchor_coordinate,
                rule,
                tenor,
                "anchor",
            )?;
            if candidate >= end {
                break;
            }
            if candidate > start {
                rolling.push_limited(
                    candidate,
                    regularity(previous, candidate, anchor_coordinate, stride, rule),
                    max_boundaries,
                )?;
                previous = candidate;
            }
            coordinate += stride;
        }
        rolling.push_limited(
            end,
            regularity(previous, end, anchor_coordinate, stride, rule),
            max_boundaries,
        )?;
        let mut schedule = self.finish(
            rolling,
            DateGenerationRule::Roll(rule),
            Some(tenor),
            None,
            None,
            None,
        )?;
        schedule.roll_provenance = Some(provenance);
        Ok(schedule)
    }
}

fn month_coordinate(date: Date) -> i64 {
    12 * i64::from(date.year() - 1) + i64::from(date.month() - 1)
}

fn roll_date(
    input: Date,
    months: i64,
    rule: SpecialRollRule,
    tenor: Period,
    role: &'static str,
) -> Result<Date, ScheduleError> {
    let result = input.checked_add_months(months).and_then(|month| {
        if rule.is_wednesday() {
            Date::nth_weekday(month.year(), month.month(), 3, Weekday::Wednesday)
        } else {
            Date::from_ymd(month.year(), month.month(), 20)
        }
    });
    result.map_err(|cause| ScheduleError::RollDate {
        rule,
        tenor,
        role,
        input,
        cause,
    })
}

fn twentieth_on_or_after(
    input: Date,
    rule: SpecialRollRule,
    tenor: Period,
    role: &'static str,
) -> Result<Date, ScheduleError> {
    let mut months = i64::from(input.day() > 20);
    if rule == SpecialRollRule::TwentiethIMM {
        // Month numbers divisible by three are the eligible quarter months.
        months += (3 - (i64::from(input.month()) + months) % 3) % 3;
    }
    roll_date(input, months, rule, tenor, role)
}

fn regularity(
    left: Date,
    right: Date,
    anchor: i64,
    stride: i64,
    rule: SpecialRollRule,
) -> PeriodRegularity {
    let left_month = month_coordinate(left);
    let right_month = month_coordinate(right);
    let on_roll = |date: Date| {
        if rule.is_wednesday() {
            (15..=21).contains(&date.day()) && date.weekday() == Weekday::Wednesday
        } else {
            date.day() == 20
                && (rule != SpecialRollRule::TwentiethIMM || date.month().is_multiple_of(3))
        }
    };
    if right_month - left_month == stride
        && (left_month - anchor).rem_euclid(stride) == 0
        && on_roll(left)
        && on_roll(right)
    {
        PeriodRegularity::Regular
    } else {
        PeriodRegularity::Stub
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::{BusinessDayConvention, Calendar};
    use crate::schedule::ScheduleCalendar;

    #[test]
    fn roll_and_generation_identifier_tables_are_complete_and_well_formed() {
        assert_eq!(ROLL_IDS.check(), Ok(()));
        assert_eq!(
            ROLL_IDS.check_round_trip(&SpecialRollRule::ALL, |rule| rule.to_string()),
            Ok(())
        );
        assert_eq!(super::super::RULE_IDS.check(), Ok(()));
        assert_eq!(
            super::super::RULE_IDS
                .check_round_trip(&DateGenerationRule::ALL, |rule| rule.to_string()),
            Ok(())
        );
    }

    #[test]
    fn supplied_end_month_roll_can_be_an_off_stride_stub() -> Result<(), crate::error::Error> {
        let start = Date::from_ymd(2024, 1, 20)?;
        let requested_end = Date::from_ymd(2024, 6, 21)?;
        let schedule = ScheduleBuilder::new(
            start,
            requested_end,
            ScheduleCalendar::Plain(Calendar::null()),
            BusinessDayConvention::Unadjusted,
            BusinessDayConvention::Unadjusted,
        )?
        .build_roll(
            Period::new(3, TimeUnit::Months),
            SpecialRollRule::Twentieth,
            4,
        )?;
        assert_eq!(
            schedule.contractual_dates(),
            [
                start,
                Date::from_ymd(2024, 4, 20)?,
                Date::from_ymd(2024, 7, 20)?
            ]
        );
        assert_eq!(schedule.regularity(), [PeriodRegularity::Regular; 2]);
        let off_stride = ScheduleBuilder::new(
            start,
            Date::from_ymd(2024, 5, 21)?,
            ScheduleCalendar::Plain(Calendar::null()),
            BusinessDayConvention::Unadjusted,
            BusinessDayConvention::Unadjusted,
        )?
        .build_roll(
            Period::new(3, TimeUnit::Months),
            SpecialRollRule::Twentieth,
            3,
        )?;
        assert_eq!(
            off_stride.contractual_dates(),
            [
                start,
                Date::from_ymd(2024, 4, 20)?,
                Date::from_ymd(2024, 6, 20)?
            ]
        );
        assert_eq!(
            off_stride.regularity(),
            [PeriodRegularity::Regular, PeriodRegularity::Stub]
        );
        Ok(())
    }
}
