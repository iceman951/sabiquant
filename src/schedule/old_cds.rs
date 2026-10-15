//! Automatic OldCDS twentieth schedules on quarterly anchors, with explicit provenance.

use super::roll::{checked_roll_date, month_coordinate, regularity, twentieth_months_ahead};
use super::{
    DateGenerationRule, Rolling, Schedule, ScheduleBuilder, ScheduleError, SpecialRollRule,
    validate_limit,
};
use crate::time::{Date, DateError, Period, TimeUnit};

/// Calendar days before the initial quarter within which it is not retained.
const CUTOFF_DAYS: i32 = 30;

/// Q(x): the first March/June/September/December twentieth on or after x.
fn quarter_on_or_after(input: Date) -> Result<Date, DateError> {
    checked_roll_date(
        input,
        twentieth_months_ahead(input, true),
        SpecialRollRule::Twentieth,
    )
}

/// Immutable inputs and resolved quarters of one automatic OldCDS schedule.
///
/// The generation start and requested end are generic builder operands, not
/// legal trade, effective, protection or maturity dates. Equality/hash keep
/// the tenor's STORED length/unit, so 1Y and 12M remain distinct definitions.
/// The recurrence anchor is the lattice origin used for regularity and ICMA
/// references; it is not the ordinary clamped seed of
/// [`Schedule::lattice_anchor`], which stays None for OldCDS.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OldCdsProvenance {
    generation_start: Date,
    requested_end: Date,
    initial_quarter: Date,
    recurrence_anchor: Date,
    resolved_end: Date,
    omitted_quarter: Option<Date>,
    tenor_length: i32,
    tenor_unit: TimeUnit,
}

impl OldCdsProvenance {
    /// Returns the builder start S, retained as the first raw boundary.
    #[must_use]
    pub const fn generation_start(&self) -> Date {
        self.generation_start
    }

    /// Returns the builder end E, which stops interior generation.
    #[must_use]
    pub const fn requested_end(&self) -> Date {
        self.requested_end
    }

    /// Returns A, the first quarterly twentieth on or after the start.
    #[must_use]
    pub const fn initial_quarter(&self) -> Date {
        self.initial_quarter
    }

    /// Returns B, the lattice origin: A, or A plus three months after the cutoff.
    #[must_use]
    pub const fn recurrence_anchor(&self) -> Date {
        self.recurrence_anchor
    }

    /// Returns R, the first quarterly twentieth on or after the requested end.
    ///
    /// This is the last raw boundary, before business-day adjustment.
    #[must_use]
    pub const fn resolved_end(&self) -> Date {
        self.resolved_end
    }

    /// Returns A when the cutoff removed it from the interior, otherwise None.
    ///
    /// None when A equals the retained start, is the retained terminal, or
    /// lies at least thirty calendar days after the start.
    #[must_use]
    pub const fn omitted_quarter(&self) -> Option<Date> {
        self.omitted_quarter
    }

    /// Returns the original stored tenor, without normalizing its unit.
    #[must_use]
    pub const fn tenor(&self) -> Period {
        Period::new(self.tenor_length, self.tenor_unit)
    }
}

