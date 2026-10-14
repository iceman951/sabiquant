//! Explicit contractual schedules with separate business-day adjustment.
//!
//! This local tranche supports Forward, Backward, Zero and four civil rolls. Recurrence is
//! anchored to the original directional endpoint or explicitly named coupon;
//! repeated month clamping and
//! business-day advancement are not recurrence rules. Dates and regularity
//! remain contractual even when adjustment changes interval lengths.
//!
//! The [local M7 contract](https://github.com/iceman951/sabiquant/blob/main/docs/M7_LOCAL_READINESS.md)
//! and [explicit coupon contract](https://github.com/iceman951/sabiquant/blob/main/docs/M7_EXPLICIT_STUB_CONTRACT.md)
//! record scope and unresolved acceptance gates. Generated ICMA contexts use
//! the [retained contractual lattice](https://github.com/iceman951/sabiquant/blob/main/docs/M7_ICMA_REFERENCE_CONTRACT.md)
//! and an explicit matching frequency. The four civil rolls retain separate
//! requested/resolved provenance and generate references on their exact roll
//! points. CDS plans remain deferred.

use crate::calendar::{
    BusinessDayConvention, Calendar, CalendarError, EndOfMonth, EndpointInclusion, TargetCalendar,
};
use crate::daycount::{ActualActualIcma, DeterminationPeriod, IcmaError};
use crate::error::IdentifierError;
use crate::identifier::{Entry, Identifiers};
use crate::time::{Date, DateError, Frequency, Period, TimeUnit};
use std::error::Error as StdError;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

mod roll;
pub use roll::{RollProvenance, SpecialRollRule};

/// The explicitly selected contractual recurrence, Zero or civil roll rule.
///
/// Canonical text is `Forward`, `Backward`, `Zero`, or a [`SpecialRollRule`]
/// identifier. There are no aliases, case folding or whitespace trimming.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DateGenerationRule {
    /// Generate from the original start; an automatic residual is at the end.
    Forward,
    /// Generate from the original end; return chronological dates and a first residual.
    Backward,
    /// Exactly start/end, with no periodic tenor, stub or EOM configuration.
    Zero,
    /// A civil month roll, built only by [`ScheduleBuilder::build_roll`].
    Roll(SpecialRollRule),
}

impl DateGenerationRule {
    /// All currently supported rule labels in canonical order.
    pub const ALL: [Self; 7] = [
        Self::Forward,
        Self::Backward,
        Self::Zero,
        Self::Roll(SpecialRollRule::ThirdWednesday),
        Self::Roll(SpecialRollRule::ThirdWednesdayInclusive),
        Self::Roll(SpecialRollRule::Twentieth),
        Self::Roll(SpecialRollRule::TwentiethIMM),
    ];

    /// Returns the exact canonical identifier.
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::Forward => "Forward",
            Self::Backward => "Backward",
            Self::Zero => "Zero",
            Self::Roll(rule) => rule.canonical_identifier(),
        }
    }
}

const RULE_IDS: Identifiers<DateGenerationRule> = Identifiers::new(
    "date generation rule",
    &[
        Entry {
            value: DateGenerationRule::Forward,
            canonical: "Forward",
            aliases: &[],
        },
        Entry {
            value: DateGenerationRule::Backward,
            canonical: "Backward",
            aliases: &[],
        },
        Entry {
            value: DateGenerationRule::Zero,
            canonical: "Zero",
            aliases: &[],
        },
        Entry {
            value: DateGenerationRule::Roll(SpecialRollRule::ThirdWednesday),
            canonical: "ThirdWednesday",
            aliases: &[],
        },
        Entry {
            value: DateGenerationRule::Roll(SpecialRollRule::ThirdWednesdayInclusive),
            canonical: "ThirdWednesdayInclusive",
            aliases: &[],
        },
        Entry {
            value: DateGenerationRule::Roll(SpecialRollRule::Twentieth),
            canonical: "Twentieth",
            aliases: &[],
        },
        Entry {
            value: DateGenerationRule::Roll(SpecialRollRule::TwentiethIMM),
            canonical: "TwentiethIMM",
            aliases: &[],
        },
    ],
    &[],
);

impl fmt::Display for DateGenerationRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_identifier())
    }
}

impl FromStr for DateGenerationRule {
    type Err = IdentifierError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        RULE_IDS.parse(text)
    }
}

/// The policy for an automatic off-lattice first or final residual interval.
///
/// Exact canonical text is `None`, `Short` or `Long`, with no aliases or
/// trimming. An exact lattice never creates an artificial stub, even when
/// `Long` is requested.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StubPolicy {
    /// Reject a residual rather than infer a coupon interval.
    None,
    /// Retain the neighboring regular boundary and the original endpoint.
    Short,
    /// Remove the neighboring regular boundary, merging its interval with the residual.
    Long,
}

impl StubPolicy {
    /// All currently supported stub labels in canonical order.
    pub const ALL: [Self; 3] = [Self::None, Self::Short, Self::Long];

    /// Returns the exact canonical identifier.
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Short => "Short",
            Self::Long => "Long",
        }
    }
}

const STUB_IDS: Identifiers<StubPolicy> = Identifiers::new(
    "stub policy",
    &[
        Entry {
            value: StubPolicy::None,
            canonical: "None",
            aliases: &[],
        },
        Entry {
            value: StubPolicy::Short,
            canonical: "Short",
            aliases: &[],
        },
        Entry {
            value: StubPolicy::Long,
            canonical: "Long",
            aliases: &[],
        },
    ],
    &[],
);

impl fmt::Display for StubPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_identifier())
    }
}

impl FromStr for StubPolicy {
    type Err = IdentifierError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        STUB_IDS.parse(text)
    }
}

/// Contractual provenance of one adjacent boundary pair.
///
/// Canonical text is exactly `Regular`, `Stub` or `NonPeriodic`, with no
/// aliases or trimming. Business-day adjustment never changes this flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PeriodRegularity {
    /// One complete interval on the retained recurrence or civil roll lattice.
    Regular,
    /// An automatic residual, explicit outer or civil roll interval off the named lattice.
    Stub,
    /// Zero's single interval, for which no periodic lattice is claimed.
    NonPeriodic,
}

impl PeriodRegularity {
    /// All currently supported regularity labels in canonical order.
    pub const ALL: [Self; 3] = [Self::Regular, Self::Stub, Self::NonPeriodic];

    /// Returns the exact canonical identifier.
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::Regular => "Regular",
            Self::Stub => "Stub",
            Self::NonPeriodic => "NonPeriodic",
        }
    }
}

const REGULARITY_IDS: Identifiers<PeriodRegularity> = Identifiers::new(
    "period regularity",
    &[
        Entry {
            value: PeriodRegularity::Regular,
            canonical: "Regular",
            aliases: &[],
        },
        Entry {
            value: PeriodRegularity::Stub,
            canonical: "Stub",
            aliases: &[],
        },
        Entry {
            value: PeriodRegularity::NonPeriodic,
            canonical: "NonPeriodic",
            aliases: &[],
        },
    ],
    &[],
);

impl fmt::Display for PeriodRegularity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_identifier())
    }
}

impl FromStr for PeriodRegularity {
    type Err = IdentifierError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        REGULARITY_IDS.parse(text)
    }
}

/// An owned calendar, retaining any TARGET validity, coverage and joint guard.
///
/// Calendar names cannot supply the required definition or projection policy.
/// This contextual value therefore has neither text parsing nor a default.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ScheduleCalendar {
    /// An immutable custom, weekend, null or plain joint definition.
    Plain(Calendar),
    /// A guarded TARGET definition, including explicit overrides/joint members.
    Target(TargetCalendar),
}

impl ScheduleCalendar {
    fn validate_span(&self, start: Date, end: Date) -> Result<(), ScheduleError> {
        self.validate_span_for(start, end, "coverage")
    }

