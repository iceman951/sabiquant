//! The crate-level error, and the error types that every domain shares.
//!
//! # Conventions
//!
//! SabiQuant reports failure with typed errors, as decided in
//! [ADR-0007](https://github.com/iceman951/sabiquant/blob/main/docs/adr/ADR-0007-error-handling.md):
//!
//! - Every operation that can fail for some input returns [`Result`]. Failure
//!   is never signalled by a sentinel value, by NaN, or by a panic
//!   (FR-CORE-01).
//! - Each domain defines one non-exhaustive error enumeration next to its
//!   operations. Every variant carries structured context, such as the name
//!   of the rejected parameter and its value (FR-CORE-02).
//! - [`Error`] wraps the error of every domain, so code that spans several
//!   domains can propagate any failure with `?`.
//! - Every error type implements [`std::error::Error`],
//!   [`Display`](fmt::Display), [`Debug`](fmt::Debug), [`Clone`] and
//!   [`PartialEq`], and is [`Send`], [`Sync`] and `'static`.
//!
//! # Kind names are stable; messages are not
//!
//! Every error variant has a **kind name**, returned by the `kind` method of
//! its type, for example `"value.not_finite"`. Kind names have the form
//! `<domain>.<variant>`, in lowercase ASCII letters and underscores. They are
//! part of the stable API, and reference fixtures use them to state the error
//! that a scenario expects
//! ([ADR-0008](https://github.com/iceman951/sabiquant/blob/main/docs/adr/ADR-0008-validation-strategy.md)).
//! The wording of `Display` messages is **not** stable and may change in any
//! release.
//!
//! | Kind name | Variant | Reported when |
//! |---|---|---|
//! | `value.not_finite` | [`ValueError::NotFinite`] | A real-valued input is NaN or infinite |
//! | `value.out_of_domain` | [`ValueError::OutOfDomain`] | A finite real-valued input lies outside its documented domain |
//! | `value.result_not_finite` | [`ValueError::ResultNotFinite`] | A computed result overflows, or is otherwise not finite |
//! | `value.result_out_of_domain` | [`ValueError::ResultOutOfDomain`] | A computed finite result lies outside its domain |
//! | `identifier.unknown` | [`IdentifierError::Unknown`] | Text is neither a canonical identifier nor a documented alias |
//! | `identifier.ambiguous` | [`IdentifierError::Ambiguous`] | Text could name more than one convention |
//!
//! M2 also provides [`DateError`](crate::time::DateError) and
//! [`PeriodError`](crate::time::PeriodError); their documentation lists stable
//! `date.*` and `period.*` kinds. [`Error`] wraps each domain error.
//! Local M7 [`ScheduleError`](crate::schedule::ScheduleError) lists the
//! corresponding stable `schedule.*` kinds and retained calendar/date context.
//!
//! # Error sources
//!
//! [`Error`] displays the domain error and exposes that typed error through
//! [`source`](std::error::Error::source). A domain error that was caused by
//! another error returns that error from `source`.
//!
//! # QuantLib correspondence
//!
//! QuantLib 1.43 signals failure by throwing `QuantLib::Error` exceptions from
//! the `QL_REQUIRE`, `QL_ENSURE` and `QL_FAIL` macros (`ql/errors.hpp`), and
//! some of its numerical code returns NaN instead. SabiQuant returns typed
//! errors and never returns NaN. Where QuantLib returns a value for an input
//! that SabiQuant rejects, the difference is a documented intentional
//! divergence
//! ([ADR-0003](https://github.com/iceman951/sabiquant/blob/main/docs/adr/ADR-0003-quantlib-compatibility.md)).

use std::error::Error as StdError;
use std::fmt;
use std::ops::Bound;

/// Any error reported by SabiQuant.
///
/// `Error` wraps the error type of each domain, so that code which combines
/// several domains can propagate any failure with `?`. Match on the variant to
/// inspect the domain error, or call [`kind`](Self::kind) for its stable kind
/// name.
///
/// Its `Display` output is that of the domain error. Its
/// [`source`](std::error::Error::source) is the wrapped domain error, which
/// callers can downcast to inspect the structured context.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// A real number was rejected by the numeric validation policy.
    Value(ValueError),
    /// An exact decimal definition, explicit rounding or binary64 conversion failed.
    Amount(crate::amounts::AmountError),
    /// Text did not name exactly one identifier.
    Identifier(IdentifierError),
    /// A date or date arithmetic operation was rejected.
    Date(crate::time::DateError),
    /// A tenor or tenor arithmetic operation was rejected.
    Period(crate::time::PeriodError),
    /// A calendar or business-day operation was rejected.
    Calendar(crate::calendar::CalendarError),
    /// A day-count result violated the numeric policy.
    DayCount(crate::daycount::DayCountError),
    /// An explicit ICMA reference context or accrual query was invalid.
    Icma(crate::daycount::IcmaError),
    /// A rate convention, input, accrual context or computed result failed.
    Rate(crate::rates::RateError),
    /// A numerical summation input or arithmetic result failed.
    Sum(crate::math::SumError),
    /// A normal-distribution input, result or bounded computation failed.
    Normal(crate::math::NormalError),
    /// An interpolation definition, query or numerical support condition failed.
    Interpolation(crate::math::InterpolationError),
    /// A solver input, callback or bounded computation failed.
    Solver(crate::math::SolverError),
    /// A schedule definition, generation, resource or calendar adjustment failed.
    Schedule(crate::schedule::ScheduleError),
    /// A cash-flow definition, resource or analytical calculation failed.
    CashFlow(crate::cashflows::CashFlowError),
    /// A bond definition, settlement, resource request or valuation failed.
    Bond(crate::bonds::BondError),
    /// A bond yield-to-maturity inversion failed.
    BondYield(crate::bonds::BondYieldError),
    /// A conventional gilt definition, price, risk or exact cash operation failed.
    Gilt(crate::bonds::GiltError),
    /// A conventional gilt target resolution or counted yield inversion failed.
    GiltYield(crate::bonds::GiltYieldError),
    /// An explicit fixed-principal Treasury source, analytical or cash operation failed.
    Treasury(crate::bonds::TreasuryError),
    /// A Treasury target, callback or bounded yield inversion failed.
    TreasuryYield(crate::bonds::TreasuryYieldError),
    /// Explicit settlement evidence, par nominal or exact staged arithmetic failed.
    Settlement(crate::settlement::SettlementError),
}

impl Error {
    /// Returns the stable kind name of the wrapped error, for example
    /// `"value.not_finite"`.
    ///
    /// The [module documentation](crate::error) links to every domain's kind names.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Value(error) => error.kind(),
            Self::Amount(error) => error.kind(),
            Self::Identifier(error) => error.kind(),
            Self::Date(error) => error.kind(),
            Self::Period(error) => error.kind(),
            Self::Calendar(error) => error.kind(),
            Self::DayCount(error) => error.kind(),
            Self::Icma(error) => error.kind(),
            Self::Rate(error) => error.kind(),
            Self::Sum(error) => error.kind(),
            Self::Normal(error) => error.kind(),
            Self::Interpolation(error) => error.kind(),
            Self::Solver(error) => error.kind(),
            Self::Schedule(error) => error.kind(),
            Self::CashFlow(error) => error.kind(),
            Self::Bond(error) => error.kind(),
            Self::BondYield(error) => error.kind(),
            Self::Gilt(error) => error.kind(),
            Self::GiltYield(error) => error.kind(),
            Self::Treasury(error) => error.kind(),
            Self::TreasuryYield(error) => error.kind(),
            Self::Settlement(error) => error.kind(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(error) => fmt::Display::fmt(error, f),
            Self::Amount(error) => fmt::Display::fmt(error, f),
            Self::Identifier(error) => fmt::Display::fmt(error, f),
            Self::Date(error) => fmt::Display::fmt(error, f),
            Self::Period(error) => fmt::Display::fmt(error, f),
            Self::Calendar(error) => fmt::Display::fmt(error, f),
            Self::DayCount(error) => fmt::Display::fmt(error, f),
            Self::Icma(error) => fmt::Display::fmt(error, f),
            Self::Rate(error) => fmt::Display::fmt(error, f),
            Self::Sum(error) => fmt::Display::fmt(error, f),
            Self::Normal(error) => fmt::Display::fmt(error, f),
            Self::Interpolation(error) => fmt::Display::fmt(error, f),
            Self::Solver(error) => fmt::Display::fmt(error, f),
            Self::Schedule(error) => fmt::Display::fmt(error, f),
            Self::CashFlow(error) => fmt::Display::fmt(error, f),
            Self::Bond(error) => fmt::Display::fmt(error, f),
            Self::BondYield(error) => fmt::Display::fmt(error, f),
            Self::Gilt(error) => fmt::Display::fmt(error, f),
            Self::GiltYield(error) => fmt::Display::fmt(error, f),
            Self::Treasury(error) => fmt::Display::fmt(error, f),
            Self::TreasuryYield(error) => fmt::Display::fmt(error, f),
            Self::Settlement(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Value(error) => Some(error),
            Self::Amount(error) => Some(error),
            Self::Identifier(error) => Some(error),
            Self::Date(error) => Some(error),
            Self::Period(error) => Some(error),
            Self::Calendar(error) => Some(error),
            Self::DayCount(error) => Some(error),
            Self::Icma(error) => Some(error),
            Self::Rate(error) => Some(error),
            Self::Sum(error) => Some(error),
            Self::Normal(error) => Some(error),
            Self::Interpolation(error) => Some(error),
            Self::Solver(error) => Some(error),
            Self::Schedule(error) => Some(error),
            Self::CashFlow(error) => Some(error),
            Self::Bond(error) => Some(error),
            Self::BondYield(error) => Some(error),
            Self::Gilt(error) => Some(error),
            Self::GiltYield(error) => Some(error),
            Self::Treasury(error) => Some(error),
            Self::TreasuryYield(error) => Some(error),
            Self::Settlement(error) => Some(error),
        }
    }
}

impl From<crate::amounts::AmountError> for Error {
    fn from(error: crate::amounts::AmountError) -> Self {
        Self::Amount(error)
    }
}

impl From<ValueError> for Error {
    fn from(error: ValueError) -> Self {
        Self::Value(error)
    }
}

impl From<IdentifierError> for Error {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<crate::time::DateError> for Error {
    fn from(error: crate::time::DateError) -> Self {
        Self::Date(error)
    }
}

impl From<crate::time::PeriodError> for Error {
    fn from(error: crate::time::PeriodError) -> Self {
        Self::Period(error)
    }
}

impl From<crate::calendar::CalendarError> for Error {
    fn from(error: crate::calendar::CalendarError) -> Self {
        Self::Calendar(error)
    }
}

impl From<crate::daycount::DayCountError> for Error {
    fn from(error: crate::daycount::DayCountError) -> Self {
        Self::DayCount(error)
    }
}

impl From<crate::daycount::IcmaError> for Error {
    fn from(error: crate::daycount::IcmaError) -> Self {
        Self::Icma(error)
    }
}

impl From<crate::rates::RateError> for Error {
    fn from(error: crate::rates::RateError) -> Self {
        Self::Rate(error)
    }
}

impl From<crate::math::SumError> for Error {
    fn from(error: crate::math::SumError) -> Self {
        Self::Sum(error)
    }
}

impl From<crate::math::NormalError> for Error {
    fn from(error: crate::math::NormalError) -> Self {
        Self::Normal(error)
    }
}

impl From<crate::math::InterpolationError> for Error {
    fn from(error: crate::math::InterpolationError) -> Self {
        Self::Interpolation(error)
    }
}

impl From<crate::math::SolverError> for Error {
    fn from(error: crate::math::SolverError) -> Self {
        Self::Solver(error)
    }
}

impl From<crate::schedule::ScheduleError> for Error {
    fn from(error: crate::schedule::ScheduleError) -> Self {
        Self::Schedule(error)
    }
}

impl From<crate::cashflows::CashFlowError> for Error {
    fn from(error: crate::cashflows::CashFlowError) -> Self {
        Self::CashFlow(error)
    }
}

impl From<crate::bonds::BondError> for Error {
    fn from(error: crate::bonds::BondError) -> Self {
        Self::Bond(error)
    }
}

impl From<crate::bonds::BondYieldError> for Error {
    fn from(error: crate::bonds::BondYieldError) -> Self {
        Self::BondYield(error)
    }
}

impl From<crate::bonds::GiltError> for Error {
    fn from(error: crate::bonds::GiltError) -> Self {
        Self::Gilt(error)
    }
}

impl From<crate::bonds::GiltYieldError> for Error {
    fn from(error: crate::bonds::GiltYieldError) -> Self {
        Self::GiltYield(error)
    }
}

impl From<crate::bonds::TreasuryError> for Error {
    fn from(error: crate::bonds::TreasuryError) -> Self {
        Self::Treasury(error)
    }
}

impl From<crate::bonds::TreasuryYieldError> for Error {
    fn from(error: crate::bonds::TreasuryYieldError) -> Self {
        Self::TreasuryYield(error)
    }
}

impl From<crate::settlement::SettlementError> for Error {
    fn from(error: crate::settlement::SettlementError) -> Self {
        Self::Settlement(error)
    }
}

/// A real number rejected by the numeric validation policy.
///
/// Operations check the real numbers they receive at the public boundary, and
/// check the real numbers they compute before returning them
/// ([NUMERICAL_CORRECTNESS §5](https://github.com/iceman951/sabiquant/blob/main/docs/NUMERICAL_CORRECTNESS.md#5-numerical-policies)):
///
/// - **NP-01**: every real-valued input must be finite (FR-CORE-03);
/// - **NP-02**: every result must be finite and inside its domain, so an
///   overflow or underflow that makes a result meaningless is an error, never
///   an infinity or a zero.
///
/// Parameter and quantity names are the names used in the documentation of
/// the operation that reports the error.
///
/// # QuantLib correspondence
///
/// QuantLib 1.43 often performs none of these checks and returns NaN or an
/// infinity instead. Rejecting such inputs and results is an intentional
/// divergence.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ValueError {
    /// A real-valued input is NaN or infinite (NP-01).
    ///
    /// Kind name: `value.not_finite`.
    #[non_exhaustive]
    NotFinite {
        /// The name of the rejected parameter.
        parameter: &'static str,
        /// The non-finite value that was given.
        value: NonFinite,
    },
    /// A finite real-valued input lies outside its documented domain.
    ///
    /// Kind name: `value.out_of_domain`.
    #[non_exhaustive]
    OutOfDomain {
        /// The name of the rejected parameter.
        parameter: &'static str,
        /// The value that was given. It is always finite.
        value: f64,
        /// The domain that the value must lie in.
        domain: Interval,
    },
    /// A computed result is not finite, for example after an overflow (NP-02).
    ///
    /// Kind name: `value.result_not_finite`.
    #[non_exhaustive]
    ResultNotFinite {
        /// The name of the computed quantity, for example `"compound factor"`.
        quantity: &'static str,
        /// The non-finite value that the computation produced.
        value: NonFinite,
    },
    /// A computed finite result lies outside its domain, for example a
    /// compound factor that underflowed to zero (NP-02).
    ///
    /// Kind name: `value.result_out_of_domain`.
    #[non_exhaustive]
    ResultOutOfDomain {
        /// The name of the computed quantity, for example `"compound factor"`.
        quantity: &'static str,
        /// The value that the computation produced. It is always finite.
        value: f64,
        /// The domain that the result must lie in.
        domain: Interval,
    },
}

