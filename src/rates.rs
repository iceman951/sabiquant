//! Explicit interest-rate quotes, stable compounding and rate-based discounting.
//!
//! Definitions and domains follow FS-RATE/FS-DISC in the project requirements.
//! Rates are decimal fractions, not percentages; horizons are year fractions.
//! The quote owns its day-count context and has no default market convention.
//! Periodic and hybrid rates must exceed minus their annual frequency.
//! Factor outputs are finite and strictly positive. Equivalent conversions
//! avoid forming an intermediate factor where its logarithm suffices.
//!
//! # Context and reference mapping
//!
//! [`InterestRate`] owns an [`AccrualConvention`] and [`Compounding`]. The
//! latter stores the frequency for periodic methods; Simple and Continuous
//! have no implicit frequency. Date forms compute both required accruals
//! when converting conventions. Time forms deliberately accept a caller's
//! externally computed year fraction.
//!
//! | Operation | QuantLib reference concept |
//! |---|---|
//! | [`InterestRate::compound_factor`] and its date form | `InterestRate::compoundFactor` |
//! | [`InterestRate::discount_factor`] and its date form | `InterestRate::discountFactor` |
//! | [`InterestRate::implied_rate`] | `InterestRate::impliedRate` |
//! | [`InterestRate::equivalent_rate`] and its date form | `InterestRate::equivalentRate` |
//!
//! The mathematical definitions are in the project
//! [requirements](https://github.com/iceman951/sabiquant/blob/main/docs/REQUIREMENTS.md).
//! Public reference observations and source-rounding differences are recorded
//! in the [local numerical report](https://github.com/iceman951/sabiquant/blob/main/docs/M4_NUMERICAL_VALIDATION.md).
//! QuantLib is an external reference; no runtime dependency is introduced.
//!
//! # Numerical behavior
//!
//! Simple factors use fused arithmetic. Periodic inference and conversion use
//! `ln_1p`/`exp_m1`, preserving tiny growth and avoiding rounded intermediate
//! factors where possible. Outputs are finite and in domain. Elementary
//! factors reject unresolved range boundaries; near a rounded hybrid switch
//! or a zero simple factor, representation conditioning can dominate relative
//! error. Platform elementary functions have unspecified precision, so finite
//! comparisons use the measured conditional policy rather than a bitwise
//! equivalence or a universal few-ULP promise.

use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;

use crate::daycount::{
    ActualActualIcma, Business252, DayCount, DayCountError, IcmaError, ThirtyE360Isda,
};
use crate::error::{IdentifierError, Interval, ValueError};
use crate::identifier::{Entry, Identifiers};
use crate::numeric;
use crate::time::{Date, Frequency};

/// Converts a finite percentage to a decimal rate by one division by 100.
///
/// For example, 5 percent gives a decimal rate of 0.05. Quote constructors
/// always take decimal rates; there is no implicit percent input mode.
///
/// # Errors
/// [`RateError::Value`] for nonfinite input or a nonzero result rounded to zero.
pub fn decimal_from_percent(percent: f64) -> Result<f64, RateError> {
    let percent = numeric::finite("percent", percent)?;
    unit_result("decimal rate", percent / 100.0, percent)
}

/// Converts finite basis points to a decimal rate by one division by 10,000.
///
/// One basis point is a decimal rate of 0.0001; 500 basis points is 0.05.
///
/// # Errors
/// [`RateError::Value`] for nonfinite input or a nonzero result rounded to zero.
pub fn decimal_from_basis_points(basis_points: f64) -> Result<f64, RateError> {
    let basis_points = numeric::finite("basis points", basis_points)?;
    unit_result("decimal rate", basis_points / 10_000.0, basis_points)
}

/// Converts a finite decimal rate to percent by one multiplication by 100.
///
/// # Errors
/// [`RateError::Value`] for nonfinite input or output overflow.
pub fn percent_from_decimal(rate: f64) -> Result<f64, RateError> {
    let rate = numeric::finite("decimal rate", rate)?;
    unit_result("percent", rate * 100.0, rate)
}

