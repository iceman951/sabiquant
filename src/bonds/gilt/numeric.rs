//! Exact source coefficients, observed quasi-period discounts and retained risk.
//!
//! The finite elementary comparison profile is conditional. Exact supplied-value
//! rounding here does not produce an enclosure of an ideal financial price.

use std::cmp::Ordering;
use std::error::Error as StdError;
use std::fmt;

use crate::amounts::uint::Wide;
use crate::bonds::{BondPrice, BondPriceKind, BondYieldSensitivity};
use crate::error::Interval;
use crate::math::{
    EvaluationCounts, EvaluationStage, FallibleSolverError, RootBracket, SolveReport, SolverError,
    SolverOptions, brent_with_payload, exact_sum,
};
use crate::numeric as validation;
use crate::rates::{Compounding, RateError, validate_rate};
use crate::time::{Date, Frequency};

use super::super::yield_risk::round_wide_ratio_to_f64;
use super::{
    GiltError, GiltPricingPlan, Metadata, PreparedBundle, SOURCE_VERSION, Scalar, SourceRational,
    capacity,
};

const SIGN: u64 = 1_u64 << 63;
const FRACTION: u64 = (1_u64 << 52) - 1;
const INFINITY: u64 = 0x7ff0_0000_0000_0000;
const SCALE_THRESHOLD: u64 = 1993_u64 << 52;
const TINY: f64 = f64::from_bits(971_u64 << 52); // 2^-52
const GUARD_SCALE: f64 = f64::from_bits(973_u64 << 52); // 2^-50 = 8u
const CONSTANT_RADIUS: f64 = f64::from_bits(979_u64 << 52); // 2^-44
const LOG_OVERFLOW: f64 = f64::from_bits(0x4086_2e42_fefa_39ef);
const LOG_HALF_MINIMUM: f64 = f64::from_bits(0xc087_4910_d52d_3052);
type RiskWide = Wide<243>;
type ResidualWide = Wide<34>;

/// A conventional gilt price observation in native per100 units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GiltValuation {
    metadata: Metadata,
    yield_rate: f64,
    dirty: f64,
    accrued: f64,
}

/// Retained conventional gilt yield derivatives and independently admitted ratios.
///
/// Raw IEEE range failure does not discard the extended numerator required by
/// a normalized getter. These are derivatives of the ideal source model, not
/// of the discontinuous floating program. The value is immutable and not Copy.
#[derive(Clone, PartialEq)]
pub struct GiltRisk {
    metadata: Metadata,
    yield_rate: f64,
    dirty: f64,
    first: RiskWide,
    second: RiskWide,
    moment: RiskWide,
}

impl fmt::Debug for GiltRisk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GiltRisk")
            .field("metadata", &self.metadata)
            .field("yield_rate", &self.yield_rate)
            .field("dirty_per_100", &self.dirty)
            .finish_non_exhaustive()
    }
}

/// A counted conventional gilt inverse result with the selected cached price.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GiltYieldReport {
    metadata: Metadata,
    yield_rate: f64,
    dirty: f64,
    accrued: f64,
    target: BondPrice,
    dirty_target: f64,
    initial_bracket: RootBracket,
    options: SolverOptions,
    solve_report: SolveReport,
    residual_scale: u8,
}

/// A rejected target, preparation, callback or bounded conventional gilt solve.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum GiltYieldError {
    /// Target clean-to-dirty conversion failed; kind `gilt_yield.target`.
    #[non_exhaustive]
    Target {
        /// Original caller target and quote kind.
        target: BondPrice,
        /// Target operation (`target clean shift`).
        phase: &'static str,
        /// Immediate typed source failure.
        cause: GiltError,
    },
    /// PV workspace preparation failed; kind `gilt_yield.preparation`.
    #[non_exhaustive]
    Preparation {
        /// Workspace phase (`yield present value terms`).
        phase: &'static str,
        /// Immediate allocation or capacity failure.
        cause: GiltError,
    },
    /// Bounded solving failed on admitted finite callbacks; kind `gilt_yield.solver`.
    Solver(SolverError),
    /// An actually invoked callback failed; kind `gilt_yield.callback`.
    #[non_exhaustive]
    Callback {
        /// Solver invocation stage.
        stage: EvaluationStage,
        /// Canonical attempted finite yield coordinate.
        point: f64,
        /// Counts including this failed function invocation.
        evaluations: EvaluationCounts,
        /// Immediate source, factor, PV, reduction or residual failure.
        cause: GiltError,
    },
    /// The positive-price model cannot attain the resolved target.
    /// Kind `gilt_yield.nonpositive_target`; no callback was performed.
    #[non_exhaustive]
    NonPositiveTarget {
        /// Original caller target.
        target: BondPrice,
        /// Once-resolved native dirty target per100.
        dirty_target_per_100: f64,
        /// Cached source AI binary observation used by any clean shift.
        accrued_per_100: f64,
        /// Zero function and derivative evaluations.
        evaluations: EvaluationCounts,
    },
}