impl ValueError {
    /// Returns the stable kind name of the error: `"value.not_finite"`,
    /// `"value.out_of_domain"`, `"value.result_not_finite"` or
    /// `"value.result_out_of_domain"`.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::NotFinite { .. } => "value.not_finite",
            Self::OutOfDomain { .. } => "value.out_of_domain",
            Self::ResultNotFinite { .. } => "value.result_not_finite",
            Self::ResultOutOfDomain { .. } => "value.result_out_of_domain",
        }
    }
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFinite { parameter, value } => {
                write!(f, "`{parameter}` must be finite, but is {value}")
            }
            Self::OutOfDomain {
                parameter,
                value,
                domain,
            } => write!(f, "`{parameter}` is {value:?}, outside its domain {domain}"),
            Self::ResultNotFinite { quantity, value } => {
                write!(f, "the computed {quantity} is not finite: {value}")
            }
            Self::ResultOutOfDomain {
                quantity,
                value,
                domain,
            } => write!(
                f,
                "the computed {quantity} is {value:?}, outside its domain {domain}"
            ),
        }
    }
}

impl StdError for ValueError {}

/// The non-finite values of IEEE 754 binary64 (`f64`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "IEEE 754 has exactly these three kinds of non-finite value"
)]
pub enum NonFinite {
    /// Not a number, whatever its sign and payload.
    NaN,
    /// Positive infinity.
    PositiveInfinity,
    /// Negative infinity.
    NegativeInfinity,
}

impl NonFinite {
    /// Classifies `value`, or returns `None` when it is finite.
    pub(crate) fn classify(value: f64) -> Option<Self> {
        if value.is_nan() {
            Some(Self::NaN)
        } else if value.is_infinite() {
            if value.is_sign_positive() {
                Some(Self::PositiveInfinity)
            } else {
                Some(Self::NegativeInfinity)
            }
        } else {
            None
        }
    }
}

impl fmt::Display for NonFinite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NaN => "NaN",
            Self::PositiveInfinity => "inf",
            Self::NegativeInfinity => "-inf",
        })
    }
}

/// An interval of real numbers: the documented domain of a parameter or a
/// result.
///
/// Each end is unbounded, inclusive or exclusive, as described by [`Bound`].
/// The bounds themselves are finite. An interval contains only finite real
/// numbers: NaN and the infinities are never inside it, even at an unbounded
/// end.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    lower: Bound<f64>,
    upper: Bound<f64>,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the constructors are used by the domain modules from M2"
    )
)]
impl Interval {
    /// Creates the interval between `lower` and `upper`.
    ///
    /// The bounds must be finite, and the interval must not be empty. A
    /// violation is a defect in SabiQuant, caught by a debug assertion.
    pub(crate) const fn new(lower: Bound<f64>, upper: Bound<f64>) -> Self {
        debug_assert!(
            Self::is_valid(lower, upper),
            "interval bounds must be finite and describe a non-empty interval"
        );
        Self { lower, upper }
    }

    /// The interval `(value, +inf)`.
    pub(crate) const fn greater_than(value: f64) -> Self {
        Self::new(Bound::Excluded(value), Bound::Unbounded)
    }

    /// The interval `[value, +inf)`.
    pub(crate) const fn at_least(value: f64) -> Self {
        Self::new(Bound::Included(value), Bound::Unbounded)
    }

    /// The interval `(-inf, value)`.
    pub(crate) const fn less_than(value: f64) -> Self {
        Self::new(Bound::Unbounded, Bound::Excluded(value))
    }

    /// The interval `(-inf, value]`.
    pub(crate) const fn at_most(value: f64) -> Self {
        Self::new(Bound::Unbounded, Bound::Included(value))
    }

    const fn is_valid(lower: Bound<f64>, upper: Bound<f64>) -> bool {
        match (lower, upper) {
            (Bound::Unbounded, Bound::Unbounded) => true,
            (Bound::Included(end) | Bound::Excluded(end), Bound::Unbounded)
            | (Bound::Unbounded, Bound::Included(end) | Bound::Excluded(end)) => end.is_finite(),
            (Bound::Included(low), Bound::Included(high)) => {
                low.is_finite() && high.is_finite() && low <= high
            }
            (
                Bound::Included(low) | Bound::Excluded(low),
                Bound::Included(high) | Bound::Excluded(high),
            ) => low.is_finite() && high.is_finite() && low < high,
        }
    }
}

impl Interval {
    /// Returns the lower end of the interval.
    #[must_use]
    pub const fn lower(&self) -> Bound<f64> {
        self.lower
    }

    /// Returns the upper end of the interval.
    #[must_use]
    pub const fn upper(&self) -> Bound<f64> {
        self.upper
    }

    /// Returns whether `value` lies in the interval.
    ///
    /// NaN and the infinities are never contained. `-0.0` and `+0.0` are the
    /// same number here (NP-08).
    #[must_use]
    pub fn contains(&self, value: f64) -> bool {
        let above_lower = match self.lower {
            Bound::Unbounded => true,
            Bound::Included(lower) => value >= lower,
            Bound::Excluded(lower) => value > lower,
        };
        let below_upper = match self.upper {
            Bound::Unbounded => true,
            Bound::Included(upper) => value <= upper,
            Bound::Excluded(upper) => value < upper,
        };
        value.is_finite() && above_lower && below_upper
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.lower {
            Bound::Unbounded => f.write_str("(-inf")?,
            Bound::Included(lower) => write!(f, "[{lower:?}")?,
            Bound::Excluded(lower) => write!(f, "({lower:?}")?,
        }
        f.write_str(", ")?;
        match self.upper {
            Bound::Unbounded => f.write_str("+inf)"),
            Bound::Included(upper) => write!(f, "{upper:?}]"),
            Bound::Excluded(upper) => write!(f, "{upper:?})"),
        }
    }
}

/// Text that does not name exactly one identifier.
///
/// Conventions, frequencies, time units, compounding methods and calendars
/// each have exactly one canonical identifier and a documented list of
/// aliases (FR-CORE-05). Parsing accepts exactly those spellings: it is
/// case-sensitive and does not trim whitespace. A label that could name more
/// than one convention, such as `30/360` or `Actual/Actual`, is always
/// rejected rather than mapped to a default (FR-CORE-06).
///
/// # QuantLib correspondence
///
/// QuantLib has no equivalent: its convention names are display strings, and
/// it has no parser that rejects ambiguous labels.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum IdentifierError {
    /// The text is neither a canonical identifier nor a documented alias.
    ///
    /// Kind name: `identifier.unknown`.
    #[non_exhaustive]
    Unknown {
        /// What was being parsed, for example `"frequency"`.
        concept: &'static str,
        /// The rejected text, exactly as given.
        text: String,
    },
    /// The text could name more than one convention, so it is rejected.
    ///
    /// Kind name: `identifier.ambiguous`.
    #[non_exhaustive]
    Ambiguous {
        /// What was being parsed, for example `"day-count convention"`.
        concept: &'static str,
        /// The rejected text, exactly as given.
        text: String,
        /// The canonical identifiers of the conventions that the text could
        /// name.
        candidates: &'static [&'static str],
    },
}

impl IdentifierError {
    /// Returns the stable kind name of the error: `"identifier.unknown"` or
    /// `"identifier.ambiguous"`.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Unknown { .. } => "identifier.unknown",
            Self::Ambiguous { .. } => "identifier.ambiguous",
        }
    }
}

impl fmt::Display for IdentifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown { concept, text } => {
                write!(f, "unknown {concept} identifier {text:?}")
            }
            Self::Ambiguous {
                concept,
                text,
                candidates,
            } => {
                write!(
                    f,
                    "ambiguous {concept} identifier {text:?}; name one of the candidates: "
                )?;
                for (index, candidate) in candidates.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{candidate:?}")?;
                }
                Ok(())
            }
        }
    }
}