    fn validate_span_for(
        &self,
        start: Date,
        end: Date,
        phase: &'static str,
    ) -> Result<(), ScheduleError> {
        if let Self::Target(calendar) = self {
            calendar
                .business_days_between(
                    start,
                    end,
                    EndpointInclusion::Include,
                    EndpointInclusion::Include,
                )
                .map_err(|cause| ScheduleError::Calendar {
                    index: None,
                    phase,
                    cause,
                })?;
        }
        Ok(())
    }

    fn adjust(
        &self,
        date: Date,
        convention: BusinessDayConvention,
        index: usize,
    ) -> Result<Date, ScheduleError> {
        let result = match self {
            Self::Plain(calendar) => calendar.adjust(date, convention),
            Self::Target(calendar) => calendar.adjust(date, convention),
        };
        result.map_err(|cause| ScheduleError::Calendar {
            index: Some(index),
            phase: "adjustment",
            cause,
        })
    }
}

/// Explicit first/penultimate coupon roles, independent of a builder's range.
///
/// At least one role is present. When both are present, first cannot follow
/// penultimate. Equal roles name one interior coupon, allowing two outer
/// intervals without a regular middle interval. Each supplied date must be
/// strictly interior to the range passed to [`ScheduleBuilder::build_explicit`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExplicitStubDates {
    first: Option<Date>,
    penultimate: Option<Date>,
}

impl ExplicitStubDates {
    /// Validates presence and role ordering, without inferring a schedule range.
    ///
    /// # Errors
    /// Returns [`ScheduleError::EmptyExplicitStubDates`] for two absent roles,
    /// then [`ScheduleError::ReversedExplicitStubDates`] if first is later.
    pub fn new(first: Option<Date>, penultimate: Option<Date>) -> Result<Self, ScheduleError> {
        if first.is_none() && penultimate.is_none() {
            return Err(ScheduleError::EmptyExplicitStubDates);
        }
        if let (Some(first), Some(penultimate)) = (first, penultimate)
            && first > penultimate
        {
            return Err(ScheduleError::ReversedExplicitStubDates { first, penultimate });
        }
        Ok(Self { first, penultimate })
    }

    /// Returns the supplied first coupon date, if present.
    #[must_use]
    pub const fn first(self) -> Option<Date> {
        self.first
    }

    /// Returns the supplied penultimate coupon date, if present.
    #[must_use]
    pub const fn penultimate(self) -> Option<Date> {
        self.penultimate
    }
}

/// An immutable explicit range, calendar and pair of adjustment conventions.
///
/// Construction validates only `start < end`. Building checks options before
/// TARGET coverage, generates contractual dates, adjusts them, then rejects
/// collisions/reversals. No convention, frequency or current date is inferred.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ScheduleBuilder {
    start: Date,
    end: Date,
    calendar: ScheduleCalendar,
    convention: BusinessDayConvention,
    termination_convention: BusinessDayConvention,
}

impl ScheduleBuilder {
    /// Minimum final boundary count for every supported schedule.
    pub const MIN_BOUNDARIES: u32 = 2;

    /// Maximum final boundary count: every date in the supported civil range.
    pub const MAX_BOUNDARIES: u32 = 3_652_059;

    /// Owns the explicit inputs after validating an increasing date range.
    ///
    /// Coverage and recurrence-specific options are checked by the build
    /// operation, so a builder is not a claim that any schedule is valid.
    ///
    /// # Errors
    /// Returns [`ScheduleError::InvalidRange`] for equal or reversed endpoints.
    pub fn new(
        start: Date,
        end: Date,
        calendar: ScheduleCalendar,
        convention: BusinessDayConvention,
        termination_convention: BusinessDayConvention,
    ) -> Result<Self, ScheduleError> {
        if start >= end {
            return Err(ScheduleError::InvalidRange { start, end });
        }
        Ok(Self {
            start,
            end,
            calendar,
            convention,
            termination_convention,
        })
    }

    /// Returns the original contractual start.
    #[must_use]
    pub const fn start(&self) -> Date {
        self.start
    }

    /// Returns the original contractual end.
    #[must_use]
    pub const fn end(&self) -> Date {
        self.end
    }

    /// Borrows the retained calendar definition and any market guard.
    #[must_use]
    pub const fn calendar(&self) -> &ScheduleCalendar {
        &self.calendar
    }

    /// Returns the adjustment for start and interior boundaries.
    #[must_use]
    pub const fn convention(&self) -> BusinessDayConvention {
        self.convention
    }

    /// Returns the separate final-boundary adjustment.
    #[must_use]
    pub const fn termination_convention(&self) -> BusinessDayConvention {
        self.termination_convention
    }

    /// Builds an anchored Forward or Backward contractual lattice.
    ///
    /// The stored tenor length must be positive. Days and Weeks count calendar
    /// days; Months and Years use checked date arithmetic from the same
    /// original start/end on every step. `Preserve` is accepted only for month
    /// families, and activates only when the directional anchor is a calendar
    /// month end. It rolls generated regular candidates to calendar month end;
    /// original start/end remain unchanged. Business month end is not inferred.
    ///
    /// `None` rejects a residual; `Short` retains it; `Long` merges it with a
    /// preceding generated regular interval in that direction. Long fails
    /// when the span is shorter than one tenor. An exact lattice has only
    /// Regular intervals regardless of the retained stub policy.
    ///
    /// `max_boundaries` bounds the FINAL count, including endpoints. Long
    /// removes its neighboring regular boundary before adding the endpoint,
    /// so it may fit where Short would not. Generation terminates after at
    /// most O(final boundaries) steps plus the declared calendar costs.
    /// Schedule-vector growth uses fallible reservation and O(boundaries)
    /// storage; cloning an existing calendar follows its standard Clone
    /// allocation semantics. No partially generated schedule is returned.
    ///
    /// ```
    /// use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth};
    /// use sabiquant::schedule::{DateGenerationRule, ScheduleBuilder, ScheduleCalendar, StubPolicy};
    /// use sabiquant::time::{Date, Period, TimeUnit};
    /// let builder = ScheduleBuilder::new(Date::from_ymd(2024, 1, 31)?,
    ///     Date::from_ymd(2024, 4, 30)?, ScheduleCalendar::Plain(Calendar::null()),
    ///     BusinessDayConvention::Unadjusted, BusinessDayConvention::Unadjusted)?;
    /// let schedule = builder.build_recurrent(Period::new(1, TimeUnit::Months),
    ///     DateGenerationRule::Forward, StubPolicy::None, EndOfMonth::Preserve, 4)?;
    /// assert_eq!(schedule.contractual_dates()[2], Date::from_ymd(2024, 3, 31)?);
    /// assert_eq!(schedule.contractual_dates(), schedule.adjusted_dates());
    /// # Ok::<(), sabiquant::error::Error>(())
    /// ```
    ///
    /// # Errors
    /// Checks the limit, rule, tenor and EOM compatibility in that order, then
    /// validates the whole TARGET span (including omitted holidays) before
    /// generation. Reports residual policy, final-count or allocation errors,
    /// checked date/calendar causes and adjusted-order conflicts. Unused
    /// candidates provably beyond the supported civil range terminate the
    /// lattice normally; unrelated date errors are preserved.
    pub fn build_recurrent(
        &self,
        tenor: Period,
        rule: DateGenerationRule,
        stub: StubPolicy,
        eom: EndOfMonth,
        max_boundaries: u32,
    ) -> Result<Schedule, ScheduleError> {
        validate_limit(max_boundaries)?;
        let forward = match rule {
            DateGenerationRule::Forward => true,
            DateGenerationRule::Backward => false,
            DateGenerationRule::Zero | DateGenerationRule::Roll(_) => {
                return Err(ScheduleError::InvalidRule { rule });
            }
        };
        if tenor.length() <= 0 {
            return Err(ScheduleError::InvalidTenor { tenor });
        }
        if eom == EndOfMonth::Preserve && matches!(tenor.unit(), TimeUnit::Days | TimeUnit::Weeks) {
            return Err(ScheduleError::IncompatibleEndOfMonth {
                tenor,
                end_of_month: eom,
            });
        }
        self.calendar.validate_span(self.start, self.end)?;
        let (anchor, endpoint) = if forward {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        };
        let preserve = eom == EndOfMonth::Preserve && anchor.is_end_of_month();
        let mut rolling = Rolling::new(anchor)?;
        loop {
            // Before residual handling, count is both the next recurrence step
            // and the current number of stored boundaries. It cannot exceed
            // MAX_BOUNDARIES, making all i64 scaled-tenor products safe.
            let candidate = anchored_candidate(
                anchor,
                tenor,
                rolling.count,
                forward,
                preserve,
                rolling.dates.len(),
            )?;
            if let Some(date) = candidate
                && if forward {
                    date < endpoint
                } else {
                    date > endpoint
                }
            {
                let required = rolling.count + if stub == StubPolicy::Long { 1 } else { 2 };
                if required > max_boundaries {
                    return Err(ScheduleError::BoundaryLimitExceeded {
                        maximum: max_boundaries,
                        required_at_least: required,
                    });
                }
                rolling.push(date, PeriodRegularity::Regular)?;
                continue;
            }
            if candidate == Some(endpoint) {
                rolling.push_limited(endpoint, PeriodRegularity::Regular, max_boundaries)?;
            } else {
                match stub {
                    StubPolicy::None => {
                        return Err(ScheduleError::StubRequired {
                            rule,
                            tenor,
                            start: self.start,
                            end: self.end,
                            regular_boundary: rolling.dates.last().copied().unwrap_or(anchor),
                        });
                    }
                    StubPolicy::Short => {
                        rolling.push_limited(endpoint, PeriodRegularity::Stub, max_boundaries)?
                    }
                    StubPolicy::Long => {
                        if rolling.regularity.is_empty() {
                            return Err(ScheduleError::LongStubWithoutRegularPeriod {
                                rule,
                                tenor,
                                start: self.start,
                                end: self.end,
                            });
                        }
                        // The final-count cap is applied after removing the
                        // neighboring regular boundary, never to a Short draft.
                        rolling.dates.pop();
                        rolling.regularity.pop();
                        rolling.count -= 1;
                        rolling.push_limited(endpoint, PeriodRegularity::Stub, max_boundaries)?;
                    }
                }
            }
            break;
        }
        if !forward {
            rolling.dates.reverse();
            rolling.regularity.reverse();
        }
        self.finish(rolling, rule, Some(tenor), Some(stub), Some(eom), None)
    }