impl GiltYieldError {
    /// Returns the stable `gilt_yield.*` error kind.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Target { .. } => "gilt_yield.target",
            Self::Preparation { .. } => "gilt_yield.preparation",
            Self::Solver(_) => "gilt_yield.solver",
            Self::Callback { .. } => "gilt_yield.callback",
            Self::NonPositiveTarget { .. } => "gilt_yield.nonpositive_target",
        }
    }
}

impl fmt::Display for GiltYieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Target {
                target,
                phase,
                cause,
                ..
            } => write!(f, "{phase} for {target:?}: {cause}"),
            Self::Preparation { phase, cause, .. } => write!(f, "{phase}: {cause}"),
            Self::Solver(cause) => fmt::Display::fmt(cause, f),
            Self::Callback {
                stage,
                point,
                evaluations,
                cause,
                ..
            } => write!(
                f,
                "gilt callback at {stage:?}, y={point}, after {} function evaluations: {cause}",
                evaluations.function()
            ),
            Self::NonPositiveTarget {
                dirty_target_per_100,
                ..
            } => write!(
                f,
                "nonpositive conventional gilt dirty target {dirty_target_per_100}"
            ),
        }
    }
}

impl StdError for GiltYieldError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Target { cause, .. }
            | Self::Preparation { cause, .. }
            | Self::Callback { cause, .. } => Some(cause),
            Self::Solver(cause) => Some(cause),
            Self::NonPositiveTarget { .. } => None,
        }
    }
}

fn zero(value: f64) -> f64 {
    if value.to_bits() & !SIGN == 0 {
        0.0
    } else {
        value
    }
}

fn quote(value: f64) -> Result<f64, GiltError> {
    let compounding =
        Compounding::compounded(Frequency::Semiannual).map_err(|cause| GiltError::Rate {
            index: None,
            phase: "yield quote",
            cause,
        })?;
    validate_rate(value, compounding).map_err(|cause| GiltError::Rate {
        index: None,
        phase: "yield quote",
        cause,
    })
}

fn reserve(count: usize, phase: &'static str) -> Result<Vec<f64>, GiltError> {
    let mut values = Vec::new();
    values
        .try_reserve(count)
        .map_err(|_| GiltError::AllocationFailed {
            phase,
            requested: count,
        })?;
    Ok(values)
}

fn decode(value: f64, index: Option<usize>, phase: &'static str) -> Result<(u64, i32), GiltError> {
    if !value.is_finite() || value.to_bits() & !SIGN == 0 {
        return Err(capacity(index, phase, "decode binary"));
    }
    let bits = value.to_bits();
    let field =
        i32::try_from((bits >> 52) & 0x7ff).map_err(|_| capacity(index, phase, "decode binary"))?;
    let fraction = bits & FRACTION;
    Ok(if field == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1_u64 << 52), field - 1075)
    })
}

/// One direct IEEE admission of an exact signed source rational, including AI.
pub(super) fn source_binary(
    source: SourceRational,
    index: Option<usize>,
    phase: &'static str,
    operation: &'static str,
) -> Result<f64, GiltError> {
    let bits = round_wide_ratio_to_f64(source.numerator, source.denominator, 0)
        .ok_or_else(|| capacity(index, phase, operation))?;
    let sign = if source.negative && bits != 0 {
        SIGN
    } else {
        0
    };
    Ok(f64::from_bits(sign | bits))
}

fn quasi_exponent(g: f64, l: u32, s: u32, index: usize) -> Result<f64, GiltError> {
    if g.to_bits() & !SIGN == 0 {
        return Ok(0.0);
    }
    let (sig, exponent) = decode(g, Some(index), "discount factor")?;
    let numerator = Scalar::from_u128(u128::from(sig))
        .checked_mul_small(l)
        .ok_or_else(|| capacity(Some(index), "discount factor", "round quasi exponent"))?;
    let denominator = Scalar::from_u128(2 * u128::from(s));
    let bits = round_wide_ratio_to_f64(numerator, denominator, exponent)
        .ok_or_else(|| capacity(Some(index), "discount factor", "round quasi exponent"))?;
    let sign = if !g.is_sign_negative() && bits != 0 {
        SIGN
    } else {
        0
    };
    Ok(f64::from_bits(sign | bits))
}

fn guard(z: f64, index: usize) -> Result<(), GiltError> {
    let a = z.abs() * GUARD_SCALE;
    let radius = ((a + CONSTANT_RADIUS).next_up() + GUARD_SCALE).next_up();
    for boundary in [LOG_OVERFLOW, LOG_HALF_MINIMUM] {
        let distance = (z - boundary).abs();
        let lower = if distance.to_bits() & !SIGN == 0 {
            0.0
        } else {
            distance.next_down().max(0.0)
        };
        if lower <= radius {
            return Err(GiltError::Rate {
                index: Some(index),
                phase: "discount factor",
                cause: RateError::NumericalBoundary {
                    quantity: "discount factor",
                    logarithm: z,
                },
            });
        }
    }
    Ok(())
}

