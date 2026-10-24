//! Prescribed Canadian observations, retained selected risk and cached inversion.
//!
//! Native elementary accuracy is conditional on the adopted primitive profile.
//! Exact represented-input rounding is not an ideal-price enclosure.

use std::cmp::Ordering;
use std::error::Error as StdError;
use std::fmt;

use crate::bonds::BondYieldSensitivity;
use crate::error::Interval;
use crate::math::{
    EvaluationCounts, EvaluationStage, FallibleSolverError, RootBracket, SolveReport, SolverError,
    SolverOptions, brent_with_payload, exact_sum,
};
use crate::numeric as validation;
use crate::rates::RateError;

use super::wide;
use super::{
    Bundle, CanadianError, CanadianPrice, CanadianPriceBasis, CanadianPricingPlan,
    CanadianYieldConvention, Metadata, SOURCE_VERSION, capacity,
};

const SIGN: u64 = 1_u64 << 63;
const FRACTION: u64 = (1_u64 << 52) - 1;
const TINY: f64 = f64::from_bits(973_u64 << 52); // 2^-50
const FAR_EXPONENT: f64 = 16_777_216.0; // 2^24
const SCALE_THRESHOLD: u64 = 1993_u64 << 52; // 2^970
const TERM_WORDS: usize = 112;
const RETAINED_WORDS: usize = 243;
type Magnitude = [u64; TERM_WORDS];
type Retained = [u64; RETAINED_WORDS];

/// Three immutable native per100 price views of one Canadian observation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanadianValuation {
    metadata: Metadata,
    yield_rate: f64,
    dirty: f64,
}

/// Selected PV-seeded risk, with independently admitted raw and normalized views.
///
/// Three exact retained sums remain available when a different getter cannot
/// represent its result. Debug omits their fixed large buffers.
#[derive(Clone, PartialEq)]
pub struct CanadianRisk {
    metadata: Metadata,
    yield_rate: f64,
    dirty: f64,
    sums: [Retained; 3],
}

impl fmt::Debug for CanadianRisk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CanadianRisk")
            .field("metadata", &self.metadata)
            .field("yield_rate", &self.yield_rate)
            .field("model_dirty_per_100", &self.dirty)
            .finish_non_exhaustive()
    }
}

/// Counted Canadian inverse result containing its selected cached dirty payload.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanadianYieldReport {
    metadata: Metadata,
    yield_rate: f64,
    dirty: f64,
    target: CanadianPrice,
    dirty_target: f64,
    initial_bracket: RootBracket,
    options: SolverOptions,
    solve_report: SolveReport,
    residual_scale: u8,
}

/// A typed Canadian target, preparation, solver or invoked-callback failure.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CanadianYieldError {
    /// Typed target resolution failed; kind `canadian_yield.target`.
    #[non_exhaustive]
    Target {
        /// Original supplied scalar and price basis.
        target: CanadianPrice,
        /// Exact target operation phase.
        phase: &'static str,
        /// Immediate typed model error.
        cause: CanadianError,
    },
    /// PV workspace reservation failed; kind `canadian_yield.preparation`.
    #[non_exhaustive]
    Preparation {
        /// Workspace preparation phase.
        phase: &'static str,
        /// Immediate allocation/capacity failure.
        cause: CanadianError,
    },
    /// Bounded shared Brent failure; kind `canadian_yield.solver`.
    Solver(SolverError),
    /// An invoked financial callback failed; kind `canadian_yield.callback`.
    #[non_exhaustive]
    Callback {
        /// Solver invocation stage.
        stage: EvaluationStage,
        /// Canonical finite attempted quote.
        point: f64,
        /// Actual counts, including this failed invocation.
        evaluations: EvaluationCounts,
        /// Immediate financial/numerical cause.
        cause: CanadianError,
    },
    /// The resolved model-dirty target is nonpositive, with zero callbacks.
    /// Kind `canadian_yield.nonpositive_target`.
    #[non_exhaustive]
    NonPositiveTarget {
        /// Original supplied scalar and price basis.
        target: CanadianPrice,
        /// Once-resolved represented model-dirty target.
        model_dirty_target_per_100: f64,
        /// Cached quote-accrued observation used by the shift.
        quote_accrued_per_100: f64,
        /// Cached settlement-accrued observation used by the shift.
        settlement_accrued_per_100: f64,
        /// Explicit source policy.
        source_policy: super::CanadianSourcePolicy,
        /// Resolved numerical convention.
        convention: CanadianYieldConvention,
        /// Zero function and derivative invocations.
        evaluations: EvaluationCounts,
    },
}

impl CanadianYieldError {
    /// Returns the stable `canadian_yield.*` identifier.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Target { .. } => "canadian_yield.target",
            Self::Preparation { .. } => "canadian_yield.preparation",
            Self::Solver(_) => "canadian_yield.solver",
            Self::Callback { .. } => "canadian_yield.callback",
            Self::NonPositiveTarget { .. } => "canadian_yield.nonpositive_target",
        }
    }
}

impl fmt::Display for CanadianYieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Target {
                target,
                phase,
                cause,
                ..
            } => {
                write!(f, "{phase} for {target:?}: {cause}")
            }
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
                "Canadian callback at {stage:?}, y={point}, after {} function evaluations: {cause}",
                evaluations.function()
            ),
            Self::NonPositiveTarget {
                model_dirty_target_per_100,
                ..
            } => {
                write!(
                    f,
                    "nonpositive Canadian model-dirty target {model_dirty_target_per_100}"
                )
            }
        }
    }
}

impl StdError for CanadianYieldError {
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

/// Decodes only represented finite magnitudes; sign remains caller metadata.
fn decode(value: f64) -> (u64, i32) {
    let bits = value.to_bits();
    let field = ((bits >> 52) & 0x7ff) as u16;
    let fraction = bits & FRACTION;
    if field == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1_u64 << 52), i32::from(field) - 1075)
    }
}

fn checked(
    ok: bool,
    index: Option<usize>,
    phase: &'static str,
    operation: &'static str,
) -> Result<(), CanadianError> {
    if ok {
        Ok(())
    } else {
        Err(capacity(index, phase, operation))
    }
}

fn reserve(count: usize, phase: &'static str) -> Result<Vec<f64>, CanadianError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| CanadianError::AllocationFailed {
            phase,
            requested: count,
        })?;
    Ok(values)
}