/// Converts a finite decimal rate to basis points by multiplying by 10,000.
///
/// # Errors
/// [`RateError::Value`] for nonfinite input or output overflow.
pub fn basis_points_from_decimal(rate: f64) -> Result<f64, RateError> {
    let rate = numeric::finite("decimal rate", rate)?;
    unit_result("basis points", rate * 10_000.0, rate)
}

fn unit_result(quantity: &'static str, value: f64, source: f64) -> Result<f64, RateError> {
    if source == 0.0 {
        return Ok(0.0);
    }
    let domain = if source > 0.0 {
        Interval::greater_than(0.0)
    } else {
        Interval::less_than(0.0)
    };
    Ok(numeric::result_within(quantity, value, domain)?)
}

/// The five mathematical compounding methods specified by FS-RATE.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "FS-RATE specifies exactly five methods"
)]
pub enum CompoundingMethod {
    /// `1 + rate × time`.
    Simple,
    /// `(1 + rate / frequency)^(frequency × time)`.
    Compounded,
    /// `exp(rate × time)`.
    Continuous,
    /// Simple through `time <= 1/frequency`, then compounded.
    SimpleThenCompounded,
    /// Compounded through `time <= 1/frequency`, then simple.
    CompoundedThenSimple,
}

impl CompoundingMethod {
    /// Every method, without the frequency required by periodic methods.
    pub const ALL: [Self; 5] = [
        Self::Simple,
        Self::Compounded,
        Self::Continuous,
        Self::SimpleThenCompounded,
        Self::CompoundedThenSimple,
    ];

    /// Returns the exact, case-sensitive method identifier.
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::Simple => "Simple",
            Self::Compounded => "Compounded",
            Self::Continuous => "Continuous",
            Self::SimpleThenCompounded => "SimpleThenCompounded",
            Self::CompoundedThenSimple => "CompoundedThenSimple",
        }
    }
}

impl fmt::Display for CompoundingMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_identifier())
    }
}

impl FromStr for CompoundingMethod {
    type Err = IdentifierError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        METHOD_IDENTIFIERS.parse(text)
    }
}

/// A compounding method with its required, validated periodic frequency.
///
/// The method identifier alone does not supply a frequency. Simple and
/// Continuous are constants; periodic constructors reject Once explicitly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Compounding {
    kind: CompoundingKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum CompoundingKind {
    Simple,
    Continuous,
    Periodic {
        method: CompoundingMethod,
        frequency: Frequency,
        count: u32,
    },
}

impl Compounding {
    /// Simple compounding; no frequency applies.
    pub const SIMPLE: Self = Self {
        kind: CompoundingKind::Simple,
    };
    /// Continuous compounding; no frequency applies.
    pub const CONTINUOUS: Self = Self {
        kind: CompoundingKind::Continuous,
    };

    /// Constructs periodic compounding with an explicit frequency.
    ///
    /// # Errors
    /// [`RateError::InvalidFrequency`] for Once.
    pub fn compounded(frequency: Frequency) -> Result<Self, RateError> {
        Self::periodic(CompoundingMethod::Compounded, frequency)
    }

    /// Constructs SimpleThenCompounded with an explicit switch frequency.
    ///
    /// # Errors
    /// [`RateError::InvalidFrequency`] for Once.
    pub fn simple_then_compounded(frequency: Frequency) -> Result<Self, RateError> {
        Self::periodic(CompoundingMethod::SimpleThenCompounded, frequency)
    }

    /// Constructs CompoundedThenSimple with an explicit switch frequency.
    ///
    /// # Errors
    /// [`RateError::InvalidFrequency`] for Once.
    pub fn compounded_then_simple(frequency: Frequency) -> Result<Self, RateError> {
        Self::periodic(CompoundingMethod::CompoundedThenSimple, frequency)
    }

    fn periodic(method: CompoundingMethod, frequency: Frequency) -> Result<Self, RateError> {
        let Some(count) = frequency.events_per_year().filter(|&count| count > 0) else {
            return Err(RateError::InvalidFrequency { frequency });
        };
        Ok(Self {
            kind: CompoundingKind::Periodic {
                method,
                frequency,
                count,
            },
        })
    }

    /// Returns the mathematical method.
    #[must_use]
    pub const fn method(self) -> CompoundingMethod {
        match self.kind {
            CompoundingKind::Simple => CompoundingMethod::Simple,
            CompoundingKind::Continuous => CompoundingMethod::Continuous,
            CompoundingKind::Periodic { method, .. } => method,
        }
    }

