//! Generic prepared-plan bond yield-to-maturity inversion and analytical risk.
//!
//! Inversion solves for the annual decimal yield rate `y` whose prepared dirty
//! present value matches the resolved dirty target amount `T`, using the exact
//! fixed-scale binary64 residual `R(y) = RN_even((D_hat(y) - T) * 2^-s)` with
//! `s = 0` when `|T| < 2^970` and `s = 1` when `|T| >= 2^970`. Analytical risk
//! accumulates 53-bit rounded extended-exponent per-flow contributions on the
//! exact common basis `2^-8192` and normalizes duration and convexity views
//! directly by the observed dirty present value `D_hat(y)`.

use std::cmp::Ordering;
use std::error::Error as StdError;
use std::fmt;

use crate::amounts::uint::Wide;
use crate::error::{Interval, NonFinite, ValueError};
use crate::math::{
    EvaluationCounts, EvaluationStage, FallibleSolverError, RootBracket, SolveReport, SolverError,
    SolverOptions, brent_with_payload, exact_sum,
};
use crate::rates::{ActiveBranch, Compounding, observe_present_value_at, validate_rate};

use super::{
    BondError, BondPrice, BondPriceKind, BondPricingPlan, BondSettlement, canonical_zero,
    rate_error, reserved, value_error,
};

const SIGN_BIT: u64 = 1_u64 << 63;
const FRACTION_MASK: u64 = (1_u64 << 52) - 1;
const INFINITY_BITS: u64 = 0x7ff0_0000_0000_0000;
/// Exact IEEE 754 magnitude bit pattern of `2^970` (`(970 + 1023) << 52`).
const RESIDUAL_SCALE_THRESHOLD_BITS: u64 = 1993_u64 << 52;

/// Thirty-four limbs (2,176 bits) hold any `2^-1074` basis sum or difference.
type MinSubWide = Wide<34>;
/// Five limbs (320 bits) hold at most five numerator and two denominator significands.
type TermWide = Wide<5>;
/// Fixed accumulator width on the `2^-8192` extended risk basis (`243` limbs on 32/64-bit targets).
const WORDS: usize = (15461 + usize::BITS as usize).div_ceil(64);
type RiskWide = Wide<WORDS>;

/// Sufficient mathematical monotonicity classification of a prepared bond plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BondYieldSensitivity {
    /// All buyer-entitled flow amounts are nonnegative and at least one strictly
    /// positive amount has a strictly positive discount horizon.
    MathematicallyStrictlyDecreasing,
    /// Signed or mixed-sign flows do not carry a global monotonicity certificate.
    Uncertified,
}

/// Owned report of a successful prepared-plan yield-to-maturity inversion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BondYieldReport {
    target: BondPrice,
    converted_target_amount: f64,
    dirty_target_amount: f64,
    dirty_amount: f64,
    accrued_amount: f64,
    face: f64,
    settlement: BondSettlement,
    compounding: Compounding,
    yield_rate: f64,
    initial_bracket: RootBracket,
    options: SolverOptions,
    solve_report: SolveReport,
    residual_scale_exponent: u8,
    sensitivity: BondYieldSensitivity,
}

impl BondYieldReport {
    /// Returns the original caller-supplied target price quote and kind.
    #[must_use]
    pub const fn target(self) -> BondPrice {
        self.target
    }

    /// Returns the target quote converted once to amount units for `face`.
    #[must_use]
    pub const fn converted_target_amount(self) -> f64 {
        self.converted_target_amount
    }

    /// Returns the resolved dirty target amount `T` matched by the solver.
    #[must_use]
    pub const fn dirty_target_amount(self) -> f64 {
        self.dirty_target_amount
    }

    /// Returns the cached dirty present value `D_hat` at [`yield_rate`](Self::yield_rate).
    #[must_use]
    pub const fn dirty_amount(self) -> f64 {
        self.dirty_amount
    }

    /// Returns the plan's cached yield-independent accrued interest amount.
    #[must_use]
    pub const fn accrued_amount(self) -> f64 {
        self.accrued_amount
    }

    /// Returns the bond's strictly positive finite face amount.
    #[must_use]
    pub const fn face(self) -> f64 {
        self.face
    }

    /// Returns the settlement specification of the pricing plan.
    #[must_use]
    pub const fn settlement(self) -> BondSettlement {
        self.settlement
    }

    /// Returns the compounding convention of the solved yield.
    #[must_use]
    pub const fn compounding(self) -> Compounding {
        self.compounding
    }

    /// Returns the solved annual decimal yield rate.
    #[must_use]
    pub const fn yield_rate(self) -> f64 {
        self.yield_rate
    }

    /// Returns the caller's initial search bracket.
    #[must_use]
    pub const fn initial_bracket(self) -> RootBracket {
        self.initial_bracket
    }

    /// Returns the caller's solver tolerances and evaluation cap.
    #[must_use]
    pub const fn options(self) -> SolverOptions {
        self.options
    }

    /// Returns the underlying solver termination report on the scaled residual.
    #[must_use]
    pub const fn solve_report(self) -> SolveReport {
        self.solve_report
    }

    /// Returns the fixed binary scale exponent `s` (`0` or `1`) used in `R(y)`.
    #[must_use]
    pub const fn residual_scale_exponent(self) -> u8 {
        self.residual_scale_exponent
    }

    /// Returns the plan's mathematical yield-sensitivity classification.
    #[must_use]
    pub const fn sensitivity(self) -> BondYieldSensitivity {
        self.sensitivity
    }

    /// Computes the unscaled amount-unit residual `RN_even(dirty_amount - dirty_target_amount)`.
    ///
    /// # Errors
    /// [`BondError::Value`] if the unscaled difference overflows `f64`, or
    /// [`BondError::InternalCapacity`] on a defensive invariant failure.
    pub fn amount_residual(self) -> Result<f64, BondError> {
        const PHASE: &str = "yield amount residual";
        let (mag, negative) =
            min_sub_difference(self.dirty_amount, self.dirty_target_amount, PHASE)?;
        if mag.is_zero() {
            return Ok(0.0);
        }
        let one = MinSubWide::from_u128(1);
        let bits = round_wide_ratio_to_f64(mag, one, -1074).ok_or(BondError::InternalCapacity {
            index: None,
            phase: PHASE,
            operation: "round residual",
        })?;
        if bits == INFINITY_BITS {
            return Err(value_error(
                None,
                PHASE,
                ValueError::ResultNotFinite {
                    quantity: PHASE,
                    value: if negative {
                        NonFinite::NegativeInfinity
                    } else {
                        NonFinite::PositiveInfinity
                    },
                },
            ));
        }
        let sign = if negative { SIGN_BIT } else { 0 };
        Ok(canonical_zero(f64::from_bits(sign | bits)))
    }

    /// Computes the clean amount `exact_sum([dirty_amount, -accrued_amount])` without repricing.
    ///
    /// # Errors
    /// [`BondError::Sum`] if the clean reduction overflows `f64`.
    pub fn clean_amount(self) -> Result<f64, BondError> {
        exact_sum(&[self.dirty_amount, -self.accrued_amount])
            .map(canonical_zero)
            .map_err(|cause| BondError::Sum {
                phase: "clean price reduction",
                cause,
            })
    }

    /// Normalizes the cached [`dirty_amount`](Self::dirty_amount) per 100 of face.
    ///
    /// # Errors
    /// [`BondError::Value`] if a nonzero normalized quote underflows or overflows.
    pub fn dirty_price(self) -> Result<BondPrice, BondError> {
        BondPrice::from_amount(self.dirty_amount, self.face, BondPriceKind::Dirty)
    }