    /// Builds a regular middle with explicitly named first/penultimate coupons.
    ///
    /// Forward anchors at the supplied first date, otherwise start. Backward
    /// anchors at the supplied penultimate date, otherwise end. The opposite
    /// middle endpoint MUST lie on that same original-anchor lattice; no
    /// additional implicit stub is generated. Equal first/penultimate roles
    /// produce one interior boundary and no middle interval.
    ///
    /// Calendar EOM preservation activates from this middle anchor, and never
    /// rewrites the supplied coupon dates or range endpoints. Each outer
    /// interval is Regular only if its endpoint equals the adjacent boundary
    /// of this SAME lattice; otherwise it is Stub. A clamped date is never
    /// reused as a new seed. Thus Forward with first February 28 and start
    /// January 31 is Stub under Ignore: February 28 minus one month is January
    /// 28. This records named lattice provenance, not general equivalence of
    /// date conventions. An out-of-range classification-only adjacent date
    /// cannot match the in-range endpoint, so that interval is Stub.
    ///
    /// `max_boundaries` counts ALL final boundaries including explicit dates
    /// and range endpoints. Generation uses O(boundaries) steps/storage and
    /// the same fallible reservations as recurrent construction. Adjustment
    /// retains one boundary per contractual date and rejects collisions.
    /// The result retains the supplied explicit roles, tenor, rule and EOM;
    /// [`Schedule::stub_policy`] is None because no automatic policy applies.
    ///
    /// ```
    /// use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth};
    /// use sabiquant::schedule::{DateGenerationRule, ExplicitStubDates,
    ///     PeriodRegularity, ScheduleBuilder, ScheduleCalendar};
    /// use sabiquant::time::{Date, Period, TimeUnit};
    /// let builder = ScheduleBuilder::new(Date::from_ymd(2026, 1, 15)?,
    ///     Date::from_ymd(2026, 11, 15)?, ScheduleCalendar::Plain(Calendar::null()),
    ///     BusinessDayConvention::Unadjusted, BusinessDayConvention::Unadjusted)?;
    /// let explicit = ExplicitStubDates::new(Some(Date::from_ymd(2026, 1, 31)?),
    ///     Some(Date::from_ymd(2026, 10, 31)?))?;
    /// let schedule = builder.build_explicit(Period::new(3, TimeUnit::Months),
    ///     DateGenerationRule::Forward, explicit, EndOfMonth::Ignore, 6)?;
    /// assert_eq!(schedule.regularity(), &[PeriodRegularity::Stub,
    ///     PeriodRegularity::Regular, PeriodRegularity::Regular,
    ///     PeriodRegularity::Regular, PeriodRegularity::Stub]);
    /// assert_eq!(schedule.explicit_stub_dates(), Some(explicit));
    /// # Ok::<(), sabiquant::error::Error>(())
    /// ```
    ///
    /// # Errors
    /// Checks the limit, rule, tenor, EOM compatibility, first then penultimate
    /// strictly-interior roles, then whole original TARGET-span coverage.
    /// Reports off-lattice middle endpoints with contextual metadata, final
    /// cap/allocation errors, checked causes and adjusted-order conflicts.
    pub fn build_explicit(
        &self,
        tenor: Period,
        rule: DateGenerationRule,
        dates: ExplicitStubDates,
        eom: EndOfMonth,
        max_boundaries: u32,
    ) -> Result<Schedule, ScheduleError> {
        validate_limit(max_boundaries)?;
        let forward = match rule {
            DateGenerationRule::Forward => true,
            DateGenerationRule::Backward => false,
            DateGenerationRule::Zero | DateGenerationRule::Roll(_) => {
                return Err(ScheduleError::InvalidRule { rule });
            }
        };
        if tenor.length() <= 0 {
            return Err(ScheduleError::InvalidTenor { tenor });
        }
        if eom == EndOfMonth::Preserve && matches!(tenor.unit(), TimeUnit::Days | TimeUnit::Weeks) {
            return Err(ScheduleError::IncompatibleEndOfMonth {
                tenor,
                end_of_month: eom,
            });
        }
        for (role, date) in [("first", dates.first), ("penultimate", dates.penultimate)] {
            if let Some(date) = date
                && (date <= self.start || date >= self.end)
            {
                return Err(ScheduleError::ExplicitStubOutsideRange {
                    role,
                    date,
                    start: self.start,
                    end: self.end,
                });
            }
        }
        self.calendar.validate_span(self.start, self.end)?;
        let regular_start = dates.first.unwrap_or(self.start);
        let regular_end = dates.penultimate.unwrap_or(self.end);
        let (anchor, endpoint, leading, trailing) = if forward {
            (
                regular_start,
                regular_end,
                dates.first.map(|_| self.start),
                dates.penultimate.map(|_| self.end),
            )
        } else {
            (
                regular_end,
                regular_start,
                dates.penultimate.map(|_| self.end),
                dates.first.map(|_| self.start),
            )
        };
        let preserve = eom == EndOfMonth::Preserve && anchor.is_end_of_month();
        let extra = u32::from(leading.is_some()) + u32::from(trailing.is_some());
        let required = extra + if anchor == endpoint { 1 } else { 2 };
        if required > max_boundaries {
            return Err(ScheduleError::BoundaryLimitExceeded {
                maximum: max_boundaries,
                required_at_least: required,
            });
        }
        let mut rolling = Rolling::new(leading.unwrap_or(anchor))?;
        if let Some(outer) = leading {
            let adjacent = anchored_candidate(anchor, tenor, 1, !forward, preserve, 0)?;
            rolling.push(anchor, outer_regularity(adjacent, outer))?;
        }
        // Unlike automatic construction, the count also contains any leading
        // endpoint. Keep the recurrence step independent of vector length.
        let mut step = 0;
        if anchor != endpoint {
            loop {
                step += 1;
                let candidate = anchored_candidate(
                    anchor,
                    tenor,
                    step,
                    forward,
                    preserve,
                    rolling.dates.len(),
                )?;
                if candidate == Some(endpoint) {
                    // The initial minimum or the preceding strict-interior check
                    // already reserves this endpoint and any trailing boundary.
                    rolling.push(endpoint, PeriodRegularity::Regular)?;
                    break;
                }
                if let Some(date) = candidate
                    && if forward {
                        date < endpoint
                    } else {
                        date > endpoint
                    }
                {
                    // This interior and the middle endpoint must still be stored,
                    // as must the outer trailing boundary if supplied.
                    let required = rolling.count + 2 + u32::from(trailing.is_some());
                    if required > max_boundaries {
                        return Err(ScheduleError::BoundaryLimitExceeded {
                            maximum: max_boundaries,
                            required_at_least: required,
                        });
                    }
                    rolling.push(date, PeriodRegularity::Regular)?;
                } else {
                    return Err(ScheduleError::IrregularExplicitMiddle {
                        rule,
                        tenor,
                        anchor,
                        endpoint,
                        regular_boundary: rolling.dates.last().copied().unwrap_or(anchor),
                    });
                }
            }
        }
        if let Some(outer) = trailing {
            let adjacent = anchored_candidate(
                anchor,
                tenor,
                step + 1,
                forward,
                preserve,
                rolling.dates.len(),
            )?;
            rolling.push_limited(outer, outer_regularity(adjacent, outer), max_boundaries)?;
        }
        if !forward {
            rolling.dates.reverse();
            rolling.regularity.reverse();
        }
        self.finish(rolling, rule, Some(tenor), None, Some(eom), Some(dates))
    }