    /// Returns the declared periodic frequency, or None for Simple/Continuous.
    #[must_use]
    pub const fn frequency(self) -> Option<Frequency> {
        match self.kind {
            CompoundingKind::Periodic { frequency, .. } => Some(frequency),
            _ => None,
        }
    }

    fn periodic_count(self) -> Option<f64> {
        match self.kind {
            CompoundingKind::Periodic { count, .. } => Some(f64::from(count)),
            _ => None,
        }
    }

    fn phase(self, time: f64) -> Phase {
        match self.kind {
            CompoundingKind::Simple => Phase::Simple,
            CompoundingKind::Continuous => Phase::Continuous,
            CompoundingKind::Periodic { method, count, .. } => {
                let count = f64::from(count);
                match method {
                    CompoundingMethod::SimpleThenCompounded if time <= 1.0 / count => Phase::Simple,
                    CompoundingMethod::CompoundedThenSimple if time > 1.0 / count => Phase::Simple,
                    _ => Phase::Periodic(count),
                }
            }
        }
    }
}

impl fmt::Display for Compounding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.method().fmt(f)?;
        if let Some(frequency) = self.frequency() {
            write!(f, "({frequency})")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Simple,
    Continuous,
    Periodic(f64),
}

/// An explicitly owned accrual convention, including any required context.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AccrualConvention {
    /// A stateless scalar day-count convention.
    Scalar(DayCount),
    /// ICMA with explicit determination periods and frequency.
    Icma(ActualActualIcma),
    /// ISDA 30E/360 with an explicit termination date.
    ThirtyE360Isda(ThirtyE360Isda),
    /// Business/252 with an explicit plain or guarded market calendar.
    Business252(Business252),
}

impl AccrualConvention {
    /// Computes the fraction while preserving its required context checks.
    ///
    /// # Errors
    /// [`RateError::DayCount`] or [`RateError::Icma`] wraps the typed cause.
    pub fn year_fraction(&self, start: Date, end: Date) -> Result<f64, RateError> {
        match self {
            Self::Scalar(value) => Ok(value.year_fraction(start, end)?),
            Self::Icma(value) => Ok(value.year_fraction(start, end)?),
            Self::ThirtyE360Isda(value) => Ok(value.year_fraction(start, end)?),
            Self::Business252(value) => Ok(value.year_fraction(start, end)?),
        }
    }
}

impl From<DayCount> for AccrualConvention {
    fn from(value: DayCount) -> Self {
        Self::Scalar(value)
    }
}
impl From<ActualActualIcma> for AccrualConvention {
    fn from(value: ActualActualIcma) -> Self {
        Self::Icma(value)
    }
}
impl From<ThirtyE360Isda> for AccrualConvention {
    fn from(value: ThirtyE360Isda) -> Self {
        Self::ThirtyE360Isda(value)
    }
}
impl From<Business252> for AccrualConvention {
    fn from(value: Business252) -> Self {
        Self::Business252(value)
    }
}

/// An immutable decimal interest rate with explicit compounding and accrual.
///
/// Negative rates are accepted within the convention domain. Zero rates and
/// horizons produce factor one exactly. Real outputs remain `f64`; there is
/// no implicit convention, default quote, clock or global evaluation date.
///
/// Equality compares the decimal input rate, complete accrual definition and
/// declared compounding/frequency. It does not test whether two quotes give
/// financially equivalent factors at one or every horizon. Signed input zero
/// is canonicalized; this type has no ordering or hashing because it stores
/// a real-valued rate.
///
/// ```
/// use sabiquant::rates::{Compounding, InterestRate};
/// use sabiquant::daycount::DayCount;
/// let quote = InterestRate::new(0.05, DayCount::Actual365Fixed, Compounding::SIMPLE)?;
/// assert_eq!(quote.compound_factor(2.0)?, 1.1);
/// # Ok::<(), sabiquant::rates::RateError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct InterestRate {
    rate: f64,
    accrual: AccrualConvention,
    compounding: Compounding,
}