// Eleven borrowed-output112-word magnitudes. No operation accepts/returns one
// by value. exact_sum adds two34-word arrays; the residual helper adds five34-
// word arrays. Even concurrent frames stay below the24-slot traversal budget.
struct Workspace {
    y: Magnitude,
    d: Magnitude,
    n: Magnitude,
    base: Magnitude,
    numerator: Magnitude,
    denominator: Magnitude,
    product: Magnitude,
    temporary: Magnitude,
    u: Magnitude,
    v: Magnitude,
    aligned: Magnitude,
}

impl Workspace {
    fn new() -> Self {
        Self {
            y: [0; TERM_WORDS],
            d: [0; TERM_WORDS],
            n: [0; TERM_WORDS],
            base: [0; TERM_WORDS],
            numerator: [0; TERM_WORDS],
            denominator: [0; TERM_WORDS],
            product: [0; TERM_WORDS],
            temporary: [0; TERM_WORDS],
            u: [0; TERM_WORDS],
            v: [0; TERM_WORDS],
            aligned: [0; TERM_WORDS],
        }
    }

    fn quote(&mut self, metadata: Metadata, value: f64) -> Result<f64, CanadianError> {
        const PHASE: &str = "yield quotation";
        let y = validation::finite("rate", value).map_err(|cause| CanadianError::Value {
            index: None,
            phase: PHASE,
            cause,
        })?;
        let y = zero(y);
        if metadata.yield_convention == CanadianYieldConvention::Compound {
            let f = f64::from(metadata.frequency.payments_per_year());
            validation::within("rate", y, Interval::greater_than(-f)).map_err(|cause| {
                CanadianError::Value {
                    index: None,
                    phase: PHASE,
                    cause,
                }
            })?;
        } else {
            let (sig, exponent) = decode(y);
            let shift = u32::try_from(exponent + 1074)
                .map_err(|_| capacity(None, PHASE, "construct affine"))?;
            checked(
                wide::set_u128(&mut self.temporary, u128::from(sig)),
                None,
                PHASE,
                "construct affine",
            )?;
            checked(
                wide::shl_into(&self.temporary, shift, &mut self.y),
                None,
                PHASE,
                "construct affine",
            )?;
            checked(
                wide::set_u128(&mut self.base, 365),
                None,
                PHASE,
                "construct affine",
            )?;
            checked(
                wide::shl_assign(&mut self.base, 1074),
                None,
                PHASE,
                "construct affine",
            )?;
            checked(
                wide::copy_into(&self.y, &mut self.d),
                None,
                PHASE,
                "construct affine",
            )?;
            checked(
                wide::mul_small_assign(&mut self.d, u64::from(metadata.days_to_true_maturity)),
                None,
                PHASE,
                "construct affine",
            )?;
            if y.is_sign_negative() {
                if wide::cmp(&self.base, &self.d) != Ordering::Greater {
                    return Err(CanadianError::MoneyMarketDomain {
                        yield_rate: y,
                        days_to_true_maturity: metadata.days_to_true_maturity,
                    });
                }
                checked(
                    wide::copy_into(&self.base, &mut self.temporary),
                    None,
                    PHASE,
                    "construct affine",
                )?;
                checked(
                    wide::sub_assign(&mut self.temporary, &self.d),
                    None,
                    PHASE,
                    "construct affine",
                )?;
                checked(
                    wide::copy_into(&self.temporary, &mut self.d),
                    None,
                    PHASE,
                    "construct affine",
                )?;
            } else {
                checked(
                    wide::add_assign(&mut self.d, &self.base),
                    None,
                    PHASE,
                    "construct affine",
                )?;
            }
        }
        Ok(y)
    }

    fn mm_numerator(&mut self, y: f64, bundle: &Bundle) -> Result<(), CanadianError> {
        let index = Some(bundle.index);
        const PHASE: &str = "discount factor";
        checked(
            wide::copy_into(&self.y, &mut self.n),
            index,
            PHASE,
            "construct affine",
        )?;
        checked(
            wide::mul_small_assign(&mut self.n, u64::from(bundle.days_payment_to_true)),
            index,
            PHASE,
            "construct affine",
        )?;
        if y.is_sign_negative() {
            checked(
                wide::copy_into(&self.base, &mut self.temporary),
                index,
                PHASE,
                "construct affine",
            )?;
            checked(
                wide::sub_assign(&mut self.temporary, &self.n),
                index,
                PHASE,
                "construct affine",
            )?;
            checked(
                wide::copy_into(&self.temporary, &mut self.n),
                index,
                PHASE,
                "construct affine",
            )?;
        } else {
            checked(
                wide::add_assign(&mut self.n, &self.base),
                index,
                PHASE,
                "construct affine",
            )?;
        }
        Ok(())
    }

    /// One exact represented scaled-log/coordinate product, with its IEEE sign.
    /// A negative nonzero exact exponent rounding to zero retains -0 at this
    /// primitive stage; the later identity branch accepts either zero sign.
    fn logarithmic_exponent(
        &mut self,
        g: f64,
        frequency: u32,
        lambda: super::Coordinate,
        ordinal: usize,
    ) -> Result<f64, CanadianError> {
        if g.to_bits() & !SIGN == 0 {
            return Ok(0.0);
        }
        const PHASE: &str = "discount factor";
        const OP: &str = "round logarithmic exponent";
        let index = Some(ordinal);
        let (sig, exponent) = decode(g);
        let numerator = u128::from(sig) * u128::from(lambda.numerator);
        let denominator = u128::from(lambda.denominator) * u128::from(frequency);
        checked(
            wide::set_u128(&mut self.numerator, numerator),
            index,
            PHASE,
            OP,
        )?;
        checked(
            wide::set_u128(&mut self.denominator, denominator),
            index,
            PHASE,
            OP,
        )?;
        let bits = wide::round_to_f64(
            &self.numerator,
            &self.denominator,
            exponent,
            &mut self.u,
            &mut self.v,
            &mut self.aligned,
        )
        .ok_or_else(|| capacity(index, PHASE, OP))?;
        Ok(f64::from_bits(
            bits | if g.is_sign_positive() { SIGN } else { 0 },
        ))
    }