    /// Computes [`clean_amount`](Self::clean_amount) and normalizes it per 100 of face.
    ///
    /// # Errors
    /// [`BondError::Sum`] if the clean reduction overflows, or [`BondError::Value`]
    /// if a nonzero normalized quote underflows or overflows.
    pub fn clean_price(self) -> Result<BondPrice, BondError> {
        let clean = self.clean_amount()?;
        BondPrice::from_amount(clean, self.face, BondPriceKind::Clean)
    }
}

/// Canonical signed sum on the `2^-8192` extended risk basis.
#[derive(Clone, Copy, PartialEq, Eq)]
struct SignedExtendedSum {
    negative: bool,
    words: RiskWide,
}

/// Analytical raw and dirty-normalized yield risk measures at a trial yield rate.
#[derive(Clone, PartialEq)]
pub struct BondRisk {
    yield_rate: f64,
    settlement: BondSettlement,
    face: f64,
    compounding: Compounding,
    dirty_amount: f64,
    price_derivative: SignedExtendedSum,
    price_second_derivative: SignedExtendedSum,
    time_weighted_amount: SignedExtendedSum,
}

impl fmt::Debug for BondRisk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BondRisk")
            .field("yield_rate", &self.yield_rate)
            .field("settlement", &self.settlement)
            .field("face", &self.face)
            .field("compounding", &self.compounding)
            .field("dirty_amount", &self.dirty_amount)
            .finish_non_exhaustive()
    }
}

impl BondRisk {
    /// Returns the canonicalized annual decimal yield rate.
    #[must_use]
    pub const fn yield_rate(&self) -> f64 {
        self.yield_rate
    }

    /// Returns the settlement specification of the pricing plan.
    #[must_use]
    pub const fn settlement(&self) -> BondSettlement {
        self.settlement
    }

    /// Returns the bond's strictly positive finite face amount.
    #[must_use]
    pub const fn face(&self) -> f64 {
        self.face
    }

    /// Returns the compounding convention of the yield quote.
    #[must_use]
    pub const fn compounding(&self) -> Compounding {
        self.compounding
    }

    /// Returns the buyer's dirty present value `D_hat` in amount units.
    #[must_use]
    pub const fn dirty_amount(&self) -> f64 {
        self.dirty_amount
    }

    /// Rounds the retained first yield derivative `dP/dy` to `f64` in amount units per unit yield.
    ///
    /// # Errors
    /// [`BondError::Value`] if a nonzero sum underflows to zero or overflows to infinity,
    /// or [`BondError::InternalCapacity`] on a defensive invariant failure.
    pub fn price_derivative(&self) -> Result<f64, BondError> {
        round_raw_risk(self.price_derivative, "price derivative")
    }

    /// Rounds the retained second yield derivative `d^2P/dy^2` to `f64` in amount units per squared unit yield.
    ///
    /// # Errors
    /// [`BondError::Value`] if a nonzero sum underflows to zero or overflows to infinity,
    /// or [`BondError::InternalCapacity`] on a defensive invariant failure.
    pub fn price_second_derivative(&self) -> Result<f64, BondError> {
        round_raw_risk(self.price_second_derivative, "price second derivative")
    }

    /// Rounds the retained time-weighted present value `sum(t_i * PV_i)` to `f64` in amount-year units.
    ///
    /// # Errors
    /// [`BondError::Value`] if a nonzero sum underflows to zero or overflows to infinity,
    /// or [`BondError::InternalCapacity`] on a defensive invariant failure.
    pub fn time_weighted_amount(&self) -> Result<f64, BondError> {
        round_raw_risk(self.time_weighted_amount, "time weighted amount")
    }

    /// Computes modified duration `-(dP/dy) / D_hat` directly from the extended sum.
    ///
    /// # Errors
    /// [`BondError::ZeroDirtyPrice`] if `dirty_amount == 0.0`, [`BondError::Value`]
    /// if a nonzero ratio underflows or overflows, or [`BondError::InternalCapacity`].
    pub fn modified_duration(&self) -> Result<f64, BondError> {
        round_normalized_risk(
            self.price_derivative,
            true,
            self.dirty_amount,
            self.yield_rate,
            "modified duration",
        )
    }

    /// Computes Macaulay duration `sum(t_i * PV_i) / D_hat` directly from the extended sum.
    ///
    /// # Errors
    /// [`BondError::ZeroDirtyPrice`] if `dirty_amount == 0.0`, [`BondError::Value`]
    /// if a nonzero ratio underflows or overflows, or [`BondError::InternalCapacity`].
    pub fn macaulay_duration(&self) -> Result<f64, BondError> {
        round_normalized_risk(
            self.time_weighted_amount,
            false,
            self.dirty_amount,
            self.yield_rate,
            "macaulay duration",
        )
    }

    /// Computes convexity `(d^2P/dy^2) / D_hat` (without a `1/2` Taylor factor) directly from the extended sum.
    ///
    /// # Errors
    /// [`BondError::ZeroDirtyPrice`] if `dirty_amount == 0.0`, [`BondError::Value`]
    /// if a nonzero ratio underflows or overflows, or [`BondError::InternalCapacity`].
    pub fn convexity(&self) -> Result<f64, BondError> {
        round_normalized_risk(
            self.price_second_derivative,
            false,
            self.dirty_amount,
            self.yield_rate,
            "convexity",
        )
    }
}

/// A failure during prepared-plan bond yield-to-maturity inversion.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum BondYieldError {
    /// Target quote conversion or clean-to-dirty shift failed; kind `bond_yield.target`.
    #[non_exhaustive]
    Target {
        /// Original caller target quote.
        target: BondPrice,
        /// Target resolution stage (`"target conversion"` or `"target clean shift"`).
        phase: &'static str,
        /// Underlying bond quote or summation error.
        cause: BondError,
    },
    /// Workspace reservation or constant-plan classification failed; kind `bond_yield.preparation`.
    #[non_exhaustive]
    Preparation {
        /// Preparation stage (`"yield present value terms"`, `"yield sensitivity keys"`, or `"yield constant classification"`).
        phase: &'static str,
        /// Underlying allocation or capacity error.
        cause: BondError,
    },
    /// Root-finding failed on finite callback values; kind `bond_yield.solver`.
    Solver(SolverError),
    /// A trial yield evaluation failed during root-finding; kind `bond_yield.callback`.
    #[non_exhaustive]
    Callback {
        /// Solver stage that invoked the callback.
        stage: EvaluationStage,
        /// Canonical finite trial yield passed to the callback.
        point: f64,
        /// Cumulative callback counts including the failed call.
        evaluations: EvaluationCounts,
        /// Underlying bond valuation or residual error.
        cause: BondError,
    },
    /// The plan's mathematical price is yield-independent and equals the target; kind `bond_yield.no_yield_sensitivity`.
    #[non_exhaustive]
    NoYieldSensitivity {
        /// Caller's initial search bracket.
        bracket: RootBracket,
        /// Resolved dirty target amount `T`.
        dirty_target_amount: f64,
        /// Dirty amount observed at the lower bracket endpoint.
        observed_lower_dirty_amount: f64,
        /// Rounded diagnostic of the exact constant price `C`, or `None` on overflow.
        constant_amount: Option<f64>,
        /// Callback counts performed (`function = 1`, `derivative = 0`).
        evaluations: EvaluationCounts,
    },
    /// The plan's mathematical price is yield-independent and differs from the target; kind `bond_yield.constant_price_mismatch`.
    #[non_exhaustive]
    ConstantPriceMismatch {
        /// Caller's initial search bracket.
        bracket: RootBracket,
        /// Resolved dirty target amount `T`.
        dirty_target_amount: f64,
        /// Dirty amount observed at the lower bracket endpoint.
        observed_lower_dirty_amount: f64,
        /// Rounded diagnostic of the exact constant price `C`, or `None` on overflow.
        constant_amount: Option<f64>,
        /// Callback counts performed (`function = 1`, `derivative = 0`).
        evaluations: EvaluationCounts,
    },
}