fn discount(y: f64, l: u32, s: u32, index: usize) -> Result<f64, GiltError> {
    if y.to_bits() & !SIGN == 0 {
        return Ok(1.0);
    }
    let tiny = y.abs() < TINY;
    let g = if tiny {
        y
    } else if y <= -1.0 {
        2.0 * ((2.0 + y) / 2.0).ln()
    } else {
        2.0 * (y / 2.0).ln_1p()
    };
    validation::finite_result("gilt scaled logarithm", g).map_err(|cause| GiltError::Rate {
        index: Some(index),
        phase: "discount factor",
        cause: RateError::Value(cause),
    })?;
    let z = quasi_exponent(g, l, s, index)?;
    if z.to_bits() & !SIGN == 0 {
        return Ok(1.0);
    }
    if !tiny {
        guard(z, index)?;
    }
    validation::result_within("discount factor", z.exp(), Interval::greater_than(0.0)).map_err(
        |cause| GiltError::Rate {
            index: Some(index),
            phase: "discount factor",
            cause: RateError::Value(cause),
        },
    )
}

fn source_pv(amount: SourceRational, discount: f64, index: usize) -> Result<f64, GiltError> {
    const PHASE: &str = "source present value";
    if amount.numerator.is_zero() {
        return Ok(0.0);
    }
    let (sig, exponent) = decode(discount, Some(index), PHASE)?;
    let numerator = amount
        .numerator
        .checked_mul(Scalar::from_u128(u128::from(sig)))
        .ok_or_else(|| capacity(Some(index), PHASE, "round source present value"))?;
    let bits = round_wide_ratio_to_f64(numerator, amount.denominator, exponent)
        .ok_or_else(|| capacity(Some(index), PHASE, "round source present value"))?;
    validation::result_within(
        "gilt present value",
        f64::from_bits(bits),
        Interval::greater_than(0.0),
    )
    .map_err(|cause| GiltError::Value {
        index: Some(index),
        phase: PHASE,
        cause,
    })
}