    /// Builds exactly the original start/end pair and one NonPeriodic interval.
    ///
    /// There is no artificial tenor, stub or EOM metadata. Both endpoints and
    /// the entire raw TARGET span are still guarded, including Unadjusted and
    /// joint calendars. Adjustment conventions remain explicit and separate.
    ///
    /// ```
    /// use sabiquant::calendar::{BusinessDayConvention, Calendar};
    /// use sabiquant::schedule::{PeriodRegularity, ScheduleBuilder, ScheduleCalendar};
    /// use sabiquant::time::Date;
    /// let builder = ScheduleBuilder::new(Date::from_ymd(2026, 1, 2)?,
    ///     Date::from_ymd(2026, 2, 2)?, ScheduleCalendar::Plain(Calendar::weekends_only()),
    ///     BusinessDayConvention::Following, BusinessDayConvention::Following)?;
    /// let schedule = builder.build_zero(2)?;
    /// assert_eq!(schedule.regularity(), &[PeriodRegularity::NonPeriodic]);
    /// assert_eq!(schedule.tenor(), None);
    /// # Ok::<(), sabiquant::error::Error>(())
    /// ```
    ///
    /// # Errors
    /// Returns a limit error before coverage, then typed coverage/adjustment,
    /// allocation or adjusted-order errors. No partial value is returned.
    pub fn build_zero(&self, max_boundaries: u32) -> Result<Schedule, ScheduleError> {
        validate_limit(max_boundaries)?;
        self.calendar.validate_span(self.start, self.end)?;
        let mut rolling = Rolling::new(self.start)?;
        rolling.push_limited(self.end, PeriodRegularity::NonPeriodic, max_boundaries)?;
        self.finish(rolling, DateGenerationRule::Zero, None, None, None, None)
    }

    fn finish(
        &self,
        rolling: Rolling,
        rule: DateGenerationRule,
        tenor: Option<Period>,
        stub: Option<StubPolicy>,
        eom: Option<EndOfMonth>,
        explicit: Option<ExplicitStubDates>,
    ) -> Result<Schedule, ScheduleError> {
        let mut adjusted = Vec::new();
        adjusted
            .try_reserve_exact(rolling.dates.len())
            .map_err(|_| ScheduleError::AllocationFailed {
                phase: "adjusted boundaries",
                requested: rolling.dates.len(),
            })?;
        for (index, &date) in rolling.dates.iter().enumerate() {
            let convention = if index + 1 == rolling.dates.len() {
                self.termination_convention
            } else {
                self.convention
            };
            let value = self.calendar.adjust(date, convention, index)?;
            if let Some(&previous_adjusted) = adjusted.last()
                && value <= previous_adjusted
            {
                return Err(ScheduleError::AdjustedOrder {
                    index,
                    previous_contractual: rolling.dates[index - 1],
                    contractual: date,
                    previous_adjusted,
                    adjusted: value,
                });
            }
            adjusted.push(value);
        }
        Ok(Schedule {
            contractual: rolling.dates,
            adjusted,
            regularity: rolling.regularity,
            generation_rule: rule,
            tenor,
            stub_policy: stub,
            end_of_month: eom,
            explicit_stub_dates: explicit,
            roll_provenance: None,
            calendar: self.calendar.clone(),
            convention: self.convention,
            termination_convention: self.termination_convention,
        })
    }
}

/// A fully built chronological contractual/adjusted schedule with provenance.
///
/// Both date slices have equal length, at least two, with strictly increasing
/// dates; regularity has exactly one fewer element. No adjustment collision
/// is repaired by sorting or deduplication. Metadata remains the supplied
/// configuration, including an inactive Preserve or an unused Long policy.
///
/// Equality and hashing compare the defined vectors and retained metadata,
/// including the tenor's STORED length and unit. Therefore 12M and 1Y can
/// generate identical dates and still define unequal schedules, even though
/// their Period values compare equal. This is definition identity, not general
/// financial equivalence; clone and inspection introduce no mutable cache.
#[derive(Clone, Debug)]
pub struct Schedule {
    contractual: Vec<Date>,
    adjusted: Vec<Date>,
    regularity: Vec<PeriodRegularity>,
    generation_rule: DateGenerationRule,
    tenor: Option<Period>,
    stub_policy: Option<StubPolicy>,
    end_of_month: Option<EndOfMonth>,
    explicit_stub_dates: Option<ExplicitStubDates>,
    roll_provenance: Option<RollProvenance>,
    calendar: ScheduleCalendar,
    convention: BusinessDayConvention,
    termination_convention: BusinessDayConvention,
}

impl Schedule {
    /// Minimum supported final determination-period count.
    pub const MIN_ICMA_PERIODS: u32 = 1;

    /// Maximum positive one-day determination periods in the civil date range.
    pub const MAX_ICMA_PERIODS: u32 = 3_652_058;