impl BondYieldError {
    /// Returns the stable `bond_yield.<variant>` kind name.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Target { .. } => "bond_yield.target",
            Self::Preparation { .. } => "bond_yield.preparation",
            Self::Solver(_) => "bond_yield.solver",
            Self::Callback { .. } => "bond_yield.callback",
            Self::NoYieldSensitivity { .. } => "bond_yield.no_yield_sensitivity",
            Self::ConstantPriceMismatch { .. } => "bond_yield.constant_price_mismatch",
        }
    }
}

impl fmt::Display for BondYieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Target {
                target,
                phase,
                cause,
            } => write!(
                f,
                "{phase} for {:?} target {}: {cause}",
                target.kind(),
                target.value_per_100_face()
            ),
            Self::Preparation { phase, cause } => write!(f, "{phase}: {cause}"),
            Self::Solver(cause) => fmt::Display::fmt(cause, f),
            Self::Callback {
                stage,
                point,
                evaluations,
                cause,
            } => write!(
                f,
                "yield callback at {stage:?} ({point:?}, {} evaluations): {cause}",
                evaluations.total()
            ),
            Self::NoYieldSensitivity {
                bracket,
                dirty_target_amount,
                observed_lower_dirty_amount,
                constant_amount,
                evaluations,
            } => write!(
                f,
                "bond price is mathematically constant ({constant_amount:?}, observed lower {observed_lower_dirty_amount:?}) and matches dirty target {dirty_target_amount:?} on [{:?}, {:?}] after {} evaluation",
                bracket.lower(),
                bracket.upper(),
                evaluations.total()
            ),
            Self::ConstantPriceMismatch {
                bracket,
                dirty_target_amount,
                observed_lower_dirty_amount,
                constant_amount,
                evaluations,
            } => write!(
                f,
                "bond price is mathematically constant ({constant_amount:?}, observed lower {observed_lower_dirty_amount:?}) and cannot match dirty target {dirty_target_amount:?} on [{:?}, {:?}] after {} evaluation",
                bracket.lower(),
                bracket.upper(),
                evaluations.total()
            ),
        }
    }
}

impl StdError for BondYieldError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Target { cause, .. }
            | Self::Preparation { cause, .. }
            | Self::Callback { cause, .. } => Some(cause),
            Self::Solver(cause) => Some(cause),
            Self::NoYieldSensitivity { .. } | Self::ConstantPriceMismatch { .. } => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SensitivityKey {
    branch: ActiveBranch,
    horizon_bits: u64,
    magnitude_bits: u64,
    negative: bool,
}

