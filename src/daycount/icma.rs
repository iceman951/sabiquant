//! Actual/Actual ICMA with explicitly supplied determination periods.
//!
//! The financial definition is ICMA Primary Market Handbook,
//! [Appendix A5, March 2022](https://www.icmagroup.org/assets/documents/ICMA-PMH-Appendix-A5-March-2022.pdf),
//! Version 2 §§1.3(a)–(b). Accrual within each supplied period is
//! divided by its actual length and the declared annual event frequency.
//! No payment dates, reference boundaries or frequencies are inferred.

use crate::time::{Date, Frequency};
use std::error::Error as StdError;
use std::fmt;

/// One positive, half-open contractual determination period.
///
/// The dates must be supplied explicitly, including quasi-coupon periods for
/// irregular stubs. Construction does not generate a schedule or adjust dates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DeterminationPeriod {
    start: Date,
    end: Date,
}

impl DeterminationPeriod {
    /// Constructs a determination period with `start < end`.
    ///
    /// # Errors
    ///
    /// [`IcmaError::InvalidPeriod`] if the interval is zero or reversed.
    pub fn new(start: Date, end: Date) -> Result<Self, IcmaError> {
        if start >= end {
            return Err(IcmaError::InvalidPeriod { start, end });
        }
        Ok(Self { start, end })
    }

    /// Returns the inclusive contractual start date.
    #[must_use]
    pub const fn start(self) -> Date {
        self.start
    }

    /// Returns the exclusive contractual end date.
    #[must_use]
    pub const fn end(self) -> Date {
        self.end
    }
}

/// Actual/Actual ICMA over an ordered, contiguous set of explicit periods.
///
/// The annual frequency is a contractual input, not a quantity inferred from
/// the reference dates. Any named periodic [`Frequency`] is accepted; Once
/// is rejected. Date/frequency consistency beyond these structural checks is
/// the caller's responsibility because schedule generation is outside M3.
///
/// ## Accrual formula
///
/// For each overlapping period `p`, contribute
/// `actual_days(intersection) / (events_per_year × actual_days(p))`.
/// Contributions are accumulated in chronological order with compensated
/// summation. Reversing endpoints negates the forward fraction.
///
/// Coverage includes both outer boundary dates for zero-length queries. A
/// query outside those boundaries fails even when its endpoints are equal.
/// The actual [`day_count`](Self::day_count) does not require coverage.
///
/// ```
/// use sabiquant::daycount::{ActualActualIcma, DeterminationPeriod};
/// use sabiquant::time::{Date, Frequency};
/// let start = Date::from_ymd(2003, 11, 1)?;
/// let end = Date::from_ymd(2004, 5, 1)?;
/// let period = DeterminationPeriod::new(start, end)?;
/// let convention = ActualActualIcma::new(vec![period], Frequency::Semiannual)?;
/// assert_eq!(convention.year_fraction(start, end)?.to_bits(), 0.5_f64.to_bits());
/// # Ok::<(), sabiquant::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActualActualIcma {
    periods: Vec<DeterminationPeriod>,
    frequency: Frequency,
    events_per_year: u32,
    coverage_start: Date,
    coverage_end: Date,
}

impl ActualActualIcma {
    /// Constructs an immutable convention from explicitly ordered periods.
    ///
    /// Periods must be nonempty and adjacent: each period's start equals the
    /// preceding period's end. Inputs are neither sorted nor expanded.
    ///
    /// # Errors
    ///
    /// [`IcmaError::InvalidFrequency`] for Once, [`IcmaError::EmptyPeriods`]
    /// for no periods, or [`IcmaError::NonContiguousPeriods`] for a gap,
    /// overlap, duplicate or unordered input. Frequency is checked first.
    pub fn new(periods: Vec<DeterminationPeriod>, frequency: Frequency) -> Result<Self, IcmaError> {
        if frequency.period().is_none_or(|period| period.length() <= 0) {
            return Err(IcmaError::InvalidFrequency { frequency });
        }
        let Some(events_per_year) = frequency.events_per_year().filter(|&count| count > 0) else {
            return Err(IcmaError::InvalidFrequency { frequency });
        };
        let Some(first) = periods.first() else {
            return Err(IcmaError::EmptyPeriods);
        };
        let coverage_start = first.start;
        let mut coverage_end = first.end;
        for (index, period) in periods.iter().enumerate().skip(1) {
            if period.start != coverage_end {
                return Err(IcmaError::NonContiguousPeriods {
                    index,
                    previous_end: coverage_end,
                    start: period.start,
                });
            }
            coverage_end = period.end;
        }
        Ok(Self {
            periods,
            frequency,
            events_per_year,
            coverage_start,
            coverage_end,
        })
    }

    /// Returns the explicitly supplied periods in contractual order.
    #[must_use]
    pub fn periods(&self) -> &[DeterminationPeriod] {
        &self.periods
    }

    /// Returns the explicitly supplied annual frequency.
    #[must_use]
    pub const fn frequency(&self) -> Frequency {
        self.frequency
    }

    /// Returns the earliest covered boundary date.
    #[must_use]
    pub const fn coverage_start(&self) -> Date {
        self.coverage_start
    }