    /// Builds the minimal same-anchor ICMA reference hull over raw accrual dates.
    ///
    /// The frequency is explicit and periodic, and its period must compare
    /// normalized-equal to the retained tenor. Thus 12M/1Y or 7D/1W may match;
    /// no frequency is inferred from a stub. The declared event counts for
    /// day/week frequencies remain 365/52/26/13, even in leap years. This is
    /// an analytical contractual definition, not approval of every market roll.
    ///
    /// References use the original named lattice anchor and calendar-EOM
    /// policy, or the retained civil roll's exact Wednesday/twentieth points.
    /// A roll's requested endpoints do not replace its resolved raw endpoints.
    /// The greatest lattice date no later than raw start and the least
    /// lattice date no earlier than raw end bound the MINIMAL contiguous hull.
    /// Exact endpoints need no extra period. Long/equal-role outer spans may
    /// require multiple quasi-periods, including dates absent from coupons.
    /// Adjusted payment dates and business-day conventions do not alter them.
    ///
    /// Every required extension uses checked civil arithmetic; an out-of-range
    /// quasi-date is an error. The entire extended TARGET span is validated
    /// under the owned strict/projected policy before returning the immutable
    /// context. Its subsequent queries use only supplied references and their
    /// coverage; they do not query a calendar or grant new market coverage.
    ///
    /// `max_periods` bounds ALL final positive reference intervals, shared by
    /// both extension directions. Work/storage is O(reference periods), plus
    /// the existing market-span validation cost. Boundary/period vectors use
    /// fallible reservation. No partial context or mutable cache is returned.
    ///
    /// ```
    /// use sabiquant::calendar::{BusinessDayConvention, Calendar, EndOfMonth};
    /// use sabiquant::schedule::{DateGenerationRule, ScheduleBuilder,
    ///     ScheduleCalendar, StubPolicy};
    /// use sabiquant::time::{Date, Frequency, Period, TimeUnit};
    /// let start = Date::from_ymd(1999, 2, 1)?;
    /// let end = Date::from_ymd(1999, 7, 1)?;
    /// let schedule = ScheduleBuilder::new(start, end,
    ///     ScheduleCalendar::Plain(Calendar::null()),
    ///     BusinessDayConvention::Unadjusted, BusinessDayConvention::Unadjusted)?
    ///     .build_recurrent(Period::new(1, TimeUnit::Years),
    ///         DateGenerationRule::Backward, StubPolicy::Short, EndOfMonth::Ignore, 2)?;
    /// let icma = schedule.build_icma_context(Frequency::Annual, 1)?;
    /// assert_eq!(icma.coverage_start(), Date::from_ymd(1998, 7, 1)?);
    /// assert_eq!(icma.year_fraction(start, end)?.to_bits(), (150.0_f64 / 365.0).to_bits());
    /// # Ok::<(), sabiquant::error::Error>(())
    /// ```
    ///
    /// Civil rolls recompute each month's roll date instead of carrying the
    /// anchor's day of month into subsequent references:
    ///
    /// ```
    /// use sabiquant::calendar::{BusinessDayConvention as Bdc, Calendar};
    /// use sabiquant::schedule::{ScheduleBuilder, ScheduleCalendar, SpecialRollRule};
    /// use sabiquant::time::{Date, Frequency, Period, TimeUnit};
    /// let schedule = ScheduleBuilder::new(Date::from_ymd(2024, 1, 16)?,
    ///     Date::from_ymd(2024, 1, 18)?, ScheduleCalendar::Plain(Calendar::null()),
    ///     Bdc::Unadjusted, Bdc::Unadjusted)?
    ///     .build_roll(Period::new(1, TimeUnit::Months), SpecialRollRule::ThirdWednesday, 2)?;
    /// let icma = schedule.build_icma_context(Frequency::Monthly, 2)?;
    /// assert_eq!(icma.coverage_start(), Date::from_ymd(2023, 12, 20)?);
    /// assert_eq!(icma.periods()[0].end(), Date::from_ymd(2024, 1, 17)?);
    /// assert_eq!(icma.coverage_end(), Date::from_ymd(2024, 2, 21)?);
    /// # Ok::<(), sabiquant::error::Error>(())
    /// ```
    ///
    /// # Errors
    /// Checks the cap, periodic schedule, frequency and tenor match in order.
    /// A proved cap insufficiency precedes computing the disallowed required
    /// date. Then reports checked Date causes, extended Calendar coverage,
    /// allocation failures or preserved ICMA constructor causes. Zero and Once
    /// are rejected. Required roll extensions retain the exact roll anchor
    /// and signed month offset in their Date cause, including stored Years.
    /// No unused-candidate absence hides a required quasi-date.
    pub fn build_icma_context(
        &self,
        frequency: Frequency,
        max_periods: u32,
    ) -> Result<ActualActualIcma, ScheduleError> {
        if !(Self::MIN_ICMA_PERIODS..=Self::MAX_ICMA_PERIODS).contains(&max_periods) {
            return Err(ScheduleError::InvalidIcmaPeriodLimit {
                requested: max_periods,
                minimum: Self::MIN_ICMA_PERIODS,
                maximum: Self::MAX_ICMA_PERIODS,
            });
        }
        let lattice = match self.generation_rule {
            DateGenerationRule::Forward | DateGenerationRule::Backward => self
                .end_of_month
                .zip(self.lattice_anchor())
                .map(|(eom, anchor)| IcmaLattice::Anchored {
                    anchor,
                    preserve: eom == EndOfMonth::Preserve && anchor.is_end_of_month(),
                }),
            DateGenerationRule::Roll(_) => self.roll_provenance.map(IcmaLattice::Roll),
            DateGenerationRule::Zero => None,
        };
        let (Some(tenor), Some(lattice)) = (self.tenor, lattice) else {
            return Err(ScheduleError::NonPeriodicIcma {
                rule: self.generation_rule,
            });
        };
        let Some(expected_tenor) = frequency.period() else {
            return Err(ScheduleError::InvalidIcmaFrequency { frequency });
        };
        if tenor != expected_tenor {
            return Err(ScheduleError::IcmaFrequencyMismatch {
                tenor,
                frequency,
                expected_tenor,
            });
        }
        // Private construction guarantees chronological nonempty raw slices.
        let (Some(&start), Some(&end)) = (self.contractual.first(), self.contractual.last()) else {
            return Err(ScheduleError::NonPeriodicIcma {
                rule: self.generation_rule,
            });
        };
        let anchor = lattice.anchor()?;
        // Each supported definition places q(0) inside its minimal hull.
        // Roll starts share the Wednesday anchor month or precede the first
        // eligible twentieth by less than one stride. Their raw end can lie
        // before q(0), so only anchor < end proves an upper period is needed.
        let remaining_upper = u32::from(anchor < end);
        let mut count = 0;
        let mut boundaries = Vec::new();
        reserve_one(&mut boundaries, "ICMA reference boundaries")?;
        boundaries.push(anchor);
        let mut lower = anchor;
        let mut step = 0;
        while lower > start {
            check_icma_count(count + 1 + remaining_upper, max_periods)?;
            step += 1;
            lower = lattice.date(anchor, tenor, step, false, boundaries.len())?;
            reserve_one(&mut boundaries, "ICMA reference boundaries")?;
            boundaries.push(lower);
            count += 1;
        }
        boundaries.reverse();
        let mut upper = anchor;
        step = 0;
        while upper < end {
            check_icma_count(count + 1, max_periods)?;
            step += 1;
            upper = lattice.date(anchor, tenor, step, true, boundaries.len())?;
            reserve_one(&mut boundaries, "ICMA reference boundaries")?;
            boundaries.push(upper);
            count += 1;
        }
        self.calendar
            .validate_span_for(lower, upper, "ICMA reference coverage")?;
        let mut periods = Vec::new();
        periods
            .try_reserve_exact(boundaries.len() - 1)
            .map_err(|_| ScheduleError::AllocationFailed {
                phase: "ICMA determination periods",
                requested: boundaries.len() - 1,
            })?;
        for pair in boundaries.windows(2) {
            let period = DeterminationPeriod::new(pair[0], pair[1])
                .map_err(|cause| ScheduleError::Icma { cause })?;
            periods.push(period);
        }
        ActualActualIcma::new(periods, frequency).map_err(|cause| ScheduleError::Icma { cause })
    }

    /// Borrows chronological raw contractual boundaries, including endpoints.
    #[must_use]
    pub fn contractual_dates(&self) -> &[Date] {
        &self.contractual
    }

    /// Borrows one adjusted date per contractual boundary, in the same order.
    #[must_use]
    pub fn adjusted_dates(&self) -> &[Date] {
        &self.adjusted
    }

    /// Borrows contractual interval provenance; adjustment never changes it.
    #[must_use]
    pub fn regularity(&self) -> &[PeriodRegularity] {
        &self.regularity
    }

    /// Returns the selected rule; Zero is explicit, not missing metadata.
    #[must_use]
    pub const fn generation_rule(&self) -> DateGenerationRule {
        self.generation_rule
    }

    /// Returns the supplied stored tenor, or None for Zero.
    #[must_use]
    pub const fn tenor(&self) -> Option<Period> {
        self.tenor
    }

    /// Returns the supplied automatic residual policy, or None for explicit/Zero/roll.
    #[must_use]
    pub const fn stub_policy(&self) -> Option<StubPolicy> {
        self.stub_policy
    }

    /// Returns the supplied calendar EOM policy, or None for Zero/roll.
    #[must_use]
    pub const fn end_of_month(&self) -> Option<EndOfMonth> {
        self.end_of_month
    }