    fn factor<F: FnMut(usize, f64) -> f64>(
        &mut self,
        metadata: Metadata,
        y: f64,
        g: f64,
        bundle: &Bundle,
        observed_exp: &mut F,
    ) -> Result<f64, CanadianError> {
        const PHASE: &str = "discount factor";
        let index = Some(bundle.index);
        let factor = if metadata.yield_convention == CanadianYieldConvention::MoneyMarket {
            self.mm_numerator(y, bundle)?;
            let bits = wide::round_to_f64(
                &self.n,
                &self.d,
                0,
                &mut self.u,
                &mut self.v,
                &mut self.aligned,
            )
            .ok_or_else(|| capacity(index, PHASE, "round rational"))?;
            f64::from_bits(bits)
        } else if g.to_bits() & !SIGN == 0 {
            1.0
        } else {
            let z = self.logarithmic_exponent(
                g,
                metadata.frequency.payments_per_year(),
                bundle.lambda,
                bundle.index,
            )?;
            validation::finite_result("canadian logarithmic exponent", z).map_err(|cause| {
                CanadianError::Rate {
                    index,
                    phase: PHASE,
                    cause: RateError::Value(cause),
                }
            })?;
            if requires_exponential(y, z, metadata.frequency.payments_per_year(), bundle.index)? {
                observed_exp(bundle.index, z)
            } else {
                1.0
            }
        };
        validation::result_within("discount factor", factor, Interval::greater_than(0.0)).map_err(
            |cause| CanadianError::Rate {
                index,
                phase: PHASE,
                cause: RateError::Value(cause),
            },
        )
    }

    fn present_value(&mut self, factor: f64, bundle: &Bundle) -> Result<f64, CanadianError> {
        const PHASE: &str = "source present value";
        let index = Some(bundle.index);
        if wide::is_zero(&bundle.amount.numerator) {
            return Ok(0.0);
        }
        let (sig, exponent) = decode(factor);
        checked(
            wide::copy_into(&bundle.amount.numerator, &mut self.numerator),
            index,
            PHASE,
            "round source present value",
        )?;
        checked(
            wide::mul_small_assign(&mut self.numerator, sig),
            index,
            PHASE,
            "round source present value",
        )?;
        let bits = wide::round_to_f64(
            &self.numerator,
            &bundle.amount.denominator,
            exponent,
            &mut self.u,
            &mut self.v,
            &mut self.aligned,
        )
        .ok_or_else(|| capacity(index, PHASE, "round source present value"))?;
        validation::result_within(
            "canadian present value",
            f64::from_bits(bits),
            Interval::greater_than(0.0),
        )
        .map_err(|cause| CanadianError::Value {
            index,
            phase: PHASE,
            cause,
        })
    }

    fn add_term(
        &mut self,
        exponent: i32,
        retained: &mut Retained,
        index: usize,
    ) -> Result<(), CanadianError> {
        const PHASE: &str = "risk present value terms";
        let (sig, exponent) = wide::round_to_extended53(
            &self.numerator,
            &self.denominator,
            exponent,
            &mut self.u,
            &mut self.v,
            &mut self.aligned,
        )
        .ok_or_else(|| capacity(Some(index), PHASE, "round scaled term"))?;
        if sig != 0 {
            let shift = u32::try_from(exponent + 8192)
                .map_err(|_| capacity(Some(index), PHASE, "accumulate risk"))?;
            checked(
                wide::add_shifted_u64(retained, sig, shift),
                Some(index),
                PHASE,
                "accumulate risk",
            )?;
        }
        Ok(())
    }

    fn risk_terms(
        &mut self,
        metadata: Metadata,
        y: f64,
        bundle: &Bundle,
        pv: f64,
        sums: &mut [Retained; 3],
    ) -> Result<(), CanadianError> {
        if pv.to_bits() & !SIGN == 0 {
            return Ok(());
        }
        const PHASE: &str = "risk present value terms";
        let index = Some(bundle.index);
        let (sig, exponent) = decode(pv);
        let h = u64::from(bundle.days_to_payment);
        if metadata.yield_convention == CanadianYieldConvention::Compound {
            let f = metadata.frequency.payments_per_year();
            let (delta_sig, delta_exp) = decode(f64::from(f) + y);
            let l = bundle.lambda.numerator;
            let k = u64::from(bundle.lambda.denominator);
            checked(
                wide::set_u128(&mut self.numerator, u128::from(sig) * u128::from(l)),
                index,
                PHASE,
                "round scaled term",
            )?;
            checked(
                wide::set_u128(&mut self.denominator, u128::from(k) * u128::from(delta_sig)),
                index,
                PHASE,
                "round scaled term",
            )?;
            self.add_term(exponent - delta_exp, &mut sums[0], bundle.index)?;
            checked(
                wide::set_u128(
                    &mut self.numerator,
                    u128::from(sig) * u128::from(l) * u128::from(l + k),
                ),
                index,
                PHASE,
                "round scaled term",
            )?;
            checked(
                wide::set_u128(&mut self.denominator, u128::from(k) * u128::from(k)),
                index,
                PHASE,
                "round scaled term",
            )?;
            checked(
                wide::mul_small_assign(&mut self.denominator, delta_sig),
                index,
                PHASE,
                "round scaled term",
            )?;
            checked(
                wide::mul_small_assign(&mut self.denominator, delta_sig),
                index,
                PHASE,
                "round scaled term",
            )?;
            self.add_term(exponent - 2 * delta_exp, &mut sums[1], bundle.index)?;
            checked(
                wide::set_u128(&mut self.numerator, u128::from(sig) * u128::from(l)),
                index,
                PHASE,
                "round scaled term",
            )?;
            checked(
                wide::set_u128(&mut self.denominator, u128::from(k) * u128::from(f)),
                index,
                PHASE,
                "round scaled term",
            )?;
            self.add_term(exponent, &mut sums[2], bundle.index)?;
        } else {
            checked(
                wide::set_u128(&mut self.numerator, u128::from(sig) * u128::from(h) * 365),
                index,
                PHASE,
                "round scaled term",
            )?;
            checked(
                wide::shl_assign(&mut self.numerator, 2148),
                index,
                PHASE,
                "round scaled term",
            )?;
            checked(
                wide::mul_into(&self.d, &self.n, &mut self.denominator),
                index,
                PHASE,
                "round scaled term",
            )?;
            self.add_term(exponent, &mut sums[0], bundle.index)?;
            let d = u128::from(metadata.days_to_true_maturity);
            checked(
                wide::set_u128(
                    &mut self.numerator,
                    2 * u128::from(sig) * d * u128::from(h) * 365,
                ),
                index,
                PHASE,
                "round scaled term",
            )?;
            checked(
                wide::shl_assign(&mut self.numerator, 3222),
                index,
                PHASE,
                "round scaled term",
            )?;
            checked(
                wide::mul_into(&self.d, &self.d, &mut self.product),
                index,
                PHASE,
                "round scaled term",
            )?;
            checked(
                wide::mul_into(&self.product, &self.n, &mut self.denominator),
                index,
                PHASE,
                "round scaled term",
            )?;
            self.add_term(exponent, &mut sums[1], bundle.index)?;
            checked(
                wide::set_u128(&mut self.numerator, u128::from(sig) * u128::from(h)),
                index,
                PHASE,
                "round scaled term",
            )?;
            checked(
                wide::set_u128(&mut self.denominator, 365),
                index,
                PHASE,
                "round scaled term",
            )?;
            self.add_term(exponent, &mut sums[2], bundle.index)?;
        }
        Ok(())
    }