impl InterestRate {
    /// Creates a quote from a decimal rate, required accrual and compounding.
    ///
    /// # Errors
    /// [`RateError::Value`] for a nonfinite rate or periodic/hybrid rate not
    /// greater than minus its annual frequency. Signed zero is canonicalized.
    pub fn new(
        rate: f64,
        accrual: impl Into<AccrualConvention>,
        compounding: Compounding,
    ) -> Result<Self, RateError> {
        let rate = numeric::finite("rate", rate)?;
        if let Some(count) = compounding.periodic_count() {
            numeric::within("rate", rate, Interval::greater_than(-count))?;
        }
        Ok(Self {
            rate: canonical_zero(rate),
            accrual: accrual.into(),
            compounding,
        })
    }

    /// Returns the decimal rate.
    #[must_use]
    pub const fn rate(&self) -> f64 {
        self.rate
    }
    /// Returns the complete immutable day-count context.
    #[must_use]
    pub const fn accrual(&self) -> &AccrualConvention {
        &self.accrual
    }
    /// Returns the declared compounding and frequency.
    #[must_use]
    pub const fn compounding(&self) -> Compounding {
        self.compounding
    }

    /// Computes the compound factor at a nonnegative finite year fraction.
    ///
    /// Simple uses a fused `1 + rate*time`; periodic growth uses `ln_1p`.
    /// Tiny periodic rates retain their first-order value when division by
    /// frequency would lose a subnormal input. Zero time/rate is exact one.
    ///
    /// # Errors
    /// [`RateError::Value`] for invalid time, nonpositive simple factor,
    /// overflow or factor underflow to zero. [`RateError::NumericalBoundary`]
    /// when elementary-function uncertainty straddles a range boundary.
    #[doc(alias = "compoundFactor")]
    pub fn compound_factor(&self, time: f64) -> Result<f64, RateError> {
        let time = time_input(time)?;
        if time == 0.0 || self.rate == 0.0 {
            return Ok(1.0);
        }
        let value = match self.compounding.phase(time) {
            Phase::Simple => self.rate.mul_add(time, 1.0),
            Phase::Continuous => exponential_factor(self.rate * time)?,
            Phase::Periodic(count) => exponential_factor(periodic_growth(self.rate, count) * time)?,
        };
        Ok(numeric::result_within(
            "compound factor",
            value,
            Interval::greater_than(0.0),
        )?)
    }

    /// Computes one correctly rounded reciprocal of the compound factor.
    ///
    /// # Errors
    /// Compound-factor errors, or [`RateError::Value`] for an unrepresentable
    /// positive reciprocal. [`RateError::NumericalBoundary`] when a factor
    /// evaluated through elementary functions is too close to reciprocal
    /// overflow for its range classification to be resolved.
    #[doc(alias = "discountFactor")]
    pub fn discount_factor(&self, time: f64) -> Result<f64, RateError> {
        let factor = self.compound_factor(time)?;
        if time != 0.0 && self.rate != 0.0 && !matches!(self.compounding.phase(time), Phase::Simple)
        {
            require_resolved_boundary("discount factor", -factor.ln(), f64::MAX.ln())?;
        }
        Ok(numeric::result_within(
            "discount factor",
            1.0 / factor,
            Interval::greater_than(0.0),
        )?)
    }

    /// Computes the compound factor using the quote's own date accrual.
    ///
    /// # Errors
    /// [`RateError::ReversedDates`], accrual context failures or factor errors.
    #[doc(alias = "compoundFactor")]
    pub fn compound_factor_between(&self, start: Date, end: Date) -> Result<f64, RateError> {
        self.compound_factor(self.date_time(start, end)?)
    }
    /// Computes the period discount factor using the quote's own accrual.
    ///
    /// # Errors
    /// Reversed dates, accrual failures or discount-factor errors.
    #[doc(alias = "discountFactor")]
    pub fn discount_factor_between(&self, start: Date, end: Date) -> Result<f64, RateError> {
        self.discount_factor(self.date_time(start, end)?)
    }

    fn date_time(&self, start: Date, end: Date) -> Result<f64, RateError> {
        require_order(start, end)?;
        time_input(self.accrual.year_fraction(start, end)?)
    }