fn observe(bundle: &PreparedBundle, y: f64, s: u32) -> Result<f64, GiltError> {
    let factor = discount(y, bundle.l, s, bundle.index)?;
    source_pv(bundle.amount, factor, bundle.index)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Extended {
    significand: u64,
    exponent: i32,
}

fn extended_ratio(
    numerator: Scalar,
    denominator: Scalar,
    exponent: i32,
    index: usize,
    phase: &'static str,
) -> Result<Extended, GiltError> {
    if numerator.is_zero() {
        return Ok(Extended {
            significand: 0,
            exponent: 0,
        });
    }
    if denominator.is_zero() {
        return Err(capacity(Some(index), phase, "normalize rational"));
    }
    let nb = i32::try_from(numerator.bits())
        .map_err(|_| capacity(Some(index), phase, "normalize rational"))?;
    let db = i32::try_from(denominator.bits())
        .map_err(|_| capacity(Some(index), phase, "normalize rational"))?;
    let k0 = nb - db;
    let at_least = if k0 >= 0 {
        numerator
            >= denominator
                .checked_shl(k0.unsigned_abs())
                .ok_or_else(|| capacity(Some(index), phase, "normalize rational"))?
    } else {
        numerator
            .checked_shl(k0.unsigned_abs())
            .ok_or_else(|| capacity(Some(index), phase, "normalize rational"))?
            >= denominator
    };
    let k = if at_least { k0 } else { k0 - 1 };
    let shift = 52 - k;
    let (num, den) = if shift >= 0 {
        (
            numerator
                .checked_shl(shift.unsigned_abs())
                .ok_or_else(|| capacity(Some(index), phase, "round scaled term"))?,
            denominator,
        )
    } else {
        (
            numerator,
            denominator
                .checked_shl(shift.unsigned_abs())
                .ok_or_else(|| capacity(Some(index), phase, "round scaled term"))?,
        )
    };
    let (q, r) = num
        .div_rem(den)
        .ok_or_else(|| capacity(Some(index), phase, "round scaled term"))?;
    let mut sig = q
        .to_u128()
        .and_then(|v| u64::try_from(v).ok())
        .ok_or_else(|| capacity(Some(index), phase, "round scaled term"))?;
    let complement = den
        .checked_sub(r)
        .ok_or_else(|| capacity(Some(index), phase, "round scaled term"))?;
    let mut output_exp = exponent + k - 52;
    if r > complement || (r == complement && sig & 1 != 0) {
        sig += 1;
        if sig == 1_u64 << 53 {
            sig >>= 1;
            output_exp += 1;
        }
    }
    Ok(Extended {
        significand: sig,
        exponent: output_exp,
    })
}

fn accumulate(
    sum: &mut RiskWide,
    term: Extended,
    index: usize,
    phase: &'static str,
) -> Result<(), GiltError> {
    if term.significand == 0 {
        return Ok(());
    }
    let shift = u32::try_from(term.exponent + 8192)
        .map_err(|_| capacity(Some(index), phase, "accumulate risk"))?;
    let value = RiskWide::from_u128(u128::from(term.significand))
        .checked_shl(shift)
        .ok_or_else(|| capacity(Some(index), phase, "accumulate risk"))?;
    *sum = sum
        .checked_add(value)
        .ok_or_else(|| capacity(Some(index), phase, "accumulate risk"))?;
    Ok(())
}

fn scalar_product(values: &[u64], index: usize, phase: &'static str) -> Result<Scalar, GiltError> {
    let mut result = Scalar::from_u128(1);
    for &value in values {
        result = result
            .checked_mul(Scalar::from_u128(u128::from(value)))
            .ok_or_else(|| capacity(Some(index), phase, "normalize rational"))?;
    }
    Ok(result)
}

fn risk_terms(
    pv: f64,
    delta: f64,
    l: u32,
    s: u32,
    index: usize,
) -> Result<[Extended; 3], GiltError> {
    if pv.to_bits() & !SIGN == 0 {
        return Ok([Extended {
            significand: 0,
            exponent: 0,
        }; 3]);
    }
    let (p, pe) = decode(pv, Some(index), "price derivative")?;
    let (d, de) = decode(delta, Some(index), "price derivative")?;
    let l = u64::from(l);
    let s = u64::from(s);
    let first = extended_ratio(
        scalar_product(&[p, l], index, "price derivative")?,
        scalar_product(&[s, d], index, "price derivative")?,
        pe - de,
        index,
        "price derivative",
    )?;
    let second = extended_ratio(
        scalar_product(&[p, l, l + s], index, "price second derivative")?,
        scalar_product(&[s, s, d, d], index, "price second derivative")?,
        pe - 2 * de,
        index,
        "price second derivative",
    )?;
    let moment = extended_ratio(
        scalar_product(&[p, l], index, "time weighted amount")?,
        Scalar::from_u128(2 * u128::from(s)),
        pe,
        index,
        "time weighted amount",
    )?;
    Ok([first, second, moment])
}

fn finish_risk(bits: u64, negative: bool, phase: &'static str) -> Result<f64, GiltError> {
    let sign = if negative { SIGN } else { 0 };
    let value = f64::from_bits(sign | bits);
    validation::result_within(
        phase,
        value,
        if negative {
            Interval::less_than(0.0)
        } else {
            Interval::greater_than(0.0)
        },
    )
    .map_err(|cause| GiltError::Value {
        index: None,
        phase,
        cause,
    })
}

fn raw(sum: RiskWide, negative: bool, phase: &'static str) -> Result<f64, GiltError> {
    if sum.is_zero() {
        return Ok(0.0);
    }
    let bits = round_wide_ratio_to_f64(sum, RiskWide::from_u128(1), -8192)
        .ok_or_else(|| capacity(None, phase, "round risk value"))?;
    finish_risk(bits, negative, phase)
}

fn normalized(sum: RiskWide, dirty: f64, y: f64, phase: &'static str) -> Result<f64, GiltError> {
    if dirty.to_bits() & !SIGN == 0 {
        return Err(GiltError::ZeroDirtyPrice {
            measure: phase,
            yield_rate: y,
        });
    }
    if sum.is_zero() {
        return Ok(0.0);
    }
    let (sig, exponent) = decode(dirty, None, phase)?;
    let bits = round_wide_ratio_to_f64(sum, RiskWide::from_u128(u128::from(sig)), -8192 - exponent)
        .ok_or_else(|| capacity(None, phase, "normalize risk value"))?;
    finish_risk(bits, false, phase)
}

fn decode_residual(value: f64, phase: &'static str) -> Result<(ResidualWide, bool), GiltError> {
    if !value.is_finite() {
        return Err(capacity(None, phase, "residual difference"));
    }
    let bits = value.to_bits();
    let field = u32::try_from((bits >> 52) & 0x7ff)
        .map_err(|_| capacity(None, phase, "residual difference"))?;
    let fraction = bits & FRACTION;
    let (sig, shift) = if field == 0 {
        (fraction, 0)
    } else {
        (fraction | (1_u64 << 52), field - 1)
    };
    let value = ResidualWide::from_u128(u128::from(sig))
        .checked_shl(shift)
        .ok_or_else(|| capacity(None, phase, "residual difference"))?;
    Ok((value, bits & SIGN != 0 && sig != 0))
}

fn difference(
    first: f64,
    second: f64,
    phase: &'static str,
) -> Result<(ResidualWide, bool), GiltError> {
    let (a, an) = decode_residual(first, phase)?;
    let (b, bn) = decode_residual(second, phase)?;
    if an != bn {
        Ok((
            a.checked_add(b)
                .ok_or_else(|| capacity(None, phase, "residual difference"))?,
            an,
        ))
    } else {
        match a.cmp(&b) {
            Ordering::Equal => Ok((ResidualWide::ZERO, false)),
            Ordering::Greater => Ok((
                a.checked_sub(b)
                    .ok_or_else(|| capacity(None, phase, "residual difference"))?,
                an,
            )),
            Ordering::Less => Ok((
                b.checked_sub(a)
                    .ok_or_else(|| capacity(None, phase, "residual difference"))?,
                !an,
            )),
        }
    }
}

fn residual(dirty: f64, target: f64, scale: u8) -> Result<f64, GiltError> {
    const PHASE: &str = "yield residual";
    let (magnitude, negative) = difference(dirty, target, PHASE)?;
    if magnitude.is_zero() {
        return Ok(0.0);
    }
    let bits = round_wide_ratio_to_f64(
        magnitude,
        ResidualWide::from_u128(1),
        -1074 - i32::from(scale),
    )
    .ok_or_else(|| capacity(None, PHASE, "round residual"))?;
    if bits == 0 || bits == INFINITY {
        return Err(capacity(None, PHASE, "round residual"));
    }
    Ok(f64::from_bits(bits | if negative { SIGN } else { 0 }))
}

fn amount_residual(dirty: f64, target: f64) -> Result<f64, GiltError> {
    const PHASE: &str = "yield amount residual";
    let (magnitude, negative) = difference(dirty, target, PHASE)?;
    if magnitude.is_zero() {
        return Ok(0.0);
    }
    let bits = round_wide_ratio_to_f64(magnitude, ResidualWide::from_u128(1), -1074)
        .ok_or_else(|| capacity(None, PHASE, "round residual"))?;
    let value = f64::from_bits(bits | if negative { SIGN } else { 0 });
    validation::finite_result(PHASE, value).map_err(|cause| GiltError::Value {
        index: None,
        phase: PHASE,
        cause,
    })
}

fn clean_price(dirty: f64, accrued: f64) -> Result<BondPrice, GiltError> {
    let value = exact_sum(&[dirty, -accrued]).map_err(|cause| GiltError::Sum {
        phase: "clean price reduction",
        cause,
    })?;
    Ok(BondPrice {
        value_per_100_face: zero(value),
        kind: BondPriceKind::Clean,
    })
}

impl GiltValuation {
    /// Returns the immutable source/model identifier; performs no source fetch.
    #[must_use]
    pub const fn source_version(self) -> &'static str {
        SOURCE_VERSION
    }
    /// Returns the original issue date.
    #[must_use]
    pub const fn issue_date(self) -> Date {
        self.metadata.issue_date
    }
    /// Returns the unadjusted maturity date.
    #[must_use]
    pub const fn maturity_date(self) -> Date {
        self.metadata.maturity_date
    }
    /// Returns the original exact annual coupon per100.
    #[must_use]
    pub const fn coupon_per_100(self) -> crate::amounts::DecimalAmount {
        self.metadata.coupon_per_100
    }
    /// Returns the first actual dividend date.
    #[must_use]
    pub const fn first_dividend_date(self) -> Date {
        self.metadata.first_dividend_date
    }
    /// Returns the inferred first-period source role.
    #[must_use]
    pub const fn first_period(self) -> super::GiltFirstPeriod {
        self.metadata.first_period
    }
    /// Returns the retained explicit calendar-EOM policy.
    #[must_use]
    pub const fn end_of_month(self) -> crate::calendar::EndOfMonth {
        self.metadata.end_of_month
    }
    /// Returns the prepared settlement date.
    #[must_use]
    pub const fn settlement_date(self) -> Date {
        self.metadata.settlement_date
    }
    /// Returns the strictly next unadjusted quasi date.
    #[must_use]
    pub const fn next_quasi_date(self) -> Date {
        self.metadata.next_quasi_date
    }
    /// Returns n, the full quasi periods after the next date.
    #[must_use]
    pub const fn remaining_quasi_periods(self) -> u32 {
        self.metadata.n
    }
    /// Returns r, days from settlement to the next quasi date.
    #[must_use]
    pub const fn remaining_days(self) -> u32 {
        self.metadata.r
    }
    /// Returns s, the full current quasi interval in civil days.
    #[must_use]
    pub const fn quasi_period_days(self) -> u32 {
        self.metadata.s
    }
    /// Returns the annual decimal yield coordinate of this observation.
    #[must_use]
    pub const fn yield_rate(self) -> f64 {
        self.yield_rate
    }
    /// Returns the cached observed dirty price in native per100 units.
    #[must_use]
    pub const fn dirty_per_100(self) -> f64 {
        self.dirty
    }
}