    /// Returns the latest covered boundary date.
    #[must_use]
    pub const fn coverage_end(&self) -> Date {
        self.coverage_end
    }

    /// Returns the signed actual calendar-day difference, regardless of
    /// reference-period coverage. Equal endpoints give zero.
    #[must_use]
    pub const fn day_count(&self, start: Date, end: Date) -> i32 {
        end.days_since(start)
    }

    /// Computes the fraction from actual days in supplied determination periods.
    ///
    /// Both endpoints must lie in the closed outer boundary range, including
    /// when equal or reversed. Equal endpoints return positive zero. Each
    /// positive overlap is divided once, then the rounded contributions are
    /// added chronologically using Neumaier compensated summation.
    ///
    /// The denominator is an exact integer product below `2^53`: at most
    /// `365 × 3_652_058`. All contributions are finite and positive and their
    /// sum is at most 3,652,058, so intermediate overflow cannot occur.
    ///
    /// # Errors
    ///
    /// [`IcmaError::OutOfCoverage`] if either endpoint is outside the supplied
    /// outer boundaries; the error retains the original endpoint order.
    pub fn year_fraction(&self, start: Date, end: Date) -> Result<f64, IcmaError> {
        let lower = start.min(end);
        let upper = start.max(end);
        if lower < self.coverage_start || upper > self.coverage_end {
            return Err(IcmaError::OutOfCoverage {
                start,
                end,
                coverage_start: self.coverage_start,
                coverage_end: self.coverage_end,
            });
        }
        if start == end {
            return Ok(0.0);
        }
        let first = self.periods.partition_point(|period| period.end <= lower);
        let mut sum = 0.0_f64;
        let mut correction = 0.0_f64;
        for period in self.periods.iter().skip(first) {
            if period.start >= upper {
                break;
            }
            let overlap_start = lower.max(period.start);
            let overlap_end = upper.min(period.end);
            let numerator = f64::from(overlap_end.days_since(overlap_start));
            let denominator =
                f64::from(self.events_per_year) * f64::from(period.end.days_since(period.start));
            let term = numerator / denominator;
            let next = sum + term;
            // FastTwoSum recovers the exact rounding residual. The branch
            // ensures the larger-magnitude addend is subtracted first.
            correction += if sum >= term {
                (sum - next) + term
            } else {
                (term - next) + sum
            };
            sum = next;
        }
        let result = sum + correction;
        Ok(if start > end { -result } else { result })
    }
}

/// A rejected ICMA reference context or accrual interval.
///
/// Stable kind names are returned by [`kind`](Self::kind); display wording
/// may change. All rejected dates, indices and frequencies remain structured.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum IcmaError {
    /// A determination period is zero or reversed. Kind: `icma.invalid_period`.
    #[non_exhaustive]
    InvalidPeriod {
        /// Rejected inclusive start.
        start: Date,
        /// Rejected exclusive end.
        end: Date,
    },
    /// No determination periods were supplied. Kind: `icma.empty_periods`.
    EmptyPeriods,
    /// Frequency is not periodic. Kind: `icma.invalid_frequency`.
    #[non_exhaustive]
    InvalidFrequency {
        /// Rejected contractual frequency.
        frequency: Frequency,
    },
    /// Supplied periods do not adjoin in order. Kind: `icma.non_contiguous_periods`.
    #[non_exhaustive]
    NonContiguousPeriods {
        /// Zero-based index of the offending period.
        index: usize,
        /// End of the preceding supplied period.
        previous_end: Date,
        /// Start of the offending supplied period.
        start: Date,
    },
    /// Requested endpoints are outside the references. Kind: `icma.out_of_coverage`.
    #[non_exhaustive]
    OutOfCoverage {
        /// Originally requested start, before reversal normalization.
        start: Date,
        /// Originally requested end, before reversal normalization.
        end: Date,
        /// Earliest supplied boundary.
        coverage_start: Date,
        /// Latest supplied boundary.
        coverage_end: Date,
    },
}

impl IcmaError {
    /// Returns the stable `icma.<variant>` kind name.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::InvalidPeriod { .. } => "icma.invalid_period",
            Self::EmptyPeriods => "icma.empty_periods",
            Self::InvalidFrequency { .. } => "icma.invalid_frequency",
            Self::NonContiguousPeriods { .. } => "icma.non_contiguous_periods",
            Self::OutOfCoverage { .. } => "icma.out_of_coverage",
        }
    }
}

impl fmt::Display for IcmaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPeriod { start, end } => write!(
                formatter,
                "ICMA determination period must be positive: {start}..{end}"
            ),
            Self::EmptyPeriods => formatter.write_str("ICMA needs explicit determination periods"),
            Self::InvalidFrequency { frequency } => write!(
                formatter,
                "ICMA needs a periodic annual frequency, got {frequency}"
            ),
            Self::NonContiguousPeriods {
                index,
                previous_end,
                start,
            } => write!(
                formatter,
                "ICMA period {index} starts at {start}, expected {previous_end}"
            ),
            Self::OutOfCoverage {
                start,
                end,
                coverage_start,
                coverage_end,
            } => write!(
                formatter,
                "ICMA accrual {start}..{end} lies outside {coverage_start}..{coverage_end}"
            ),
        }
    }
}

impl StdError for IcmaError {}