    /// Infers a quote from a positive finite compound factor and horizon.
    ///
    /// At zero time, factor one chooses zero rate; other factors are rejected.
    /// Periodic inference uses `exp_m1` without subtracting nearly equal powers.
    /// A rounded rate at its active simple boundary is rejected. Otherwise,
    /// reconstruction of a very small simple factor is sensitive to rate
    /// rounding; a small relative factor error is not promised there.
    ///
    /// # Errors
    /// Invalid inputs, [`RateError::ZeroTime`], or an unrepresentable rate.
    #[expect(
        clippy::float_cmp,
        reason = "zero-time inference requires exactly the supplied unit factor, not approximate closeness"
    )]
    #[doc(alias = "impliedRate")]
    pub fn implied_rate(
        factor: f64,
        time: f64,
        accrual: impl Into<AccrualConvention>,
        compounding: Compounding,
    ) -> Result<Self, RateError> {
        let factor = numeric::within("compound factor", factor, Interval::greater_than(0.0))?;
        let time = time_input(time)?;
        if time == 0.0 {
            if factor != 1.0 {
                return Err(RateError::ZeroTime {
                    quantity: "compound factor",
                    factor,
                });
            }
            return Self::new(0.0, accrual, compounding);
        }
        let log_growth = factor.ln();
        let rate = if matches!(compounding.phase(time), Phase::Simple) {
            (factor - 1.0) / time
        } else {
            rate_from_annual(log_growth / time, time, compounding.phase(time))
        };
        let rate = rate_result("implied rate", rate, log_growth, compounding, time)?;
        Self::new(rate, accrual, compounding)
    }

    /// Infers a quote directly from a positive finite discount factor.
    ///
    /// Uses `-ln(discount)` without forming a possibly overflowing reciprocal.
    /// At zero time, only discount one is admitted and chooses zero rate.
    /// A rounded rate must remain in its active simple domain at this horizon.
    ///
    /// # Errors
    /// Invalid inputs, [`RateError::ZeroTime`], or an unrepresentable rate.
    #[expect(
        clippy::float_cmp,
        reason = "zero-time inference requires exactly the supplied unit discount, not approximate closeness"
    )]
    pub fn implied_rate_from_discount(
        discount: f64,
        time: f64,
        accrual: impl Into<AccrualConvention>,
        compounding: Compounding,
    ) -> Result<Self, RateError> {
        let discount = numeric::within("discount factor", discount, Interval::greater_than(0.0))?;
        let time = time_input(time)?;
        if time == 0.0 {
            if discount != 1.0 {
                return Err(RateError::ZeroTime {
                    quantity: "discount factor",
                    factor: discount,
                });
            }
            return Self::new(0.0, accrual, compounding);
        }
        let log_growth = -discount.ln();
        let rate = if matches!(compounding.phase(time), Phase::Simple) {
            // Rescale before dividing by a small discount, and avoid a
            // near-one reciprocal subtraction that loses significant bits.
            if discount < 0.5 {
                ((1.0 - discount) / time) / discount
            } else {
                ((1.0 - discount) / discount) / time
            }
        } else {
            rate_from_annual(log_growth / time, time, compounding.phase(time))
        };
        let rate = rate_result("implied rate", rate, log_growth, compounding, time)?;
        Self::new(rate, accrual, compounding)
    }

    /// Converts compounding at the same horizon, retaining accrual context.
    ///
    /// Periodic/continuous conversions are horizon independent for positive
    /// time. Zero time chooses zero rate. Equivalent conversions need not
    /// represent an intermediate compound factor.
    /// The rounded target rate must remain in its active simple domain.
    /// Factor reconstruction near that boundary is ill-conditioned with
    /// respect to the rounded target rate.
    ///
    /// # Errors
    /// Invalid time or active simple domain, or an unrepresentable target rate.
    #[doc(alias = "equivalentRate")]
    pub fn equivalent_rate(&self, target: Compounding, time: f64) -> Result<Self, RateError> {
        let time = time_input(time)?;
        if time == 0.0 || self.rate == 0.0 {
            return Self::new(0.0, self.accrual.clone(), target);
        }
        validate_simple_domain(self.rate, time, self.compounding.phase(time))?;
        if target.phase(time) == self.compounding.phase(time) {
            let rate = rate_result("equivalent rate", self.rate, self.rate, target, time)?;
            return Self::new(rate, self.accrual.clone(), target);
        }
        let growth = annual_growth(self.rate, time, self.compounding.phase(time))?;
        let rate = rate_from_annual(growth, time, target.phase(time));
        let rate = rate_result("equivalent rate", rate, self.rate, target, time)?;
        Self::new(rate, self.accrual.clone(), target)
    }

    /// Converts both accrual and compounding for the same ordered date pair.
    ///
    /// Source and target year fractions are independently computed from their
    /// required contexts. Positive-horizon source factor is preserved, rather
    /// than silently assuming the year fractions are equal.
    ///
    /// # Errors
    /// Reversed dates, context errors, [`RateError::ZeroTargetTime`], invalid
    /// source domain or an unrepresentable target rate.
    #[doc(alias = "equivalentRate")]
    pub fn equivalent_rate_between(
        &self,
        accrual: impl Into<AccrualConvention>,
        target: Compounding,
        start: Date,
        end: Date,
    ) -> Result<Self, RateError> {
        let source_time = self.date_time(start, end)?;
        let accrual = accrual.into();
        let target_time = time_input(accrual.year_fraction(start, end)?)?;
        if source_time == 0.0 || self.rate == 0.0 {
            return Self::new(0.0, accrual, target);
        }
        if target_time == 0.0 {
            return Err(RateError::ZeroTargetTime {
                source_time,
                rate: self.rate,
            });
        }
        validate_simple_domain(self.rate, source_time, self.compounding.phase(source_time))?;
        if source_time.to_bits() == target_time.to_bits() {
            let rate = self.equivalent_rate(target, source_time)?.rate();
            return Self::new(rate, accrual, target);
        }
        let growth = annual_growth(self.rate, source_time, self.compounding.phase(source_time))?;
        let target_growth = growth * (source_time / target_time);
        let rate = rate_from_annual(target_growth, target_time, target.phase(target_time));
        let rate = rate_result("equivalent rate", rate, self.rate, target, target_time)?;
        Self::new(rate, accrual, target)
    }

    /// Discounts a finite signed amount at a year-fraction horizon.
    ///
    /// # Errors
    /// Invalid amount/horizon/factor, product overflow or zero underflow of a
    /// nonzero amount. Negative cash flows are permitted.
    pub fn present_value(&self, amount: f64, time: f64) -> Result<f64, RateError> {
        let amount = numeric::finite("amount", amount)?;
        let value = amount * self.discount_factor(time)?;
        if amount == 0.0 {
            return Ok(0.0);
        }
        let domain = if amount > 0.0 {
            Interval::greater_than(0.0)
        } else {
            Interval::less_than(0.0)
        };
        Ok(numeric::result_within("present value", value, domain)?)
    }

    /// Discounts a finite signed amount between ordered dates.
    ///
    /// # Errors
    /// Reversed dates, accrual context errors or present-value errors.
    pub fn present_value_between(
        &self,
        amount: f64,
        start: Date,
        end: Date,
    ) -> Result<f64, RateError> {
        self.present_value(amount, self.date_time(start, end)?)
    }
}