impl BondPricingPlan<'_> {
    /// Solves for the annual decimal yield rate that prices the prepared plan to `target`.
    ///
    /// # Errors
    /// [`BondYieldError::Target`] if target quote conversion or clean-to-dirty shift
    /// fails, [`BondYieldError::Preparation`] on workspace reservation failure,
    /// [`BondYieldError::Callback`] if any queried trial yield fails valuation,
    /// [`BondYieldError::NoYieldSensitivity`] or [`BondYieldError::ConstantPriceMismatch`]
    /// if the plan's mathematical price is yield-independent, or
    /// [`BondYieldError::Solver`] on bracket, budget or precision failure.
    pub fn yield_from_price(
        &self,
        target: BondPrice,
        bracket: RootBracket,
        options: SolverOptions,
    ) -> Result<BondYieldReport, BondYieldError> {
        let converted_target_amount =
            canonical_zero(target.to_amount(self.bond.face).map_err(|cause| {
                BondYieldError::Target {
                    target,
                    phase: "target conversion",
                    cause,
                }
            })?);
        let dirty_target_amount = match target.kind() {
            BondPriceKind::Dirty => converted_target_amount,
            BondPriceKind::Clean => exact_sum(&[converted_target_amount, self.accrued_amount])
                .map(canonical_zero)
                .map_err(|cause| BondYieldError::Target {
                    target,
                    phase: "target clean shift",
                    cause: BondError::Sum {
                        phase: "yield target shift",
                        cause,
                    },
                })?,
        };
        let residual_scale_exponent =
            u8::from((dirty_target_amount.to_bits() & !SIGN_BIT) >= RESIDUAL_SCALE_THRESHOLD_BITS);
        let mut pv_terms = reserved(self.discount_flows.len(), "yield present value terms")
            .map_err(|cause| BondYieldError::Preparation {
                phase: "yield present value terms",
                cause,
            })?;
        let mut keys =
            reserved(self.discount_flows.len(), "yield sensitivity keys").map_err(|cause| {
                BondYieldError::Preparation {
                    phase: "yield sensitivity keys",
                    cause,
                }
            })?;

        let compounding = self.yield_convention.compounding();
        let mut all_nonnegative = true;
        let mut has_positive_amount_at_positive_time = false;
        for flow in &self.discount_flows {
            if flow.amount < 0.0 {
                all_nonnegative = false;
            }
            if flow.amount != 0.0 && flow.horizon > 0.0 {
                if flow.amount > 0.0 {
                    has_positive_amount_at_positive_time = true;
                }
                keys.push(SensitivityKey {
                    branch: compounding.active_branch(flow.horizon),
                    horizon_bits: canonical_zero(flow.horizon).to_bits(),
                    magnitude_bits: flow.amount.abs().to_bits(),
                    negative: flow.amount.is_sign_negative(),
                });
            }
        }
        keys.sort_unstable();
        if keys_pair_off(&keys) {
            return Err(self.finish_constant_plan(bracket, dirty_target_amount, &mut pv_terms));
        }
        let sensitivity = if all_nonnegative && has_positive_amount_at_positive_time {
            BondYieldSensitivity::MathematicallyStrictlyDecreasing
        } else {
            BondYieldSensitivity::Uncertified
        };

        let (solve_report, dirty_amount) = brent_with_payload(
            |trial_yield| {
                let (_, dirty) = self.evaluate_dirty_into(trial_yield, &mut pv_terms)?;
                let residual =
                    compute_scaled_residual(dirty, dirty_target_amount, residual_scale_exponent)?;
                Ok((residual, dirty))
            },
            bracket,
            options,
        )
        .map_err(|error| match error {
            FallibleSolverError::Solver(cause) => BondYieldError::Solver(cause),
            FallibleSolverError::Callback {
                stage,
                point,
                evaluations,
                cause,
            } => BondYieldError::Callback {
                stage,
                point,
                evaluations,
                cause,
            },
        })?;

        Ok(BondYieldReport {
            target,
            converted_target_amount,
            dirty_target_amount,
            dirty_amount,
            accrued_amount: self.accrued_amount,
            face: self.bond.face,
            settlement: self.settlement,
            compounding,
            yield_rate: solve_report.point(),
            initial_bracket: bracket,
            options,
            solve_report,
            residual_scale_exponent,
            sensitivity,
        })
    }

    /// Evaluates analytical raw and dirty-normalized yield risk measures at `yield_rate`.
    ///
    /// # Errors
    /// [`BondError::Rate`] for an invalid `yield_rate` or any per-flow discount
    /// factor/present-value failure, [`BondError::AllocationFailed`] for workspace
    /// reservation, [`BondError::Sum`] for dirty present-value reduction overflow,
    /// or [`BondError::InternalCapacity`] on a defensive invariant failure.
    pub fn risk(&self, yield_rate: f64) -> Result<BondRisk, BondError> {
        let compounding = self.yield_convention.compounding();
        let rate = validate_rate(yield_rate, compounding)
            .map_err(|cause| rate_error(None, "yield quote", cause))?;
        let mut pv_terms = reserved(self.discount_flows.len(), "risk present value terms")?;
        let mut p_prime_pos = RiskWide::ZERO;
        let mut p_prime_neg = RiskWide::ZERO;
        let mut p_second_pos = RiskWide::ZERO;
        let mut p_second_neg = RiskWide::ZERO;
        let mut moment_pos = RiskWide::ZERO;
        let mut moment_neg = RiskWide::ZERO;

        for flow in &self.discount_flows {
            let obs = observe_present_value_at(rate, compounding, flow.amount, flow.horizon)
                .map_err(|cause| rate_error(Some(flow.index), "discount factor", cause))?;
            pv_terms.push(obs.present_value);
            if flow.horizon == 0.0 || obs.present_value == 0.0 {
                continue;
            }
            let t = flow.horizon;
            let v_hat = obs.present_value;
            let v_neg = v_hat.is_sign_negative();

            let moment_term = round_extended_term(
                &[v_hat, t],
                &[],
                0,
                v_neg,
                flow.index,
                "time weighted amount",
            )?;
            accumulate_risk_term(
                &mut moment_pos,
                &mut moment_neg,
                moment_term,
                flow.index,
                "time weighted amount",
            )?;

            match obs.branch {
                ActiveBranch::Simple => {
                    let q = obs.simple_compound_factor;
                    let d1 = round_extended_term(
                        &[v_hat, t],
                        &[q],
                        0,
                        !v_neg,
                        flow.index,
                        "price derivative",
                    )?;
                    accumulate_risk_term(
                        &mut p_prime_pos,
                        &mut p_prime_neg,
                        d1,
                        flow.index,
                        "price derivative",
                    )?;
                    let d2 = round_extended_term(
                        &[v_hat, t, t],
                        &[q, q],
                        1,
                        v_neg,
                        flow.index,
                        "price second derivative",
                    )?;
                    accumulate_risk_term(
                        &mut p_second_pos,
                        &mut p_second_neg,
                        d2,
                        flow.index,
                        "price second derivative",
                    )?;
                }
                ActiveBranch::Continuous => {
                    let d1 = round_extended_term(
                        &[v_hat, t],
                        &[],
                        0,
                        !v_neg,
                        flow.index,
                        "price derivative",
                    )?;
                    accumulate_risk_term(
                        &mut p_prime_pos,
                        &mut p_prime_neg,
                        d1,
                        flow.index,
                        "price derivative",
                    )?;
                    let d2 = round_extended_term(
                        &[v_hat, t, t],
                        &[],
                        0,
                        v_neg,
                        flow.index,
                        "price second derivative",
                    )?;
                    accumulate_risk_term(
                        &mut p_second_pos,
                        &mut p_second_neg,
                        d2,
                        flow.index,
                        "price second derivative",
                    )?;
                }
                ActiveBranch::Periodic(count) => {
                    let m = f64::from(count);
                    let delta = m + rate;
                    let w = t + 1.0 / m;
                    let d1 = round_extended_term(
                        &[v_hat, t, m],
                        &[delta],
                        0,
                        !v_neg,
                        flow.index,
                        "price derivative",
                    )?;
                    accumulate_risk_term(
                        &mut p_prime_pos,
                        &mut p_prime_neg,
                        d1,
                        flow.index,
                        "price derivative",
                    )?;
                    let d2 = round_extended_term(
                        &[v_hat, t, w, m, m],
                        &[delta, delta],
                        0,
                        v_neg,
                        flow.index,
                        "price second derivative",
                    )?;
                    accumulate_risk_term(
                        &mut p_second_pos,
                        &mut p_second_neg,
                        d2,
                        flow.index,
                        "price second derivative",
                    )?;
                }
            }
        }

        let dirty_amount =
            exact_sum(&pv_terms)
                .map(canonical_zero)
                .map_err(|cause| BondError::Sum {
                    phase: "dirty price reduction",
                    cause,
                })?;
        let price_derivative = subtract_risk_sums(p_prime_pos, p_prime_neg, "price derivative")?;
        let price_second_derivative =
            subtract_risk_sums(p_second_pos, p_second_neg, "price second derivative")?;
        let time_weighted_amount =
            subtract_risk_sums(moment_pos, moment_neg, "time weighted amount")?;

        Ok(BondRisk {
            yield_rate: rate,
            settlement: self.settlement,
            face: self.bond.face,
            compounding,
            dirty_amount,
            price_derivative,
            price_second_derivative,
            time_weighted_amount,
        })
    }

    fn prepare_constant_plan(
        &self,
        dirty_target_amount: f64,
    ) -> Result<(MinSubWide, bool, MinSubWide, bool, Option<f64>), BondError> {
        const PHASE: &str = "yield constant classification";
        let (const_mag, const_neg) = self.accumulate_zero_horizon_constant()?;
        let (target_mag, target_neg) =
            decode_min_sub(dirty_target_amount, PHASE, "compare constant")?;
        let constant_amount = round_constant_diagnostic(const_mag, const_neg)?;
        Ok((
            const_mag,
            const_neg,
            target_mag,
            target_neg,
            constant_amount,
        ))
    }

    fn finish_constant_plan(
        &self,
        bracket: RootBracket,
        dirty_target_amount: f64,
        pv_terms: &mut Vec<f64>,
    ) -> BondYieldError {
        let (const_mag, const_neg, target_mag, target_neg, constant_amount) =
            match self.prepare_constant_plan(dirty_target_amount) {
                Ok(prepared) => prepared,
                Err(cause) => {
                    return BondYieldError::Preparation {
                        phase: "yield constant classification",
                        cause,
                    };
                }
            };

        let evaluations = EvaluationCounts {
            function: 1,
            derivative: 0,
        };
        let lower_point = canonical_zero(bracket.lower());
        let observed_lower_dirty_amount = match self.evaluate_dirty_into(lower_point, pv_terms) {
            Ok((_, dirty)) => dirty,
            Err(cause) => {
                return BondYieldError::Callback {
                    stage: EvaluationStage::LowerEndpoint,
                    point: lower_point,
                    evaluations,
                    cause,
                };
            }
        };

        let equal = (const_mag.is_zero() && target_mag.is_zero())
            || (const_neg == target_neg && const_mag == target_mag);
        if equal {
            BondYieldError::NoYieldSensitivity {
                bracket,
                dirty_target_amount,
                observed_lower_dirty_amount,
                constant_amount,
                evaluations,
            }
        } else {
            BondYieldError::ConstantPriceMismatch {
                bracket,
                dirty_target_amount,
                observed_lower_dirty_amount,
                constant_amount,
                evaluations,
            }
        }
    }

    fn accumulate_zero_horizon_constant(&self) -> Result<(MinSubWide, bool), BondError> {
        const PHASE: &str = "yield constant classification";
        const OP: &str = "accumulate constant";
        let mut pos = MinSubWide::ZERO;
        let mut neg = MinSubWide::ZERO;
        for flow in &self.discount_flows {
            if flow.horizon == 0.0 && flow.amount != 0.0 {
                let (mag, is_neg) = decode_min_sub(flow.amount, PHASE, OP)?;
                if is_neg {
                    neg = neg.checked_add(mag).ok_or(BondError::InternalCapacity {
                        index: None,
                        phase: PHASE,
                        operation: OP,
                    })?;
                } else {
                    pos = pos.checked_add(mag).ok_or(BondError::InternalCapacity {
                        index: None,
                        phase: PHASE,
                        operation: OP,
                    })?;
                }
            }
        }
        match pos.cmp(&neg) {
            Ordering::Equal => Ok((MinSubWide::ZERO, false)),
            Ordering::Greater => {
                let diff = pos.checked_sub(neg).ok_or(BondError::InternalCapacity {
                    index: None,
                    phase: PHASE,
                    operation: OP,
                })?;
                Ok((diff, false))
            }
            Ordering::Less => {
                let diff = neg.checked_sub(pos).ok_or(BondError::InternalCapacity {
                    index: None,
                    phase: PHASE,
                    operation: OP,
                })?;
                Ok((diff, true))
            }
        }
    }
}