    fn run(
        &mut self,
        plan: &CanadianPricingPlan<'_>,
        y: f64,
        terms: &mut Vec<f64>,
        sums: Option<&mut [Retained; 3]>,
    ) -> Result<f64, CanadianError> {
        let g = if plan.metadata.yield_convention == CanadianYieldConvention::Compound {
            scaled_logarithm(y, plan.metadata.frequency.payments_per_year())?
        } else {
            0.0
        };
        self.traverse(plan, y, g, terms, sums, &mut |_, z: f64| z.exp())
    }

    /// Core traversal receives already observed primitive inputs. Production
    /// always supplies its shared native logarithm and exactly the native exp
    /// above; private replay can bind explicit seeds without a public hook.
    fn traverse<F: FnMut(usize, f64) -> f64>(
        &mut self,
        plan: &CanadianPricingPlan<'_>,
        y: f64,
        g: f64,
        terms: &mut Vec<f64>,
        mut sums: Option<&mut [Retained; 3]>,
        observed_exp: &mut F,
    ) -> Result<f64, CanadianError> {
        terms.clear();
        for bundle in &plan.bundles {
            let factor = self.factor(plan.metadata, y, g, bundle, observed_exp)?;
            let pv = self.present_value(factor, bundle)?;
            if let Some(retained) = sums.as_deref_mut() {
                self.risk_terms(plan.metadata, y, bundle, pv, retained)?;
            }
            terms.push(pv);
        }
        dirty_price(terms)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LogarithmBranch {
    Zero,
    Tiny,
    Natural,
    NaturalOnePlus,
}

/// Exact prescribed argument observation, before the selected native logarithm.
/// The caller has already admitted the finite compound quote y>-f.
fn logarithm_argument(y: f64, f: u32) -> (LogarithmBranch, f64) {
    if y.to_bits() & !SIGN == 0 {
        (LogarithmBranch::Zero, 0.0)
    } else if y.abs() < TINY {
        (LogarithmBranch::Tiny, y)
    } else if y <= -f64::from(f) / 2.0 {
        // Sterbenz gives the exact positive numerator on this branch.
        (LogarithmBranch::Natural, (f64::from(f) + y) / f64::from(f))
    } else {
        (LogarithmBranch::NaturalOnePlus, y / f64::from(f))
    }
}

fn logarithm_scale(ell: f64, f: u32) -> f64 {
    f64::from(f) * ell
}

fn dirty_price(terms: &[f64]) -> Result<f64, CanadianError> {
    exact_sum(terms)
        .map(zero)
        .map_err(|cause| CanadianError::Sum {
            phase: "dirty price reduction",
            cause,
        })
}

fn scaled_logarithm(y: f64, f: u32) -> Result<f64, CanadianError> {
    let (branch, argument) = logarithm_argument(y, f);
    let g = match branch {
        LogarithmBranch::Zero | LogarithmBranch::Tiny => argument,
        LogarithmBranch::Natural => logarithm_scale(argument.ln(), f),
        LogarithmBranch::NaturalOnePlus => logarithm_scale(argument.ln_1p(), f),
    };
    validation::finite_result("canadian scaled logarithm", g).map_err(|cause| CanadianError::Rate {
        index: None,
        phase: "discount factor",
        cause: RateError::Value(cause),
    })
}

/// True requests exactly one native exp; false is the zero-exponent identity.
fn requires_exponential(y: f64, z: f64, f: u32, ordinal: usize) -> Result<bool, CanadianError> {
    if z.to_bits() & !SIGN == 0 {
        return Ok(false);
    }
    if y.abs() >= TINY && z.abs() < FAR_EXPONENT {
        guard(z, f, ordinal)?;
    }
    Ok(true)
}

// Four certified signed threshold magnitudes, little-endian. No floating log
// approximation or runtime text parsing participates in the closed comparison.
const THRESHOLDS: [[u64; 5]; 4] = [
    [
        0xb4f9_156b_7ef8_865f,
        0xbd82_fa73_3745_013a,
        0xc72b_2758_9541_e1ed,
        0x221a_a5a6_0a3b_ec60,
        0x2e9,
    ],
    [
        0xb4f9_156b_7ef8_865e,
        0xbd82_fa73_3745_013a,
        0xc72b_2758_9541_e1ed,
        0x221a_a5a6_0a3b_ec60,
        0x2e9,
    ],
    [
        0xdf08_1899_5693_5a49,
        0xcd0c_99ca_4d83_60d2,
        0x8ece_600f_cbd2_bd03,
        0xc85f_df47_3de6_ab27,
        0x2c5,
    ],
    [
        0xdf08_1899_5693_5a4a,
        0xcd0c_99ca_4d83_60d2,
        0x8ece_600f_cbd2_bd03,
        0xc85f_df47_3de6_ab27,
        0x2c5,
    ],
];

fn signed_cmp(a: &[u64], negative_a: bool, b: &[u64], negative_b: bool) -> Ordering {
    let negative_a = negative_a && !wide::is_zero(a);
    let negative_b = negative_b && !wide::is_zero(b);
    match (negative_a, negative_b) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => wide::cmp(a, b),
        (true, true) => wide::cmp(b, a),
    }
}

fn signed_sub(a: &[u64], b: &[u64], out: &mut [u64]) -> Option<bool> {
    let negative = wide::cmp(a, b) == Ordering::Less;
    let (large, small) = if negative { (b, a) } else { (a, b) };
    if !wide::copy_into(large, out) || !wide::sub_assign(out, small) {
        return None;
    }
    Some(negative && !wide::is_zero(out))
}

fn reaches_lower_endpoint(comparison: Ordering) -> bool {
    comparison != Ordering::Less
}

fn reaches_upper_endpoint(comparison: Ordering) -> bool {
    comparison != Ordering::Greater
}

fn guard(z: f64, f: u32, index: usize) -> Result<(), CanadianError> {
    const PHASE: &str = "discount factor";
    const OP: &str = "guard discount range";
    let error = || capacity(Some(index), PHASE, OP);
    // Six20-word magnitudes; helpers borrow and return scalars only. Zero,
    // tiny-quote and far-exponent cases bypass this construction at the caller.
    let mut a = [0_u64; 20];
    let mut c = [0_u64; 20];
    let mut n = [0_u64; 20];
    let mut lower = [0_u64; 20];
    let mut upper = [0_u64; 20];
    let mut threshold = [0_u64; 20];
    let (sig, exponent) = decode(z);
    let shift = u32::try_from(exponent + 1075).map_err(|_| error())?;
    let k = if f == 12 { 8 } else { 4 };
    let b = (1_u64 << 53) - k;
    if !wide::set_u128(&mut a, u128::from(sig))
        || !wide::shl_assign(&mut a, shift)
        || !wide::copy_into(&a, &mut c)
        || !wide::mul_small_assign(&mut c, b)
        || !wide::copy_into(&a, &mut n)
        || !wide::mul_small_assign(&mut n, k)
        || !wide::add_small_assign(&mut n, 1_u64 << 53)
    {
        return Err(error());
    }
    let (lower_negative, upper_negative) = if z.is_sign_negative() {
        if !wide::copy_into(&c, &mut lower) || !wide::add_assign(&mut lower, &n) {
            return Err(error());
        }
        (true, signed_sub(&n, &c, &mut upper).ok_or_else(error)?)
    } else {
        let negative = signed_sub(&c, &n, &mut lower).ok_or_else(error)?;
        if !wide::copy_into(&c, &mut upper) || !wide::add_assign(&mut upper, &n) {
            return Err(error());
        }
        (negative, false)
    };
    let mut intersections = [false; 4];
    for (i, limbs) in THRESHOLDS.iter().enumerate() {
        wide::clear(&mut threshold);
        for limb in limbs.iter().rev() {
            if !wide::shl_assign(&mut threshold, 64)
                || !wide::add_small_assign(&mut threshold, *limb)
            {
                return Err(error());
            }
        }
        if !wide::mul_small_assign(&mut threshold, b) || !wide::shl_assign(&mut threshold, 819) {
            return Err(error());
        }
        intersections[i] = if i % 2 == 0 {
            reaches_lower_endpoint(signed_cmp(&upper, upper_negative, &threshold, i < 2))
        } else {
            reaches_upper_endpoint(signed_cmp(&lower, lower_negative, &threshold, i < 2))
        };
    }
    if (intersections[0] && intersections[1]) || (intersections[2] && intersections[3]) {
        return Err(CanadianError::Rate {
            index: Some(index),
            phase: PHASE,
            cause: RateError::NumericalBoundary {
                quantity: "discount factor",
                logarithm: z,
            },
        });
    }
    Ok(())
}

fn bridge(
    metadata: Metadata,
    dirty: f64,
    basis: CanadianPriceBasis,
) -> Result<CanadianPrice, CanadianError> {
    let (phase, value) = match basis {
        CanadianPriceBasis::ModelDirty => return Ok(model_price(dirty)),
        CanadianPriceBasis::Clean => (
            "clean price reduction",
            exact_sum(&[dirty, -metadata.quote_accrued_per_100]),
        ),
        CanadianPriceBasis::SettlementDirty
            if metadata.yield_convention == CanadianYieldConvention::MoneyMarket =>
        {
            return Ok(CanadianPrice::observed(zero(dirty), basis));
        }
        CanadianPriceBasis::SettlementDirty => (
            "settlement dirty price reduction",
            exact_sum(&[
                dirty,
                -metadata.quote_accrued_per_100,
                metadata.settlement_accrued_per_100,
            ]),
        ),
    };
    let value = value.map_err(|cause| CanadianError::Sum { phase, cause })?;
    Ok(CanadianPrice::observed(zero(value), basis))
}

fn model_price(dirty: f64) -> CanadianPrice {
    CanadianPrice::observed(zero(dirty), CanadianPriceBasis::ModelDirty)
}

/// Resolve the represented target once, before its positivity/allocation gates.
fn resolve_target(metadata: Metadata, target: CanadianPrice) -> Result<f64, CanadianError> {
    let shifted = match target.basis() {
        CanadianPriceBasis::ModelDirty => return Ok(target.value()),
        CanadianPriceBasis::Clean => exact_sum(&[target.value(), metadata.quote_accrued_per_100]),
        CanadianPriceBasis::SettlementDirty => exact_sum(&[
            target.value(),
            -metadata.settlement_accrued_per_100,
            metadata.quote_accrued_per_100,
        ]),
    };
    shifted.map(zero).map_err(|cause| CanadianError::Sum {
        phase: "yield target shift",
        cause,
    })
}

fn residual(dirty: f64, target: f64, scale: u8) -> Result<f64, CanadianError> {
    const PHASE: &str = "yield amount residual";
    const OP: &str = "derive residual";
    // Five34-word arrays, distinct from the112-word traversal frame. The exact
    // signed difference remains on the MIN_SUB lattice until one final round.
    let mut a = [0_u64; 34];
    let mut b = [0_u64; 34];
    let mut u = [0_u64; 34];
    let mut v = [0_u64; 34];
    let mut aligned = [0_u64; 34];
    for (value, out) in [(dirty, &mut a), (target, &mut b)] {
        let (sig, exponent) = decode(value);
        let shift = u32::try_from(exponent + 1074).map_err(|_| capacity(None, PHASE, OP))?;
        checked(wide::set_u128(out, u128::from(sig)), None, PHASE, OP)?;
        checked(wide::shl_assign(out, shift), None, PHASE, OP)?;
    }
    let negative = if dirty.is_sign_negative() != target.is_sign_negative() {
        checked(wide::add_assign(&mut a, &b), None, PHASE, OP)?;
        dirty.is_sign_negative()
    } else if wide::cmp(&a, &b) == Ordering::Less {
        checked(wide::sub_assign(&mut b, &a), None, PHASE, OP)?;
        checked(wide::copy_into(&b, &mut a), None, PHASE, OP)?;
        !dirty.is_sign_negative()
    } else {
        checked(wide::sub_assign(&mut a, &b), None, PHASE, OP)?;
        dirty.is_sign_negative()
    };
    let bits = wide::round_to_f64(
        &a,
        &[1],
        -1074 - i32::from(scale),
        &mut u,
        &mut v,
        &mut aligned,
    )
    .ok_or_else(|| capacity(None, PHASE, OP))?;
    let value = f64::from_bits(bits | if negative && bits != 0 { SIGN } else { 0 });
    validation::finite_result(PHASE, value).map_err(|cause| CanadianError::Value {
        index: None,
        phase: PHASE,
        cause,
    })
}

impl CanadianPricingPlan<'_> {
    /// Observes all required factors/PVs and reduces the positive model price once.
    ///
    /// # Errors
    /// Preserves quote/domain, factor, source-PV, allocation and reduction errors
    /// in their prescribed order and original actual-coupon ordinal context.
    pub fn evaluate(&self, yield_rate: f64) -> Result<CanadianValuation, CanadianError> {
        let mut workspace = Workspace::new();
        let y = workspace.quote(self.metadata, yield_rate)?;
        let mut terms = reserve(self.bundles.len(), "present value terms")?;
        let dirty = workspace.run(self, y, &mut terms, None)?;
        Ok(CanadianValuation {
            metadata: self.metadata,
            yield_rate: y,
            dirty,
        })
    }