impl GiltRisk {
    /// Returns the immutable source/model identifier; performs no source fetch.
    #[must_use]
    pub const fn source_version(&self) -> &'static str {
        SOURCE_VERSION
    }
    /// Returns the original issue date.
    #[must_use]
    pub const fn issue_date(&self) -> Date {
        self.metadata.issue_date
    }
    /// Returns the unadjusted maturity date.
    #[must_use]
    pub const fn maturity_date(&self) -> Date {
        self.metadata.maturity_date
    }
    /// Returns the original exact annual coupon per100.
    #[must_use]
    pub const fn coupon_per_100(&self) -> crate::amounts::DecimalAmount {
        self.metadata.coupon_per_100
    }
    /// Returns the first actual dividend date.
    #[must_use]
    pub const fn first_dividend_date(&self) -> Date {
        self.metadata.first_dividend_date
    }
    /// Returns the inferred first-period source role.
    #[must_use]
    pub const fn first_period(&self) -> super::GiltFirstPeriod {
        self.metadata.first_period
    }
    /// Returns the retained explicit calendar-EOM policy.
    #[must_use]
    pub const fn end_of_month(&self) -> crate::calendar::EndOfMonth {
        self.metadata.end_of_month
    }
    /// Returns the prepared settlement date.
    #[must_use]
    pub const fn settlement_date(&self) -> Date {
        self.metadata.settlement_date
    }
    /// Returns the strictly next unadjusted quasi date.
    #[must_use]
    pub const fn next_quasi_date(&self) -> Date {
        self.metadata.next_quasi_date
    }
    /// Returns n, the full quasi periods after the next date.
    #[must_use]
    pub const fn remaining_quasi_periods(&self) -> u32 {
        self.metadata.n
    }
    /// Returns r, days from settlement to the next quasi date.
    #[must_use]
    pub const fn remaining_days(&self) -> u32 {
        self.metadata.r
    }
    /// Returns s, the full current quasi interval in civil days.
    #[must_use]
    pub const fn quasi_period_days(&self) -> u32 {
        self.metadata.s
    }
    /// Returns the annual decimal yield coordinate of this observation.
    #[must_use]
    pub const fn yield_rate(&self) -> f64 {
        self.yield_rate
    }
    /// Returns the cached observed dirty price in native per100 units.
    #[must_use]
    pub const fn dirty_per_100(&self) -> f64 {
        self.dirty
    }
}

