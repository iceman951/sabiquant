//! Prescribed rational Treasury factors, retained risk numerators and bounded inversion.
//!
//! All factor operations use fixed-precision directed integer endpoints. No
//! elementary floating function participates in this observation. Raw getter
//! range admission is independent of each retained normalized numerator.

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
use crate::time::Date;

use super::super::yield_risk::round_wide_ratio_to_f64;
use super::{
    Metadata, PreparedBundle, SOURCE_VERSION, Scalar, SourceRational, TreasuryError,
    TreasuryPricingPlan, capacity,
};

const SIGN: u64 = 1_u64 << 63;
const FRACTION: u64 = (1_u64 << 52) - 1;
const INFINITY: u64 = 0x7ff0_0000_0000_0000;
const SCALE_THRESHOLD: u64 = 1993_u64 << 52;
type RiskWide = Wide<243>;
type ResidualWide = Wide<34>;

/// A Treasury price observation in native per100 units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreasuryValuation {
    metadata: Metadata,
    yield_rate: f64,
    dirty: f64,
}

/// Retained Treasury yield derivatives and independently admitted ratios.
///
/// Raw IEEE range failure does not discard the extended numerator required by
/// a normalized getter. These are derivatives of the ideal source model, not
/// of the discontinuous floating program. The value is immutable and not Copy.
#[derive(Clone, PartialEq)]
pub struct TreasuryRisk {
    metadata: Metadata,
    yield_rate: f64,
    dirty: f64,
    first: RiskWide,
    second: RiskWide,
    moment: RiskWide,
}

impl fmt::Debug for TreasuryRisk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TreasuryRisk")
            .field("metadata", &self.metadata)
            .field("yield_rate", &self.yield_rate)
            .field("dirty_per_100", &self.dirty)
            .finish_non_exhaustive()
    }
}

/// A counted Treasury inverse result with the selected cached price.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreasuryYieldReport {
    metadata: Metadata,
    yield_rate: f64,
    dirty: f64,
    target: BondPrice,
    dirty_target: f64,
    initial_bracket: RootBracket,
    options: SolverOptions,
    solve_report: SolveReport,
    residual_scale: u8,
}

/// A rejected target, preparation, callback or bounded Treasury solve.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum TreasuryYieldError {
    /// Target clean-to-dirty conversion failed; kind `treasury_yield.target`.
    #[non_exhaustive]
    Target {
        /// Original caller target and quote kind.
        target: BondPrice,
        /// Target operation (`target clean shift`).
        phase: &'static str,
        /// Immediate typed source failure.
        cause: TreasuryError,
    },
    /// PV workspace preparation failed; kind `treasury_yield.preparation`.
    #[non_exhaustive]
    Preparation {
        /// Workspace phase (`yield present value terms`).
        phase: &'static str,
        /// Immediate allocation or capacity failure.
        cause: TreasuryError,
    },
    /// Bounded solving failed on admitted finite callbacks; kind `treasury_yield.solver`.
    Solver(SolverError),
    /// An actually invoked callback failed; kind `treasury_yield.callback`.
    #[non_exhaustive]
    Callback {
        /// Solver invocation stage.
        stage: EvaluationStage,
        /// Canonical attempted finite yield coordinate.
        point: f64,
        /// Counts including this failed function invocation.
        evaluations: EvaluationCounts,
        /// Immediate source, factor, PV, reduction or residual failure.
        cause: TreasuryError,
    },
    /// The positive-price model cannot attain the resolved target.
    /// Kind `treasury_yield.nonpositive_target`; no callback was performed.
    #[non_exhaustive]
    NonPositiveTarget {
        /// Original caller target.
        target: BondPrice,
        /// Once-resolved native dirty target per100.
        dirty_target_per_100: f64,
        /// Cached source AI binary observation used by any clean shift.
        accrued_per_100: f64,
        /// Retained publication policy used to resolve the target.
        source_policy: super::TreasurySourcePolicy,
        /// Zero function and derivative evaluations.
        evaluations: EvaluationCounts,
    },
}

impl TreasuryYieldError {
    /// Returns the stable `treasury_yield.*` error kind.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Target { .. } => "treasury_yield.target",
            Self::Preparation { .. } => "treasury_yield.preparation",
            Self::Solver(_) => "treasury_yield.solver",
            Self::Callback { .. } => "treasury_yield.callback",
            Self::NonPositiveTarget { .. } => "treasury_yield.nonpositive_target",
        }
    }
}

impl fmt::Display for TreasuryYieldError {
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
                "treasury callback at {stage:?}, y={point}, after {} function evaluations: {cause}",
                evaluations.function()
            ),
            Self::NonPositiveTarget {
                dirty_target_per_100,
                ..
            } => write!(
                f,
                "nonpositive Treasury dirty target {dirty_target_per_100}"
            ),
        }
    }
}

impl StdError for TreasuryYieldError {
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

fn quote(value: f64) -> Result<f64, TreasuryError> {
    validation::within("rate", value, Interval::greater_than(-2.0))
        .map(zero)
        .map_err(|cause| TreasuryError::Value {
            index: None,
            phase: "yield quote",
            cause,
        })
}

fn reserve(count: usize, phase: &'static str) -> Result<Vec<f64>, TreasuryError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| TreasuryError::AllocationFailed {
            phase,
            requested: count,
        })?;
    Ok(values)
}