fn canonical_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}
fn time_input(time: f64) -> Result<f64, RateError> {
    Ok(canonical_zero(numeric::within(
        "time",
        time,
        Interval::at_least(0.0),
    )?))
}
fn require_order(start: Date, end: Date) -> Result<(), RateError> {
    if start > end {
        Err(RateError::ReversedDates { start, end })
    } else {
        Ok(())
    }
}

fn periodic_growth(rate: f64, count: f64) -> f64 {
    let scaled = rate / count;
    if scaled.abs() < f64::EPSILON {
        rate
    } else if scaled <= -0.5 {
        // Sterbenz subtraction makes count + rate exact in this range.
        // Dividing the small positive difference avoids rounding rate/count
        // to a value whose distance from -1 has lost most significant bits.
        count * ((count + rate) / count).ln()
    } else {
        count * scaled.ln_1p()
    }
}

fn exponential_factor(logarithm: f64) -> Result<f64, RateError> {
    require_resolved_boundary("compound factor", logarithm, f64::MAX.ln())?;
    // Half the minimum subnormal is the exact round-to-zero threshold.
    // Form its logarithm without first rounding that half to zero.
    let zero_boundary = f64::from_bits(1).ln() - std::f64::consts::LN_2;
    require_resolved_boundary("compound factor", logarithm, zero_boundary)?;
    Ok(logarithm.exp())
}