    /// Retains three selected extended53 risk sums from the same accepted PV seeds.
    ///
    /// # Errors
    /// Preserves quote/domain and ordered factor/PV/term failures. An individual
    /// raw getter's IEEE range is not checked during this construction.
    pub fn risk(&self, yield_rate: f64) -> Result<CanadianRisk, CanadianError> {
        let mut workspace = Workspace::new();
        let y = workspace.quote(self.metadata, yield_rate)?;
        let mut terms = reserve(self.bundles.len(), "risk present value terms")?;
        let mut risk = CanadianRisk {
            metadata: self.metadata,
            yield_rate: y,
            dirty: 0.0,
            sums: [[0; RETAINED_WORDS]; 3],
        };
        risk.dirty = workspace.run(self, y, &mut terms, Some(&mut risk.sums))?;
        Ok(risk)
    }

    /// Runs bounded shared Brent with one PV vector and cached selected payload.
    ///
    /// All three external bases resolve once before target positivity and the
    /// workspace reservation. Only actually invoked points undergo quote checks.
    /// No ideal-image precheck or final repricing is performed.
    ///
    /// # Errors
    /// Preserves Target/Preparation/Solver/Callback and NonPositiveTarget with
    /// immediate typed causes and exact attempted invocation counts.
    pub fn yield_from_price(
        &self,
        target: CanadianPrice,
        bracket: RootBracket,
        options: SolverOptions,
    ) -> Result<CanadianYieldReport, CanadianYieldError> {
        let dirty_target =
            resolve_target(self.metadata, target).map_err(|cause| CanadianYieldError::Target {
                target,
                phase: "yield target shift",
                cause,
            })?;
        if dirty_target <= 0.0 {
            return Err(CanadianYieldError::NonPositiveTarget {
                target,
                model_dirty_target_per_100: dirty_target,
                quote_accrued_per_100: self.metadata.quote_accrued_per_100,
                settlement_accrued_per_100: self.metadata.settlement_accrued_per_100,
                source_policy: self.metadata.source_policy,
                convention: self.metadata.yield_convention,
                evaluations: EvaluationCounts {
                    function: 0,
                    derivative: 0,
                },
            });
        }
        let residual_scale = u8::from(dirty_target.to_bits() >= SCALE_THRESHOLD);
        let mut terms =
            reserve(self.bundles.len(), "yield present value terms").map_err(|cause| {
                CanadianYieldError::Preparation {
                    phase: "yield present value terms",
                    cause,
                }
            })?;
        let mut workspace = Workspace::new();
        let (solve_report, dirty) = brent_with_payload(
            |coordinate| {
                let y = workspace.quote(self.metadata, coordinate)?;
                let dirty = workspace.run(self, y, &mut terms, None)?;
                Ok((residual(dirty, dirty_target, residual_scale)?, dirty))
            },
            bracket,
            options,
        )
        .map_err(|failure| match failure {
            FallibleSolverError::Solver(cause) => CanadianYieldError::Solver(cause),
            FallibleSolverError::Callback {
                stage,
                point,
                evaluations,
                cause,
            } => CanadianYieldError::Callback {
                stage,
                point,
                evaluations,
                cause,
            },
        })?;
        Ok(CanadianYieldReport {
            metadata: self.metadata,
            yield_rate: zero(solve_report.point()),
            dirty,
            target,
            dirty_target,
            initial_bracket: bracket,
            options,
            solve_report,
            residual_scale,
        })
    }
}