impl GiltYieldReport {
    /// Returns the immutable source/model identifier; performs no source fetch.
    #[must_use]
    pub const fn source_version(self) -> &'static str {
        SOURCE_VERSION
    }
    /// Returns the original issue date.
    #[must_use]
    pub const fn issue_date(self) -> Date {
        self.metadata.issue_date
    }
    /// Returns the unadjusted maturity date.
    #[must_use]
    pub const fn maturity_date(self) -> Date {
        self.metadata.maturity_date
    }
    /// Returns the original exact annual coupon per100.
    #[must_use]
    pub const fn coupon_per_100(self) -> crate::amounts::DecimalAmount {
        self.metadata.coupon_per_100
    }
    /// Returns the first actual dividend date.
    #[must_use]
    pub const fn first_dividend_date(self) -> Date {
        self.metadata.first_dividend_date
    }
    /// Returns the inferred first-period source role.
    #[must_use]
    pub const fn first_period(self) -> super::GiltFirstPeriod {
        self.metadata.first_period
    }
    /// Returns the retained explicit calendar-EOM policy.
    #[must_use]
    pub const fn end_of_month(self) -> crate::calendar::EndOfMonth {
        self.metadata.end_of_month
    }
    /// Returns the prepared settlement date.
    #[must_use]
    pub const fn settlement_date(self) -> Date {
        self.metadata.settlement_date
    }
    /// Returns the strictly next unadjusted quasi date.
    #[must_use]
    pub const fn next_quasi_date(self) -> Date {
        self.metadata.next_quasi_date
    }
    /// Returns n, the full quasi periods after the next date.
    #[must_use]
    pub const fn remaining_quasi_periods(self) -> u32 {
        self.metadata.n
    }
    /// Returns r, days from settlement to the next quasi date.
    #[must_use]
    pub const fn remaining_days(self) -> u32 {
        self.metadata.r
    }
    /// Returns s, the full current quasi interval in civil days.
    #[must_use]
    pub const fn quasi_period_days(self) -> u32 {
        self.metadata.s
    }
    /// Returns the annual decimal yield coordinate of this observation.
    #[must_use]
    pub const fn yield_rate(self) -> f64 {
        self.yield_rate
    }
    /// Returns the cached observed dirty price in native per100 units.
    #[must_use]
    pub const fn dirty_per_100(self) -> f64 {
        self.dirty
    }
}

impl GiltValuation {
    /// Returns the cached nearest binary64 view of exact source AI per100.
    #[must_use]
    pub const fn accrued_per_100(self) -> f64 {
        self.accrued
    }

    /// Returns the already admitted native dirty quote without repricing.
    #[must_use]
    pub const fn dirty_price(self) -> BondPrice {
        BondPrice {
            value_per_100_face: self.dirty,
            kind: BondPriceKind::Dirty,
        }
    }

    /// Reduces cached dirty minus cached source AI once, without new queries.
    ///
    /// # Errors
    /// GiltError::Sum on an unadmittable reduction; financial overflow is
    /// proof-excluded on the admitted source domain.
    pub fn clean_price(self) -> Result<BondPrice, GiltError> {
        clean_price(self.dirty, self.accrued)
    }
}