impl ScheduleBuilder {
    /// Builds the automatic local OldCDS schedule from quarterly-twentieth anchors.
    ///
    /// Let Q(x) be the first March/June/September/December twentieth on or
    /// after x, A = Q(start) and R = Q(end). The recurrence anchor B is A plus
    /// three months when A < R and A is fewer than thirty calendar days after
    /// start, otherwise A. Raw boundaries are start, every B + k*stride months
    /// (k >= 0) strictly before the requested end, and finally R. Thus a near
    /// terminal quarter is always retained and the terminal never moves to
    /// lengthen a short first interval. Positive stored Months/Years define
    /// the stride; no divisibility by three is required.
    ///
    /// An interval is Regular only when both raw endpoints lie on the
    /// B-anchored stride lattice exactly one stride apart; every other
    /// interval is Stub. Start and interiors use the ordinary convention, R
    /// the termination convention; collisions are rejected, never repaired.
    /// The operands carry no legal trade/effective role and no historical
    /// premium convention; see the
    /// [OldCDS contract](https://github.com/iceman951/sabiquant/blob/main/docs/M7_OLD_CDS_CONTRACT.md).
    ///
    /// Work/storage is O(final boundaries), plus existing calendar costs; the
    /// exact final count is checked against `max_boundaries` before generation.
    ///
    /// ```
    /// use sabiquant::calendar::{BusinessDayConvention as Bdc, Calendar};
    /// use sabiquant::schedule::{PeriodRegularity, ScheduleBuilder, ScheduleCalendar};
    /// use sabiquant::time::{Date, Period, TimeUnit};
    /// let schedule = ScheduleBuilder::new(Date::from_ymd(2026, 2, 19)?,
    ///     Date::from_ymd(2026, 11, 17)?, ScheduleCalendar::Plain(Calendar::null()),
    ///     Bdc::Unadjusted, Bdc::Unadjusted)?
    ///     .build_old_cds(Period::new(3, TimeUnit::Months), 4)?;
    /// // March 20 is 29 days after the start, so the first coupon is June 20.
    /// assert_eq!(schedule.contractual_dates()[1], Date::from_ymd(2026, 6, 20)?);
    /// assert_eq!(schedule.contractual_dates()[3], Date::from_ymd(2026, 12, 20)?);
    /// assert_eq!(schedule.regularity()[0], PeriodRegularity::Stub);
    /// let provenance = schedule.old_cds_provenance().ok_or("missing provenance")?;
    /// assert_eq!(provenance.omitted_quarter(), Some(Date::from_ymd(2026, 3, 20)?));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    /// Checks the cap, stored tenor domain, original TARGET span, required
    /// terminal/initial quarters and anchor, the expanded start..R span, the
    /// exact final count, fallible generation/storage (`AllocationFailed`)
    /// and then adjustment, in that order. A requested end
    /// after 9999-12-20 needs an unrepresentable terminal and fails with its
    /// Date cause instead of truncating. No partial value is returned.
    pub fn build_old_cds(
        &self,
        tenor: Period,
        max_boundaries: u32,
    ) -> Result<Schedule, ScheduleError> {
        validate_limit(max_boundaries)?;
        let stride = match tenor.unit() {
            TimeUnit::Months => i64::from(tenor.length()),
            TimeUnit::Years => 12 * i64::from(tenor.length()),
            TimeUnit::Days | TimeUnit::Weeks => 0,
        };
        if stride <= 0 {
            return Err(ScheduleError::InvalidOldCdsTenor {
                generation_start: self.start,
                requested_end: self.end,
                tenor,
            });
        }
        self.calendar.validate_span(self.start, self.end)?;
        let required_date = |role: &'static str, input: Date| {
            move |cause: DateError| ScheduleError::OldCdsDate {
                generation_start: self.start,
                requested_end: self.end,
                tenor,
                role,
                input,
                cause,
            }
        };
        // Resolve the terminal first: once R exists, start < end proves that
        // the initial quarter and its three-month successor are representable.
        let resolved_end =
            quarter_on_or_after(self.end).map_err(required_date("terminal quarter", self.end))?;
        let initial_quarter = quarter_on_or_after(self.start)
            .map_err(required_date("initial quarter", self.start))?;
        let recurrence_anchor = if initial_quarter < resolved_end
            && initial_quarter.days_since(self.start) < CUTOFF_DAYS
        {
            initial_quarter
                .checked_add_months(3)
                .map_err(required_date("recurrence anchor", initial_quarter))?
        } else {
            initial_quarter
        };
        self.calendar
            .validate_span_for(self.start, resolved_end, "old CDS coverage")?;
        // Interior twentieths lie strictly before the requested end. Count
        // them from civil month coordinates, without constructing unused dates.
        let anchor_coordinate = month_coordinate(recurrence_anchor);
        let last_coordinate = month_coordinate(self.end) - i64::from(self.end.day() <= 20);
        let interior = if anchor_coordinate > last_coordinate {
            0
        } else {
            (last_coordinate - anchor_coordinate) / stride + 1
        };
        let required = interior + 2;
        if required > i64::from(max_boundaries) {
            return Err(ScheduleError::BoundaryLimitExceeded {
                maximum: max_boundaries,
                // At most 119,987 boundaries fit the civil range.
                required_at_least: u32::try_from(required).unwrap_or(u32::MAX),
            });
        }
        let mut rolling = Rolling::new(self.start)?;
        let mut previous = self.start;
        for step in 0..interior {
            // Every twentieth exists, so month arithmetic from B never clamps.
            let point = recurrence_anchor
                .checked_add_months(step * stride)
                .map_err(required_date("stride point", recurrence_anchor))?;
            rolling.push(
                point,
                regularity(
                    previous,
                    point,
                    anchor_coordinate,
                    stride,
                    SpecialRollRule::Twentieth,
                ),
            )?;
            previous = point;
        }
        rolling.push(
            resolved_end,
            regularity(
                previous,
                resolved_end,
                anchor_coordinate,
                stride,
                SpecialRollRule::Twentieth,
            ),
        )?;
        let provenance = OldCdsProvenance {
            generation_start: self.start,
            requested_end: self.end,
            initial_quarter,
            recurrence_anchor,
            resolved_end,
            omitted_quarter: (recurrence_anchor != initial_quarter
                && initial_quarter != self.start)
                .then_some(initial_quarter),
            tenor_length: tenor.length(),
            tenor_unit: tenor.unit(),
        };
        let mut schedule = self.finish(
            rolling,
            DateGenerationRule::OldCds,
            Some(tenor),
            None,
            None,
            None,
        )?;
        schedule.old_cds_provenance = Some(provenance);
        Ok(schedule)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::{BusinessDayConvention, Calendar};
    use crate::schedule::{PeriodRegularity, ScheduleCalendar};