fn risk_value(
    sum: &Retained,
    negative: bool,
    denominator: Option<f64>,
    quantity: &'static str,
    y: f64,
) -> Result<f64, CanadianError> {
    if wide::is_zero(sum) {
        return Ok(0.0);
    }
    let (sig, exponent) = if let Some(dirty) = denominator {
        if dirty.to_bits() & !SIGN == 0 {
            return Err(CanadianError::ZeroDirtyPrice {
                measure: quantity,
                yield_rate: y,
            });
        }
        decode(dirty)
    } else {
        (1, 0)
    };
    // Exactly three243-word scratch magnitudes; the retained input is borrowed.
    let mut u = [0_u64; RETAINED_WORDS];
    let mut v = [0_u64; RETAINED_WORDS];
    let mut aligned = [0_u64; RETAINED_WORDS];
    let bits = wide::round_to_f64(sum, &[sig], -8192 - exponent, &mut u, &mut v, &mut aligned)
        .ok_or_else(|| {
            capacity(
                None,
                quantity,
                if denominator.is_some() {
                    "normalize risk value"
                } else {
                    "round risk value"
                },
            )
        })?;
    let value = f64::from_bits(bits | if negative { SIGN } else { 0 });
    validation::result_within(
        quantity,
        value,
        if negative {
            Interval::less_than(0.0)
        } else {
            Interval::greater_than(0.0)
        },
    )
    .map_err(|cause| CanadianError::Value {
        index: None,
        phase: quantity,
        cause,
    })
}