fn require_resolved_boundary(
    quantity: &'static str,
    logarithm: f64,
    boundary: f64,
) -> Result<(), RateError> {
    // TOL-E3's measured exponent-error allowance. This is an explicit
    // conservative range policy, not a proof of a platform libm error bound.
    // Avoid claiming a finite result when that allowance straddles the
    // representable boundary; exact simple arithmetic needs no such guard.
    let allowance = 8.0 * (f64::EPSILON / 2.0) * (1.0 + logarithm.abs());
    if logarithm.is_finite() && (logarithm - boundary).abs() <= allowance {
        Err(RateError::NumericalBoundary {
            quantity,
            logarithm,
        })
    } else {
        Ok(())
    }
}

fn validate_simple_domain(rate: f64, time: f64, phase: Phase) -> Result<(), RateError> {
    if matches!(phase, Phase::Simple) {
        let factor = rate.mul_add(time, 1.0);
        if factor <= 0.0 {
            numeric::result_within("compound factor", factor, Interval::greater_than(0.0))?;
        }
    }
    Ok(())
}

fn annual_growth(rate: f64, time: f64, phase: Phase) -> Result<f64, RateError> {
    let growth = match phase {
        Phase::Continuous => rate,
        Phase::Periodic(count) => periodic_growth(rate, count),
        Phase::Simple => {
            let product = rate * time;
            if product.abs() < f64::EPSILON {
                return Ok(rate);
            }
            let log = if product.is_infinite() {
                rate.ln() + time.ln()
            } else if product <= -0.5 {
                rate.mul_add(time, 1.0).ln()
            } else {
                product.ln_1p()
            };
            log / time
        }
    };
    Ok(numeric::finite_result("annual log growth", growth)?)
}

fn rate_from_annual(growth: f64, time: f64, phase: Phase) -> f64 {
    match phase {
        Phase::Continuous => growth,
        Phase::Periodic(count) => {
            let scaled = growth / count;
            if scaled.abs() < f64::EPSILON {
                growth
            } else {
                count * scaled.exp_m1()
            }
        }
        Phase::Simple => {
            let log_growth = growth * time;
            if log_growth.abs() < f64::EPSILON {
                growth
            } else {
                simple_rate(log_growth, time)
            }
        }
    }
}

fn simple_rate(log_growth: f64, time: f64) -> f64 {
    let increment = log_growth.exp_m1();
    if increment.is_infinite() && log_growth > 0.0 && time > 1.0 {
        (log_growth - time.ln()).exp() - 1.0 / time
    } else {
        increment / time
    }
}

fn rate_result(
    quantity: &'static str,
    rate: f64,
    sign: f64,
    compounding: Compounding,
    time: f64,
) -> Result<f64, RateError> {
    let rate = numeric::finite_result(quantity, rate)?;
    if sign > 0.0 {
        numeric::result_within(quantity, rate, Interval::greater_than(0.0))?;
    } else if sign < 0.0 {
        numeric::result_within(quantity, rate, Interval::less_than(0.0))?;
    }
    if let Some(count) = compounding.periodic_count() {
        numeric::result_within(quantity, rate, Interval::greater_than(-count))?;
    }
    // A mathematically positive source factor can imply a rounded rate at
    // the target's simple boundary. Reject that unusable horizon explicitly;
    // conversion still need not materialize a finite intermediate factor.
    validate_simple_domain(rate, time, compounding.phase(time))?;
    Ok(canonical_zero(rate))
}