    /// Returns the supplied explicit coupon roles, or None for automatic/Zero/roll.
    #[must_use]
    pub const fn explicit_stub_dates(&self) -> Option<ExplicitStubDates> {
        self.explicit_stub_dates
    }

    /// Borrows civil roll provenance, or None for recurrent/explicit/Zero.
    #[must_use]
    pub const fn roll_provenance(&self) -> Option<&RollProvenance> {
        self.roll_provenance.as_ref()
    }

    /// Returns the original clamped-date lattice seed, or None for Zero/roll.
    ///
    /// Forward uses explicit first, otherwise original start. Backward uses
    /// explicit penultimate, otherwise original end. Adjustment does not alter
    /// this seed. The slices are privately guaranteed nonempty.
    #[must_use]
    pub fn lattice_anchor(&self) -> Option<Date> {
        match self.generation_rule {
            DateGenerationRule::Forward => self
                .explicit_stub_dates
                .and_then(ExplicitStubDates::first)
                .or_else(|| self.contractual.first().copied()),
            DateGenerationRule::Backward => self
                .explicit_stub_dates
                .and_then(ExplicitStubDates::penultimate)
                .or_else(|| self.contractual.last().copied()),
            DateGenerationRule::Zero | DateGenerationRule::Roll(_) => None,
        }
    }

    /// Borrows the owned calendar definition and retained market guard.
    #[must_use]
    pub const fn calendar(&self) -> &ScheduleCalendar {
        &self.calendar
    }

    /// Returns the adjustment for start/interior boundaries.
    #[must_use]
    pub const fn convention(&self) -> BusinessDayConvention {
        self.convention
    }

    /// Returns the distinct adjustment applied only to the last boundary.
    #[must_use]
    pub const fn termination_convention(&self) -> BusinessDayConvention {
        self.termination_convention
    }
}

impl PartialEq for Schedule {
    fn eq(&self, other: &Self) -> bool {
        self.contractual == other.contractual
            && self.adjusted == other.adjusted
            && self.regularity == other.regularity
            && self.generation_rule == other.generation_rule
            && stored_tenor(self.tenor) == stored_tenor(other.tenor)
            && self.stub_policy == other.stub_policy
            && self.end_of_month == other.end_of_month
            && self.explicit_stub_dates == other.explicit_stub_dates
            && self.roll_provenance == other.roll_provenance
            && self.calendar == other.calendar
            && self.convention == other.convention
            && self.termination_convention == other.termination_convention
    }
}

impl Eq for Schedule {}

impl Hash for Schedule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.contractual.hash(state);
        self.adjusted.hash(state);
        self.regularity.hash(state);
        self.generation_rule.hash(state);
        stored_tenor(self.tenor).hash(state);
        self.stub_policy.hash(state);
        self.end_of_month.hash(state);
        self.explicit_stub_dates.hash(state);
        self.roll_provenance.hash(state);
        self.calendar.hash(state);
        self.convention.hash(state);
        self.termination_convention.hash(state);
    }
}

fn stored_tenor(tenor: Option<Period>) -> Option<(i32, TimeUnit)> {
    tenor.map(|value| (value.length(), value.unit()))
}

/// A rejected schedule configuration, resource request or generated boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScheduleError {
    /// A stored tenor is incompatible with a roll; `schedule.invalid_roll_tenor`.
    #[non_exhaustive]
    InvalidRollTenor {
        /// Requested civil rule.
        rule: SpecialRollRule,
        /// Original stored tenor.
        tenor: Period,
    },
    /// Projected endpoints fail to increase; `schedule.resolved_roll_range`.
    #[non_exhaustive]
    ResolvedRollRange {
        /// Requested civil rule.
        rule: SpecialRollRule,
        /// Original stored tenor.
        tenor: Period,
        /// Original start.
        requested_start: Date,
        /// Original end.
        requested_end: Date,
        /// Projected contractual start.
        resolved_start: Date,
        /// Projected contractual end.
        resolved_end: Date,
    },
    /// A required roll date is unavailable; `schedule.roll_date`.
    #[non_exhaustive]
    RollDate {
        /// Requested civil rule.
        rule: SpecialRollRule,
        /// Original stored tenor.
        tenor: Period,
        /// Required date role, such as `anchor`, `start` or `end`.
        role: &'static str,
        /// Date from which the required roll was resolved.
        input: Date,
        /// Preserved checked civil-date cause.
        cause: DateError,
    },
    /// Equal or reversed endpoints; `schedule.invalid_range`.
    #[non_exhaustive]
    InvalidRange {
        /// Requested original start.
        start: Date,
        /// Requested original end.
        end: Date,
    },
    /// An unsupported final-boundary cap; `schedule.invalid_boundary_limit`.
    #[non_exhaustive]
    InvalidBoundaryLimit {
        /// Requested cap.
        requested: u32,
        /// Minimum supported cap.
        minimum: u32,
        /// Maximum supported cap.
        maximum: u32,
    },
    /// Zero/roll passed to recurrent or explicit generation; `schedule.invalid_rule`.
    #[non_exhaustive]
    InvalidRule {
        /// Rejected rule; use build_zero for Zero.
        rule: DateGenerationRule,
    },
    /// A stored tenor length is nonpositive; `schedule.invalid_tenor`.
    #[non_exhaustive]
    InvalidTenor {
        /// Rejected period, preserving its stored representation.
        tenor: Period,
    },
    /// Preserve used for Days/Weeks; `schedule.incompatible_end_of_month`.
    #[non_exhaustive]
    IncompatibleEndOfMonth {
        /// Rejected recurrence period.
        tenor: Period,
        /// Rejected EOM option.
        end_of_month: EndOfMonth,
    },
    /// Neither coupon role is supplied; `schedule.empty_explicit_stub_dates`.
    EmptyExplicitStubDates,
    /// First follows penultimate; `schedule.reversed_explicit_stub_dates`.
    #[non_exhaustive]
    ReversedExplicitStubDates {
        /// Rejected first coupon date.
        first: Date,
        /// Rejected penultimate coupon date.
        penultimate: Date,
    },
    /// A coupon is not strictly interior; `schedule.explicit_stub_outside_range`.
    #[non_exhaustive]
    ExplicitStubOutsideRange {
        /// First or penultimate role.
        role: &'static str,
        /// Rejected coupon date.
        date: Date,
        /// Original range start.
        start: Date,
        /// Original range end.
        end: Date,
    },
    /// Middle endpoint is off the named lattice; `schedule.irregular_explicit_middle`.
    #[non_exhaustive]
    IrregularExplicitMiddle {
        /// Generation direction.
        rule: DateGenerationRule,
        /// Supplied stored tenor.
        tenor: Period,
        /// Original directional regular seed.
        anchor: Date,
        /// Opposite middle endpoint.
        endpoint: Date,
        /// Last generated regular boundary in the directional sequence.
        regular_boundary: Date,
    },
    /// Unsupported reference-period cap; `schedule.invalid_icma_period_limit`.
    #[non_exhaustive]
    InvalidIcmaPeriodLimit {
        /// Requested final positive-period cap.
        requested: u32,
        /// Minimum supported count.
        minimum: u32,
        /// Maximum supported count.
        maximum: u32,
    },
    /// A schedule has no recurring lattice; `schedule.non_periodic_icma`.
    #[non_exhaustive]
    NonPeriodicIcma {
        /// Rejected schedule rule, currently Zero.
        rule: DateGenerationRule,
    },
    /// Frequency is not periodic; `schedule.invalid_icma_frequency`.
    #[non_exhaustive]
    InvalidIcmaFrequency {
        /// Rejected named frequency, currently Once.
        frequency: Frequency,
    },
    /// Tenor does not match the explicit frequency; `schedule.icma_frequency_mismatch`.
    #[non_exhaustive]
    IcmaFrequencyMismatch {
        /// Stored schedule tenor.
        tenor: Period,
        /// Supplied annual frequency.
        frequency: Frequency,
        /// Period defined by that frequency.
        expected_tenor: Period,
    },
    /// Required references cannot fit the cap; `schedule.icma_period_limit_exceeded`.
    #[non_exhaustive]
    IcmaPeriodLimitExceeded {
        /// Original caller-supplied final-period cap.
        maximum: u32,
        /// Proved minimum required final-period count.
        required_at_least: u32,
    },
    /// A generated period/context failed validation; `schedule.icma`.
    #[non_exhaustive]
    Icma {
        /// Preserved typed ICMA constructor cause.
        cause: IcmaError,
    },
    /// An off-lattice endpoint requires a permitted residual; `schedule.stub_required`.
    #[non_exhaustive]
    StubRequired {
        /// Generation direction.
        rule: DateGenerationRule,
        /// Requested tenor.
        tenor: Period,
        /// Original start.
        start: Date,
        /// Original end.
        end: Date,
        /// Last generated boundary in the directional sequence.
        regular_boundary: Date,
    },
    /// Long has no complete neighboring regular interval; `schedule.long_stub_without_regular_period`.
    #[non_exhaustive]
    LongStubWithoutRegularPeriod {
        /// Generation direction.
        rule: DateGenerationRule,
        /// Requested tenor.
        tenor: Period,
        /// Original start.
        start: Date,
        /// Original end.
        end: Date,
    },
    /// Final boundaries cannot fit the supplied cap; `schedule.boundary_limit_exceeded`.
    #[non_exhaustive]
    BoundaryLimitExceeded {
        /// Supplied final cap.
        maximum: u32,
        /// A proved lower bound on required final boundaries.
        required_at_least: u32,
    },
    /// A schedule vector could not reserve memory; `schedule.allocation_failed`.
    #[non_exhaustive]
    AllocationFailed {
        /// Vector being grown.
        phase: &'static str,
        /// Requested element count; no partially usable schedule is returned.
        requested: usize,
    },
    /// Unexpected required date arithmetic failure; `schedule.date`.
    #[non_exhaustive]
    Date {
        /// Boundary-generation position before final chronological ordering.
        index: usize,
        /// Generation, ICMA anchor reconstruction or required reference extension.
        phase: &'static str,
        /// Preserved date cause.
        cause: DateError,
    },
    /// Market coverage or adjustment failure; `schedule.calendar`.
    #[non_exhaustive]
    Calendar {
        /// None for full-span coverage, otherwise the chronological boundary index.
        index: Option<usize>,
        /// Coverage or adjustment phase.
        phase: &'static str,
        /// Preserved calendar cause, including any nested date or TARGET failure.
        cause: CalendarError,
    },
    /// Adjustment collapsed or reversed adjacent dates; `schedule.adjusted_order`.
    #[non_exhaustive]
    AdjustedOrder {
        /// Index of the second offending boundary.
        index: usize,
        /// First raw boundary.
        previous_contractual: Date,
        /// Second raw boundary.
        contractual: Date,
        /// First adjusted boundary.
        previous_adjusted: Date,
        /// Second adjusted boundary.
        adjusted: Date,
    },
}