impl CanadianRisk {
    /// Returns the selected negative first price derivative.
    /// # Errors
    /// A nonzero result rounded to signed zero/infinity is a named Value error.
    pub fn price_derivative(&self) -> Result<f64, CanadianError> {
        risk_value(
            &self.sums[0],
            true,
            None,
            "price derivative",
            self.yield_rate,
        )
    }
    /// Returns the selected second price derivative, without a half factor.
    /// # Errors
    /// A nonzero result rounded to zero/infinity is a named Value error.
    pub fn price_second_derivative(&self) -> Result<f64, CanadianError> {
        risk_value(
            &self.sums[1],
            false,
            None,
            "price second derivative",
            self.yield_rate,
        )
    }
    /// Returns the selected time-weighted amount in the convention's clock.
    /// # Errors
    /// A nonzero result rounded to zero/infinity is a named Value error.
    pub fn time_weighted_amount(&self) -> Result<f64, CanadianError> {
        risk_value(
            &self.sums[2],
            false,
            None,
            "time weighted amount",
            self.yield_rate,
        )
    }
    /// Directly normalizes the retained negative derivative by observed dirty.
    /// # Errors
    /// Preserves this getter's independent range or defensive zero-dirty error.
    pub fn modified_duration(&self) -> Result<f64, CanadianError> {
        risk_value(
            &self.sums[0],
            false,
            Some(self.dirty),
            "modified duration",
            self.yield_rate,
        )
    }
    /// Directly normalizes the retained time moment by observed dirty.
    /// # Errors
    /// Preserves this getter's independent range or defensive zero-dirty error.
    pub fn macaulay_duration(&self) -> Result<f64, CanadianError> {
        risk_value(
            &self.sums[2],
            false,
            Some(self.dirty),
            "macaulay duration",
            self.yield_rate,
        )
    }
    /// Directly normalizes the retained second derivative by observed dirty.
    /// # Errors
    /// Preserves this getter's independent range or defensive zero-dirty error.
    pub fn convexity(&self) -> Result<f64, CanadianError> {
        risk_value(
            &self.sums[1],
            false,
            Some(self.dirty),
            "convexity",
            self.yield_rate,
        )
    }
    /// Returns the cached positive model-dirty quote without repricing.
    #[must_use]
    pub fn model_dirty_price(&self) -> CanadianPrice {
        model_price(self.dirty)
    }
    /// Observes clean by one exact cached-price/accrual reduction.
    /// # Errors
    /// Preserves the independent clean price reduction failure.
    pub fn clean_price(&self) -> Result<CanadianPrice, CanadianError> {
        bridge(self.metadata, self.dirty, CanadianPriceBasis::Clean)
    }
    /// Observes all-in settlement dirty by its one-shot cached bridge.
    /// # Errors
    /// Preserves the independent settlement dirty price reduction failure.
    pub fn settlement_dirty_price(&self) -> Result<CanadianPrice, CanadianError> {
        bridge(
            self.metadata,
            self.dirty,
            CanadianPriceBasis::SettlementDirty,
        )
    }
}

macro_rules! scalar_prices {
    ($type:ty) => {
        impl $type {
            /// Returns the cached positive model-dirty quote without repricing.
            #[must_use]
            pub fn model_dirty_price(self) -> CanadianPrice {
                model_price(self.dirty)
            }
            /// Observes clean by one exact cached-price/accrual reduction.
            /// # Errors
            /// Preserves this independent clean reduction's error.
            pub fn clean_price(self) -> Result<CanadianPrice, CanadianError> {
                bridge(self.metadata, self.dirty, CanadianPriceBasis::Clean)
            }
            /// Observes settlement dirty by one exact three-term bridge (MM identity).
            /// # Errors
            /// Preserves this independent settlement reduction's error.
            pub fn settlement_dirty_price(self) -> Result<CanadianPrice, CanadianError> {
                bridge(
                    self.metadata,
                    self.dirty,
                    CanadianPriceBasis::SettlementDirty,
                )
            }
        }
    };
}
scalar_prices!(CanadianValuation);
scalar_prices!(CanadianYieldReport);

impl CanadianYieldReport {
    /// Returns the original externally supplied typed target.
    #[must_use]
    pub const fn target(self) -> CanadianPrice {
        self.target
    }
    /// Returns the once-resolved represented model-dirty target per100.
    #[must_use]
    pub const fn model_dirty_target_per_100(self) -> f64 {
        self.dirty_target
    }
    /// Returns the caller's original finite coordinate bracket.
    #[must_use]
    pub const fn initial_bracket(self) -> RootBracket {
        self.initial_bracket
    }
    /// Returns the caller's exact solver tolerances and total invocation budget.
    #[must_use]
    pub const fn options(self) -> SolverOptions {
        self.options
    }
    /// Returns the shared solver's cached report and exact counts.
    #[must_use]
    pub const fn solve_report(self) -> SolveReport {
        self.solve_report
    }
    /// Returns the power-of-two residual scaling exponent0 or1.
    #[must_use]
    pub const fn residual_scale_exponent(self) -> u8 {
        self.residual_scale
    }
    /// Returns ideal mathematical sensitivity, not a rounded-root certificate.
    #[must_use]
    pub const fn sensitivity(self) -> BondYieldSensitivity {
        BondYieldSensitivity::MathematicallyStrictlyDecreasing
    }
    /// Observes the exact represented cached dirty-target difference once.
    ///
    /// This rounded getter is not the exact-rational residual predicate required
    /// by the independent inverse accuracy certificate. It performs no callback.
    /// # Errors
    /// Preserves defensive capacity/nonfinite result errors. Positive finite
    /// cached dirty and target exclude actual range failure on admitted states.
    pub fn amount_residual(self) -> Result<f64, CanadianError> {
        residual(self.dirty, self.dirty_target, 0)
    }
}