    fn build(start: Date, end: Date, tenor: Period) -> Result<Schedule, crate::error::Error> {
        Ok(ScheduleBuilder::new(
            start,
            end,
            ScheduleCalendar::Plain(Calendar::null()),
            BusinessDayConvention::Unadjusted,
            BusinessDayConvention::Unadjusted,
        )?
        .build_old_cds(tenor, ScheduleBuilder::MAX_BOUNDARIES)?)
    }

    #[test]
    fn cutoff_and_terminal_retention_use_civil_quarter_distances()
    -> Result<(), Box<dyn std::error::Error>> {
        let quarter = Date::from_ymd(2026, 3, 20)?;
        let end = Date::from_ymd(2026, 12, 31)?;
        let months = Period::new(3, TimeUnit::Months);
        for (distance, first_interior, omitted) in [
            (0, Date::from_ymd(2026, 6, 20)?, None),
            (1, Date::from_ymd(2026, 6, 20)?, Some(quarter)),
            (29, Date::from_ymd(2026, 6, 20)?, Some(quarter)),
            (30, quarter, None),
            (31, quarter, None),
        ] {
            let start = quarter.checked_add_days(-distance)?;
            let schedule = build(start, end, months)?;
            assert_eq!(schedule.contractual_dates()[0], start);
            assert_eq!(schedule.contractual_dates()[1], first_interior);
            assert_eq!(
                schedule.contractual_dates().last(),
                Some(&Date::from_ymd(2027, 3, 20)?)
            );
            let provenance = schedule.old_cds_provenance().ok_or("provenance")?;
            assert_eq!(provenance.omitted_quarter(), omitted);
            assert_eq!(provenance.initial_quarter(), quarter);
        }
        // A near quarter that is also the terminal is retained, never extended.
        let schedule = build(
            Date::from_ymd(2026, 3, 17)?,
            Date::from_ymd(2026, 3, 19)?,
            months,
        )?;
        assert_eq!(
            schedule.contractual_dates(),
            [Date::from_ymd(2026, 3, 17)?, quarter]
        );
        assert_eq!(schedule.regularity(), [PeriodRegularity::Stub]);
        let provenance = schedule.old_cds_provenance().ok_or("provenance")?;
        assert_eq!(provenance.recurrence_anchor(), quarter);
        assert_eq!(provenance.omitted_quarter(), None);
        Ok(())
    }

    #[test]
    fn generation_stops_at_the_requested_end_before_the_projected_terminal()
    -> Result<(), Box<dyn std::error::Error>> {
        let schedule = build(
            Date::from_ymd(2026, 2, 6)?,
            Date::from_ymd(2026, 10, 21)?,
            Period::new(1, TimeUnit::Months),
        )?;
        let raw = schedule.contractual_dates();
        assert_eq!(raw.len(), 10);
        assert_eq!(raw[8], Date::from_ymd(2026, 10, 20)?);
        // November 20 is on the stride lattice but after the requested end.
        assert_eq!(raw[9], Date::from_ymd(2026, 12, 20)?);
        assert_eq!(schedule.regularity()[8], PeriodRegularity::Stub);
        assert_eq!(schedule.regularity()[7], PeriodRegularity::Regular);
        Ok(())
    }
}