fn keys_pair_off(keys: &[SensitivityKey]) -> bool {
    if !keys.len().is_multiple_of(2) {
        return false;
    }
    let mut index = 0;
    while index < keys.len() {
        let first = keys[index];
        let mut pos_count = 0_usize;
        let mut neg_count = 0_usize;
        while index < keys.len()
            && keys[index].branch == first.branch
            && keys[index].horizon_bits == first.horizon_bits
            && keys[index].magnitude_bits == first.magnitude_bits
        {
            if keys[index].negative {
                neg_count += 1;
            } else {
                pos_count += 1;
            }
            index += 1;
        }
        if pos_count != neg_count {
            return false;
        }
    }
    true
}

fn round_constant_diagnostic(mag: MinSubWide, negative: bool) -> Result<Option<f64>, BondError> {
    if mag.is_zero() {
        return Ok(Some(0.0));
    }
    let bits = round_wide_ratio_to_f64(mag, MinSubWide::from_u128(1), -1074).ok_or(
        BondError::InternalCapacity {
            index: None,
            phase: "yield constant classification",
            operation: "round constant diagnostic",
        },
    )?;
    if bits == 0 || bits == INFINITY_BITS {
        Ok(None)
    } else {
        let sign = if negative { SIGN_BIT } else { 0 };
        Ok(Some(canonical_zero(f64::from_bits(sign | bits))))
    }
}

fn compute_scaled_residual(
    dirty_amount: f64,
    dirty_target_amount: f64,
    scale_exponent: u8,
) -> Result<f64, BondError> {
    const PHASE: &str = "yield residual";
    let (mag, negative) = min_sub_difference(dirty_amount, dirty_target_amount, PHASE)?;
    if mag.is_zero() {
        return Ok(0.0);
    }
    let exponent = -1074 - i32::from(scale_exponent);
    let bits = round_wide_ratio_to_f64(mag, MinSubWide::from_u128(1), exponent).ok_or(
        BondError::InternalCapacity {
            index: None,
            phase: PHASE,
            operation: "round residual",
        },
    )?;
    if bits == 0 || bits == INFINITY_BITS {
        return Err(BondError::InternalCapacity {
            index: None,
            phase: PHASE,
            operation: "round residual",
        });
    }
    let sign = if negative { SIGN_BIT } else { 0 };
    Ok(canonical_zero(f64::from_bits(sign | bits)))
}

fn min_sub_difference(
    first: f64,
    second: f64,
    phase: &'static str,
) -> Result<(MinSubWide, bool), BondError> {
    const OP: &str = "residual difference";
    let (first_mag, first_neg) = decode_min_sub(first, phase, OP)?;
    let (second_mag, second_neg) = decode_min_sub(second, phase, OP)?;
    if first_mag.is_zero() && second_mag.is_zero() {
        return Ok((MinSubWide::ZERO, false));
    }
    if first_mag.is_zero() {
        return Ok((second_mag, !second_neg));
    }
    if second_mag.is_zero() {
        return Ok((first_mag, first_neg));
    }
    let sub_neg = !second_neg;
    if first_neg == sub_neg {
        let sum = first_mag
            .checked_add(second_mag)
            .ok_or(BondError::InternalCapacity {
                index: None,
                phase,
                operation: OP,
            })?;
        Ok((sum, first_neg))
    } else {
        match first_mag.cmp(&second_mag) {
            Ordering::Equal => Ok((MinSubWide::ZERO, false)),
            Ordering::Greater => {
                let diff =
                    first_mag
                        .checked_sub(second_mag)
                        .ok_or(BondError::InternalCapacity {
                            index: None,
                            phase,
                            operation: OP,
                        })?;
                Ok((diff, first_neg))
            }
            Ordering::Less => {
                let diff =
                    second_mag
                        .checked_sub(first_mag)
                        .ok_or(BondError::InternalCapacity {
                            index: None,
                            phase,
                            operation: OP,
                        })?;
                Ok((diff, sub_neg))
            }
        }
    }
}

fn decode_min_sub(
    value: f64,
    phase: &'static str,
    operation: &'static str,
) -> Result<(MinSubWide, bool), BondError> {
    if !value.is_finite() {
        return Err(BondError::InternalCapacity {
            index: None,
            phase,
            operation,
        });
    }
    let bits = value.to_bits();
    let negative = (bits & SIGN_BIT) != 0;
    let field = ((bits >> 52) & 0x7ff) as u32;
    let fraction = bits & FRACTION_MASK;
    let (significand, shift) = if field == 0 {
        (fraction, 0_u32)
    } else {
        (fraction | (1_u64 << 52), field - 1)
    };
    if significand == 0 {
        return Ok((MinSubWide::ZERO, false));
    }
    let wide = MinSubWide::from_u128(u128::from(significand))
        .checked_shl(shift)
        .ok_or(BondError::InternalCapacity {
            index: None,
            phase,
            operation,
        })?;
    Ok((wide, negative))
}

#[derive(Clone, Copy)]
struct ExtendedTerm {
    negative: bool,
    significand: u64,
    exponent: i32,
}

fn decompose_nonzero(
    value: f64,
    index: Option<usize>,
    phase: &'static str,
) -> Result<(u64, i32), BondError> {
    if !value.is_finite() || value == 0.0 {
        return Err(BondError::InternalCapacity {
            index,
            phase,
            operation: "decode binary",
        });
    }
    let bits = value.to_bits();
    let field = i32::try_from((bits >> 52) & 0x7ff).unwrap_or(0);
    let fraction = bits & FRACTION_MASK;
    if field == 0 {
        Ok((fraction, -1074))
    } else {
        Ok((fraction | (1_u64 << 52), field - 1075))
    }
}