macro_rules! report_metadata {
    ($type:ty, $($receiver:tt)*) => {
        impl $type {
            /// Returns the immutable local model identifier without source I/O.
            #[must_use]
            pub const fn source_version($($receiver)* self) -> &'static str { SOURCE_VERSION }
            /// Returns the explicit local source policy.
            #[must_use]
            pub const fn source_policy($($receiver)* self) -> super::CanadianSourcePolicy { self.metadata.source_policy }
            /// Returns the source accrual start D.
            #[must_use]
            pub const fn accrual_start($($receiver)* self) -> crate::time::Date { self.metadata.accrual_start }
            /// Returns the original issue date I.
            #[must_use]
            pub const fn original_issue_date($($receiver)* self) -> crate::time::Date { self.metadata.original_issue_date }
            /// Returns the contractual maturity M.
            #[must_use]
            pub const fn stated_maturity_date($($receiver)* self) -> crate::time::Date { self.metadata.stated_maturity_date }
            /// Returns the explicit true payment date T.
            #[must_use]
            pub const fn true_maturity_date($($receiver)* self) -> crate::time::Date { self.metadata.true_maturity_date }
            /// Returns the first actual coupon date F.
            #[must_use]
            pub const fn first_coupon_date($($receiver)* self) -> crate::time::Date { self.metadata.first_coupon_date }
            /// Returns the retained final source interval start L.
            #[must_use]
            pub const fn last_coupon_start_date($($receiver)* self) -> crate::time::Date { self.metadata.last_coupon_start_date }
            /// Returns the inferred first-period role.
            #[must_use]
            pub const fn first_period($($receiver)* self) -> super::CanadianPeriodKind { self.metadata.first_period }
            /// Returns the inferred final-period role.
            #[must_use]
            pub const fn last_period($($receiver)* self) -> super::CanadianPeriodKind { self.metadata.last_period }
            /// Returns the explicit frequency.
            #[must_use]
            pub const fn frequency($($receiver)* self) -> super::CanadianFrequency { self.metadata.frequency }
            /// Returns the exact annual coupon per100.
            #[must_use]
            pub const fn coupon_per_100($($receiver)* self) -> crate::amounts::DecimalAmount { self.metadata.coupon_per_100 }
            /// Returns the explicit direct-cycle EOM policy.
            #[must_use]
            pub const fn end_of_month($($receiver)* self) -> crate::calendar::EndOfMonth { self.metadata.end_of_month }
            /// Returns the retained direct-cycle anchor.
            #[must_use]
            pub const fn direct_cycle_anchor($($receiver)* self) -> crate::time::Date { self.metadata.direct_cycle_anchor }
            /// Returns the prepared settlement date S.
            #[must_use]
            pub const fn settlement_date($($receiver)* self) -> crate::time::Date { self.metadata.settlement_date }
            /// Returns the original explicit convention selection.
            #[must_use]
            pub const fn yield_selection($($receiver)* self) -> super::CanadianYieldSelection { self.metadata.yield_selection }
            /// Returns the resolved numerical convention.
            #[must_use]
            pub const fn yield_convention($($receiver)* self) -> CanadianYieldConvention { self.metadata.yield_convention }
            /// Returns the cached valuation-accrued binary observation.
            #[must_use]
            pub const fn valuation_accrued_per_100($($receiver)* self) -> f64 { self.metadata.valuation_accrued_per_100 }
            /// Returns the cached settlement-accrued binary observation.
            #[must_use]
            pub const fn settlement_accrued_per_100($($receiver)* self) -> f64 { self.metadata.settlement_accrued_per_100 }
            /// Returns the convention-selected cached quote accrual.
            #[must_use]
            pub const fn quote_accrued_per_100($($receiver)* self) -> f64 { self.metadata.quote_accrued_per_100 }
            /// Returns the canonical annual decimal yield coordinate.
            #[must_use]
            pub const fn yield_rate($($receiver)* self) -> f64 { self.yield_rate }
            /// Returns the cached positive model-dirty observation per100.
            #[must_use]
            pub const fn model_dirty_per_100($($receiver)* self) -> f64 { self.dirty }
        }
    };
}
report_metadata!(CanadianValuation,);
report_metadata!(CanadianYieldReport,);
report_metadata!(CanadianRisk, &);

#[cfg(test)]
mod tests {
    use super::super::{
        CanadianBond, CanadianCouponSchedule, CanadianFrequency, CanadianSourcePolicy,
        CanadianYieldSelection, Coordinate,
    };
    use super::*;
    use crate::amounts::DecimalAmount;
    use crate::calendar::EndOfMonth;
    use crate::error::ValueError;
    use crate::time::Date;

    /// FR-CAD-36/40: the prescribed exponent preserves an IEEE underflow sign,
    /// while the later discount identity consumes either zero without exp.
    #[test]
    fn signed_underflow_exponent_is_preserved_before_zero_identity() -> Result<(), Box<dyn StdError>>
    {
        let mut workspace = Workspace::new();
        for frequency in [2, 4, 12] {
            // These are independently supplied primitive coordinates, not a
            // claim that every frequency attains a184-day quasi cell.
            for denominator in [2, 184] {
                let lambda = Coordinate {
                    numerator: 1,
                    denominator,
                };
                for (g, expected) in [(f64::from_bits(1), SIGN), (-f64::from_bits(1), 0)] {
                    let z = workspace.logarithmic_exponent(g, frequency, lambda, 17)?;
                    assert_eq!(z.to_bits(), expected);
                    assert!(!requires_exponential(g, z, frequency, 17)?);
                }
            }
            let zero = workspace.logarithmic_exponent(
                0.0,
                frequency,
                Coordinate {
                    numerator: 1,
                    denominator: 2,
                },
                17,
            )?;
            assert_eq!(zero.to_bits(), 0);
        }
        Ok(())
    }

    /// FR-CAD-39/41: a genuinely dated zero coupon still observes and validates
    /// its factor, using the original ordinal after an earlier coupon ceased.
    #[test]
    fn zero_source_still_observes_factor_before_present_value_shortcut()
    -> Result<(), Box<dyn StdError>> {
        let start = Date::from_ymd(2025, 1, 1)?;
        let first = Date::from_ymd(2025, 7, 1)?;
        let next = Date::from_ymd(2026, 1, 1)?;
        let maturity = Date::from_ymd(2026, 7, 1)?;
        let schedule = CanadianCouponSchedule::new(
            start,
            maturity,
            CanadianFrequency::Semiannual,
            vec![start, first, next, maturity],
            vec![first, next, maturity],
            EndOfMonth::Ignore,
            4,
        )?;
        let bond = CanadianBond::new(
            start,
            DecimalAmount::new(0, 0)?,
            schedule,
            maturity,
            CanadianSourcePolicy::CanonicalIiac13LocalV1,
        )?;
        let plan = CanadianPricingPlan::new(
            &bond,
            first,
            CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound),
            2,
        )?;
        let bundle = &plan.bundles[0];
        assert_eq!(bundle.index, 1);
        assert!(wide::is_zero(&bundle.amount.numerator));
        let mut workspace = Workspace::new();
        let y = workspace.quote(plan.metadata, 0.05)?;
        let g = scaled_logarithm(y, 2)?;
        let mut calls = 0;
        let factor = workspace.factor(plan.metadata, y, g, bundle, &mut |ordinal, z: f64| {
            calls += 1;
            assert_eq!(ordinal, 1);
            z.exp()
        })?;
        assert_eq!(calls, 1);
        assert_eq!(workspace.present_value(factor, bundle)?.to_bits(), 0);

        let mut invalid_calls = 0;
        let invalid = workspace.factor(plan.metadata, y, g, bundle, &mut |ordinal, _| {
            invalid_calls += 1;
            assert_eq!(ordinal, 1);
            0.0
        });
        assert_eq!(invalid_calls, 1);
        assert!(matches!(
            invalid,
            Err(CanadianError::Rate {
                index: Some(1),
                phase: "discount factor",
                cause: RateError::Value(ValueError::ResultOutOfDomain {
                    quantity: "discount factor",
                    value,
                    ..
                }),
                ..
            }) if value.to_bits() == 0
        ));
        Ok(())
    }
}