impl GiltRisk {
    /// Admits the retained first derivative per annual-decimal yield.
    ///
    /// # Errors
    /// GiltError::Value on signed IEEE overflow/underflow, or InternalCapacity
    /// on a defensive invariant failure.
    pub fn price_derivative(&self) -> Result<f64, GiltError> {
        raw(self.first, true, "price derivative")
    }

    /// Admits the retained second derivative per squared annual-decimal yield.
    ///
    /// # Errors
    /// GiltError::Value on positive IEEE overflow/underflow, or InternalCapacity.
    pub fn price_second_derivative(&self) -> Result<f64, GiltError> {
        raw(self.second, false, "price second derivative")
    }

    /// Admits the retained source-quasi-year weighted price numerator.
    ///
    /// # Errors
    /// GiltError::Value on IEEE range failure, or InternalCapacity. Positive
    /// moment underflow is excluded conditionally on the elementary profile.
    pub fn time_weighted_amount(&self) -> Result<f64, GiltError> {
        raw(self.moment, false, "time weighted amount")
    }

    /// Divides the retained negated first derivative directly by observed dirty.
    ///
    /// # Errors
    /// GiltError::Value on IEEE range failure, ZeroDirtyPrice on zero dirty,
    /// or InternalCapacity. Financial range/zero-dirty failures are excluded
    /// on successful admitted positive source plans.
    pub fn modified_duration(&self) -> Result<f64, GiltError> {
        normalized(self.first, self.dirty, self.yield_rate, "modified duration")
    }

    /// Divides the retained moment directly by observed dirty, in quasi years.
    ///
    /// # Errors
    /// GiltError::Value, ZeroDirtyPrice or InternalCapacity; financial
    /// range/zero-dirty failures are proof-excluded on successful source plans.
    pub fn macaulay_duration(&self) -> Result<f64, GiltError> {
        normalized(
            self.moment,
            self.dirty,
            self.yield_rate,
            "macaulay duration",
        )
    }

    /// Divides the retained second derivative directly by dirty, without a half.
    ///
    /// # Errors
    /// GiltError::Value for positive IEEE underflow, ZeroDirtyPrice or
    /// InternalCapacity. Overflow/zero-dirty are excluded on successful plans.
    pub fn convexity(&self) -> Result<f64, GiltError> {
        normalized(self.second, self.dirty, self.yield_rate, "convexity")
    }
}

impl GiltYieldReport {
    /// Returns the original caller target, retaining its clean/dirty kind.
    #[must_use]
    pub const fn target(self) -> BondPrice {
        self.target
    }
    /// Returns the once-resolved native dirty target per100.
    #[must_use]
    pub const fn dirty_target_per_100(self) -> f64 {
        self.dirty_target
    }
    /// Returns the cached source AI binary observation used for target resolution.
    #[must_use]
    pub const fn accrued_per_100(self) -> f64 {
        self.accrued
    }
    /// Returns the original explicit finite coordinate bracket.
    #[must_use]
    pub const fn initial_bracket(self) -> RootBracket {
        self.initial_bracket
    }
    /// Returns the original explicit tolerances and callback cap.
    #[must_use]
    pub const fn options(self) -> SolverOptions {
        self.options
    }
    /// Returns the counted cached-payload solver result; does not re-evaluate.
    #[must_use]
    pub const fn solve_report(self) -> SolveReport {
        self.solve_report
    }
    /// Returns the target-fixed exponent0/1 in the callback residual units.
    #[must_use]
    pub const fn residual_scale_exponent(self) -> u8 {
        self.residual_scale
    }
    /// Returns the positive-source ideal strict-decrease certificate.
    #[must_use]
    pub const fn sensitivity(self) -> BondYieldSensitivity {
        BondYieldSensitivity::MathematicallyStrictlyDecreasing
    }

    /// Independently rounds cached dirty minus target in unscaled per100 units.
    ///
    /// # Errors
    /// GiltError::Value on signed overflow, or InternalCapacity. Overflow is
    /// excluded here because both cached dirty and resolved target are positive.
    pub fn amount_residual(self) -> Result<f64, GiltError> {
        amount_residual(self.dirty, self.dirty_target)
    }

    /// Returns the selected cached native dirty quote without a callback.
    #[must_use]
    pub const fn dirty_price(self) -> BondPrice {
        BondPrice {
            value_per_100_face: self.dirty,
            kind: BondPriceKind::Dirty,
        }
    }

    /// Reduces cached dirty and AI once without new pricing or date work.
    ///
    /// # Errors
    /// GiltError::Sum on reduction failure, proof-excluded for valid source data.
    pub fn clean_price(self) -> Result<BondPrice, GiltError> {
        clean_price(self.dirty, self.accrued)
    }
}