impl ScheduleError {
    /// Returns the stable schedule-domain kind.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::InvalidRollTenor { .. } => "schedule.invalid_roll_tenor",
            Self::ResolvedRollRange { .. } => "schedule.resolved_roll_range",
            Self::RollDate { .. } => "schedule.roll_date",
            Self::InvalidRange { .. } => "schedule.invalid_range",
            Self::InvalidBoundaryLimit { .. } => "schedule.invalid_boundary_limit",
            Self::InvalidRule { .. } => "schedule.invalid_rule",
            Self::InvalidTenor { .. } => "schedule.invalid_tenor",
            Self::IncompatibleEndOfMonth { .. } => "schedule.incompatible_end_of_month",
            Self::EmptyExplicitStubDates => "schedule.empty_explicit_stub_dates",
            Self::ReversedExplicitStubDates { .. } => "schedule.reversed_explicit_stub_dates",
            Self::ExplicitStubOutsideRange { .. } => "schedule.explicit_stub_outside_range",
            Self::IrregularExplicitMiddle { .. } => "schedule.irregular_explicit_middle",
            Self::InvalidIcmaPeriodLimit { .. } => "schedule.invalid_icma_period_limit",
            Self::NonPeriodicIcma { .. } => "schedule.non_periodic_icma",
            Self::InvalidIcmaFrequency { .. } => "schedule.invalid_icma_frequency",
            Self::IcmaFrequencyMismatch { .. } => "schedule.icma_frequency_mismatch",
            Self::IcmaPeriodLimitExceeded { .. } => "schedule.icma_period_limit_exceeded",
            Self::Icma { .. } => "schedule.icma",
            Self::StubRequired { .. } => "schedule.stub_required",
            Self::LongStubWithoutRegularPeriod { .. } => {
                "schedule.long_stub_without_regular_period"
            }
            Self::BoundaryLimitExceeded { .. } => "schedule.boundary_limit_exceeded",
            Self::AllocationFailed { .. } => "schedule.allocation_failed",
            Self::Date { .. } => "schedule.date",
            Self::Calendar { .. } => "schedule.calendar",
            Self::AdjustedOrder { .. } => "schedule.adjusted_order",
        }
    }
}

impl fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRollTenor { rule, tenor } => write!(
                f,
                "schedule roll {rule} does not accept stored tenor {tenor}; use positive Months/Years and a quarter-multiple stride for TwentiethIMM"
            ),
            Self::ResolvedRollRange {
                rule,
                tenor,
                requested_start,
                requested_end,
                resolved_start,
                resolved_end,
            } => write!(
                f,
                "schedule roll {rule} {tenor} resolves {requested_start} to {requested_end} into non-increasing {resolved_start} to {resolved_end}"
            ),
            Self::RollDate {
                rule,
                tenor,
                role,
                input,
                cause,
            } => write!(
                f,
                "schedule roll {rule} {tenor} required {role} from {input}: {cause}"
            ),
            Self::InvalidRange { start, end } => {
                write!(f, "schedule range must increase: {start} to {end}")
            }
            Self::InvalidBoundaryLimit {
                requested,
                minimum,
                maximum,
            } => write!(
                f,
                "schedule cap {requested} must be in {minimum}..={maximum}"
            ),
            Self::InvalidRule { rule } => write!(
                f,
                "schedule recurrence does not accept {rule}; use build_zero or build_roll for its rule family"
            ),
            Self::InvalidTenor { tenor } => write!(
                f,
                "schedule tenor {tenor} must have a positive stored length"
            ),
            Self::IncompatibleEndOfMonth {
                tenor,
                end_of_month,
            } => write!(
                f,
                "schedule EOM {end_of_month} is incompatible with {tenor}"
            ),
            Self::EmptyExplicitStubDates => {
                f.write_str("schedule needs a first or penultimate coupon date")
            }
            Self::ReversedExplicitStubDates { first, penultimate } => write!(
                f,
                "schedule first coupon {first} follows penultimate coupon {penultimate}"
            ),
            Self::ExplicitStubOutsideRange {
                role,
                date,
                start,
                end,
            } => write!(
                f,
                "schedule {role} coupon {date} must be strictly inside {start} to {end}"
            ),
            Self::IrregularExplicitMiddle {
                rule,
                tenor,
                anchor,
                endpoint,
                regular_boundary,
            } => write!(
                f,
                "schedule explicit {rule} {tenor} middle from {anchor} to {endpoint} is off lattice after {regular_boundary}"
            ),
            Self::InvalidIcmaPeriodLimit {
                requested,
                minimum,
                maximum,
            } => write!(
                f,
                "schedule ICMA period cap {requested} must be in {minimum}..={maximum}"
            ),
            Self::NonPeriodicIcma { rule } => {
                write!(f, "schedule {rule} has no recurring ICMA reference lattice")
            }
            Self::InvalidIcmaFrequency { frequency } => {
                write!(f, "schedule ICMA frequency {frequency} must be periodic")
            }
            Self::IcmaFrequencyMismatch {
                tenor,
                frequency,
                expected_tenor,
            } => write!(
                f,
                "schedule tenor {tenor} does not match ICMA frequency {frequency} ({expected_tenor})"
            ),
            Self::IcmaPeriodLimitExceeded {
                maximum,
                required_at_least,
            } => write!(
                f,
                "schedule ICMA needs at least {required_at_least} periods, exceeding cap {maximum}"
            ),
            Self::Icma { cause } => write!(f, "schedule ICMA reference context: {cause}"),
            Self::StubRequired {
                rule,
                tenor,
                start,
                end,
                regular_boundary,
            } => write!(
                f,
                "schedule {rule} {tenor} from {start} to {end} needs a stub after directional boundary {regular_boundary}"
            ),
            Self::LongStubWithoutRegularPeriod {
                rule,
                tenor,
                start,
                end,
            } => write!(
                f,
                "schedule {rule} {tenor} from {start} to {end} has no complete regular interval to merge into Long"
            ),
            Self::BoundaryLimitExceeded {
                maximum,
                required_at_least,
            } => write!(
                f,
                "schedule needs at least {required_at_least} final boundaries, exceeding cap {maximum}"
            ),
            Self::AllocationFailed { phase, requested } => {
                write!(f, "schedule {phase} cannot reserve {requested} elements")
            }
            Self::Date {
                index,
                phase,
                cause,
            } => write!(f, "schedule {phase} step {index}: {cause}"),
            Self::Calendar {
                index,
                phase,
                cause,
            } => write!(f, "schedule {phase} boundary {index:?}: {cause}"),
            Self::AdjustedOrder {
                index,
                previous_contractual,
                contractual,
                previous_adjusted,
                adjusted,
            } => write!(
                f,
                "schedule adjustment conflicts at boundary {index}: raw {previous_contractual}, {contractual}; adjusted {previous_adjusted}, {adjusted}"
            ),
        }
    }
}