fn decode(
    value: f64,
    index: Option<usize>,
    phase: &'static str,
) -> Result<(u64, i32), TreasuryError> {
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

/// A directed endpoint with exactly 256 significant bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Endpoint {
    significand: Wide<4>,
    exponent: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Endpoints {
    low: Endpoint,
    high: Endpoint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Seeds {
    b: Endpoints,
    q: Endpoints,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Extended {
    significand: u64,
    exponent: i32,
}

/// Exact normalized ratio rounding in nine limbs, including the possible
/// 513th quotient-alignment bit.
fn endpoint<const N: usize>(
    numerator: Wide<N>,
    denominator: Wide<N>,
    exponent: i32,
    upward: bool,
    index: Option<usize>,
    phase: &'static str,
    operation: &'static str,
) -> Result<Endpoint, TreasuryError> {
    let fail = || capacity(index, phase, operation);
    if numerator.is_zero() || denominator.is_zero() {
        return Err(fail());
    }
    let nb = i32::try_from(numerator.bits()).map_err(|_| fail())?;
    let db = i32::try_from(denominator.bits()).map_err(|_| fail())?;
    let k0 = nb - db;
    let at_least = if k0 >= 0 {
        numerator
            >= denominator
                .checked_shl(k0.unsigned_abs())
                .ok_or_else(fail)?
    } else {
        numerator.checked_shl(k0.unsigned_abs()).ok_or_else(fail)? >= denominator
    };
    let k = if at_least { k0 } else { k0 - 1 };
    let unit = k - 255;
    let (num, den) = if unit < 0 {
        (
            numerator
                .checked_shl(unit.unsigned_abs())
                .ok_or_else(fail)?,
            denominator,
        )
    } else {
        (
            numerator,
            denominator
                .checked_shl(unit.unsigned_abs())
                .ok_or_else(fail)?,
        )
    };
    let (mut retained, remainder) = num.div_rem(den).ok_or_else(fail)?;
    let mut output_exp = exponent.checked_add(unit).ok_or_else(fail)?;
    if upward && !remainder.is_zero() {
        retained = retained.checked_add(Wide::from_u128(1)).ok_or_else(fail)?;
        if retained.bits() == 257 {
            retained = retained.div_rem_small(2).ok_or_else(fail)?.0;
            output_exp = output_exp.checked_add(1).ok_or_else(fail)?;
        }
    }
    let significand = retained.resize::<4>().ok_or_else(fail)?;
    if significand.bits() != 256 {
        return Err(fail());
    }
    Ok(Endpoint {
        significand,
        exponent: output_exp,
    })
}

fn exact_y(
    value: f64,
    phase: &'static str,
    operation: &'static str,
) -> Result<ResidualWide, TreasuryError> {
    let bits = value.to_bits();
    if !value.is_finite() {
        return Err(capacity(None, phase, operation));
    }
    let field =
        u32::try_from((bits >> 52) & 0x7ff).map_err(|_| capacity(None, phase, operation))?;
    let fraction = bits & FRACTION;
    let (sig, shift) = if field == 0 {
        (fraction, 0)
    } else {
        (fraction | (1_u64 << 52), field - 1)
    };
    ResidualWide::from_u128(u128::from(sig))
        .checked_shl(shift)
        .ok_or_else(|| capacity(None, phase, operation))
}

fn prefix_numerators(
    y: f64,
    r: u32,
    s: u32,
    phase: &'static str,
) -> Result<(ResidualWide, ResidualWide), TreasuryError> {
    let magnitude = exact_y(
        y,
        phase,
        if phase == "discount seeds" {
            "decode yield"
        } else {
            "derive prefix"
        },
    )?;
    let fail = || capacity(None, phase, "derive prefix");
    let two_b = ResidualWide::from_u128(1)
        .checked_shl(1075)
        .ok_or_else(fail)?;
    let weighted = magnitude.checked_mul_small(r).ok_or_else(fail)?;
    let base = two_b.checked_mul_small(s).ok_or_else(fail)?;
    let (nb, nq) = if y.is_sign_negative() {
        (two_b.checked_sub(magnitude), base.checked_sub(weighted))
    } else {
        (two_b.checked_add(magnitude), base.checked_add(weighted))
    };
    let nb = nb.ok_or_else(fail)?;
    let nq = nq.ok_or_else(fail)?;
    if nb.is_zero() || nq.is_zero() || s == 0 {
        return Err(fail());
    }
    Ok((nb, nq))
}

/// Shifts a fixed integer in at most 71 bounded chunks, preserving the
/// exact midpoint bit and all lower discarded bits. The integer is overwritten
/// so argument copies cannot retain a second full seed through the loop.
fn discard_bits<const N: usize>(
    value: &mut Wide<N>,
    mut shift: u32,
    mut sticky: bool,
    phase: &'static str,
    operation: &'static str,
) -> Result<(bool, bool), TreasuryError> {
    let mut half = false;
    while shift != 0 {
        let chunk = shift.min(31);
        let (quotient, remainder) = value
            .div_rem_small(1_u32 << chunk)
            .ok_or_else(|| capacity(None, phase, operation))?;
        *value = quotient;
        if shift == chunk {
            let midpoint_bit = 1_u32 << (chunk - 1);
            half = remainder & midpoint_bit != 0;
            sticky |= remainder & (midpoint_bit - 1) != 0;
        } else {
            sticky |= remainder != 0;
        }
        shift -= chunk;
    }
    Ok((half, sticky))
}

/// Extracts a seed grid without a 34-limb long-division helper chain.
fn seed_rounding(
    mut numerator: ResidualWide,
    denominator: u32,
    precision: u32,
    phase: &'static str,
    operation: &'static str,
) -> Result<(Wide<4>, i32, bool, bool), TreasuryError> {
    let fail = || capacity(None, phase, operation);
    let remainder = {
        let (quotient, remainder) = numerator.div_rem_small(denominator).ok_or_else(fail)?;
        numerator = quotient;
        remainder
    };
    let shift = numerator.bits().checked_sub(precision).ok_or_else(fail)?;
    let unit = i32::try_from(shift).map_err(|_| fail())?;
    let (half, sticky) = discard_bits(&mut numerator, shift, remainder != 0, phase, operation)?;
    Ok((
        numerator.resize::<4>().ok_or_else(fail)?,
        unit - 1074,
        half,
        sticky,
    ))
}

fn seed_endpoints(numerator: ResidualWide, denominator: u32) -> Result<Endpoints, TreasuryError> {
    const PHASE: &str = "discount seeds";
    let (significand, exponent, half, sticky) =
        seed_rounding(numerator, denominator, 256, PHASE, "round seed")?;
    let low = Endpoint {
        significand,
        exponent,
    };
    let high = if half || sticky {
        let fail = || capacity(None, PHASE, "round seed");
        let mut rounded = significand
            .resize::<9>()
            .ok_or_else(fail)?
            .checked_add(Wide::from_u128(1))
            .ok_or_else(fail)?;
        let mut exponent = exponent;
        if rounded.bits() == 257 {
            rounded = rounded.div_rem_small(2).ok_or_else(fail)?.0;
            exponent = exponent.checked_add(1).ok_or_else(fail)?;
        }
        Endpoint {
            significand: rounded.resize::<4>().ok_or_else(fail)?,
            exponent,
        }
    } else {
        low
    };
    Ok(Endpoints { low, high })
}

fn seeds(y: f64, r: u32, s: u32) -> Result<Seeds, TreasuryError> {
    let (nb, nq) = prefix_numerators(y, r, s, "discount seeds")?;
    let denominator = s
        .checked_mul(2)
        .ok_or_else(|| capacity(None, "discount seeds", "derive prefix"))?;
    Ok(Seeds {
        b: seed_endpoints(nb, 2)?,
        q: seed_endpoints(nq, denominator)?,
    })
}

fn seed_observation(numerator: ResidualWide, denominator: u32) -> Result<f64, TreasuryError> {
    const PHASE: &str = "risk prefix observations";
    let fail = || capacity(None, PHASE, "round prefix observation");
    let (retained, mut exponent, half, sticky) = seed_rounding(
        numerator,
        denominator,
        53,
        PHASE,
        "round prefix observation",
    )?;
    let mut sig = retained
        .to_u128()
        .and_then(|value| u64::try_from(value).ok())
        .ok_or_else(fail)?;
    if half && (sticky || sig & 1 != 0) {
        sig += 1;
        if sig == 1_u64 << 53 {
            sig >>= 1;
            exponent = exponent.checked_add(1).ok_or_else(fail)?;
        }
    }
    let bits = round_wide_ratio_to_f64(
        Scalar::from_u128(u128::from(sig)),
        Scalar::from_u128(1),
        exponent,
    )
    .ok_or_else(fail)?;
    Ok(f64::from_bits(bits))
}

fn risk_prefix(y: f64, r: u32, s: u32) -> Result<(f64, f64), TreasuryError> {
    const PHASE: &str = "risk prefix observations";
    let (nb, nq) = prefix_numerators(y, r, s, PHASE)?;
    let denominator = s
        .checked_mul(2)
        .ok_or_else(|| capacity(None, PHASE, "derive prefix"))?;
    Ok((seed_observation(nb, 1)?, seed_observation(nq, denominator)?))
}

fn endpoint_product(
    a: Endpoint,
    b: Endpoint,
    upward: bool,
    index: usize,
) -> Result<Endpoint, TreasuryError> {
    const PHASE: &str = "discount factor";
    let fail = || capacity(Some(index), PHASE, "multiply endpoint");
    let numerator = a
        .significand
        .resize::<9>()
        .ok_or_else(fail)?
        .checked_mul(b.significand.resize::<9>().ok_or_else(fail)?)
        .ok_or_else(fail)?;
    let exponent = a
        .exponent
        .checked_add(b.exponent)
        .ok_or_else(|| capacity(Some(index), PHASE, "power exponent"))?;
    endpoint(
        numerator,
        Wide::<9>::from_u128(1),
        exponent,
        upward,
        Some(index),
        PHASE,
        "round endpoint",
    )
}

fn product(a: Endpoints, b: Endpoints, index: usize) -> Result<Endpoints, TreasuryError> {
    Ok(Endpoints {
        low: endpoint_product(a.low, b.low, false, index)?,
        high: endpoint_product(a.high, b.high, true, index)?,
    })
}

fn reciprocal_endpoint(
    value: Endpoint,
    upward: bool,
    index: usize,
) -> Result<Endpoint, TreasuryError> {
    const PHASE: &str = "discount factor";
    let denominator = value
        .significand
        .resize::<9>()
        .ok_or_else(|| capacity(Some(index), PHASE, "divide endpoint"))?;
    let exponent = value
        .exponent
        .checked_neg()
        .ok_or_else(|| capacity(Some(index), PHASE, "power exponent"))?;
    endpoint(
        Wide::<9>::from_u128(1),
        denominator,
        exponent,
        upward,
        Some(index),
        PHASE,
        "divide endpoint",
    )
}

fn factor_observation(low: Endpoint, index: usize) -> Result<Extended, TreasuryError> {
    let fail = || capacity(Some(index), "discount factor", "round factor observation");
    let divisor = Wide::<4>::from_u128(1).checked_shl(203).ok_or_else(fail)?;
    let (q, remainder) = low.significand.div_rem(divisor).ok_or_else(fail)?;
    let mut sig = q
        .to_u128()
        .and_then(|v| u64::try_from(v).ok())
        .ok_or_else(fail)?;
    let complement = divisor.checked_sub(remainder).ok_or_else(fail)?;
    let mut exponent = low.exponent.checked_add(203).ok_or_else(fail)?;
    if remainder > complement || (remainder == complement && sig & 1 != 0) {
        sig += 1;
        if sig == 1_u64 << 53 {
            sig >>= 1;
            exponent = exponent.checked_add(1).ok_or_else(fail)?;
        }
    }
    Ok(Extended {
        significand: sig,
        exponent,
    })
}

fn discount(seeds: &Seeds, j: u32, index: usize) -> Result<Extended, TreasuryError> {
    let denominator = if j == 0 {
        seeds.q
    } else {
        let leading = 31 - j.leading_zeros();
        let mut power = seeds.b;
        for bit in (0..leading).rev() {
            power = product(power, power, index)?;
            if j & (1_u32 << bit) != 0 {
                power = product(power, seeds.b, index)?;
            }
        }
        product(seeds.q, power, index)?
    };
    // Both endpoints are computed, preserving the prescribed enclosing tree.
    let low = reciprocal_endpoint(denominator.high, false, index)?;
    let _high = reciprocal_endpoint(denominator.low, true, index)?;
    factor_observation(low, index)
}

fn source_pv(amount: SourceRational, factor: Extended, index: usize) -> Result<f64, TreasuryError> {
    const PHASE: &str = "source present value";
    if amount.numerator.is_zero() {
        return Ok(0.0);
    }
    let numerator = amount
        .numerator
        .checked_mul(Scalar::from_u128(u128::from(factor.significand)))
        .ok_or_else(|| capacity(Some(index), PHASE, "derive source"))?;
    let bits = round_wide_ratio_to_f64(numerator, amount.denominator, factor.exponent)
        .ok_or_else(|| capacity(Some(index), PHASE, "round source present value"))?;
    validation::result_within(
        "treasury present value",
        f64::from_bits(bits),
        Interval::greater_than(0.0),
    )
    .map_err(|cause| TreasuryError::Value {
        index: Some(index),
        phase: PHASE,
        cause,
    })
}

fn observe(bundle: &PreparedBundle, seeds: &Seeds) -> Result<f64, TreasuryError> {
    let factor = discount(seeds, bundle.j, bundle.index)?;
    source_pv(bundle.amount, factor, bundle.index)
}

fn scalar_product(
    values: &[u64],
    index: usize,
    phase: &'static str,
) -> Result<Scalar, TreasuryError> {
    let mut result = Scalar::from_u128(1);
    for &value in values {
        result = result
            .checked_mul(Scalar::from_u128(u128::from(value)))
            .ok_or_else(|| capacity(Some(index), phase, "normalize rational"))?;
    }
    Ok(result)
}

fn scalar_shift(
    value: Scalar,
    shift: i32,
    index: usize,
    phase: &'static str,
) -> Result<Scalar, TreasuryError> {
    let shift =
        u32::try_from(shift).map_err(|_| capacity(Some(index), phase, "normalize rational"))?;
    value
        .checked_shl(shift)
        .ok_or_else(|| capacity(Some(index), phase, "normalize rational"))
}

fn risk_terms(
    pv: f64,
    delta: f64,
    q: f64,
    j: u32,
    r: u32,
    s: u32,
    index: usize,
) -> Result<[Extended; 3], TreasuryError> {
    if pv.to_bits() & !SIGN == 0 {
        return Ok([Extended {
            significand: 0,
            exponent: 0,
        }; 3]);
    }
    let j = u64::from(j);
    let r = u64::from(r);
    let s = u64::from(s);
    let first = {
        const PHASE: &str = "price derivative";
        let (p, pe) = decode(pv, Some(index), PHASE)?;
        let (d, de) = decode(delta, Some(index), PHASE)?;
        let (q, qe) = decode(q, Some(index), PHASE)?;
        let m = de.min(qe);
        // End coefficient summands before the nested ratio division.
        let numerator = {
            let a = scalar_shift(
                scalar_product(&[2, s, j, q], index, PHASE)?,
                qe - m,
                index,
                PHASE,
            )?;
            let b = scalar_shift(scalar_product(&[r, d], index, PHASE)?, de - m, index, PHASE)?;
            let coefficient = a
                .checked_add(b)
                .ok_or_else(|| capacity(Some(index), PHASE, "normalize rational"))?;
            coefficient
                .checked_mul(Scalar::from_u128(u128::from(p)))
                .ok_or_else(|| capacity(Some(index), PHASE, "normalize rational"))?
        };
        extended_ratio(
            numerator,
            scalar_product(&[2, s, d, q], index, PHASE)?,
            pe + m - de - qe,
            index,
            PHASE,
        )?
    };
    let second = {
        const PHASE: &str = "price second derivative";
        let (p, pe) = decode(pv, Some(index), PHASE)?;
        let (d, de) = decode(delta, Some(index), PHASE)?;
        let (q, qe) = decode(q, Some(index), PHASE)?;
        let m = 2 * de.min(qe);
        // End coefficient summands before the nested ratio division.
        let numerator = {
            let a = scalar_shift(
                scalar_product(&[2, j, j + 1, s, s, q, q], index, PHASE)?,
                2 * qe - m,
                index,
                PHASE,
            )?;
            let b = scalar_shift(
                scalar_product(&[2, j, r, s, d, q], index, PHASE)?,
                de + qe - m,
                index,
                PHASE,
            )?;
            let c = scalar_shift(
                scalar_product(&[r, r, d, d], index, PHASE)?,
                2 * de - m,
                index,
                PHASE,
            )?;
            let coefficient = a
                .checked_add(b)
                .and_then(|value| value.checked_add(c))
                .ok_or_else(|| capacity(Some(index), PHASE, "normalize rational"))?;
            coefficient
                .checked_mul(Scalar::from_u128(u128::from(p)))
                .ok_or_else(|| capacity(Some(index), PHASE, "normalize rational"))?
        };
        extended_ratio(
            numerator,
            scalar_product(&[2, s, s, d, d, q, q], index, PHASE)?,
            pe + m - 2 * de - 2 * qe,
            index,
            PHASE,
        )?
    };
    let moment = {
        const PHASE: &str = "time weighted amount";
        let (p, pe) = decode(pv, Some(index), PHASE)?;
        let numerator = scalar_product(&[p, r + j * s], index, PHASE)?;
        extended_ratio(
            numerator,
            Scalar::from_u128(2 * u128::from(s)),
            pe,
            index,
            PHASE,
        )?
    };
    Ok([first, second, moment])
}

fn extended_ratio(
    numerator: Scalar,
    denominator: Scalar,
    exponent: i32,
    index: usize,
    phase: &'static str,
) -> Result<Extended, TreasuryError> {
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
) -> Result<(), TreasuryError> {
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

fn finish_risk(bits: u64, negative: bool, phase: &'static str) -> Result<f64, TreasuryError> {
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
    .map_err(|cause| TreasuryError::Value {
        index: None,
        phase,
        cause,
    })
}

fn raw(sum: RiskWide, negative: bool, phase: &'static str) -> Result<f64, TreasuryError> {
    let bits = round_wide_ratio_to_f64(sum, RiskWide::from_u128(1), -8192)
        .ok_or_else(|| capacity(None, phase, "round risk value"))?;
    finish_risk(bits, negative, phase)
}

fn normalized(
    sum: RiskWide,
    dirty: f64,
    y: f64,
    phase: &'static str,
) -> Result<f64, TreasuryError> {
    if dirty.to_bits() & !SIGN == 0 {
        return Err(TreasuryError::ZeroDirtyPrice {
            measure: phase,
            yield_rate: y,
        });
    }
    let (sig, exponent) = decode(dirty, None, phase)?;
    let bits = round_wide_ratio_to_f64(sum, RiskWide::from_u128(u128::from(sig)), -8192 - exponent)
        .ok_or_else(|| capacity(None, phase, "normalize risk value"))?;
    finish_risk(bits, false, phase)
}

fn decode_residual(value: f64, phase: &'static str) -> Result<(ResidualWide, bool), TreasuryError> {
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
) -> Result<(ResidualWide, bool), TreasuryError> {
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

/// Direct IEEE rounding of an integer times a binary power. Chunk extraction
/// keeps the entire residual helper chain below eight 34-word magnitudes.
fn residual_bits(
    mut magnitude: ResidualWide,
    exponent: i32,
    phase: &'static str,
) -> Result<u64, TreasuryError> {
    if magnitude.is_zero() {
        return Ok(0);
    }
    let fail = || capacity(None, phase, "round residual");
    let top = i32::try_from(magnitude.bits()).map_err(|_| fail())? - 1 + exponent;
    if top > 1023 {
        return Ok(INFINITY);
    }
    if top < -1075 {
        return Ok(0);
    }
    let unit = (top - 52).max(-1074);
    let shift = u32::try_from(unit - exponent).map_err(|_| fail())?;
    let (half, sticky) = discard_bits(&mut magnitude, shift, false, phase, "round residual")?;
    let mut retained = magnitude
        .to_u128()
        .and_then(|value| u64::try_from(value).ok())
        .ok_or_else(fail)?;
    if half && (sticky || retained & 1 != 0) {
        retained += 1;
    }
    if top < -1022 {
        return Ok(retained);
    }
    let mut rounded_top = top;
    if retained == 1_u64 << 53 {
        retained >>= 1;
        rounded_top += 1;
    }
    if rounded_top > 1023 {
        return Ok(INFINITY);
    }
    let biased = u64::try_from(rounded_top + 1023).map_err(|_| fail())?;
    Ok((biased << 52) | (retained & FRACTION))
}

fn residual(dirty: f64, target: f64, scale: u8) -> Result<f64, TreasuryError> {
    const PHASE: &str = "yield residual";
    let (magnitude, negative) = difference(dirty, target, PHASE)?;
    if magnitude.is_zero() {
        return Ok(0.0);
    }
    let bits = residual_bits(magnitude, -1074 - i32::from(scale), PHASE)?;
    if bits == 0 || bits == INFINITY {
        return Err(capacity(None, PHASE, "round residual"));
    }
    Ok(f64::from_bits(bits | if negative { SIGN } else { 0 }))
}

fn amount_residual(dirty: f64, target: f64) -> Result<f64, TreasuryError> {
    const PHASE: &str = "yield amount residual";
    let (magnitude, negative) = difference(dirty, target, PHASE)?;
    if magnitude.is_zero() {
        return Ok(0.0);
    }
    let bits = residual_bits(magnitude, -1074, PHASE)?;
    let value = f64::from_bits(bits | if negative { SIGN } else { 0 });
    validation::finite_result(PHASE, value).map_err(|cause| TreasuryError::Value {
        index: None,
        phase: PHASE,
        cause,
    })
}

fn clean_price(dirty: f64, accrued: f64) -> Result<BondPrice, TreasuryError> {
    let value = exact_sum(&[dirty, -accrued]).map_err(|cause| TreasuryError::Sum {
        phase: "clean price reduction",
        cause,
    })?;
    Ok(BondPrice {
        value_per_100_face: zero(value),
        kind: BondPriceKind::Clean,
    })
}

impl TreasuryValuation {
    /// Returns the immutable adopted source/model identifier.
    #[must_use]
    pub const fn source_version(self) -> &'static str {
        SOURCE_VERSION
    }
    /// Retained source publication policy.
    #[must_use]
    pub const fn source_policy(self) -> super::TreasurySourcePolicy {
        self.metadata.source_policy
    }
    /// Original dated date.
    #[must_use]
    pub const fn original_dated_date(self) -> Date {
        self.metadata.original_dated_date
    }
    /// Original issue date.
    #[must_use]
    pub const fn original_issue_date(self) -> Date {
        self.metadata.original_issue_date
    }
    /// Unadjusted maturity date.
    #[must_use]
    pub const fn maturity_date(self) -> Date {
        self.metadata.maturity_date
    }
    /// Original exact annual coupon per100.
    #[must_use]
    pub const fn coupon_per_100(self) -> crate::amounts::DecimalAmount {
        self.metadata.coupon_per_100
    }
    /// First actual coupon date.
    #[must_use]
    pub const fn first_coupon_date(self) -> Date {
        self.metadata.first_coupon_date
    }
    /// Inferred first-period source role.
    #[must_use]
    pub const fn first_period(self) -> super::TreasuryFirstPeriod {
        self.metadata.first_period
    }
    /// Explicit calendar end-of-month policy.
    #[must_use]
    pub const fn end_of_month(self) -> crate::calendar::EndOfMonth {
        self.metadata.end_of_month
    }
    /// Prepared settlement date.
    #[must_use]
    pub const fn settlement_date(self) -> Date {
        self.metadata.settlement_date
    }
    /// Strictly next unadjusted quasi date.
    #[must_use]
    pub const fn next_quasi_date(self) -> Date {
        self.metadata.next_quasi_date
    }
    /// Number of full quasi periods after the next date.
    #[must_use]
    pub const fn remaining_quasi_periods(self) -> u32 {
        self.metadata.n
    }
    /// Civil days from settlement to the next quasi date.
    #[must_use]
    pub const fn remaining_days(self) -> u32 {
        self.metadata.r
    }
    /// Civil days in the current full quasi interval.
    #[must_use]
    pub const fn quasi_period_days(self) -> u32 {
        self.metadata.s
    }
    /// Cached correctly rounded raw accrued interest per100.
    #[must_use]
    pub const fn raw_accrued_per_100(self) -> f64 {
        self.metadata.raw_accrued_per_100
    }
    /// Cached correctly rounded published accrued interest per100.
    #[must_use]
    pub const fn accrued_per_100(self) -> f64 {
        self.metadata.accrued_per_100
    }
    /// Canonical annual decimal yield coordinate.
    #[must_use]
    pub const fn yield_rate(self) -> f64 {
        self.yield_rate
    }
    /// Cached observed native dirty price per100.
    #[must_use]
    pub const fn dirty_per_100(self) -> f64 {
        self.dirty
    }
}

impl TreasuryRisk {
    /// Returns the immutable adopted source/model identifier.
    #[must_use]
    pub const fn source_version(&self) -> &'static str {
        SOURCE_VERSION
    }
    /// Retained source publication policy.
    #[must_use]
    pub const fn source_policy(&self) -> super::TreasurySourcePolicy {
        self.metadata.source_policy
    }
    /// Original dated date.
    #[must_use]
    pub const fn original_dated_date(&self) -> Date {
        self.metadata.original_dated_date
    }
    /// Original issue date.
    #[must_use]
    pub const fn original_issue_date(&self) -> Date {
        self.metadata.original_issue_date
    }
    /// Unadjusted maturity date.
    #[must_use]
    pub const fn maturity_date(&self) -> Date {
        self.metadata.maturity_date
    }
    /// Original exact annual coupon per100.
    #[must_use]
    pub const fn coupon_per_100(&self) -> crate::amounts::DecimalAmount {
        self.metadata.coupon_per_100
    }
    /// First actual coupon date.
    #[must_use]
    pub const fn first_coupon_date(&self) -> Date {
        self.metadata.first_coupon_date
    }
    /// Inferred first-period source role.
    #[must_use]
    pub const fn first_period(&self) -> super::TreasuryFirstPeriod {
        self.metadata.first_period
    }
    /// Explicit calendar end-of-month policy.
    #[must_use]
    pub const fn end_of_month(&self) -> crate::calendar::EndOfMonth {
        self.metadata.end_of_month
    }
    /// Prepared settlement date.
    #[must_use]
    pub const fn settlement_date(&self) -> Date {
        self.metadata.settlement_date
    }
    /// Strictly next unadjusted quasi date.
    #[must_use]
    pub const fn next_quasi_date(&self) -> Date {
        self.metadata.next_quasi_date
    }
    /// Number of full quasi periods after the next date.
    #[must_use]
    pub const fn remaining_quasi_periods(&self) -> u32 {
        self.metadata.n
    }
    /// Civil days from settlement to the next quasi date.
    #[must_use]
    pub const fn remaining_days(&self) -> u32 {
        self.metadata.r
    }
    /// Civil days in the current full quasi interval.
    #[must_use]
    pub const fn quasi_period_days(&self) -> u32 {
        self.metadata.s
    }
    /// Cached correctly rounded raw accrued interest per100.
    #[must_use]
    pub const fn raw_accrued_per_100(&self) -> f64 {
        self.metadata.raw_accrued_per_100
    }
    /// Cached correctly rounded published accrued interest per100.
    #[must_use]
    pub const fn accrued_per_100(&self) -> f64 {
        self.metadata.accrued_per_100
    }
    /// Canonical annual decimal yield coordinate.
    #[must_use]
    pub const fn yield_rate(&self) -> f64 {
        self.yield_rate
    }
    /// Cached observed native dirty price per100.
    #[must_use]
    pub const fn dirty_per_100(&self) -> f64 {
        self.dirty
    }
}

impl TreasuryYieldReport {
    /// Returns the immutable adopted source/model identifier.
    #[must_use]
    pub const fn source_version(self) -> &'static str {
        SOURCE_VERSION
    }
    /// Retained source publication policy.
    #[must_use]
    pub const fn source_policy(self) -> super::TreasurySourcePolicy {
        self.metadata.source_policy
    }
    /// Original dated date.
    #[must_use]
    pub const fn original_dated_date(self) -> Date {
        self.metadata.original_dated_date
    }
    /// Original issue date.
    #[must_use]
    pub const fn original_issue_date(self) -> Date {
        self.metadata.original_issue_date
    }
    /// Unadjusted maturity date.
    #[must_use]
    pub const fn maturity_date(self) -> Date {
        self.metadata.maturity_date
    }
    /// Original exact annual coupon per100.
    #[must_use]
    pub const fn coupon_per_100(self) -> crate::amounts::DecimalAmount {
        self.metadata.coupon_per_100
    }
    /// First actual coupon date.
    #[must_use]
    pub const fn first_coupon_date(self) -> Date {
        self.metadata.first_coupon_date
    }
    /// Inferred first-period source role.
    #[must_use]
    pub const fn first_period(self) -> super::TreasuryFirstPeriod {
        self.metadata.first_period
    }
    /// Explicit calendar end-of-month policy.
    #[must_use]
    pub const fn end_of_month(self) -> crate::calendar::EndOfMonth {
        self.metadata.end_of_month
    }
    /// Prepared settlement date.
    #[must_use]
    pub const fn settlement_date(self) -> Date {
        self.metadata.settlement_date
    }
    /// Strictly next unadjusted quasi date.
    #[must_use]
    pub const fn next_quasi_date(self) -> Date {
        self.metadata.next_quasi_date
    }
    /// Number of full quasi periods after the next date.
    #[must_use]
    pub const fn remaining_quasi_periods(self) -> u32 {
        self.metadata.n
    }
    /// Civil days from settlement to the next quasi date.
    #[must_use]
    pub const fn remaining_days(self) -> u32 {
        self.metadata.r
    }
    /// Civil days in the current full quasi interval.
    #[must_use]
    pub const fn quasi_period_days(self) -> u32 {
        self.metadata.s
    }
    /// Cached correctly rounded raw accrued interest per100.
    #[must_use]
    pub const fn raw_accrued_per_100(self) -> f64 {
        self.metadata.raw_accrued_per_100
    }
    /// Cached correctly rounded published accrued interest per100.
    #[must_use]
    pub const fn accrued_per_100(self) -> f64 {
        self.metadata.accrued_per_100
    }
    /// Canonical annual decimal yield coordinate.
    #[must_use]
    pub const fn yield_rate(self) -> f64 {
        self.yield_rate
    }
    /// Cached observed native dirty price per100.
    #[must_use]
    pub const fn dirty_per_100(self) -> f64 {
        self.dirty
    }
}

impl TreasuryValuation {
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
    /// TreasuryError::Sum on an unadmittable reduction; financial overflow is
    /// proof-excluded on the admitted source domain.
    pub fn clean_price(self) -> Result<BondPrice, TreasuryError> {
        clean_price(self.dirty, self.metadata.accrued_per_100)
    }
}

impl TreasuryRisk {
    /// Admits the retained first derivative per annual-decimal yield.
    ///
    /// # Errors
    /// TreasuryError::Value on signed IEEE overflow/underflow, or InternalCapacity
    /// on a defensive invariant failure.
    pub fn price_derivative(&self) -> Result<f64, TreasuryError> {
        raw(self.first, true, "price derivative")
    }

    /// Admits the retained second derivative per squared annual-decimal yield.
    ///
    /// # Errors
    /// TreasuryError::Value on positive IEEE overflow/underflow, or InternalCapacity.
    pub fn price_second_derivative(&self) -> Result<f64, TreasuryError> {
        raw(self.second, false, "price second derivative")
    }

    /// Admits the retained source-quasi-year weighted price numerator.
    ///
    /// # Errors
    /// TreasuryError::Value on IEEE range failure, or InternalCapacity. Positive
    /// moment underflow is excluded on admitted positive Treasury plans.
    pub fn time_weighted_amount(&self) -> Result<f64, TreasuryError> {
        raw(self.moment, false, "time weighted amount")
    }

    /// Divides the retained negated first derivative directly by observed dirty.
    ///
    /// # Errors
    /// TreasuryError::Value on IEEE range failure, ZeroDirtyPrice on zero dirty,
    /// or InternalCapacity. Financial range/zero-dirty failures are excluded
    /// on successful admitted positive source plans.
    pub fn modified_duration(&self) -> Result<f64, TreasuryError> {
        normalized(self.first, self.dirty, self.yield_rate, "modified duration")
    }

    /// Divides the retained moment directly by observed dirty, in quasi years.
    ///
    /// # Errors
    /// TreasuryError::Value, ZeroDirtyPrice or InternalCapacity; financial
    /// range/zero-dirty failures are proof-excluded on successful source plans.
    pub fn macaulay_duration(&self) -> Result<f64, TreasuryError> {
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
    /// TreasuryError::Value for positive IEEE underflow, ZeroDirtyPrice or
    /// InternalCapacity. Overflow/zero-dirty are excluded on successful plans.
    pub fn convexity(&self) -> Result<f64, TreasuryError> {
        normalized(self.second, self.dirty, self.yield_rate, "convexity")
    }
}

impl TreasuryYieldReport {
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
    /// TreasuryError::Value on signed overflow, or InternalCapacity. Overflow is
    /// excluded here because both cached dirty and resolved target are positive.
    pub fn amount_residual(self) -> Result<f64, TreasuryError> {
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
    /// TreasuryError::Sum on reduction failure, proof-excluded for valid source data.
    pub fn clean_price(self) -> Result<BondPrice, TreasuryError> {
        clean_price(self.dirty, self.metadata.accrued_per_100)
    }
}

impl TreasuryPricingPlan<'_> {
    /// Observes the complete source price at an explicit annual decimal yield.
    ///
    /// # Errors
    /// TreasuryError::Value for quote rejection, Value for source PV range,
    /// AllocationFailed for PV workspace, Sum for final dirty overflow, or
    /// InternalCapacity on a defensive invariant failure. Quote precedes reserve.
    pub fn evaluate(&self, yield_rate: f64) -> Result<TreasuryValuation, TreasuryError> {
        let y = quote(yield_rate)?;
        let mut terms = reserve(self.bundles.len(), "present value terms")?;
        let dirty = self.dirty_validated_into(y, &mut terms)?;
        Ok(TreasuryValuation {
            metadata: self.metadata(),
            yield_rate: y,
            dirty,
        })
    }

    /// Prepares all six source-model yield-risk views with retained wide sums.
    ///
    /// # Errors
    /// TreasuryError::Value in source-index order, AllocationFailed before
    /// traversal, Sum after all accepted PVs, or InternalCapacity. IEEE raw
    /// getter range admission is deferred until each getter is requested.
    pub fn risk(&self, yield_rate: f64) -> Result<TreasuryRisk, TreasuryError> {
        let y = quote(yield_rate)?;
        let mut terms = reserve(self.bundles.len(), "risk present value terms")?;
        let seeds = seeds(y, self.r, self.s)?;
        let (delta, q) = risk_prefix(y, self.r, self.s)?;
        let mut sums = [RiskWide::ZERO; 3];
        for bundle in &self.bundles {
            let pv = observe(bundle, &seeds)?;
            let contributions = risk_terms(pv, delta, q, bundle.j, self.r, self.s, bundle.index)?;
            for ((sum, term), phase) in sums.iter_mut().zip(contributions).zip([
                "price derivative",
                "price second derivative",
                "time weighted amount",
            ]) {
                accumulate(sum, term, bundle.index, phase)?;
            }
            terms.push(pv);
        }
        let dirty = exact_sum(&terms).map_err(|cause| TreasuryError::Sum {
            phase: "dirty price reduction",
            cause,
        })?;
        let [first, second, moment] = sums;
        Ok(TreasuryRisk {
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
    /// TreasuryYieldError preserves target/preparation/solver/callback failures and
    /// exact invocation counts. Ideal-model comparisons remain conditional;
    /// computed callback zero alone is not an ideal-price root certificate.
    pub fn yield_from_price(
        &self,
        target: BondPrice,
        bracket: RootBracket,
        options: SolverOptions,
    ) -> Result<TreasuryYieldReport, TreasuryYieldError> {
        let dirty_target = match target.kind() {
            BondPriceKind::Dirty => target.value_per_100_face(),
            BondPriceKind::Clean => exact_sum(&[target.value_per_100_face(), self.accrued_per_100])
                .map_err(|cause| TreasuryYieldError::Target {
                    target,
                    phase: "target clean shift",
                    cause: TreasuryError::Sum {
                        phase: "yield target shift",
                        cause,
                    },
                })?,
        };
        let dirty_target = zero(dirty_target);
        let residual_scale = u8::from((dirty_target.to_bits() & !SIGN) >= SCALE_THRESHOLD);
        if dirty_target <= 0.0 {
            return Err(TreasuryYieldError::NonPositiveTarget {
                target,
                dirty_target_per_100: dirty_target,
                accrued_per_100: self.accrued_per_100,
                source_policy: self.metadata().source_policy,
                evaluations: EvaluationCounts {
                    function: 0,
                    derivative: 0,
                },
            });
        }
        let mut terms =
            reserve(self.bundles.len(), "yield present value terms").map_err(|cause| {
                TreasuryYieldError::Preparation {
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
            FallibleSolverError::Solver(cause) => TreasuryYieldError::Solver(cause),
            FallibleSolverError::Callback {
                stage,
                point,
                evaluations,
                cause,
            } => TreasuryYieldError::Callback {
                stage,
                point,
                evaluations,
                cause,
            },
        })?;
        Ok(TreasuryYieldReport {
            metadata: self.metadata(),
            yield_rate: solve_report.point(),
            dirty,
            target,
            dirty_target,
            initial_bracket: bracket,
            options,
            solve_report,
            residual_scale,
        })
    }

    fn dirty_validated_into(&self, y: f64, terms: &mut Vec<f64>) -> Result<f64, TreasuryError> {
        terms.clear();
        let seeds = seeds(y, self.r, self.s)?;
        for bundle in &self.bundles {
            terms.push(observe(bundle, &seeds)?);
        }
        exact_sum(terms)
            .map(zero)
            .map_err(|cause| TreasuryError::Sum {
                phase: "dirty price reduction",
                cause,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-UST-09/10: quote admission occurs before any factor observation.
    #[test]
    fn quote_domain_and_zero_bits() {
        for y in [-0.0, 0.0] {
            assert_eq!(quote(y).expect("zero admitted").to_bits(), 0);
        }
        for y in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -2.0, -3.0] {
            assert!(matches!(
                quote(y),
                Err(TreasuryError::Value {
                    index: None,
                    phase: "yield quote",
                    ..
                })
            ));
        }
        for y in [(-2.0_f64).next_down(), (-2.0_f64).next_up(), f64::MAX] {
            assert_eq!(quote(y).is_ok(), y > -2.0);
        }
    }

    /// FR-UST-09: independent exact powers, including the j=0 q-only branch.
    #[test]
    fn rational_power_and_prefix_controls() {
        for (y, j, expected) in [
            (0.0, 19_999, 1.0),
            (2.0, 0, 0.5),
            (2.0, 1, 0.25),
            (-1.0, 1, 4.0),
        ] {
            let seeded = seeds(y, 182, 182).expect("valid exact seeds");
            let observed = discount(&seeded, j, 7).expect("bounded factor");
            let bits = round_wide_ratio_to_f64(
                Scalar::from_u128(u128::from(observed.significand)),
                Scalar::from_u128(1),
                observed.exponent,
            )
            .expect("finite exact factor");
            assert_eq!(f64::from_bits(bits), expected);
        }
        let (delta, q) = risk_prefix(0.0, 181, 184).expect("zero-yield prefixes");
        assert_eq!((delta, q), (2.0, 1.0));
        let y = (-2.0_f64).next_up();
        let (delta, q) = risk_prefix(y, 184, 184).expect("lower-domain prefixes");
        assert_eq!(delta, f64::from_bits(971_u64 << 52));
        assert_eq!(q, f64::from_bits(970_u64 << 52));
    }

    /// FR-UST-09/11: a zero amount still has an observed bounded factor;
    /// a positive amount admits direct IEEE underflow as a typed PV failure.
    #[test]
    fn extended_factor_precedes_source_ieee_admission() {
        let factor = discount(
            &seeds(f64::MAX, 182, 182).expect("maximum yield seed"),
            19_999,
            91,
        )
        .expect("unbounded factor exists");
        let zero_amount = SourceRational {
            numerator: Scalar::ZERO,
            denominator: Scalar::from_u128(1),
        };
        assert_eq!(
            source_pv(zero_amount, factor, 91).expect("zero source"),
            0.0
        );
        let positive_amount = SourceRational {
            numerator: Scalar::from_u128(100),
            denominator: Scalar::from_u128(1),
        };
        assert!(matches!(
            source_pv(positive_amount, factor, 91),
            Err(TreasuryError::Value {
                index: Some(91),
                phase: "source present value",
                ..
            })
        ));
    }

    /// FR-UST-11: all summands of kappa are combined before the sole rounding.
    #[test]
    fn complete_risk_coefficients_at_zero_yield() {
        for (j, expected) in [(0, [50.0, 50.0, 50.0]), (1, [100.0, 150.0, 100.0])] {
            let terms =
                risk_terms(100.0, 2.0, 1.0, j, 182, 182, 3).expect("coupled valid primitive");
            for (term, expected) in terms.into_iter().zip(expected) {
                let bits = round_wide_ratio_to_f64(
                    Scalar::from_u128(u128::from(term.significand)),
                    Scalar::from_u128(1),
                    term.exponent,
                )
                .expect("finite exact term");
                assert_eq!(f64::from_bits(bits), expected);
            }
        }
    }

    /// FR-UST-12: exact binary differences retain sign, half-minimum ties and
    /// the target-fixed scaled units without separately halving operands.
    #[test]
    fn residual_rounding_boundaries() {
        let minimum = f64::from_bits(1);
        assert_eq!(residual(1.0, 1.0, 0).expect("exact zero").to_bits(), 0);
        assert_eq!(
            residual(minimum * 2.0, minimum, 0).expect("positive minimum"),
            minimum
        );
        assert_eq!(
            residual(minimum, minimum * 2.0, 0).expect("negative minimum"),
            -minimum
        );
        let half_min = ResidualWide::from_u128(1);
        assert_eq!(
            residual_bits(half_min, -1075, "yield residual").expect("even zero tie"),
            0
        );
        assert_eq!(
            residual_bits(ResidualWide::from_u128(3), -1075, "yield residual")
                .expect("even two tie"),
            2
        );
        let target = f64::from_bits(SCALE_THRESHOLD);
        let above = target.next_up();
        assert_eq!(
            residual(above, target, 1).expect("scaled exact difference"),
            (above - target) / 2.0
        );
        assert_eq!(
            amount_residual(above, target).expect("unscaled exact difference"),
            above - target
        );
    }
}