impl GiltPricingPlan<'_> {
    /// Observes the complete source price at an explicit annual decimal yield.
    ///
    /// # Errors
    /// GiltError::Rate for quote/factor rejection, Value for source PV range,
    /// AllocationFailed for PV workspace, Sum for final dirty overflow, or
    /// InternalCapacity on a defensive invariant failure. Quote precedes reserve.
    pub fn evaluate(&self, yield_rate: f64) -> Result<GiltValuation, GiltError> {
        let y = quote(yield_rate)?;
        let mut terms = reserve(self.bundles.len(), "present value terms")?;
        let dirty = self.dirty_validated_into(y, &mut terms)?;
        Ok(GiltValuation {
            metadata: self.metadata(),
            yield_rate: y,
            dirty,
            accrued: self.accrued_per_100,
        })
    }

    /// Prepares all six source-model yield-risk views with retained wide sums.
    ///
    /// # Errors
    /// GiltError::Rate/Value in source-index order, AllocationFailed before
    /// traversal, Sum after all accepted PVs, or InternalCapacity. IEEE raw
    /// getter range admission is deferred until each getter is requested.
    pub fn risk(&self, yield_rate: f64) -> Result<GiltRisk, GiltError> {
        let y = quote(yield_rate)?;
        let mut terms = reserve(self.bundles.len(), "risk present value terms")?;
        let delta = 2.0 + y;
        let mut sums = [RiskWide::ZERO; 3];
        for bundle in &self.bundles {
            let pv = observe(bundle, y, self.s)?;
            let contributions = risk_terms(pv, delta, bundle.l, self.s, bundle.index)?;
            for ((sum, term), phase) in sums.iter_mut().zip(contributions).zip([
                "price derivative",
                "price second derivative",
                "time weighted amount",
            ]) {
                accumulate(sum, term, bundle.index, phase)?;
            }
            terms.push(pv);
        }
        let dirty = exact_sum(&terms).map_err(|cause| GiltError::Sum {
            phase: "dirty price reduction",
            cause,
        })?;
        let [first, second, moment] = sums;
        Ok(GiltRisk {
            metadata: self.metadata(),
            yield_rate: y,
            dirty: zero(dirty),
            first,
            second,
            moment,
        })
    }

    /// Solves the complete named model using bounded cached-payload Brent calls.
    ///
    /// Target units stay per100. A clean target receives one cached-AI shift;
    /// nonpositive resolved targets fail with zero evaluations. There is no
    /// implicit bracket/tolerance, algebraic solve or final uncounted repricing.
    ///
    /// # Errors
    /// GiltYieldError preserves target/preparation/solver/callback failures and
    /// exact invocation counts. Numerical comparisons remain conditional;
    /// computed callback zero alone is not an ideal-price root certificate.
    pub fn yield_from_price(
        &self,
        target: BondPrice,
        bracket: RootBracket,
        options: SolverOptions,
    ) -> Result<GiltYieldReport, GiltYieldError> {
        let dirty_target = match target.kind() {
            BondPriceKind::Dirty => target.value_per_100_face(),
            BondPriceKind::Clean => exact_sum(&[target.value_per_100_face(), self.accrued_per_100])
                .map_err(|cause| GiltYieldError::Target {
                    target,
                    phase: "target clean shift",
                    cause: GiltError::Sum {
                        phase: "yield target shift",
                        cause,
                    },
                })?,
        };
        let dirty_target = zero(dirty_target);
        let residual_scale = u8::from((dirty_target.to_bits() & !SIGN) >= SCALE_THRESHOLD);
        if dirty_target <= 0.0 {
            return Err(GiltYieldError::NonPositiveTarget {
                target,
                dirty_target_per_100: dirty_target,
                accrued_per_100: self.accrued_per_100,
                evaluations: EvaluationCounts {
                    function: 0,
                    derivative: 0,
                },
            });
        }
        let mut terms =
            reserve(self.bundles.len(), "yield present value terms").map_err(|cause| {
                GiltYieldError::Preparation {
                    phase: "yield present value terms",
                    cause,
                }
            })?;
        let (solve_report, dirty) = brent_with_payload(
            |coordinate| {
                let y = quote(coordinate)?;
                let dirty = self.dirty_validated_into(y, &mut terms)?;
                Ok((residual(dirty, dirty_target, residual_scale)?, dirty))
            },
            bracket,
            options,
        )
        .map_err(|failure| match failure {
            FallibleSolverError::Solver(cause) => GiltYieldError::Solver(cause),
            FallibleSolverError::Callback {
                stage,
                point,
                evaluations,
                cause,
            } => GiltYieldError::Callback {
                stage,
                point,
                evaluations,
                cause,
            },
        })?;
        Ok(GiltYieldReport {
            metadata: self.metadata(),
            yield_rate: solve_report.point(),
            dirty,
            accrued: self.accrued_per_100,
            target,
            dirty_target,
            initial_bracket: bracket,
            options,
            solve_report,
            residual_scale,
        })
    }

    fn dirty_validated_into(&self, y: f64, terms: &mut Vec<f64>) -> Result<f64, GiltError> {
        terms.clear();
        for bundle in &self.bundles {
            terms.push(observe(bundle, y, self.s)?);
        }
        exact_sum(terms).map(zero).map_err(|cause| GiltError::Sum {
            phase: "dirty price reduction",
            cause,
        })
    }
}