impl StdError for IdentifierError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amounts::{AmountError, DecimalAmount};
    use crate::bonds::{
        BondError, BondPrice, BondYieldError, GiltError, GiltYieldError, TreasuryError,
        TreasuryYieldError,
    };
    use crate::calendar::CalendarError;
    use crate::cashflows::CashFlowError;
    use crate::daycount::{DayCountError, IcmaError};
    use crate::math::{
        EvaluationCounts, EvaluationStage, InterpolationError, NormalError, RootBracket,
        SolverError, SolverOptions, SumError, bisect,
    };
    use crate::rates::RateError;
    use crate::schedule::{
        CdsMaturityConvention, DateGenerationRule, ScheduleError, SpecialRollRule,
    };
    use crate::settlement::SettlementError;
    use crate::time::{Date, DateError, Frequency, Period, PeriodError, TimeUnit, Weekday};

    const CANDIDATES: &[&str] = &["Actual/Actual (ISDA)", "Actual/Actual (ICMA)"];

    /// One error of every variant, with its documented kind name.
    fn every_variant() -> Vec<(Error, &'static str)> {
        let bracket = RootBracket::new(0.0, 1.0).unwrap();
        let options = SolverOptions::new(f64::from_bits(1), 0.0, 10).unwrap();
        vec![
            (
                ValueError::NotFinite {
                    parameter: "rate",
                    value: NonFinite::NaN,
                }
                .into(),
                "value.not_finite",
            ),
            (
                ValueError::OutOfDomain {
                    parameter: "time",
                    value: -0.5,
                    domain: Interval::at_least(0.0),
                }
                .into(),
                "value.out_of_domain",
            ),
            (
                ValueError::ResultNotFinite {
                    quantity: "compound factor",
                    value: NonFinite::PositiveInfinity,
                }
                .into(),
                "value.result_not_finite",
            ),
            (
                ValueError::ResultOutOfDomain {
                    quantity: "compound factor",
                    value: 0.0,
                    domain: Interval::greater_than(0.0),
                }
                .into(),
                "value.result_out_of_domain",
            ),
            (
                IdentifierError::Unknown {
                    concept: "frequency",
                    text: "Monthy".to_owned(),
                }
                .into(),
                "identifier.unknown",
            ),
            (
                IdentifierError::Ambiguous {
                    concept: "day-count convention",
                    text: "Actual/Actual".to_owned(),
                    candidates: CANDIDATES,
                }
                .into(),
                "identifier.ambiguous",
            ),
            (
                DateError::YearOutOfRange { year: 0 }.into(),
                "date.year_out_of_range",
            ),
            (
                DateError::InvalidDate {
                    year: 2023,
                    month: 2,
                    day: 29,
                }
                .into(),
                "date.invalid_date",
            ),
            (
                DateError::OutOfRange {
                    date: Date::MAX,
                    operation: "add days",
                    count: 1,
                }
                .into(),
                "date.out_of_range",
            ),
            (
                DateError::InvalidMonth { month: 13 }.into(),
                "date.invalid_month",
            ),
            (
                DateError::InvalidWeekday { weekday: 8 }.into(),
                "date.invalid_weekday",
            ),
            (
                DateError::InvalidOccurrence {
                    year: 2024,
                    month: 2,
                    nth: 5,
                    weekday: Weekday::Monday,
                }
                .into(),
                "date.invalid_occurrence",
            ),
            (
                DateError::Parse {
                    text: "not a date".to_owned(),
                }
                .into(),
                "date.parse",
            ),
            (
                PeriodError::Overflow {
                    operation: "negate",
                    length: 2_147_483_648,
                    unit: TimeUnit::Days,
                }
                .into(),
                "period.overflow",
            ),
            (
                PeriodError::IncompatibleUnits {
                    left: TimeUnit::Days,
                    right: TimeUnit::Months,
                }
                .into(),
                "period.incompatible_units",
            ),
            (
                PeriodError::Parse {
                    text: "1Y6M".to_owned(),
                }
                .into(),
                "period.parse",
            ),
            (
                CalendarError::Date(DateError::YearOutOfRange { year: 0 }).into(),
                "calendar.date",
            ),
            (
                CalendarError::NoBusinessWeekday.into(),
                "calendar.no_business_weekday",
            ),
            (CalendarError::EmptyJoint.into(), "calendar.empty_joint"),
            (
                CalendarError::SearchLimit {
                    start: Date::MIN,
                    direction: 1,
                    rejected: 366,
                }
                .into(),
                "calendar.search_limit",
            ),
            (
                CalendarError::NoBusinessDayInMonth { date: Date::MIN }.into(),
                "calendar.no_business_day_in_month",
            ),
            (
                CalendarError::InvalidRange {
                    start: Date::MAX,
                    end: Date::MIN,
                }
                .into(),
                "calendar.invalid_range",
            ),
            (
                CalendarError::NestingLimit {
                    depth: 65,
                    maximum: 64,
                }
                .into(),
                "calendar.nesting_limit",
            ),
            (
                DayCountError::Value(ValueError::ResultNotFinite {
                    quantity: "year fraction",
                    value: NonFinite::PositiveInfinity,
                })
                .into(),
                "daycount.value",
            ),
            (
                CalendarError::OutsideValidity { date: Date::MIN }.into(),
                "calendar.outside_validity",
            ),
            (
                CalendarError::UnverifiedCoverage { date: Date::MAX }.into(),
                "calendar.unverified_coverage",
            ),
            (
                DayCountError::Calendar(CalendarError::UnverifiedCoverage { date: Date::MAX })
                    .into(),
                "daycount.calendar",
            ),
            (
                IcmaError::InvalidPeriod {
                    start: Date::MAX,
                    end: Date::MIN,
                }
                .into(),
                "icma.invalid_period",
            ),
            (IcmaError::EmptyPeriods.into(), "icma.empty_periods"),
            (
                IcmaError::InvalidFrequency {
                    frequency: crate::time::Frequency::Once,
                }
                .into(),
                "icma.invalid_frequency",
            ),
            (
                IcmaError::NonContiguousPeriods {
                    index: 1,
                    previous_end: Date::MAX,
                    start: Date::MIN,
                }
                .into(),
                "icma.non_contiguous_periods",
            ),
            (
                IcmaError::OutOfCoverage {
                    start: Date::MIN,
                    end: Date::MAX,
                    coverage_start: Date::MAX,
                    coverage_end: Date::MAX,
                }
                .into(),
                "icma.out_of_coverage",
            ),
            (
                RateError::Value(ValueError::ResultNotFinite {
                    quantity: "compound factor",
                    value: NonFinite::PositiveInfinity,
                })
                .into(),
                "rate.value",
            ),
            (
                RateError::DayCount(DayCountError::Calendar(CalendarError::OutsideValidity {
                    date: Date::MIN,
                }))
                .into(),
                "rate.day_count",
            ),
            (RateError::Icma(IcmaError::EmptyPeriods).into(), "rate.icma"),
            (
                RateError::NumericalBoundary {
                    quantity: "compound factor",
                    logarithm: 709.782_712_893_384,
                }
                .into(),
                "rate.numerical_boundary",
            ),
            (
                RateError::InvalidFrequency {
                    frequency: crate::time::Frequency::Once,
                }
                .into(),
                "rate.invalid_frequency",
            ),
            (
                RateError::ReversedDates {
                    start: Date::MAX,
                    end: Date::MIN,
                }
                .into(),
                "rate.reversed_dates",
            ),
            (
                RateError::ZeroTime {
                    quantity: "compound factor",
                    factor: 2.0,
                }
                .into(),
                "rate.zero_time",
            ),
            (
                RateError::ZeroTargetTime {
                    source_time: 1.0,
                    rate: 0.05,
                }
                .into(),
                "rate.zero_target_time",
            ),
            (
                SumError::Input {
                    index: 2,
                    error: ValueError::NotFinite {
                        parameter: "values",
                        value: NonFinite::NaN,
                    },
                }
                .into(),
                "sum.input",
            ),
            (
                SumError::Arithmetic {
                    index: None,
                    error: ValueError::ResultNotFinite {
                        quantity: "exact sum",
                        value: NonFinite::PositiveInfinity,
                    },
                }
                .into(),
                "sum.arithmetic",
            ),
            (
                NormalError::Value(ValueError::NotFinite {
                    parameter: "normal variate",
                    value: NonFinite::NaN,
                })
                .into(),
                "normal.value",
            ),
            (
                NormalError::NumericalBoundary {
                    quantity: "normal density",
                    logarithm: 709.78,
                }
                .into(),
                "normal.numerical_boundary",
            ),
            (
                NormalError::Convergence {
                    quantity: "normal inverse probability",
                    iterations: 256,
                }
                .into(),
                "normal.convergence",
            ),
            (
                InterpolationError::NotEnoughNodes {
                    actual: 1,
                    minimum: 2,
                }
                .into(),
                "interpolation.not_enough_nodes",
            ),
            (
                InterpolationError::InvalidNode {
                    index: 1,
                    cause: ValueError::NotFinite {
                        parameter: "abscissa",
                        value: NonFinite::NaN,
                    },
                }
                .into(),
                "interpolation.invalid_node",
            ),
            (
                InterpolationError::NotStrictlyIncreasing {
                    index: 1,
                    previous: 1.0,
                    current: 1.0,
                }
                .into(),
                "interpolation.not_strictly_increasing",
            ),
            (
                InterpolationError::OutsideDomain {
                    query: -1.0,
                    lower: 0.0,
                    upper: 1.0,
                }
                .into(),
                "interpolation.outside_domain",
            ),
            (
                InterpolationError::NumericalGrid {
                    interval: 1,
                    quantity: "interval width",
                    cause: ValueError::ResultNotFinite {
                        quantity: "interval width",
                        value: NonFinite::PositiveInfinity,
                    },
                }
                .into(),
                "interpolation.numerical_grid",
            ),
            (
                InterpolationError::Value(ValueError::NotFinite {
                    parameter: "query",
                    value: NonFinite::NaN,
                })
                .into(),
                "interpolation.value",
            ),
            (
                SolverError::Value(ValueError::NotFinite {
                    parameter: "bracket lower",
                    value: NonFinite::NaN,
                })
                .into(),
                "solver.value",
            ),
            (
                SolverError::InvalidBracket {
                    lower: 1.0,
                    upper: 0.0,
                }
                .into(),
                "solver.invalid_bracket",
            ),
            (SolverError::InvalidBudget.into(), "solver.invalid_budget"),
            (
                bisect(|_| 1.0, bracket, options).unwrap_err().into(),
                "solver.not_bracketed",
            ),
            (
                bisect(|_| f64::NAN, bracket, options).unwrap_err().into(),
                "solver.non_finite_evaluation",
            ),
            (
                bisect(
                    |x| x - 0.5,
                    bracket,
                    SolverOptions::new(1e-12, 0.0, 1).unwrap(),
                )
                .unwrap_err()
                .into(),
                "solver.evaluation_limit",
            ),
            (
                bisect(
                    |x| {
                        if x.to_bits() == 1.0_f64.to_bits() {
                            -1.0
                        } else {
                            1.0
                        }
                    },
                    RootBracket::new(1.0, 1.0_f64.next_up()).unwrap(),
                    options,
                )
                .unwrap_err()
                .into(),
                "solver.precision_exhausted",
            ),
            (
                ScheduleError::InvalidCdsTenor {
                    trade_date: Date::MIN,
                    tenor: Period::new(0, TimeUnit::Days),
                    convention: CdsMaturityConvention::Semiannual2015,
                }
                .into(),
                "schedule.invalid_cds_tenor",
            ),
            (
                ScheduleError::CdsMaturityDate {
                    trade_date: Date::MAX,
                    tenor: Period::new(1, TimeUnit::Years),
                    convention: CdsMaturityConvention::Quarterly2009,
                    cause: Date::MAX.checked_add_months(15).unwrap_err(),
                }
                .into(),
                "schedule.cds_maturity_date",
            ),
            (
                ScheduleError::InvalidCdsMaturity {
                    maturity: Date::MAX,
                    convention: CdsMaturityConvention::Quarterly2009,
                }
                .into(),
                "schedule.invalid_cds_maturity",
            ),
            (
                ScheduleError::InvalidCdsCouponLimit {
                    requested: 0,
                    minimum: 1,
                    maximum: 39996,
                }
                .into(),
                "schedule.invalid_cds_coupon_limit",
            ),
            (
                ScheduleError::CdsCouponLimitExceeded {
                    maximum: 1,
                    required: 2,
                }
                .into(),
                "schedule.cds_coupon_limit_exceeded",
            ),
            (
                ScheduleError::NoEligibleCdsPayment {
                    trade_date: Date::from_ymd(2026, 3, 19).unwrap(),
                    payment_selection_cutoff: Date::from_ymd(2026, 3, 20).unwrap(),
                    unadjusted_maturity: Date::from_ymd(2026, 3, 20).unwrap(),
                    final_payment: Date::from_ymd(2026, 3, 20).unwrap(),
                }
                .into(),
                "schedule.no_eligible_cds_payment",
            ),
            (
                ScheduleError::CdsDate {
                    trade_date: Date::MIN,
                    unadjusted_maturity: Date::MAX,
                    role: "cutoff quarter",
                    input: Date::MIN,
                    cause: Date::MIN.checked_add_months(-1).unwrap_err(),
                }
                .into(),
                "schedule.cds_date",
            ),
            (
                ScheduleError::CdsCalendar {
                    trade_date: Date::from_ymd(9999, 12, 1).unwrap(),
                    unadjusted_maturity: Date::from_ymd(9999, 12, 20).unwrap(),
                    phase: "coupon adjustment",
                    index: Some(0),
                    start: Date::from_ymd(9999, 12, 20).unwrap(),
                    end: Date::from_ymd(9999, 12, 20).unwrap(),
                    cause: CalendarError::Date(Date::MAX.checked_add_days(1).unwrap_err()),
                }
                .into(),
                "schedule.cds_calendar",
            ),
            (
                ScheduleError::CdsAdjustedOrder {
                    index: 0,
                    previous_contractual: Date::from_ymd(2026, 3, 20).unwrap(),
                    contractual: Date::from_ymd(2026, 6, 20).unwrap(),
                    previous_adjusted: Date::from_ymd(2026, 6, 22).unwrap(),
                    adjusted: Date::from_ymd(2026, 6, 22).unwrap(),
                }
                .into(),
                "schedule.cds_adjusted_order",
            ),
            (
                ScheduleError::InvalidRollTenor {
                    rule: SpecialRollRule::TwentiethIMM,
                    tenor: Period::new(1, TimeUnit::Months),
                }
                .into(),
                "schedule.invalid_roll_tenor",
            ),
            (
                ScheduleError::ResolvedRollRange {
                    rule: SpecialRollRule::ThirdWednesdayInclusive,
                    tenor: Period::new(1, TimeUnit::Months),
                    requested_start: Date::from_ymd(2026, 1, 1).unwrap(),
                    requested_end: Date::from_ymd(2026, 1, 31).unwrap(),
                    resolved_start: Date::from_ymd(2026, 1, 21).unwrap(),
                    resolved_end: Date::from_ymd(2026, 1, 21).unwrap(),
                }
                .into(),
                "schedule.resolved_roll_range",
            ),
            (
                ScheduleError::RollDate {
                    rule: SpecialRollRule::Twentieth,
                    tenor: Period::new(1, TimeUnit::Months),
                    role: "end",
                    input: Date::MAX,
                    cause: Date::MAX.checked_add_months(1).unwrap_err(),
                }
                .into(),
                "schedule.roll_date",
            ),
            (
                ScheduleError::InvalidOldCdsTenor {
                    generation_start: Date::MIN,
                    requested_end: Date::MAX,
                    tenor: Period::new(1, TimeUnit::Weeks),
                }
                .into(),
                "schedule.invalid_old_cds_tenor",
            ),
            (
                ScheduleError::OldCdsDate {
                    generation_start: Date::MIN,
                    requested_end: Date::MAX,
                    tenor: Period::new(3, TimeUnit::Months),
                    role: "terminal quarter",
                    input: Date::MAX,
                    cause: Date::MAX.checked_add_months(3).unwrap_err(),
                }
                .into(),
                "schedule.old_cds_date",
            ),
            (
                ScheduleError::InvalidRange {
                    start: Date::MAX,
                    end: Date::MIN,
                }
                .into(),
                "schedule.invalid_range",
            ),
            (
                ScheduleError::InvalidBoundaryLimit {
                    requested: 1,
                    minimum: 2,
                    maximum: 3_652_059,
                }
                .into(),
                "schedule.invalid_boundary_limit",
            ),
            (
                ScheduleError::InvalidRule {
                    rule: DateGenerationRule::Zero,
                }
                .into(),
                "schedule.invalid_rule",
            ),
            (
                ScheduleError::InvalidTenor {
                    tenor: Period::new(0, TimeUnit::Days),
                }
                .into(),
                "schedule.invalid_tenor",
            ),
            (
                ScheduleError::IncompatibleEndOfMonth {
                    tenor: Period::new(1, TimeUnit::Days),
                    end_of_month: crate::calendar::EndOfMonth::Preserve,
                }
                .into(),
                "schedule.incompatible_end_of_month",
            ),
            (
                ScheduleError::EmptyExplicitStubDates.into(),
                "schedule.empty_explicit_stub_dates",
            ),
            (
                ScheduleError::ReversedExplicitStubDates {
                    first: Date::MAX,
                    penultimate: Date::MIN,
                }
                .into(),
                "schedule.reversed_explicit_stub_dates",
            ),
            (
                ScheduleError::ExplicitStubOutsideRange {
                    role: "first",
                    date: Date::MIN,
                    start: Date::MIN,
                    end: Date::MAX,
                }
                .into(),
                "schedule.explicit_stub_outside_range",
            ),
            (
                ScheduleError::IrregularExplicitMiddle {
                    rule: DateGenerationRule::Forward,
                    tenor: Period::new(2, TimeUnit::Days),
                    anchor: Date::MIN,
                    endpoint: Date::MIN.checked_add_days(3).unwrap(),
                    regular_boundary: Date::MIN.checked_add_days(2).unwrap(),
                }
                .into(),
                "schedule.irregular_explicit_middle",
            ),
            (
                ScheduleError::InvalidIcmaPeriodLimit {
                    requested: 0,
                    minimum: 1,
                    maximum: 3_652_058,
                }
                .into(),
                "schedule.invalid_icma_period_limit",
            ),
            (
                ScheduleError::NonPeriodicIcma {
                    rule: DateGenerationRule::Zero,
                }
                .into(),
                "schedule.non_periodic_icma",
            ),
            (
                ScheduleError::InvalidIcmaFrequency {
                    frequency: Frequency::Once,
                }
                .into(),
                "schedule.invalid_icma_frequency",
            ),
            (
                ScheduleError::IcmaFrequencyMismatch {
                    tenor: Period::new(2, TimeUnit::Years),
                    frequency: Frequency::Annual,
                    expected_tenor: Period::new(1, TimeUnit::Years),
                }
                .into(),
                "schedule.icma_frequency_mismatch",
            ),
            (
                ScheduleError::IcmaPeriodLimitExceeded {
                    maximum: 1,
                    required_at_least: 2,
                }
                .into(),
                "schedule.icma_period_limit_exceeded",
            ),
            (
                ScheduleError::Icma {
                    cause: IcmaError::EmptyPeriods,
                }
                .into(),
                "schedule.icma",
            ),
            (
                ScheduleError::StubRequired {
                    rule: DateGenerationRule::Forward,
                    tenor: Period::new(2, TimeUnit::Days),
                    start: Date::MIN,
                    end: Date::MIN.checked_add_days(3).unwrap(),
                    regular_boundary: Date::MIN.checked_add_days(2).unwrap(),
                }
                .into(),
                "schedule.stub_required",
            ),
            (
                ScheduleError::LongStubWithoutRegularPeriod {
                    rule: DateGenerationRule::Forward,
                    tenor: Period::new(2, TimeUnit::Days),
                    start: Date::MIN,
                    end: Date::MIN.checked_add_days(1).unwrap(),
                }
                .into(),
                "schedule.long_stub_without_regular_period",
            ),
            (
                ScheduleError::BoundaryLimitExceeded {
                    maximum: 2,
                    required_at_least: 3,
                }
                .into(),
                "schedule.boundary_limit_exceeded",
            ),
            (
                ScheduleError::AllocationFailed {
                    phase: "generation",
                    requested: 3,
                }
                .into(),
                "schedule.allocation_failed",
            ),
            (
                ScheduleError::Date {
                    index: 1,
                    phase: "generation",
                    cause: DateError::OutOfRange {
                        date: Date::MAX,
                        operation: "add days",
                        count: 1,
                    },
                }
                .into(),
                "schedule.date",
            ),
            (
                ScheduleError::Calendar {
                    index: None,
                    phase: "coverage",
                    cause: CalendarError::UnverifiedCoverage { date: Date::MIN },
                }
                .into(),
                "schedule.calendar",
            ),
            (
                ScheduleError::AdjustedOrder {
                    index: 1,
                    previous_contractual: Date::MIN,
                    contractual: Date::MIN.checked_add_days(1).unwrap(),
                    previous_adjusted: Date::MIN,
                    adjusted: Date::MIN,
                }
                .into(),
                "schedule.adjusted_order",
            ),
            (
                CashFlowError::InvalidAccrualPeriod {
                    start: Date::MIN,
                    end: Date::MIN,
                }
                .into(),
                "cashflow.invalid_accrual_period",
            ),
            (
                CashFlowError::InvalidFlowLimit {
                    requested: u32::MAX,
                    maximum: 3_652_059,
                }
                .into(),
                "cashflow.invalid_flow_limit",
            ),
            (
                CashFlowError::FlowLimitExceeded {
                    maximum: 0,
                    required: 1,
                }
                .into(),
                "cashflow.flow_limit_exceeded",
            ),
            (
                CashFlowError::PaymentOrder {
                    index: 1,
                    previous: Date::MAX,
                    current: Date::MIN,
                }
                .into(),
                "cashflow.payment_order",
            ),
            (
                CashFlowError::AllocationFailed {
                    phase: "NPV terms",
                    requested: 10,
                }
                .into(),
                "cashflow.allocation_failed",
            ),
            (
                CashFlowError::AccrualWorkLimitExceeded {
                    index: Some(2),
                    phase: "discount horizon",
                    maximum: 3,
                    used: 2,
                    required_at_least: 5,
                }
                .into(),
                "cashflow.accrual_work_limit_exceeded",
            ),
            (
                CashFlowError::Value {
                    index: None,
                    phase: "coupon quote",
                    cause: ValueError::NotFinite {
                        parameter: "coupon rate",
                        value: NonFinite::NaN,
                    },
                }
                .into(),
                "cashflow.value",
            ),
            (
                CashFlowError::Rate {
                    index: Some(1),
                    phase: "discount horizon",
                    cause: RateError::Icma(IcmaError::OutOfCoverage {
                        start: Date::MIN,
                        end: Date::MAX,
                        coverage_start: Date::MIN,
                        coverage_end: Date::MIN.checked_add_days(1).unwrap(),
                    }),
                }
                .into(),
                "cashflow.rate",
            ),
            (
                CashFlowError::Sum {
                    phase: "NPV reduction",
                    cause: SumError::Arithmetic {
                        index: None,
                        error: ValueError::ResultNotFinite {
                            quantity: "exact sum",
                            value: NonFinite::PositiveInfinity,
                        },
                    },
                }
                .into(),
                "cashflow.sum",
            ),
            (
                AmountError::TextTooLong {
                    length: 129,
                    maximum: 128,
                }
                .into(),
                "amount.text_too_long",
            ),
            (
                AmountError::InvalidSyntax {
                    index: 1,
                    reason: "unexpected byte",
                }
                .into(),
                "amount.invalid_syntax",
            ),
            (
                AmountError::InvalidScale {
                    scale: 39,
                    maximum: 38,
                }
                .into(),
                "amount.invalid_scale",
            ),
            (
                AmountError::InvalidQuantum {
                    coefficient: -1,
                    scale: 2,
                }
                .into(),
                "amount.invalid_quantum",
            ),
            (
                AmountError::ZeroDenominator { operation: "ratio" }.into(),
                "amount.zero_denominator",
            ),
            (
                AmountError::Inexact {
                    operation: "quantize",
                }
                .into(),
                "amount.inexact",
            ),
            (
                AmountError::ResultScale {
                    operation: "multiply",
                    scale: 76,
                    maximum: 38,
                }
                .into(),
                "amount.result_scale",
            ),
            (
                AmountError::OutOfRange {
                    operation: "sum",
                    negative: true,
                }
                .into(),
                "amount.out_of_range",
            ),
            (
                AmountError::NonFinite {
                    operation: "from_binary64_exact",
                    value: NonFinite::NaN,
                }
                .into(),
                "amount.non_finite",
            ),
            (
                BondError::InvalidFlowLimit {
                    requested: u32::MAX,
                    maximum: 3_652_059,
                }
                .into(),
                "bond.invalid_flow_limit",
            ),
            (
                BondError::FlowLimitExceeded {
                    maximum: 1,
                    required: 2,
                }
                .into(),
                "bond.flow_limit_exceeded",
            ),
            (
                BondError::ExCouponCountMismatch {
                    expected: 2,
                    actual: 1,
                }
                .into(),
                "bond.ex_coupon_count_mismatch",
            ),
            (
                BondError::InvalidIssueMaturity {
                    issue_date: Date::MAX,
                    contractual_maturity: Date::MIN,
                }
                .into(),
                "bond.invalid_issue_maturity",
            ),
            (
                BondError::RedemptionBeforeIssue {
                    issue_date: Date::MAX,
                    redemption_payment: Date::MIN,
                }
                .into(),
                "bond.redemption_before_issue",
            ),
            (
                BondError::RedemptionBeforeLastCoupon {
                    last_coupon_payment: Date::MAX,
                    redemption_payment: Date::MIN,
                }
                .into(),
                "bond.redemption_before_last_coupon",
            ),
            (
                BondError::ExCouponAfterPayment {
                    index: 0,
                    cutoff_date: Date::MAX,
                    payment_date: Date::MIN,
                }
                .into(),
                "bond.ex_coupon_after_payment",
            ),
            (
                BondError::SettlementBeforeIssue {
                    settlement_date: Date::MIN,
                    issue_date: Date::MAX,
                }
                .into(),
                "bond.settlement_before_issue",
            ),
            (
                BondError::AllocationFailed {
                    phase: "fixed-rate bond coupons",
                    requested: 8,
                }
                .into(),
                "bond.allocation_failed",
            ),
            (
                BondError::AccrualWorkLimitExceeded {
                    index: Some(0),
                    phase: "partial accrual",
                    maximum: 1,
                    used: 1,
                    required_at_least: 2,
                }
                .into(),
                "bond.accrual_work_limit_exceeded",
            ),
            (
                BondError::Value {
                    index: None,
                    phase: "bond face",
                    cause: ValueError::NotFinite {
                        parameter: "bond face",
                        value: NonFinite::NaN,
                    },
                }
                .into(),
                "bond.value",
            ),
            (
                BondError::Rate {
                    index: Some(0),
                    phase: "discount factor",
                    cause: RateError::Icma(IcmaError::OutOfCoverage {
                        start: Date::MIN,
                        end: Date::MAX,
                        coverage_start: Date::MIN,
                        coverage_end: Date::MIN.checked_add_days(1).unwrap(),
                    }),
                }
                .into(),
                "bond.rate",
            ),
            (
                BondError::Sum {
                    phase: "dirty price reduction",
                    cause: SumError::Arithmetic {
                        index: None,
                        error: ValueError::ResultNotFinite {
                            quantity: "exact sum",
                            value: NonFinite::PositiveInfinity,
                        },
                    },
                }
                .into(),
                "bond.sum",
            ),
            (
                BondError::ZeroDirtyPrice {
                    measure: "macaulay_duration",
                    yield_rate: 0.0,
                }
                .into(),
                "bond.zero_dirty_price",
            ),
            (
                BondError::InternalCapacity {
                    index: Some(0),
                    phase: "risk accumulation",
                    operation: "accumulate",
                }
                .into(),
                "bond.internal_capacity",
            ),
            (
                BondYieldError::Target {
                    target: BondPrice::new(100.0, crate::bonds::BondPriceKind::Dirty).unwrap(),
                    phase: "target conversion",
                    cause: BondError::Value {
                        index: None,
                        phase: "dirty price",
                        cause: ValueError::NotFinite {
                            parameter: "dirty price",
                            value: NonFinite::NaN,
                        },
                    },
                }
                .into(),
                "bond_yield.target",
            ),
            (
                BondYieldError::Preparation {
                    phase: "yield present value terms",
                    cause: BondError::AllocationFailed {
                        phase: "yield present value terms",
                        requested: 8,
                    },
                }
                .into(),
                "bond_yield.preparation",
            ),
            (
                BondYieldError::Solver(SolverError::InvalidBudget).into(),
                "bond_yield.solver",
            ),
            (
                BondYieldError::Callback {
                    stage: EvaluationStage::LowerEndpoint,
                    point: 0.05,
                    evaluations: EvaluationCounts {
                        function: 1,
                        derivative: 0,
                    },
                    cause: BondError::Sum {
                        phase: "dirty price reduction",
                        cause: SumError::Arithmetic {
                            index: None,
                            error: ValueError::ResultNotFinite {
                                quantity: "exact sum",
                                value: NonFinite::PositiveInfinity,
                            },
                        },
                    },
                }
                .into(),
                "bond_yield.callback",
            ),
            (
                BondYieldError::NoYieldSensitivity {
                    bracket,
                    dirty_target_amount: 100.0,
                    observed_lower_dirty_amount: 100.0,
                    constant_amount: Some(100.0),
                    evaluations: EvaluationCounts {
                        function: 1,
                        derivative: 0,
                    },
                }
                .into(),
                "bond_yield.no_yield_sensitivity",
            ),
            (
                BondYieldError::ConstantPriceMismatch {
                    bracket,
                    dirty_target_amount: 99.0,
                    observed_lower_dirty_amount: 100.0,
                    constant_amount: Some(100.0),
                    evaluations: EvaluationCounts {
                        function: 1,
                        derivative: 0,
                    },
                }
                .into(),
                "bond_yield.constant_price_mismatch",
            ),
            // FR-GILT-20: these error-value samples test kind/format/source,
            // including defensive variants, without constructing private
            // financial reports or claiming reachable capacity/zero-dirty states.
            (
                GiltError::InvalidDateCap {
                    cap: 0,
                    minimum: 2,
                    maximum: 20_000,
                }
                .into(),
                "gilt.invalid_date_cap",
            ),
            (
                GiltError::TooManyQuasiDates { count: 3, cap: 2 }.into(),
                "gilt.too_many_quasi_dates",
            ),
            (
                GiltError::TooFewQuasiDates {
                    count: 1,
                    minimum: 2,
                }
                .into(),
                "gilt.too_few_quasi_dates",
            ),
            (
                GiltError::NegativeCoupon {
                    coupon_per_100: DecimalAmount::new(-1, 0).unwrap(),
                }
                .into(),
                "gilt.negative_coupon",
            ),
            (
                GiltError::IssueNotBeforeMaturity {
                    issue_date: Date::MIN,
                    maturity_date: Date::MIN,
                }
                .into(),
                "gilt.issue_not_before_maturity",
            ),
            (
                GiltError::MaturityBoundary {
                    maturity_date: Date::MAX,
                    last_quasi_date: Date::MIN,
                }
                .into(),
                "gilt.maturity_boundary",
            ),
            (
                GiltError::QuasiDateOrder {
                    index: 1,
                    previous: Date::MAX,
                    date: Date::MIN,
                }
                .into(),
                "gilt.quasi_date_order",
            ),
            (
                GiltError::QuasiCoverage {
                    issue_date: Date::MIN,
                    first_quasi_date: Date::MIN,
                    next_quasi_date: Date::MIN,
                }
                .into(),
                "gilt.quasi_coverage",
            ),
            (
                GiltError::QuasiDateMismatch {
                    index: 1,
                    supplied: Date::MIN,
                    expected: Date::MAX,
                    maturity_date: Date::MAX,
                    end_of_month: crate::calendar::EndOfMonth::Ignore,
                }
                .into(),
                "gilt.quasi_date_mismatch",
            ),
            (
                GiltError::Date {
                    index: Some(0),
                    phase: "quasi cycle",
                    cause: DateError::YearOutOfRange { year: 0 },
                }
                .into(),
                "gilt.date",
            ),
            (
                GiltError::FirstDividend {
                    issue_date: Date::MIN,
                    first_dividend_date: Date::MAX,
                    next_quasi_date: Date::MIN,
                    second_quasi_date: None,
                }
                .into(),
                "gilt.first_dividend",
            ),
            (
                GiltError::ExDateCount {
                    count: 0,
                    expected: 1,
                }
                .into(),
                "gilt.ex_date_count",
            ),
            (
                GiltError::ExDateWindow {
                    dividend_index: 0,
                    cutoff: Date::MAX,
                    lower: Date::MIN,
                    dividend_date: Date::MIN,
                }
                .into(),
                "gilt.ex_date_window",
            ),
            (
                GiltError::InvalidBundleCap {
                    cap: 20_001,
                    maximum: 20_000,
                }
                .into(),
                "gilt.invalid_bundle_cap",
            ),
            (
                GiltError::TooManyBundles { count: 1, cap: 0 }.into(),
                "gilt.too_many_bundles",
            ),
            (
                GiltError::SettlementBeforeIssue {
                    settlement_date: Date::MIN,
                    issue_date: Date::MAX,
                }
                .into(),
                "gilt.settlement_before_issue",
            ),
            (
                GiltError::SettlementAtOrAfterMaturity {
                    settlement_date: Date::MAX,
                    maturity_date: Date::MAX,
                }
                .into(),
                "gilt.settlement_at_or_after_maturity",
            ),
            (
                GiltError::InvalidDividendIndex { index: 1, count: 1 }.into(),
                "gilt.invalid_dividend_index",
            ),
            (
                GiltError::Value {
                    index: Some(1),
                    phase: "source present value",
                    cause: ValueError::ResultNotFinite {
                        quantity: "gilt present value",
                        value: NonFinite::PositiveInfinity,
                    },
                }
                .into(),
                "gilt.value",
            ),
            (
                GiltError::Rate {
                    index: Some(1),
                    phase: "discount factor",
                    cause: RateError::NumericalBoundary {
                        quantity: "discount factor",
                        logarithm: 709.0,
                    },
                }
                .into(),
                "gilt.rate",
            ),
            (
                GiltError::Sum {
                    phase: "dirty price reduction",
                    cause: SumError::Arithmetic {
                        index: None,
                        error: ValueError::ResultNotFinite {
                            quantity: "sum",
                            value: NonFinite::PositiveInfinity,
                        },
                    },
                }
                .into(),
                "gilt.sum",
            ),
            (
                GiltError::AllocationFailed {
                    phase: "present value terms",
                    requested: 2,
                }
                .into(),
                "gilt.allocation_failed",
            ),
            (
                GiltError::InternalCapacity {
                    index: None,
                    phase: "convexity",
                    operation: "normalize risk value",
                }
                .into(),
                "gilt.internal_capacity",
            ),
            (
                GiltError::ZeroDirtyPrice {
                    measure: "convexity",
                    yield_rate: 0.0,
                }
                .into(),
                "gilt.zero_dirty_price",
            ),
            (
                GiltError::NonPositiveNominal {
                    nominal: DecimalAmount::new(0, 0).unwrap(),
                }
                .into(),
                "gilt.nonpositive_nominal",
            ),
            (
                GiltError::CashArithmetic {
                    stage: "cash capital",
                    endpoint: "point",
                    cause: AmountError::Inexact {
                        operation: "quantize",
                    },
                }
                .into(),
                "gilt.cash_arithmetic",
            ),
            (
                GiltYieldError::Target {
                    target: BondPrice::new(100.0, crate::bonds::BondPriceKind::Clean).unwrap(),
                    phase: "target clean shift",
                    cause: GiltError::Sum {
                        phase: "yield target shift",
                        cause: SumError::Arithmetic {
                            index: None,
                            error: ValueError::ResultNotFinite {
                                quantity: "sum",
                                value: NonFinite::PositiveInfinity,
                            },
                        },
                    },
                }
                .into(),
                "gilt_yield.target",
            ),
            (
                GiltYieldError::Preparation {
                    phase: "yield present value terms",
                    cause: GiltError::AllocationFailed {
                        phase: "yield present value terms",
                        requested: 1,
                    },
                }
                .into(),
                "gilt_yield.preparation",
            ),
            (
                GiltYieldError::Callback {
                    stage: EvaluationStage::LowerEndpoint,
                    point: -2.0,
                    evaluations: EvaluationCounts {
                        function: 1,
                        derivative: 0,
                    },
                    cause: GiltError::Rate {
                        index: None,
                        phase: "yield quote",
                        cause: RateError::Value(ValueError::OutOfDomain {
                            parameter: "rate",
                            value: -2.0,
                            domain: Interval::greater_than(-2.0),
                        }),
                    },
                }
                .into(),
                "gilt_yield.callback",
            ),
            (
                GiltYieldError::NonPositiveTarget {
                    target: BondPrice::new(0.0, crate::bonds::BondPriceKind::Dirty).unwrap(),
                    dirty_target_per_100: 0.0,
                    accrued_per_100: 0.0,
                    evaluations: EvaluationCounts {
                        function: 0,
                        derivative: 0,
                    },
                }
                .into(),
                "gilt_yield.nonpositive_target",
            ),
            (
                GiltYieldError::Solver(SolverError::InvalidBudget).into(),
                "gilt_yield.solver",
            ),
            // FR-UST-23: every error-value variant is sampled as an error
            // value, without forging private financial reports or claiming
            // allocation/capacity/zero-dirty reachability.
            (
                TreasuryError::InvalidDateCap {
                    cap: 0,
                    minimum: 2,
                    maximum: 20_000,
                }
                .into(),
                "treasury.invalid_date_cap",
            ),
            (
                TreasuryError::TooManyQuasiDates { count: 3, cap: 2 }.into(),
                "treasury.too_many_quasi_dates",
            ),
            (
                TreasuryError::TooFewQuasiDates {
                    count: 1,
                    minimum: 2,
                }
                .into(),
                "treasury.too_few_quasi_dates",
            ),
            (
                TreasuryError::NegativeCoupon {
                    coupon_per_100: DecimalAmount::new(-1, 0).unwrap(),
                }
                .into(),
                "treasury.negative_coupon",
            ),
            (
                TreasuryError::IssueNotBeforeMaturity {
                    original_issue_date: Date::MIN,
                    maturity_date: Date::MIN,
                }
                .into(),
                "treasury.issue_not_before_maturity",
            ),
            (
                TreasuryError::MaturityBoundary {
                    maturity_date: Date::MAX,
                    last_quasi_date: Date::MIN,
                }
                .into(),
                "treasury.maturity_boundary",
            ),
            (
                TreasuryError::QuasiDateOrder {
                    index: 1,
                    previous: Date::MAX,
                    date: Date::MIN,
                }
                .into(),
                "treasury.quasi_date_order",
            ),
            (
                TreasuryError::QuasiCoverage {
                    original_dated_date: Date::MIN,
                    first_quasi_date: Date::MIN,
                    next_quasi_date: Date::MIN,
                }
                .into(),
                "treasury.quasi_coverage",
            ),
            (
                TreasuryError::QuasiDateMismatch {
                    index: 1,
                    supplied: Date::MIN,
                    expected: Date::MAX,
                    maturity_date: Date::MAX,
                    end_of_month: crate::calendar::EndOfMonth::Ignore,
                }
                .into(),
                "treasury.quasi_date_mismatch",
            ),
            (
                TreasuryError::Date {
                    index: Some(0),
                    phase: "quasi cycle",
                    cause: DateError::YearOutOfRange { year: 0 },
                }
                .into(),
                "treasury.date",
            ),
            (
                TreasuryError::FirstCoupon {
                    original_dated_date: Date::MIN,
                    original_issue_date: Date::MIN,
                    first_coupon_date: Date::MAX,
                    next_quasi_date: Date::MIN,
                    second_quasi_date: None,
                }
                .into(),
                "treasury.first_coupon",
            ),
            (
                TreasuryError::InvalidBundleCap {
                    cap: 20_001,
                    maximum: 20_000,
                }
                .into(),
                "treasury.invalid_bundle_cap",
            ),
            (
                TreasuryError::TooManyBundles { count: 1, cap: 0 }.into(),
                "treasury.too_many_bundles",
            ),
            (
                TreasuryError::SettlementBeforeIssue {
                    settlement_date: Date::MIN,
                    original_issue_date: Date::MAX,
                }
                .into(),
                "treasury.settlement_before_issue",
            ),
            (
                TreasuryError::SettlementAtOrAfterMaturity {
                    settlement_date: Date::MAX,
                    maturity_date: Date::MAX,
                }
                .into(),
                "treasury.settlement_at_or_after_maturity",
            ),
            (
                TreasuryError::InvalidCouponIndex { index: 1, count: 1 }.into(),
                "treasury.invalid_coupon_index",
            ),
            (
                TreasuryError::Value {
                    index: Some(1),
                    phase: "source present value",
                    cause: ValueError::ResultNotFinite {
                        quantity: "treasury present value",
                        value: NonFinite::PositiveInfinity,
                    },
                }
                .into(),
                "treasury.value",
            ),
            (
                TreasuryError::Sum {
                    phase: "dirty price reduction",
                    cause: SumError::Arithmetic {
                        index: None,
                        error: ValueError::ResultNotFinite {
                            quantity: "sum",
                            value: NonFinite::PositiveInfinity,
                        },
                    },
                }
                .into(),
                "treasury.sum",
            ),
            (
                TreasuryError::AllocationFailed {
                    phase: "present value terms",
                    requested: 2,
                }
                .into(),
                "treasury.allocation_failed",
            ),
            (
                TreasuryError::InternalCapacity {
                    index: None,
                    phase: "convexity",
                    operation: "normalize risk value",
                }
                .into(),
                "treasury.internal_capacity",
            ),
            (
                TreasuryError::ZeroDirtyPrice {
                    measure: "convexity",
                    yield_rate: 0.0,
                }
                .into(),
                "treasury.zero_dirty_price",
            ),
            (
                TreasuryError::NonPositiveNominal {
                    nominal: DecimalAmount::new(0, 0).unwrap(),
                }
                .into(),
                "treasury.nonpositive_nominal",
            ),
            (
                TreasuryError::CashArithmetic {
                    stage: "cash capital",
                    endpoint: "point",
                    cause: AmountError::Inexact {
                        operation: "quantize",
                    },
                }
                .into(),
                "treasury.cash_arithmetic",
            ),
            (
                TreasuryYieldError::Target {
                    target: BondPrice::new(100.0, crate::bonds::BondPriceKind::Clean).unwrap(),
                    phase: "target clean shift",
                    cause: TreasuryError::Sum {
                        phase: "yield target shift",
                        cause: SumError::Arithmetic {
                            index: None,
                            error: ValueError::ResultNotFinite {
                                quantity: "sum",
                                value: NonFinite::PositiveInfinity,
                            },
                        },
                    },
                }
                .into(),
                "treasury_yield.target",
            ),
            (
                TreasuryYieldError::Preparation {
                    phase: "yield present value terms",
                    cause: TreasuryError::AllocationFailed {
                        phase: "yield present value terms",
                        requested: 1,
                    },
                }
                .into(),
                "treasury_yield.preparation",
            ),
            (
                TreasuryYieldError::Callback {
                    stage: EvaluationStage::LowerEndpoint,
                    point: -2.0,
                    evaluations: EvaluationCounts {
                        function: 1,
                        derivative: 0,
                    },
                    cause: TreasuryError::Value {
                        index: None,
                        phase: "yield quote",
                        cause: ValueError::OutOfDomain {
                            parameter: "rate",
                            value: -2.0,
                            domain: Interval::greater_than(-2.0),
                        },
                    },
                }
                .into(),
                "treasury_yield.callback",
            ),
            (
                TreasuryYieldError::NonPositiveTarget {
                    target: BondPrice::new(0.0, crate::bonds::BondPriceKind::Dirty).unwrap(),
                    dirty_target_per_100: 0.0,
                    accrued_per_100: 0.0,
                    source_policy: crate::bonds::TreasurySourcePolicy::new(
                        crate::bonds::TreasuryDailyBasis::ExactRatio,
                        crate::bonds::TreasuryTieRule::NearestTiesEven,
                    ),
                    evaluations: EvaluationCounts {
                        function: 0,
                        derivative: 0,
                    },
                }
                .into(),
                "treasury_yield.nonpositive_target",
            ),
            (
                TreasuryYieldError::Solver(SolverError::InvalidBudget).into(),
                "treasury_yield.solver",
            ),
            (
                TreasuryError::DatedAfterIssue {
                    original_dated_date: Date::MAX,
                    original_issue_date: Date::MIN,
                }
                .into(),
                "treasury.dated_after_issue",
            ),
            (
                SettlementError::NonPositiveNominal {
                    nominal: DecimalAmount::new(0, 0).unwrap(),
                }
                .into(),
                "settlement.nonpositive_nominal",
            ),
            (
                SettlementError::DecimalIntervalOrder {
                    lower: DecimalAmount::new(1, 0).unwrap(),
                    upper: DecimalAmount::new(0, 0).unwrap(),
                }
                .into(),
                "settlement.decimal_interval_order",
            ),
            (
                SettlementError::BinaryIntervalOrder {
                    lower_bits: 1.0_f64.to_bits(),
                    upper_bits: 0,
                }
                .into(),
                "settlement.binary_interval_order",
            ),
            (
                SettlementError::NonFinite {
                    endpoint: "value",
                    value: NonFinite::NaN,
                }
                .into(),
                "settlement.non_finite",
            ),
            (
                SettlementError::Arithmetic {
                    stage: "cash total",
                    endpoint: "point",
                    cause: AmountError::Inexact {
                        operation: "quantize",
                    },
                }
                .into(),
                "settlement.arithmetic",
            ),
            // An error-value catalog sample tests kind/format/source only.
            // It does not forge private arithmetic state or claim that a
            // valid public input can exhaust the proven forty-limb capacity.
            (
                SettlementError::InternalCapacity {
                    stage: "cash total",
                    operation: "cash rounding",
                }
                .into(),
                "settlement.internal_capacity",
            ),
        ]
    }

    /// Exhaustive over the variants inside this crate: adding a variant
    /// breaks the build here until `every_variant` covers it.
    fn variant_name(error: &Error) -> &'static str {
        match error {
            Error::Amount(AmountError::TextTooLong { .. }) => "AmountTextTooLong",
            Error::Amount(AmountError::InvalidSyntax { .. }) => "AmountInvalidSyntax",
            Error::Amount(AmountError::InvalidScale { .. }) => "AmountInvalidScale",
            Error::Amount(AmountError::InvalidQuantum { .. }) => "AmountInvalidQuantum",
            Error::Amount(AmountError::ZeroDenominator { .. }) => "AmountZeroDenominator",
            Error::Amount(AmountError::Inexact { .. }) => "AmountInexact",
            Error::Amount(AmountError::ResultScale { .. }) => "AmountResultScale",
            Error::Amount(AmountError::OutOfRange { .. }) => "AmountOutOfRange",
            Error::Amount(AmountError::NonFinite { .. }) => "AmountNonFinite",
            Error::Value(ValueError::NotFinite { .. }) => "NotFinite",
            Error::Value(ValueError::OutOfDomain { .. }) => "OutOfDomain",
            Error::Value(ValueError::ResultNotFinite { .. }) => "ResultNotFinite",
            Error::Value(ValueError::ResultOutOfDomain { .. }) => "ResultOutOfDomain",
            Error::Identifier(IdentifierError::Unknown { .. }) => "Unknown",
            Error::Identifier(IdentifierError::Ambiguous { .. }) => "Ambiguous",
            Error::Date(DateError::YearOutOfRange { .. }) => "DateYearOutOfRange",
            Error::Date(DateError::InvalidDate { .. }) => "DateInvalidDate",
            Error::Date(DateError::OutOfRange { .. }) => "DateOutOfRange",
            Error::Date(DateError::InvalidMonth { .. }) => "DateInvalidMonth",
            Error::Date(DateError::InvalidWeekday { .. }) => "DateInvalidWeekday",
            Error::Date(DateError::InvalidOccurrence { .. }) => "DateInvalidOccurrence",
            Error::Date(DateError::Parse { .. }) => "DateParse",
            Error::Period(PeriodError::Overflow { .. }) => "PeriodOverflow",
            Error::Period(PeriodError::IncompatibleUnits { .. }) => "PeriodIncompatibleUnits",
            Error::Period(PeriodError::Parse { .. }) => "PeriodParse",
            Error::Calendar(CalendarError::Date(_)) => "CalendarDate",
            Error::Calendar(CalendarError::NoBusinessWeekday) => "CalendarNoBusinessWeekday",
            Error::Calendar(CalendarError::EmptyJoint) => "CalendarEmptyJoint",
            Error::Calendar(CalendarError::SearchLimit { .. }) => "CalendarSearchLimit",
            Error::Calendar(CalendarError::NoBusinessDayInMonth { .. }) => {
                "CalendarNoBusinessDayInMonth"
            }
            Error::Calendar(CalendarError::InvalidRange { .. }) => "CalendarInvalidRange",
            Error::Calendar(CalendarError::NestingLimit { .. }) => "CalendarNestingLimit",
            Error::DayCount(DayCountError::Value(_)) => "DayCountValue",
            Error::DayCount(DayCountError::Calendar(_)) => "DayCountCalendar",
            Error::Calendar(CalendarError::OutsideValidity { .. }) => "CalendarOutsideValidity",
            Error::Calendar(CalendarError::UnverifiedCoverage { .. }) => {
                "CalendarUnverifiedCoverage"
            }
            Error::Icma(IcmaError::InvalidPeriod { .. }) => "IcmaInvalidPeriod",
            Error::Icma(IcmaError::EmptyPeriods) => "IcmaEmptyPeriods",
            Error::Icma(IcmaError::InvalidFrequency { .. }) => "IcmaInvalidFrequency",
            Error::Icma(IcmaError::NonContiguousPeriods { .. }) => "IcmaNonContiguousPeriods",
            Error::Icma(IcmaError::OutOfCoverage { .. }) => "IcmaOutOfCoverage",
            Error::Rate(RateError::Value(_)) => "RateValue",
            Error::Rate(RateError::DayCount(_)) => "RateDayCount",
            Error::Rate(RateError::Icma(_)) => "RateIcma",
            Error::Rate(RateError::NumericalBoundary { .. }) => "RateNumericalBoundary",
            Error::Rate(RateError::InvalidFrequency { .. }) => "RateInvalidFrequency",
            Error::Rate(RateError::ReversedDates { .. }) => "RateReversedDates",
            Error::Rate(RateError::ZeroTime { .. }) => "RateZeroTime",
            Error::Rate(RateError::ZeroTargetTime { .. }) => "RateZeroTargetTime",
            Error::Sum(SumError::Input { .. }) => "SumInput",
            Error::Sum(SumError::Arithmetic { .. }) => "SumArithmetic",
            Error::Normal(NormalError::Value(_)) => "NormalValue",
            Error::Normal(NormalError::NumericalBoundary { .. }) => "NormalNumericalBoundary",
            Error::Normal(NormalError::Convergence { .. }) => "NormalConvergence",
            Error::Interpolation(InterpolationError::NotEnoughNodes { .. }) => {
                "InterpolationNotEnoughNodes"
            }
            Error::Interpolation(InterpolationError::InvalidNode { .. }) => {
                "InterpolationInvalidNode"
            }
            Error::Interpolation(InterpolationError::NotStrictlyIncreasing { .. }) => {
                "InterpolationNotStrictlyIncreasing"
            }
            Error::Interpolation(InterpolationError::OutsideDomain { .. }) => {
                "InterpolationOutsideDomain"
            }
            Error::Interpolation(InterpolationError::NumericalGrid { .. }) => {
                "InterpolationNumericalGrid"
            }
            Error::Interpolation(InterpolationError::Value(_)) => "InterpolationValue",
            Error::Solver(SolverError::Value(_)) => "SolverValue",
            Error::Solver(SolverError::InvalidBracket { .. }) => "SolverInvalidBracket",
            Error::Solver(SolverError::InvalidBudget) => "SolverInvalidBudget",
            Error::Solver(SolverError::NotBracketed { .. }) => "SolverNotBracketed",
            Error::Solver(SolverError::NonFiniteEvaluation { .. }) => "SolverNonFiniteEvaluation",
            Error::Solver(SolverError::EvaluationLimit { .. }) => "SolverEvaluationLimit",
            Error::Solver(SolverError::PrecisionExhausted { .. }) => "SolverPrecisionExhausted",
            Error::Schedule(ScheduleError::InvalidCdsTenor { .. }) => "ScheduleInvalidCdsTenor",
            Error::Schedule(ScheduleError::CdsMaturityDate { .. }) => "ScheduleCdsMaturityDate",
            Error::Schedule(ScheduleError::InvalidCdsMaturity { .. }) => {
                "ScheduleInvalidCdsMaturity"
            }
            Error::Schedule(ScheduleError::InvalidCdsCouponLimit { .. }) => {
                "ScheduleInvalidCdsCouponLimit"
            }
            Error::Schedule(ScheduleError::CdsCouponLimitExceeded { .. }) => {
                "ScheduleCdsCouponLimitExceeded"
            }
            Error::Schedule(ScheduleError::NoEligibleCdsPayment { .. }) => {
                "ScheduleNoEligibleCdsPayment"
            }
            Error::Schedule(ScheduleError::CdsDate { .. }) => "ScheduleCdsDate",
            Error::Schedule(ScheduleError::CdsCalendar { .. }) => "ScheduleCdsCalendar",
            Error::Schedule(ScheduleError::CdsAdjustedOrder { .. }) => "ScheduleCdsAdjustedOrder",
            Error::Schedule(ScheduleError::InvalidRollTenor { .. }) => "ScheduleInvalidRollTenor",
            Error::Schedule(ScheduleError::ResolvedRollRange { .. }) => "ScheduleResolvedRollRange",
            Error::Schedule(ScheduleError::RollDate { .. }) => "ScheduleRollDate",
            Error::Schedule(ScheduleError::InvalidOldCdsTenor { .. }) => {
                "ScheduleInvalidOldCdsTenor"
            }
            Error::Schedule(ScheduleError::OldCdsDate { .. }) => "ScheduleOldCdsDate",
            Error::Schedule(ScheduleError::InvalidRange { .. }) => "ScheduleInvalidRange",
            Error::Schedule(ScheduleError::InvalidBoundaryLimit { .. }) => {
                "ScheduleInvalidBoundaryLimit"
            }
            Error::Schedule(ScheduleError::InvalidRule { .. }) => "ScheduleInvalidRule",
            Error::Schedule(ScheduleError::InvalidTenor { .. }) => "ScheduleInvalidTenor",
            Error::Schedule(ScheduleError::IncompatibleEndOfMonth { .. }) => {
                "ScheduleIncompatibleEndOfMonth"
            }
            Error::Schedule(ScheduleError::EmptyExplicitStubDates) => {
                "ScheduleEmptyExplicitStubDates"
            }
            Error::Schedule(ScheduleError::ReversedExplicitStubDates { .. }) => {
                "ScheduleReversedExplicitStubDates"
            }
            Error::Schedule(ScheduleError::ExplicitStubOutsideRange { .. }) => {
                "ScheduleExplicitStubOutsideRange"
            }
            Error::Schedule(ScheduleError::IrregularExplicitMiddle { .. }) => {
                "ScheduleIrregularExplicitMiddle"
            }
            Error::Schedule(ScheduleError::InvalidIcmaPeriodLimit { .. }) => {
                "ScheduleInvalidIcmaPeriodLimit"
            }
            Error::Schedule(ScheduleError::NonPeriodicIcma { .. }) => "ScheduleNonPeriodicIcma",
            Error::Schedule(ScheduleError::InvalidIcmaFrequency { .. }) => {
                "ScheduleInvalidIcmaFrequency"
            }
            Error::Schedule(ScheduleError::IcmaFrequencyMismatch { .. }) => {
                "ScheduleIcmaFrequencyMismatch"
            }
            Error::Schedule(ScheduleError::IcmaPeriodLimitExceeded { .. }) => {
                "ScheduleIcmaPeriodLimitExceeded"
            }
            Error::Schedule(ScheduleError::Icma { .. }) => "ScheduleIcma",
            Error::Schedule(ScheduleError::StubRequired { .. }) => "ScheduleStubRequired",
            Error::Schedule(ScheduleError::LongStubWithoutRegularPeriod { .. }) => {
                "ScheduleLongStubWithoutRegularPeriod"
            }
            Error::Schedule(ScheduleError::BoundaryLimitExceeded { .. }) => {
                "ScheduleBoundaryLimitExceeded"
            }
            Error::Schedule(ScheduleError::AllocationFailed { .. }) => "ScheduleAllocationFailed",
            Error::Schedule(ScheduleError::Date { .. }) => "ScheduleDate",
            Error::Schedule(ScheduleError::Calendar { .. }) => "ScheduleCalendar",
            Error::Schedule(ScheduleError::AdjustedOrder { .. }) => "ScheduleAdjustedOrder",
            Error::CashFlow(CashFlowError::InvalidAccrualPeriod { .. }) => {
                "CashFlowInvalidAccrualPeriod"
            }
            Error::CashFlow(CashFlowError::InvalidFlowLimit { .. }) => "CashFlowInvalidFlowLimit",
            Error::CashFlow(CashFlowError::FlowLimitExceeded { .. }) => "CashFlowFlowLimitExceeded",
            Error::CashFlow(CashFlowError::PaymentOrder { .. }) => "CashFlowPaymentOrder",
            Error::CashFlow(CashFlowError::AllocationFailed { .. }) => "CashFlowAllocationFailed",
            Error::CashFlow(CashFlowError::AccrualWorkLimitExceeded { .. }) => {
                "CashFlowAccrualWorkLimitExceeded"
            }
            Error::CashFlow(CashFlowError::Value { .. }) => "CashFlowValue",
            Error::CashFlow(CashFlowError::Rate { .. }) => "CashFlowRate",
            Error::CashFlow(CashFlowError::Sum { .. }) => "CashFlowSum",
            Error::Bond(BondError::InvalidFlowLimit { .. }) => "BondInvalidFlowLimit",
            Error::Bond(BondError::FlowLimitExceeded { .. }) => "BondFlowLimitExceeded",
            Error::Bond(BondError::ExCouponCountMismatch { .. }) => "BondExCouponCountMismatch",
            Error::Bond(BondError::InvalidIssueMaturity { .. }) => "BondInvalidIssueMaturity",
            Error::Bond(BondError::RedemptionBeforeIssue { .. }) => "BondRedemptionBeforeIssue",
            Error::Bond(BondError::RedemptionBeforeLastCoupon { .. }) => {
                "BondRedemptionBeforeLastCoupon"
            }
            Error::Bond(BondError::ExCouponAfterPayment { .. }) => "BondExCouponAfterPayment",
            Error::Bond(BondError::SettlementBeforeIssue { .. }) => "BondSettlementBeforeIssue",
            Error::Bond(BondError::AllocationFailed { .. }) => "BondAllocationFailed",
            Error::Bond(BondError::AccrualWorkLimitExceeded { .. }) => {
                "BondAccrualWorkLimitExceeded"
            }
            Error::Bond(BondError::Value { .. }) => "BondValue",
            Error::Bond(BondError::Rate { .. }) => "BondRate",
            Error::Bond(BondError::Sum { .. }) => "BondSum",
            Error::Bond(BondError::ZeroDirtyPrice { .. }) => "BondZeroDirtyPrice",
            Error::Bond(BondError::InternalCapacity { .. }) => "BondInternalCapacity",
            Error::Gilt(GiltError::InvalidDateCap { .. }) => "GiltInvalidDateCap",
            Error::Gilt(GiltError::TooManyQuasiDates { .. }) => "GiltTooManyQuasiDates",
            Error::Gilt(GiltError::TooFewQuasiDates { .. }) => "GiltTooFewQuasiDates",
            Error::Gilt(GiltError::NegativeCoupon { .. }) => "GiltNegativeCoupon",
            Error::Gilt(GiltError::IssueNotBeforeMaturity { .. }) => "GiltIssueNotBeforeMaturity",
            Error::Gilt(GiltError::MaturityBoundary { .. }) => "GiltMaturityBoundary",
            Error::Gilt(GiltError::QuasiDateOrder { .. }) => "GiltQuasiDateOrder",
            Error::Gilt(GiltError::QuasiCoverage { .. }) => "GiltQuasiCoverage",
            Error::Gilt(GiltError::QuasiDateMismatch { .. }) => "GiltQuasiDateMismatch",
            Error::Gilt(GiltError::Date { .. }) => "GiltDate",
            Error::Gilt(GiltError::FirstDividend { .. }) => "GiltFirstDividend",
            Error::Gilt(GiltError::ExDateCount { .. }) => "GiltExDateCount",
            Error::Gilt(GiltError::ExDateWindow { .. }) => "GiltExDateWindow",
            Error::Gilt(GiltError::InvalidBundleCap { .. }) => "GiltInvalidBundleCap",
            Error::Gilt(GiltError::TooManyBundles { .. }) => "GiltTooManyBundles",
            Error::Gilt(GiltError::SettlementBeforeIssue { .. }) => "GiltSettlementBeforeIssue",
            Error::Gilt(GiltError::SettlementAtOrAfterMaturity { .. }) => {
                "GiltSettlementAtOrAfterMaturity"
            }
            Error::Gilt(GiltError::InvalidDividendIndex { .. }) => "GiltInvalidDividendIndex",
            Error::Gilt(GiltError::Value { .. }) => "GiltValue",
            Error::Gilt(GiltError::Rate { .. }) => "GiltRate",
            Error::Gilt(GiltError::Sum { .. }) => "GiltSum",
            Error::Gilt(GiltError::AllocationFailed { .. }) => "GiltAllocationFailed",
            Error::Gilt(GiltError::InternalCapacity { .. }) => "GiltInternalCapacity",
            Error::Gilt(GiltError::ZeroDirtyPrice { .. }) => "GiltZeroDirtyPrice",
            Error::Gilt(GiltError::NonPositiveNominal { .. }) => "GiltNonPositiveNominal",
            Error::Gilt(GiltError::CashArithmetic { .. }) => "GiltCashArithmetic",
            Error::GiltYield(GiltYieldError::Target { .. }) => "GiltYieldTarget",
            Error::GiltYield(GiltYieldError::Preparation { .. }) => "GiltYieldPreparation",
            Error::GiltYield(GiltYieldError::Callback { .. }) => "GiltYieldCallback",
            Error::GiltYield(GiltYieldError::NonPositiveTarget { .. }) => {
                "GiltYieldNonPositiveTarget"
            }
            Error::GiltYield(GiltYieldError::Solver(_)) => "GiltYieldSolver",
            Error::Treasury(TreasuryError::InvalidDateCap { .. }) => "TreasuryInvalidDateCap",
            Error::Treasury(TreasuryError::TooManyQuasiDates { .. }) => "TreasuryTooManyQuasiDates",
            Error::Treasury(TreasuryError::TooFewQuasiDates { .. }) => "TreasuryTooFewQuasiDates",
            Error::Treasury(TreasuryError::NegativeCoupon { .. }) => "TreasuryNegativeCoupon",
            Error::Treasury(TreasuryError::DatedAfterIssue { .. }) => "TreasuryDatedAfterIssue",
            Error::Treasury(TreasuryError::IssueNotBeforeMaturity { .. }) => {
                "TreasuryIssueNotBeforeMaturity"
            }
            Error::Treasury(TreasuryError::MaturityBoundary { .. }) => "TreasuryMaturityBoundary",
            Error::Treasury(TreasuryError::QuasiDateOrder { .. }) => "TreasuryQuasiDateOrder",
            Error::Treasury(TreasuryError::QuasiCoverage { .. }) => "TreasuryQuasiCoverage",
            Error::Treasury(TreasuryError::QuasiDateMismatch { .. }) => "TreasuryQuasiDateMismatch",
            Error::Treasury(TreasuryError::Date { .. }) => "TreasuryDate",
            Error::Treasury(TreasuryError::FirstCoupon { .. }) => "TreasuryFirstCoupon",
            Error::Treasury(TreasuryError::InvalidBundleCap { .. }) => "TreasuryInvalidBundleCap",
            Error::Treasury(TreasuryError::TooManyBundles { .. }) => "TreasuryTooManyBundles",
            Error::Treasury(TreasuryError::SettlementBeforeIssue { .. }) => {
                "TreasurySettlementBeforeIssue"
            }
            Error::Treasury(TreasuryError::SettlementAtOrAfterMaturity { .. }) => {
                "TreasurySettlementAtOrAfterMaturity"
            }
            Error::Treasury(TreasuryError::InvalidCouponIndex { .. }) => {
                "TreasuryInvalidCouponIndex"
            }
            Error::Treasury(TreasuryError::Value { .. }) => "TreasuryValue",
            Error::Treasury(TreasuryError::Sum { .. }) => "TreasurySum",
            Error::Treasury(TreasuryError::AllocationFailed { .. }) => "TreasuryAllocationFailed",
            Error::Treasury(TreasuryError::InternalCapacity { .. }) => "TreasuryInternalCapacity",
            Error::Treasury(TreasuryError::ZeroDirtyPrice { .. }) => "TreasuryZeroDirtyPrice",
            Error::Treasury(TreasuryError::NonPositiveNominal { .. }) => {
                "TreasuryNonPositiveNominal"
            }
            Error::Treasury(TreasuryError::CashArithmetic { .. }) => "TreasuryCashArithmetic",
            Error::TreasuryYield(TreasuryYieldError::Target { .. }) => "TreasuryYieldTarget",
            Error::TreasuryYield(TreasuryYieldError::Preparation { .. }) => {
                "TreasuryYieldPreparation"
            }
            Error::TreasuryYield(TreasuryYieldError::Callback { .. }) => "TreasuryYieldCallback",
            Error::TreasuryYield(TreasuryYieldError::NonPositiveTarget { .. }) => {
                "TreasuryYieldNonPositiveTarget"
            }
            Error::TreasuryYield(TreasuryYieldError::Solver(_)) => "TreasuryYieldSolver",
            Error::BondYield(BondYieldError::Target { .. }) => "BondYieldTarget",
            Error::BondYield(BondYieldError::Preparation { .. }) => "BondYieldPreparation",
            Error::BondYield(BondYieldError::Solver(_)) => "BondYieldSolver",
            Error::BondYield(BondYieldError::Callback { .. }) => "BondYieldCallback",
            Error::BondYield(BondYieldError::NoYieldSensitivity { .. }) => {
                "BondYieldNoYieldSensitivity"
            }
            Error::BondYield(BondYieldError::ConstantPriceMismatch { .. }) => {
                "BondYieldConstantPriceMismatch"
            }
            Error::Settlement(SettlementError::NonPositiveNominal { .. }) => {
                "SettlementNonPositiveNominal"
            }
            Error::Settlement(SettlementError::DecimalIntervalOrder { .. }) => {
                "SettlementDecimalIntervalOrder"
            }
            Error::Settlement(SettlementError::BinaryIntervalOrder { .. }) => {
                "SettlementBinaryIntervalOrder"
            }
            Error::Settlement(SettlementError::NonFinite { .. }) => "SettlementNonFinite",
            Error::Settlement(SettlementError::Arithmetic { .. }) => "SettlementArithmetic",
            Error::Settlement(SettlementError::InternalCapacity { .. }) => {
                "SettlementInternalCapacity"
            }
        }
    }

    fn is_well_formed_kind(kind: &str) -> bool {
        let segment = |s: &str| {
            !s.is_empty()
                && !s.starts_with('_')
                && !s.ends_with('_')
                && !s.contains("__")
                && s.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
        };
        matches!(kind.split_once('.'), Some((domain, variant)) if segment(domain) && segment(variant))
    }

    /// TS-CORE-01, FR-CORE-02, ADR-0007 item 5: every variant has its
    /// documented, well-formed and unique kind name, and `Error` reports the
    /// kind of the error it wraps.
    #[test]
    fn every_variant_has_a_stable_unique_kind_name() {
        let errors = every_variant();
        let mut names: Vec<&str> = errors.iter().map(|(e, _)| variant_name(e)).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 199, "every variant is sampled exactly once");

        let mut kinds = Vec::new();
        for (error, expected) in &errors {
            assert_eq!(error.kind(), *expected);
            assert!(
                is_well_formed_kind(expected),
                "malformed kind name {expected:?}"
            );
            kinds.push(*expected);
            let inner_kind = match error {
                Error::Amount(inner) => inner.kind(),
                Error::Value(inner) => inner.kind(),
                Error::Identifier(inner) => inner.kind(),
                Error::Date(inner) => inner.kind(),
                Error::Period(inner) => inner.kind(),
                Error::Calendar(inner) => inner.kind(),
                Error::DayCount(inner) => inner.kind(),
                Error::Icma(inner) => inner.kind(),
                Error::Rate(inner) => inner.kind(),
                Error::Sum(inner) => inner.kind(),
                Error::Normal(inner) => inner.kind(),
                Error::Interpolation(inner) => inner.kind(),
                Error::Solver(inner) => inner.kind(),
                Error::Schedule(inner) => inner.kind(),
                Error::CashFlow(inner) => inner.kind(),
                Error::Bond(inner) => inner.kind(),
                Error::BondYield(inner) => inner.kind(),
                Error::Gilt(inner) => inner.kind(),
                Error::GiltYield(inner) => inner.kind(),
                Error::Treasury(inner) => inner.kind(),
                Error::TreasuryYield(inner) => inner.kind(),
                Error::Settlement(inner) => inner.kind(),
            };
            assert_eq!(inner_kind, *expected);
        }
        kinds.sort_unstable();
        kinds.dedup();
        assert_eq!(kinds.len(), errors.len(), "kind names are unique");
    }

    /// The kind-name format check itself rejects malformed names.
    #[test]
    fn kind_name_format_check_rejects_malformed_names() {
        for bad in [
            "",
            "value",
            "value.",
            ".not_finite",
            "Value.not_finite",
            "value.not-finite",
            "value.not_finite.extra",
            "value._x",
            "value.x__y",
        ] {
            assert!(!is_well_formed_kind(bad), "{bad:?} must be rejected");
        }
        assert!(is_well_formed_kind("value.not_finite"));
    }

    /// FR-CORE-02: messages carry the structured context. Only the presence
    /// of the context is checked, because message wording is not stable.
    #[test]
    fn messages_carry_the_context() {
        let messages: Vec<String> = every_variant().iter().map(|(e, _)| e.to_string()).collect();
        assert!(messages[0].contains("`rate`") && messages[0].contains("NaN"));
        assert!(messages[1].contains("`time`") && messages[1].contains("-0.5"));
        assert!(messages[1].contains("[0.0, +inf)"));
        assert!(messages[2].contains("compound factor") && messages[2].contains("inf"));
        assert!(messages[3].contains("compound factor") && messages[3].contains("(0.0, +inf)"));
        assert!(messages[4].contains("frequency") && messages[4].contains("\"Monthy\""));
        assert!(messages[5].contains("\"Actual/Actual\""));
        for candidate in CANDIDATES {
            assert!(
                messages[5].contains(candidate),
                "{candidate} missing from {}",
                messages[5]
            );
        }
    }

    /// The crate error displays the domain error and exposes it as its source
    /// (ADR-0007 item 4), so callers can inspect the typed cause.
    #[test]
    fn crate_error_is_transparent() {
        for (error, _) in every_variant() {
            let inner_message = match &error {
                Error::Amount(inner) => inner.to_string(),
                Error::Value(inner) => inner.to_string(),
                Error::Identifier(inner) => inner.to_string(),
                Error::Date(inner) => inner.to_string(),
                Error::Period(inner) => inner.to_string(),
                Error::Calendar(inner) => inner.to_string(),
                Error::DayCount(inner) => inner.to_string(),
                Error::Icma(inner) => inner.to_string(),
                Error::Rate(inner) => inner.to_string(),
                Error::Sum(inner) => inner.to_string(),
                Error::Normal(inner) => inner.to_string(),
                Error::Interpolation(inner) => inner.to_string(),
                Error::Solver(inner) => inner.to_string(),
                Error::Schedule(inner) => inner.to_string(),
                Error::CashFlow(inner) => inner.to_string(),
                Error::Bond(inner) => inner.to_string(),
                Error::BondYield(inner) => inner.to_string(),
                Error::Gilt(inner) => inner.to_string(),
                Error::GiltYield(inner) => inner.to_string(),
                Error::Treasury(inner) => inner.to_string(),
                Error::TreasuryYield(inner) => inner.to_string(),
                Error::Settlement(inner) => inner.to_string(),
            };
            assert_eq!(error.to_string(), inner_message);
            let source = error.source().expect("the domain error is the source");
            assert_eq!(source.to_string(), inner_message);
            let expected_cause = match &error {
                Error::Calendar(CalendarError::Date(date)) => Some(date.to_string()),
                Error::DayCount(DayCountError::Value(value)) => Some(value.to_string()),
                Error::DayCount(DayCountError::Calendar(calendar)) => Some(calendar.to_string()),
                Error::Rate(RateError::Value(inner)) => Some(inner.to_string()),
                Error::Rate(RateError::DayCount(inner)) => Some(inner.to_string()),
                Error::Rate(RateError::Icma(inner)) => Some(inner.to_string()),
                Error::Sum(SumError::Input { error, .. } | SumError::Arithmetic { error, .. }) => {
                    Some(error.to_string())
                }
                Error::Normal(NormalError::Value(error)) => Some(error.to_string()),
                Error::Interpolation(
                    InterpolationError::InvalidNode { cause, .. }
                    | InterpolationError::NumericalGrid { cause, .. }
                    | InterpolationError::Value(cause),
                ) => Some(cause.to_string()),
                Error::Solver(
                    SolverError::Value(cause) | SolverError::NonFiniteEvaluation { cause, .. },
                ) => Some(cause.to_string()),
                Error::Schedule(
                    ScheduleError::Date { cause, .. }
                    | ScheduleError::RollDate { cause, .. }
                    | ScheduleError::OldCdsDate { cause, .. }
                    | ScheduleError::CdsDate { cause, .. }
                    | ScheduleError::CdsMaturityDate { cause, .. },
                ) => Some(cause.to_string()),
                Error::Schedule(
                    ScheduleError::Calendar { cause, .. }
                    | ScheduleError::CdsCalendar { cause, .. },
                ) => Some(cause.to_string()),
                Error::Schedule(ScheduleError::Icma { cause }) => Some(cause.to_string()),
                Error::CashFlow(CashFlowError::Value { cause, .. }) => Some(cause.to_string()),
                Error::CashFlow(CashFlowError::Rate { cause, .. }) => Some(cause.to_string()),
                Error::CashFlow(CashFlowError::Sum { cause, .. }) => Some(cause.to_string()),
                Error::Bond(BondError::Value { cause, .. }) => Some(cause.to_string()),
                Error::Bond(BondError::Rate { cause, .. }) => Some(cause.to_string()),
                Error::Bond(BondError::Sum { cause, .. }) => Some(cause.to_string()),
                Error::BondYield(
                    BondYieldError::Target { cause, .. }
                    | BondYieldError::Preparation { cause, .. }
                    | BondYieldError::Callback { cause, .. },
                ) => Some(cause.to_string()),
                Error::BondYield(BondYieldError::Solver(cause)) => Some(cause.to_string()),
                Error::Gilt(GiltError::Date { cause, .. }) => Some(cause.to_string()),
                Error::Gilt(GiltError::Value { cause, .. }) => Some(cause.to_string()),
                Error::Gilt(GiltError::Rate { cause, .. }) => Some(cause.to_string()),
                Error::Gilt(GiltError::Sum { cause, .. }) => Some(cause.to_string()),
                Error::Gilt(GiltError::CashArithmetic { cause, .. }) => Some(cause.to_string()),
                Error::GiltYield(
                    GiltYieldError::Target { cause, .. }
                    | GiltYieldError::Preparation { cause, .. }
                    | GiltYieldError::Callback { cause, .. },
                ) => Some(cause.to_string()),
                Error::GiltYield(GiltYieldError::Solver(cause)) => Some(cause.to_string()),
                Error::Treasury(TreasuryError::Date { cause, .. }) => Some(cause.to_string()),
                Error::Treasury(TreasuryError::Value { cause, .. }) => Some(cause.to_string()),
                Error::Treasury(TreasuryError::Sum { cause, .. }) => Some(cause.to_string()),
                Error::Treasury(TreasuryError::CashArithmetic { cause, .. }) => {
                    Some(cause.to_string())
                }
                Error::TreasuryYield(
                    TreasuryYieldError::Target { cause, .. }
                    | TreasuryYieldError::Preparation { cause, .. }
                    | TreasuryYieldError::Callback { cause, .. },
                ) => Some(cause.to_string()),
                Error::TreasuryYield(TreasuryYieldError::Solver(cause)) => Some(cause.to_string()),
                Error::Settlement(SettlementError::Arithmetic { cause, .. }) => {
                    Some(cause.to_string())
                }
                _ => None,
            };
            assert_eq!(source.source().map(ToString::to_string), expected_cause);
        }
    }

    /// Formatting is fallible: a caller's failing writer must receive the
    /// error, including failures in the middle of a compound error message.
    #[test]
    fn formatting_propagates_writer_failures_for_every_domain() {
        struct LimitedWriter {
            remaining: usize,
        }
        impl fmt::Write for LimitedWriter {
            fn write_str(&mut self, text: &str) -> fmt::Result {
                if text.len() > self.remaining {
                    return Err(fmt::Error);
                }
                self.remaining -= text.len();
                Ok(())
            }
        }
        for (error, _) in every_variant() {
            let message = error.to_string();
            for remaining in 0..message.len() {
                let mut writer = LimitedWriter { remaining };
                assert!(fmt::write(&mut writer, format_args!("{error}")).is_err());
            }
            let mut writer = LimitedWriter {
                remaining: message.len(),
            };
            assert!(fmt::write(&mut writer, format_args!("{error}")).is_ok());
            assert_eq!(writer.remaining, 0);
        }
    }

    /// NP-01: convenience intervals apply strict/inclusive endpoints and
    /// always reject non-finite inputs, even with an unbounded side.
    #[test]
    fn convenience_intervals_have_correct_endpoint_and_nonfinite_semantics() {
        for bound in [f64::MIN, -1.0, -0.0, 0.0, 1.0, f64::MAX] {
            for (domain, accepts_equal) in [
                (Interval::greater_than(bound), false),
                (Interval::at_least(bound), true),
                (Interval::less_than(bound), false),
                (Interval::at_most(bound), true),
            ] {
                assert_eq!(domain.contains(bound), accepts_equal);
                for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                    assert!(!domain.contains(value));
                }
            }
            assert!(!Interval::less_than(bound).contains(f64::MAX));
            assert!(!Interval::greater_than(bound).contains(f64::MIN));
        }
        assert!(Interval::at_most(0.0).contains(-0.0));
        assert!(Interval::at_least(-0.0).contains(0.0));
    }

    /// NP-01: classification of non-finite values, whatever the NaN payload
    /// or sign, and of the finite edge cases.
    #[test]
    fn non_finite_values_are_classified() {
        let nans = [
            f64::NAN,
            -f64::NAN,
            f64::from_bits(0x7FF0_0000_0000_0001), // signalling NaN
            f64::from_bits(0xFFF8_0000_0000_0000),
            f64::from_bits(u64::MAX),
        ];
        for nan in nans {
            assert_eq!(NonFinite::classify(nan), Some(NonFinite::NaN));
        }
        assert_eq!(
            NonFinite::classify(f64::INFINITY),
            Some(NonFinite::PositiveInfinity)
        );
        assert_eq!(
            NonFinite::classify(f64::NEG_INFINITY),
            Some(NonFinite::NegativeInfinity)
        );
        let finite = [
            0.0,
            -0.0,
            f64::from_bits(1), // smallest subnormal
            f64::MIN_POSITIVE,
            f64::EPSILON,
            f64::MAX,
            f64::MIN,
        ];
        for value in finite {
            assert_eq!(NonFinite::classify(value), None, "{value:?}");
        }
        assert_eq!(NonFinite::NaN.to_string(), "NaN");
        assert_eq!(NonFinite::PositiveInfinity.to_string(), "inf");
        assert_eq!(NonFinite::NegativeInfinity.to_string(), "-inf");
    }

    /// Interval membership at every kind of bound; NaN and the infinities are
    /// never members; both zeros behave alike (NP-08).
    #[test]
    fn interval_membership() {
        let non_negative = Interval::at_least(0.0);
        assert!(non_negative.contains(0.0));
        assert!(non_negative.contains(-0.0));
        assert!(non_negative.contains(f64::from_bits(1)));
        assert!(non_negative.contains(f64::MAX));
        assert!(!non_negative.contains(-f64::from_bits(1)));

        let positive = Interval::greater_than(0.0);
        assert!(!positive.contains(0.0));
        assert!(!positive.contains(-0.0));
        assert!(positive.contains(f64::from_bits(1)));

        let below_one = Interval::less_than(1.0);
        assert!(below_one.contains(1.0_f64.next_down()));
        assert!(!below_one.contains(1.0));
        assert!(below_one.contains(f64::MIN));

        let at_most_one = Interval::at_most(1.0);
        assert!(at_most_one.contains(1.0));
        assert!(!at_most_one.contains(1.0_f64.next_up()));

        let closed = Interval::new(Bound::Included(-1.0), Bound::Included(1.0));
        assert!(closed.contains(-1.0) && closed.contains(1.0) && closed.contains(0.0));
        assert!(!closed.contains(1.0_f64.next_up()));
        let point = Interval::new(Bound::Included(2.0), Bound::Included(2.0));
        assert!(point.contains(2.0) && !point.contains(2.0_f64.next_up()));

        let everything = Interval::new(Bound::Unbounded, Bound::Unbounded);
        for interval in [
            non_negative,
            positive,
            below_one,
            at_most_one,
            closed,
            everything,
        ] {
            for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                assert!(
                    !interval.contains(value),
                    "{interval} must not contain {value}"
                );
            }
        }
        assert!(everything.contains(f64::MAX) && everything.contains(f64::MIN));
    }

    #[test]
    fn interval_accessors_and_display() {
        let interval = Interval::greater_than(-2.0);
        assert_eq!(interval.lower(), Bound::Excluded(-2.0));
        assert_eq!(interval.upper(), Bound::Unbounded);
        assert_eq!(interval.to_string(), "(-2.0, +inf)");
        assert_eq!(Interval::at_least(0.0).to_string(), "[0.0, +inf)");
        assert_eq!(Interval::less_than(1.5).to_string(), "(-inf, 1.5)");
        assert_eq!(Interval::at_most(1.5).to_string(), "(-inf, 1.5]");
        let closed = Interval::new(Bound::Included(-1.0), Bound::Excluded(1e300));
        assert_eq!(closed.to_string(), "[-1.0, 1e300)");
        let everything = Interval::new(Bound::Unbounded, Bound::Unbounded);
        assert_eq!(everything.to_string(), "(-inf, +inf)");
    }

    /// Invalid bounds are a defect, caught by the debug assertion.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "interval bounds must be finite")]
    fn invalid_interval_is_a_defect() {
        let _ = Interval::new(Bound::Included(1.0), Bound::Excluded(1.0));
    }

    #[test]
    fn interval_validity_rules() {
        use Bound::{Excluded, Included, Unbounded};
        assert!(Interval::is_valid(Unbounded, Unbounded));
        assert!(Interval::is_valid(Included(0.0), Unbounded));
        assert!(Interval::is_valid(Unbounded, Excluded(0.0)));
        assert!(Interval::is_valid(Included(1.0), Included(1.0)));
        assert!(Interval::is_valid(Excluded(0.0), Excluded(1.0)));
        assert!(!Interval::is_valid(Included(1.0), Excluded(1.0)));
        assert!(!Interval::is_valid(Excluded(1.0), Included(1.0)));
        assert!(!Interval::is_valid(Included(2.0), Included(1.0)));
        assert!(!Interval::is_valid(Included(f64::NAN), Unbounded));
        assert!(!Interval::is_valid(Unbounded, Excluded(f64::INFINITY)));
        assert!(!Interval::is_valid(
            Included(f64::NEG_INFINITY),
            Included(0.0)
        ));
    }
}
