//! Explicit contractual schedules with separate business-day adjustment.
//!
//! This local tranche supports Forward, Backward and Zero only. Recurrence is
//! anchored to the original directional endpoint; repeated month clamping and
//! business-day advancement are not recurrence rules. Dates and regularity
//! remain contractual even when adjustment changes interval lengths.
//!
//! The [local M7 contract](https://github.com/iceman951/sabiquant/blob/main/docs/M7_LOCAL_READINESS.md)
//! records scope and unresolved acceptance gates. Specialized market rolls,
//! explicit coupon dates, automatic ICMA references and cash flows are deferred.

use crate::calendar::{
    BusinessDayConvention, Calendar, CalendarError, EndOfMonth, EndpointInclusion, TargetCalendar,
};
use crate::error::IdentifierError;
use crate::identifier::{Entry, Identifiers};
use crate::time::{Date, DateError, Period, TimeUnit};
use std::error::Error as StdError;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

/// The explicitly selected contractual recurrence direction or Zero rule.
///
/// Canonical text is exactly `Forward`, `Backward` or `Zero`. There are no
/// aliases, case folding or whitespace trimming. The seven specialized market
/// rules are outside this tranche and cannot be silently selected.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DateGenerationRule {
    /// Generate from the original start; an automatic residual is at the end.
    Forward,
    /// Generate from the original end; return chronological dates and a first residual.
    Backward,
    /// Exactly start/end, with no periodic tenor, stub or EOM configuration.
    Zero,
}

impl DateGenerationRule {
    /// All currently supported rule labels in canonical order.
    pub const ALL: [Self; 3] = [Self::Forward, Self::Backward, Self::Zero];

    /// Returns the exact canonical identifier.
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::Forward => "Forward",
            Self::Backward => "Backward",
            Self::Zero => "Zero",
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
    /// One complete interval on the original directional recurrence lattice.
    Regular,
    /// The automatically retained short or merged long residual interval.
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
                    phase: "coverage",
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
            DateGenerationRule::Zero => return Err(ScheduleError::InvalidRule { rule }),
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
        self.finish(rolling, rule, Some(tenor), Some(stub), Some(eom))
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
        self.finish(rolling, DateGenerationRule::Zero, None, None, None)
    }

    fn finish(
        &self,
        rolling: Rolling,
        rule: DateGenerationRule,
        tenor: Option<Period>,
        stub: Option<StubPolicy>,
        eom: Option<EndOfMonth>,
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
    calendar: ScheduleCalendar,
    convention: BusinessDayConvention,
    termination_convention: BusinessDayConvention,
}

impl Schedule {
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

    /// Returns the supplied residual policy, or None for Zero.
    #[must_use]
    pub const fn stub_policy(&self) -> Option<StubPolicy> {
        self.stub_policy
    }

    /// Returns the supplied calendar EOM policy, or None for Zero.
    #[must_use]
    pub const fn end_of_month(&self) -> Option<EndOfMonth> {
        self.end_of_month
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
    /// Zero passed to the recurrence operation; `schedule.invalid_rule`.
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
        /// Anchor-relative recurrence step, before chronological ordering.
        index: usize,
        /// Operation phase, currently generation.
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
            Self::InvalidRange { .. } => "schedule.invalid_range",
            Self::InvalidBoundaryLimit { .. } => "schedule.invalid_boundary_limit",
            Self::InvalidRule { .. } => "schedule.invalid_rule",
            Self::InvalidTenor { .. } => "schedule.invalid_tenor",
            Self::IncompatibleEndOfMonth { .. } => "schedule.incompatible_end_of_month",
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
                "schedule recurrence does not accept {rule}; use build_zero"
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
            Self::Date { cause, .. } => Some(cause),
            Self::Calendar { cause, .. } => Some(cause),
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
    let offset = if forward { magnitude } else { -magnitude };
    let date = match tenor.unit() {
        TimeUnit::Days => anchor.checked_add_days(offset),
        TimeUnit::Weeks => anchor.checked_add_weeks(offset),
        TimeUnit::Months => anchor.checked_add_months(offset),
        TimeUnit::Years => anchor.checked_add_years(offset),
    }
    .map_err(|cause| ScheduleError::Date {
        index,
        phase: "generation",
        cause,
    })?;
    Ok(Some(if preserve {
        date.last_day_of_month()
    } else {
        date
    }))
}