fn round_extended_term(
    num_factors: &[f64],
    den_factors: &[f64],
    extra_exp: i32,
    negative: bool,
    index: usize,
    phase: &'static str,
) -> Result<ExtendedTerm, BondError> {
    let mut num = TermWide::from_u128(1);
    let mut num_exp = extra_exp;
    for &factor in num_factors {
        let (sig, exp) = decompose_nonzero(factor, Some(index), phase)?;
        num = num
            .checked_mul(TermWide::from_u128(u128::from(sig)))
            .ok_or(BondError::InternalCapacity {
                index: Some(index),
                phase,
                operation: "normalize rational",
            })?;
        num_exp += exp;
    }
    let mut den = TermWide::from_u128(1);
    let mut den_exp = 0_i32;
    for &factor in den_factors {
        let (sig, exp) = decompose_nonzero(factor, Some(index), phase)?;
        den = den
            .checked_mul(TermWide::from_u128(u128::from(sig)))
            .ok_or(BondError::InternalCapacity {
                index: Some(index),
                phase,
                operation: "normalize rational",
            })?;
        den_exp += exp;
    }

    #[expect(
        clippy::cast_possible_wrap,
        reason = "TermWide bit count is at most 320"
    )]
    let k0 = (num.bits() as i32) - (den.bits() as i32);
    let at_least_k0 = if k0 >= 0 {
        let shifted = den
            .checked_shl(k0.unsigned_abs())
            .ok_or(BondError::InternalCapacity {
                index: Some(index),
                phase,
                operation: "normalize rational",
            })?;
        num >= shifted
    } else {
        let shifted = num
            .checked_shl(k0.unsigned_abs())
            .ok_or(BondError::InternalCapacity {
                index: Some(index),
                phase,
                operation: "normalize rational",
            })?;
        shifted >= den
    };
    let k = if at_least_k0 { k0 } else { k0 - 1 };
    let shift = 52 - k;
    let (scaled_num, scaled_den) = if shift >= 0 {
        let shifted = num
            .checked_shl(shift.unsigned_abs())
            .ok_or(BondError::InternalCapacity {
                index: Some(index),
                phase,
                operation: "round scaled term",
            })?;
        (shifted, den)
    } else {
        let shifted = den
            .checked_shl(shift.unsigned_abs())
            .ok_or(BondError::InternalCapacity {
                index: Some(index),
                phase,
                operation: "round scaled term",
            })?;
        (num, shifted)
    };
    let (quotient_wide, remainder) =
        scaled_num
            .div_rem(scaled_den)
            .ok_or(BondError::InternalCapacity {
                index: Some(index),
                phase,
                operation: "round scaled term",
            })?;
    let mut significand = quotient_wide
        .to_u128()
        .and_then(|v| u64::try_from(v).ok())
        .ok_or(BondError::InternalCapacity {
            index: Some(index),
            phase,
            operation: "round scaled term",
        })?;
    let complement = scaled_den
        .checked_sub(remainder)
        .ok_or(BondError::InternalCapacity {
            index: Some(index),
            phase,
            operation: "round scaled term",
        })?;
    let mut exponent = (num_exp - den_exp) + k - 52;
    if remainder > complement || (remainder == complement && (significand & 1 != 0)) {
        significand += 1;
        if significand == (1_u64 << 53) {
            significand >>= 1;
            exponent += 1;
        }
    }
    Ok(ExtendedTerm {
        negative,
        significand,
        exponent,
    })
}

fn accumulate_risk_term(
    pos: &mut RiskWide,
    neg: &mut RiskWide,
    term: ExtendedTerm,
    index: usize,
    phase: &'static str,
) -> Result<(), BondError> {
    const OP: &str = "accumulate risk";
    let shift = u32::try_from(term.exponent + 8192).map_err(|_| BondError::InternalCapacity {
        index: Some(index),
        phase,
        operation: OP,
    })?;
    let scaled = RiskWide::from_u128(u128::from(term.significand))
        .checked_shl(shift)
        .ok_or(BondError::InternalCapacity {
            index: Some(index),
            phase,
            operation: OP,
        })?;
    let target = if term.negative { neg } else { pos };
    *target = target
        .checked_add(scaled)
        .ok_or(BondError::InternalCapacity {
            index: Some(index),
            phase,
            operation: OP,
        })?;
    Ok(())
}

fn subtract_risk_sums(
    pos: RiskWide,
    neg: RiskWide,
    phase: &'static str,
) -> Result<SignedExtendedSum, BondError> {
    const OP: &str = "subtract risk sums";
    match pos.cmp(&neg) {
        Ordering::Equal => Ok(SignedExtendedSum {
            negative: false,
            words: RiskWide::ZERO,
        }),
        Ordering::Greater => {
            let words = pos.checked_sub(neg).ok_or(BondError::InternalCapacity {
                index: None,
                phase,
                operation: OP,
            })?;
            Ok(SignedExtendedSum {
                negative: false,
                words,
            })
        }
        Ordering::Less => {
            let words = neg.checked_sub(pos).ok_or(BondError::InternalCapacity {
                index: None,
                phase,
                operation: OP,
            })?;
            Ok(SignedExtendedSum {
                negative: true,
                words,
            })
        }
    }
}

fn round_raw_risk(sum: SignedExtendedSum, phase: &'static str) -> Result<f64, BondError> {
    if sum.words.is_zero() {
        return Ok(0.0);
    }
    let bits = round_wide_ratio_to_f64(sum.words, RiskWide::from_u128(1), -8192).ok_or(
        BondError::InternalCapacity {
            index: None,
            phase,
            operation: "round risk value",
        },
    )?;
    finish_signed_risk_f64(bits, sum.negative, phase)
}

fn round_normalized_risk(
    sum: SignedExtendedSum,
    negate_numerator: bool,
    dirty_amount: f64,
    yield_rate: f64,
    measure: &'static str,
) -> Result<f64, BondError> {
    if dirty_amount == 0.0 {
        return Err(BondError::ZeroDirtyPrice {
            measure,
            yield_rate,
        });
    }
    if sum.words.is_zero() {
        return Ok(0.0);
    }
    let (d_sig, d_exp) = decompose_nonzero(dirty_amount, None, measure)?;
    let negative = (sum.negative ^ negate_numerator) ^ dirty_amount.is_sign_negative();
    let denominator = RiskWide::from_u128(u128::from(d_sig));
    let bits = round_wide_ratio_to_f64(sum.words, denominator, -8192 - d_exp).ok_or(
        BondError::InternalCapacity {
            index: None,
            phase: measure,
            operation: "normalize risk value",
        },
    )?;
    finish_signed_risk_f64(bits, negative, measure)
}

fn finish_signed_risk_f64(
    magnitude_bits: u64,
    negative: bool,
    phase: &'static str,
) -> Result<f64, BondError> {
    let sign = if negative { SIGN_BIT } else { 0 };
    if magnitude_bits == INFINITY_BITS {
        return Err(value_error(
            None,
            phase,
            ValueError::ResultNotFinite {
                quantity: phase,
                value: if negative {
                    NonFinite::NegativeInfinity
                } else {
                    NonFinite::PositiveInfinity
                },
            },
        ));
    }
    if magnitude_bits == 0 {
        let domain = if negative {
            Interval::less_than(0.0)
        } else {
            Interval::greater_than(0.0)
        };
        return Err(value_error(
            None,
            phase,
            ValueError::ResultOutOfDomain {
                quantity: phase,
                value: f64::from_bits(sign),
                domain,
            },
        ));
    }
    Ok(f64::from_bits(sign | magnitude_bits))
}