impl StdError for ScheduleError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Date { cause, .. } | Self::RollDate { cause, .. } => Some(cause),
            Self::Calendar { cause, .. } => Some(cause),
            Self::Icma { cause } => Some(cause),
            _ => None,
        }
    }
}

fn validate_limit(requested: u32) -> Result<(), ScheduleError> {
    if !(ScheduleBuilder::MIN_BOUNDARIES..=ScheduleBuilder::MAX_BOUNDARIES).contains(&requested) {
        return Err(ScheduleError::InvalidBoundaryLimit {
            requested,
            minimum: ScheduleBuilder::MIN_BOUNDARIES,
            maximum: ScheduleBuilder::MAX_BOUNDARIES,
        });
    }
    Ok(())
}

fn outer_regularity(adjacent: Option<Date>, endpoint: Date) -> PeriodRegularity {
    if adjacent == Some(endpoint) {
        PeriodRegularity::Regular
    } else {
        PeriodRegularity::Stub
    }
}

#[derive(Clone, Copy)]
enum IcmaLattice {
    Anchored { anchor: Date, preserve: bool },
    Roll(RollProvenance),
}

impl IcmaLattice {
    fn anchor(self) -> Result<Date, ScheduleError> {
        match self {
            Self::Anchored { anchor, .. } => Ok(anchor),
            // The private provenance contains a valid civil anchor month.
            // Retain a typed cause if its reconstruction is ever extended.
            Self::Roll(provenance) => {
                provenance
                    .reference_anchor()
                    .map_err(|cause| ScheduleError::Date {
                        index: 0,
                        phase: "ICMA reference anchor",
                        cause,
                    })
            }
        }
    }

    fn date(
        self,
        anchor: Date,
        tenor: Period,
        step: u32,
        forward: bool,
        index: usize,
    ) -> Result<Date, ScheduleError> {
        match self {
            Self::Anchored { preserve, .. } => checked_anchored_date(
                anchor,
                tenor,
                step,
                forward,
                preserve,
                index,
                "ICMA reference extension",
            ),
            Self::Roll(provenance) => {
                provenance
                    .reference_date(anchor, step, forward)
                    .map_err(|cause| ScheduleError::Date {
                        index,
                        phase: "ICMA reference extension",
                        cause,
                    })
            }
        }
    }
}

fn check_icma_count(required: u32, maximum: u32) -> Result<(), ScheduleError> {
    if required > maximum {
        Err(ScheduleError::IcmaPeriodLimitExceeded {
            maximum,
            required_at_least: required,
        })
    } else {
        Ok(())
    }
}

struct Rolling {
    dates: Vec<Date>,
    regularity: Vec<PeriodRegularity>,
    count: u32,
}

impl Rolling {
    fn new(anchor: Date) -> Result<Self, ScheduleError> {
        let mut dates = Vec::new();
        reserve_one(&mut dates, "contractual boundaries")?;
        dates.push(anchor);
        Ok(Self {
            dates,
            regularity: Vec::new(),
            count: 1,
        })
    }

    fn push(&mut self, date: Date, flag: PeriodRegularity) -> Result<(), ScheduleError> {
        reserve_one(&mut self.dates, "contractual boundaries")?;
        reserve_one(&mut self.regularity, "contractual regularity")?;
        self.dates.push(date);
        self.regularity.push(flag);
        self.count += 1;
        Ok(())
    }

    fn push_limited(
        &mut self,
        date: Date,
        flag: PeriodRegularity,
        maximum: u32,
    ) -> Result<(), ScheduleError> {
        if self.count + 1 > maximum {
            return Err(ScheduleError::BoundaryLimitExceeded {
                maximum,
                required_at_least: self.count + 1,
            });
        }
        self.push(date, flag)
    }
}

fn reserve_one<T>(values: &mut Vec<T>, phase: &'static str) -> Result<(), ScheduleError> {
    if values.len() == values.capacity() {
        values
            .try_reserve(1)
            .map_err(|_| ScheduleError::AllocationFailed {
                phase,
                requested: values.len() + 1,
            })?;
    }
    Ok(())
}

fn anchored_candidate(
    anchor: Date,
    tenor: Period,
    step: u32,
    forward: bool,
    preserve: bool,
    index: usize,
) -> Result<Option<Date>, ScheduleError> {
    let magnitude = i64::from(tenor.length()) * i64::from(step);
    // These supported-range comparisons prove an unused candidate is beyond
    // every valid endpoint. They do not catch and discard arbitrary DateErrors.
    // The count cap bounds magnitude by i32::MAX * 3_652_059, inside i64;
    // the subsequent factors seven/twelve also fit comfortably inside i64.
    let days_available = if forward {
        Date::MAX.days_since(anchor)
    } else {
        anchor.days_since(Date::MIN)
    };
    let months_available = if forward {
        i64::from(9999 - anchor.year()) * 12 + (12 - i64::from(anchor.month()))
    } else {
        i64::from(anchor.year() - 1) * 12 + (i64::from(anchor.month()) - 1)
    };
    let years_available = if forward {
        9999 - anchor.year()
    } else {
        anchor.year() - 1
    };
    let outside = match tenor.unit() {
        TimeUnit::Days => magnitude > i64::from(days_available),
        TimeUnit::Weeks => magnitude > i64::from(days_available) / 7,
        TimeUnit::Months => magnitude > months_available,
        TimeUnit::Years => magnitude > i64::from(years_available),
    };
    if outside {
        return Ok(None);
    }
    checked_anchored_date(anchor, tenor, step, forward, preserve, index, "generation").map(Some)
}

fn checked_anchored_date(
    anchor: Date,
    tenor: Period,
    step: u32,
    forward: bool,
    preserve: bool,
    index: usize,
    phase: &'static str,
) -> Result<Date, ScheduleError> {
    // Both construction operations cap their step before evaluating it.
    // Even the widest positive stored tenor and all factors fit i64.
    let magnitude = i64::from(tenor.length()) * i64::from(step);
    let offset = if forward { magnitude } else { -magnitude };
    let date = match tenor.unit() {
        TimeUnit::Days => anchor.checked_add_days(offset),
        TimeUnit::Weeks => anchor.checked_add_weeks(offset),
        TimeUnit::Months => anchor.checked_add_months(offset),
        TimeUnit::Years => anchor.checked_add_years(offset),
    }
    .map_err(|cause| ScheduleError::Date {
        index,
        phase,
        cause,
    })?;
    Ok(if preserve {
        date.last_day_of_month()
    } else {
        date
    })
}