/// A rejected rate input, convention context or computed result.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum RateError {
    /// Numeric validation failed; kind `rate.value`, source is the value error.
    Value(ValueError),
    /// Accrual failed; kind `rate.day_count`, source preserves the day-count error.
    DayCount(DayCountError),
    /// ICMA context failed; kind `rate.icma`, source is the ICMA error.
    Icma(IcmaError),
    /// Elementary-function uncertainty straddles a factor range boundary.
    /// Kind `rate.numerical_boundary`; the caller receives no ambiguous factor.
    #[non_exhaustive]
    NumericalBoundary {
        /// Compound or discount factor whose range classification is unresolved.
        quantity: &'static str,
        /// Computed natural logarithm used to classify the factor's range.
        logarithm: f64,
    },
    /// A periodic method received Once; kind `rate.invalid_frequency`.
    #[non_exhaustive]
    InvalidFrequency {
        /// Rejected frequency.
        frequency: Frequency,
    },
    /// Date valuation was reversed; kind `rate.reversed_dates`.
    #[non_exhaustive]
    ReversedDates {
        /// Requested start.
        start: Date,
        /// Requested end.
        end: Date,
    },
    /// Zero time cannot imply a nonunit factor; kind `rate.zero_time`.
    #[non_exhaustive]
    ZeroTime {
        /// The supplied quantity: compound factor or discount factor.
        quantity: &'static str,
        /// The nonunit supplied value.
        factor: f64,
    },
    /// A zero target horizon cannot preserve nonzero source accrual; kind `rate.zero_target_time`.
    #[non_exhaustive]
    ZeroTargetTime {
        /// Positive source horizon.
        source_time: f64,
        /// Nonzero source rate.
        rate: f64,
    },
}

impl RateError {
    /// Returns the stable `rate.<variant>` kind name.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Value(_) => "rate.value",
            Self::DayCount(_) => "rate.day_count",
            Self::Icma(_) => "rate.icma",
            Self::NumericalBoundary { .. } => "rate.numerical_boundary",
            Self::InvalidFrequency { .. } => "rate.invalid_frequency",
            Self::ReversedDates { .. } => "rate.reversed_dates",
            Self::ZeroTime { .. } => "rate.zero_time",
            Self::ZeroTargetTime { .. } => "rate.zero_target_time",
        }
    }
}

impl fmt::Display for RateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(error) => error.fmt(f),
            Self::DayCount(error) => error.fmt(f),
            Self::Icma(error) => error.fmt(f),
            Self::NumericalBoundary {
                quantity,
                logarithm,
            } => write!(
                f,
                "{quantity} range is unresolved near a binary64 boundary (logarithm {logarithm})"
            ),
            Self::InvalidFrequency { frequency } => {
                write!(f, "{frequency} is not a periodic compounding frequency")
            }
            Self::ReversedDates { start, end } => write!(
                f,
                "rate valuation requires start {start} not after end {end}"
            ),
            Self::ZeroTime { quantity, factor } => {
                write!(f, "zero time cannot imply {quantity} {factor}")
            }
            Self::ZeroTargetTime { source_time, rate } => write!(
                f,
                "zero target time cannot preserve source rate {rate} over horizon {source_time}"
            ),
        }
    }
}

impl StdError for RateError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Value(error) => Some(error),
            Self::DayCount(error) => Some(error),
            Self::Icma(error) => Some(error),
            _ => None,
        }
    }
}
impl From<ValueError> for RateError {
    fn from(value: ValueError) -> Self {
        Self::Value(value)
    }
}
impl From<DayCountError> for RateError {
    fn from(value: DayCountError) -> Self {
        Self::DayCount(value)
    }
}
impl From<IcmaError> for RateError {
    fn from(value: IcmaError) -> Self {
        Self::Icma(value)
    }
}

const METHOD_IDENTIFIERS: Identifiers<CompoundingMethod> = Identifiers::new(
    "compounding method",
    &[
        Entry {
            value: CompoundingMethod::Simple,
            canonical: "Simple",
            aliases: &[],
        },
        Entry {
            value: CompoundingMethod::Compounded,
            canonical: "Compounded",
            aliases: &[],
        },
        Entry {
            value: CompoundingMethod::Continuous,
            canonical: "Continuous",
            aliases: &[],
        },
        Entry {
            value: CompoundingMethod::SimpleThenCompounded,
            canonical: "SimpleThenCompounded",
            aliases: &[],
        },
        Entry {
            value: CompoundingMethod::CompoundedThenSimple,
            canonical: "CompoundedThenSimple",
            aliases: &[],
        },
    ],
    &[],
);