/// Rounds `(numerator / denominator) * 2^exponent` once to nearest binary64
/// magnitude bits, ties to even.
pub(super) fn round_wide_ratio_to_f64<const N: usize>(
    numerator: Wide<N>,
    denominator: Wide<N>,
    exponent: i32,
) -> Option<u64> {
    if numerator.is_zero() {
        return Some(0);
    }
    let num_bits = i32::try_from(numerator.bits()).ok()?;
    let den_bits = i32::try_from(denominator.bits()).ok()?;
    let k0 = num_bits - den_bits;
    let at_least_k0 = if k0 >= 0 {
        numerator >= denominator.checked_shl(k0.unsigned_abs())?
    } else {
        numerator.checked_shl(k0.unsigned_abs())? >= denominator
    };
    let k = if at_least_k0 { k0 } else { k0 - 1 };
    let top = k + exponent;
    if top > 1023 {
        return Some(INFINITY_BITS);
    }
    if top < -1075 {
        return Some(0);
    }
    let unit_exponent = (top - 52).max(-1074);
    let shift = exponent - unit_exponent;
    let (scaled_num, scaled_den) = if shift >= 0 {
        (numerator.checked_shl(shift.unsigned_abs())?, denominator)
    } else {
        (numerator, denominator.checked_shl(shift.unsigned_abs())?)
    };
    let (quotient, remainder) = scaled_num.div_rem(scaled_den)?;
    let mut retained = u64::try_from(quotient.to_u128()?).ok()?;
    let complement = scaled_den.checked_sub(remainder)?;
    if remainder > complement || (remainder == complement && (retained & 1 != 0)) {
        retained += 1;
    }
    if top < -1022 {
        return Some(retained);
    }
    let mut rounded_top = top;
    if retained == (1_u64 << 53) {
        retained >>= 1;
        rounded_top += 1;
    }
    if rounded_top > 1023 {
        Some(INFINITY_BITS)
    } else {
        let biased = u64::try_from(rounded_top + 1023).ok()?;
        Some((biased << 52) | (retained & FRACTION_MASK))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitivity_keys_pair_off_with_multiplicity_and_reject_mismatches() {
        assert!(!keys_pair_off(&[SensitivityKey {
            branch: ActiveBranch::Continuous,
            horizon_bits: 1.0_f64.to_bits(),
            magnitude_bits: 100.0_f64.to_bits(),
            negative: false,
        }]));

        let mut keys = vec![
            SensitivityKey {
                branch: ActiveBranch::Continuous,
                horizon_bits: 1.0_f64.to_bits(),
                magnitude_bits: 100.0_f64.to_bits(),
                negative: true,
            },
            SensitivityKey {
                branch: ActiveBranch::Continuous,
                horizon_bits: 1.0_f64.to_bits(),
                magnitude_bits: 100.0_f64.to_bits(),
                negative: false,
            },
            SensitivityKey {
                branch: ActiveBranch::Continuous,
                horizon_bits: 1.0_f64.to_bits(),
                magnitude_bits: 100.0_f64.to_bits(),
                negative: true,
            },
            SensitivityKey {
                branch: ActiveBranch::Continuous,
                horizon_bits: 1.0_f64.to_bits(),
                magnitude_bits: 100.0_f64.to_bits(),
                negative: false,
            },
        ];
        keys.sort_unstable();
        assert!(keys_pair_off(&keys));

        let mut unbalanced = vec![
            SensitivityKey {
                branch: ActiveBranch::Simple,
                horizon_bits: 1.0_f64.to_bits(),
                magnitude_bits: 50.0_f64.to_bits(),
                negative: false,
            },
            SensitivityKey {
                branch: ActiveBranch::Simple,
                horizon_bits: 1.0_f64.to_bits(),
                magnitude_bits: 50.0_f64.to_bits(),
                negative: false,
            },
        ];
        unbalanced.sort_unstable();
        assert!(!keys_pair_off(&unbalanced));
    }

    #[test]
    fn min_sub_and_scaled_residual_cover_valid_sign_and_scale_cases() {
        assert_eq!(
            decode_min_sub(-0.0, "test", "op").unwrap(),
            (MinSubWide::ZERO, false)
        );
        assert_eq!(
            decode_min_sub(f64::from_bits(1), "test", "op").unwrap(),
            (MinSubWide::from_u128(1), false)
        );

        assert_eq!(
            min_sub_difference(0.0, -0.0, "test").unwrap(),
            (MinSubWide::ZERO, false)
        );
        assert_eq!(
            min_sub_difference(0.0, 2.0, "test").unwrap(),
            decode_min_sub(-2.0, "test", "op").unwrap()
        );
        assert_eq!(
            min_sub_difference(-3.0, 0.0, "test").unwrap(),
            decode_min_sub(-3.0, "test", "op").unwrap()
        );
        assert_eq!(
            min_sub_difference(4.0, 4.0, "test").unwrap(),
            (MinSubWide::ZERO, false)
        );
        assert_eq!(
            min_sub_difference(5.0, 2.0, "test").unwrap(),
            decode_min_sub(3.0, "test", "op").unwrap()
        );
        assert_eq!(
            min_sub_difference(2.0, 5.0, "test").unwrap(),
            decode_min_sub(-3.0, "test", "op").unwrap()
        );
        assert_eq!(
            min_sub_difference(-2.0, 3.0, "test").unwrap(),
            decode_min_sub(-5.0, "test", "op").unwrap()
        );

        assert_eq!(compute_scaled_residual(10.0, 10.0, 0).unwrap(), 0.0);
        assert_eq!(compute_scaled_residual(10.0, 6.0, 0).unwrap(), 4.0);
        assert_eq!(compute_scaled_residual(10.0, 6.0, 1).unwrap(), 2.0);
        assert_eq!(
            compute_scaled_residual(f64::MAX, -f64::MAX, 1).unwrap(),
            f64::MAX
        );

        assert_eq!(
            round_constant_diagnostic(MinSubWide::ZERO, true).unwrap(),
            Some(0.0)
        );
        let (max_mag, _) = decode_min_sub(f64::MAX, "test", "op").unwrap();
        let two_max = max_mag.checked_add(max_mag).unwrap();
        assert_eq!(round_constant_diagnostic(two_max, false).unwrap(), None);
    }

    #[test]
    fn extended_term_and_wide_ratio_rounding_cover_subnormals_midpoints_and_carries() {
        assert_eq!(
            decompose_nonzero(f64::from_bits(3), None, "test").unwrap(),
            (3, -1074)
        );

        // k0 >= 0 with num < (den << k0) vs num >= (den << k0), and k0 < 0.
        let t1 = round_extended_term(&[3.0], &[2.0], 0, false, 0, "test").unwrap();
        assert_eq!(
            f64::from_bits(
                round_wide_ratio_to_f64(
                    TermWide::from_u128(u128::from(t1.significand)),
                    TermWide::from_u128(1),
                    t1.exponent
                )
                .unwrap()
            ),
            1.5
        );
        let t2 = round_extended_term(&[2.0], &[3.0], 0, true, 0, "test").unwrap();
        assert!(t2.negative);
        let t3 = round_extended_term(&[5.0], &[7.0], 0, false, 0, "test").unwrap();
        assert!(!t3.negative);
        let t4 = round_extended_term(&[7.0], &[5.0], 0, false, 0, "test").unwrap();
        assert_eq!(
            f64::from_bits(
                round_wide_ratio_to_f64(
                    TermWide::from_u128(u128::from(t4.significand)),
                    TermWide::from_u128(1),
                    t4.exponent
                )
                .unwrap()
            ),
            1.4
        );

        // Carry across 2^53 boundary in round_extended_term:
        // (2^27 - 1) * (2^27 + 1) = 2^54 - 1 (54 bits, k = 53, shift = -1, scaled_den = 2),
        // quotient = 2^53 - 1, remainder = 1 == complement -> rounds up to 2^53 -> carry!
        let f_minus = f64::from_bits((1_u64 << 27) - 1);
        let f_plus = f64::from_bits((1_u64 << 27) + 1);
        let carry_term = round_extended_term(&[f_minus, f_plus], &[], 0, false, 0, "test").unwrap();
        assert_eq!(carry_term.significand, 1_u64 << 52);

        // k0 < 0 in round_extended_term (both at_least_k0 = true and false).
        let k_neg_true = round_extended_term(
            &[f64::from_bits(2)],
            &[f64::from_bits(4)],
            0,
            false,
            0,
            "test",
        )
        .unwrap();
        assert_eq!(k_neg_true.significand, 1_u64 << 52);
        let k_neg_false = round_extended_term(
            &[f64::from_bits(1)],
            &[f64::from_bits(3)],
            0,
            false,
            0,
            "test",
        )
        .unwrap();
        assert!(k_neg_false.significand > (1_u64 << 52));

        assert_eq!(
            round_wide_ratio_to_f64(TermWide::ZERO, TermWide::from_u128(1), 0),
            Some(0)
        );
        assert_eq!(
            round_wide_ratio_to_f64(TermWide::from_u128(2), TermWide::from_u128(4), 0).unwrap(),
            0.5_f64.to_bits()
        );
        assert_eq!(
            round_wide_ratio_to_f64(TermWide::from_u128(1), TermWide::from_u128(3), 0).unwrap(),
            (1.0_f64 / 3.0_f64).to_bits()
        );

        let num_carry = TermWide::from_u128((1_u128 << 54) - 1);
        let den_two = TermWide::from_u128(2);
        assert_eq!(
            round_wide_ratio_to_f64(num_carry, den_two, -52).unwrap(),
            2.0_f64.to_bits()
        );
        // Carry past 1023 overflows to INFINITY_BITS.
        assert_eq!(
            round_wide_ratio_to_f64(num_carry, den_two, 1023 - 52).unwrap(),
            INFINITY_BITS
        );
        // Top > 1023 and top < -1075 fast paths.
        assert_eq!(
            round_wide_ratio_to_f64(TermWide::from_u128(1), TermWide::from_u128(1), 1024).unwrap(),
            INFINITY_BITS
        );
        assert_eq!(
            round_wide_ratio_to_f64(TermWide::from_u128(1), TermWide::from_u128(1), -1076).unwrap(),
            0
        );
        // Subnormal midpoint tie-to-even: 1.5 * 2^-1074 -> 2 * 2^-1074, 2.5 * 2^-1074 -> 2 * 2^-1074.
        assert_eq!(
            round_wide_ratio_to_f64(TermWide::from_u128(3), TermWide::from_u128(2), -1074).unwrap(),
            2
        );
        assert_eq!(
            round_wide_ratio_to_f64(TermWide::from_u128(5), TermWide::from_u128(2), -1074).unwrap(),
            2
        );
    }

    #[test]
    fn risk_accumulation_and_getters_report_underflow_overflow_and_zero_dirty_price() {
        let mut pos = RiskWide::ZERO;
        let mut neg = RiskWide::ZERO;
        // Obtain every retained sum from admitted finite operands. MIN_SUB/2
        // is a valid extended term even though its raw binary64 view is zero.
        let tiny_term = round_extended_term(
            &[f64::from_bits(1), 0.5],
            &[],
            0,
            true,
            0,
            "price derivative",
        )
        .unwrap();
        accumulate_risk_term(&mut pos, &mut neg, tiny_term, 0, "price derivative").unwrap();
        let tiny = subtract_risk_sums(pos, neg, "price derivative").unwrap();
        assert_eq!(
            round_raw_risk(tiny, "price derivative").unwrap_err().kind(),
            "bond.value"
        );

        let mut pos = RiskWide::ZERO;
        let mut neg = RiskWide::ZERO;
        let huge_term = round_extended_term(
            &[f64::from_bits(2046_u64 << 52), 2.0],
            &[],
            0,
            false,
            0,
            "price second derivative",
        )
        .unwrap();
        accumulate_risk_term(&mut pos, &mut neg, huge_term, 0, "price second derivative").unwrap();
        let huge = subtract_risk_sums(pos, neg, "price second derivative").unwrap();
        assert_eq!(
            round_raw_risk(huge, "price second derivative")
                .unwrap_err()
                .kind(),
            "bond.value"
        );

        assert_eq!(
            round_normalized_risk(tiny, true, 0.0, 0.05, "modified duration")
                .unwrap_err()
                .kind(),
            "bond.zero_dirty_price"
        );
        assert_eq!(
            round_normalized_risk(
                SignedExtendedSum {
                    negative: false,
                    words: RiskWide::ZERO
                },
                false,
                -100.0,
                0.05,
                "convexity"
            )
            .unwrap(),
            0.0
        );
    }

    #[test]
    fn negative_zero_horizon_constant_risk_dirty_overflow_and_debug() {
        use crate::bonds::{Bond, BondPricingPlan, BondYieldConvention};
        use crate::calendar::{BusinessDayConvention, Calendar, EndOfMonth};
        use crate::cashflows::{FixedCouponRate, PaymentDateInclusion, SimpleCashFlow};
        use crate::daycount::DayCount;
        use crate::rates::AccrualConvention;
        use crate::schedule::{DateGenerationRule, ScheduleBuilder, ScheduleCalendar, StubPolicy};
        use crate::time::{Date, Period, TimeUnit};

        let issue = Date::from_ymd(2024, 1, 1).unwrap();
        let mat = Date::from_ymd(2025, 1, 1).unwrap();
        let sched = ScheduleBuilder::new(
            issue,
            mat,
            ScheduleCalendar::Plain(Calendar::null()),
            BusinessDayConvention::Unadjusted,
            BusinessDayConvention::Unadjusted,
        )
        .unwrap()
        .build_recurrent(
            Period::new(100, TimeUnit::Years),
            DateGenerationRule::Forward,
            StubPolicy::Short,
            EndOfMonth::Ignore,
            4,
        )
        .unwrap();
        let accrual = AccrualConvention::from(DayCount::Thirty360BondBasis);
        let conv = BondYieldConvention::new(accrual.clone(), Compounding::SIMPLE);

        // 1. Negative zero-horizon net constant (-150 coupon + 100 redemption = -50 at maturity Include).
        let neg_cpn_bond = Bond::fixed_rate(
            issue,
            sched.clone(),
            100.0,
            FixedCouponRate::new(-1.5, accrual.clone()).unwrap(),
            vec![None],
            SimpleCashFlow::new(mat, 100.0).unwrap(),
            8,
        )
        .unwrap();
        let mat_plan = BondPricingPlan::new(
            &neg_cpn_bond,
            BondSettlement::new(mat, PaymentDateInclusion::Include),
            &conv,
            16,
        )
        .unwrap();
        let err_neg_const = mat_plan
            .yield_from_price(
                BondPrice::new(-50.0, BondPriceKind::Dirty).unwrap(),
                RootBracket::new(-0.05, 0.20).unwrap(),
                SolverOptions::new(1e-8, 1e-8, 50).unwrap(),
            )
            .unwrap_err();
        assert!(matches!(
            err_neg_const,
            BondYieldError::NoYieldSensitivity {
                constant_amount: Some(-50.0),
                ..
            }
        ));

        // 2. Risk dirty price reduction overflow (f64::MAX coupon + f64::MAX redemption at y = 0.0).
        let max_bond = Bond::fixed_rate(
            issue,
            sched,
            100.0,
            FixedCouponRate::new(f64::MAX / 100.0, accrual).unwrap(),
            vec![None],
            SimpleCashFlow::new(mat, f64::MAX).unwrap(),
            8,
        )
        .unwrap();
        let max_plan = BondPricingPlan::new(
            &max_bond,
            BondSettlement::new(issue, PaymentDateInclusion::Exclude),
            &conv,
            16,
        )
        .unwrap();
        let sum_err = max_plan.risk(0.0).unwrap_err();
        assert_eq!(sum_err.kind(), "bond.sum");
        assert!(matches!(
            sum_err,
            BondError::Sum {
                phase: "dirty price reduction",
                ..
            }
        ));

        // 3. Debug formatting of a publicly constructed BondRisk.
        let zc =
            Bond::zero_coupon(issue, mat, 100.0, SimpleCashFlow::new(mat, 100.0).unwrap()).unwrap();
        let zc_plan = BondPricingPlan::new(
            &zc,
            BondSettlement::new(issue, PaymentDateInclusion::Exclude),
            &conv,
            16,
        )
        .unwrap();
        let risk = zc_plan.risk(0.05).unwrap();
        assert!(format!("{risk:?}").contains("BondRisk"));
    }
}
