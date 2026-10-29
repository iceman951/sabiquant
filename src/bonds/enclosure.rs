//! Explicit budgets and controlled evidence shapes for mathematical bond prices.
//!
//! The local four-model contract fixes represented/source targets, per100 units,
//! exact admission, proof/encoding semantics and bounded work. Producer methods
//! are installed separately; these opaque shapes cannot turn caller-supplied
//! observations or settlement evidence into a proved price.
//!
//! The contract is documented in
//! [the enclosure contract](https://github.com/iceman951/sabiquant/blob/main/docs/M8_PRICE_ENCLOSURE_CONTRACT.md).
//! These are SabiQuant evidence-boundary types, not a QuantLib C++ binding.

use crate::amounts::DecimalAmount;
use crate::bonds::{
    BondPriceKind, BondPricingPlan, CanadianPriceBasis, CanadianPricingPlan, GiltPricingPlan,
    TreasuryPricingPlan,
};
use crate::error::ValueError;
use crate::settlement::SettlementEvidence;
use std::{error::Error, fmt};

mod arithmetic;
mod engine;
mod generic;
mod gilt;

// Adopted configuration minimum; actual producer conformance includes all
// admission, arithmetic, initialization and reserved terminal debits.
const MINIMUM_WORK: u64 = 53_680;

/// Explicit fixed-precision/work configuration, with no defaults.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EnclosureBudget {
    precision_bits: u16,
    max_work: u64,
}
impl EnclosureBudget {
    /// Creates an explicit fixed-precision and total-work configuration.
    ///
    /// # Errors
    /// Reports original attempted fields and work0 for unsupported precision or
    /// work below 53,680. This constructor creates no numerical arena.
    pub fn new(precision_bits: u16, max_work: u64) -> Result<Self, EnclosureBudgetError> {
        let kind = if ![64, 128, 256, 512, 1024].contains(&precision_bits) {
            Some(BudgetFailure::UnsupportedPrecision)
        } else if max_work < MINIMUM_WORK {
            Some(BudgetFailure::BelowMinimum)
        } else {
            None
        };
        match kind {
            Some(kind) => Err(EnclosureBudgetError {
                precision_bits,
                max_work,
                kind,
            }),
            None => Ok(Self {
                precision_bits,
                max_work,
            }),
        }
    }
    /// Returns original precision in bits.
    pub const fn precision_bits(self) -> u16 {
        self.precision_bits
    }
    /// Returns original total work allowance.
    pub const fn max_work(self) -> u64 {
        self.max_work
    }
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum BudgetFailure {
    UnsupportedPrecision,
    BelowMinimum,
}
/// Configuration error outside a producer call, retaining both attempted fields.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EnclosureBudgetError {
    precision_bits: u16,
    max_work: u64,
    kind: BudgetFailure,
}
impl EnclosureBudgetError {
    /// Returns attempted precision.
    pub const fn precision_bits(self) -> u16 {
        self.precision_bits
    }
    /// Returns attempted total work.
    pub const fn max_work(self) -> u64 {
        self.max_work
    }
    /// Configuration validation consumes no producer work.
    pub const fn work_used(self) -> u64 {
        0
    }
    /// Stable configuration-error classification.
    pub const fn kind(self) -> &'static str {
        match self.kind {
            BudgetFailure::UnsupportedPrecision => "enclosure_budget.precision",
            BudgetFailure::BelowMinimum => "enclosure_budget.minimum_work",
        }
    }
}
impl fmt::Display for EnclosureBudgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "enclosure budget: {} (p={}, max_work={})",
            self.kind(),
            self.precision_bits,
            self.max_work
        )
    }
}
impl Error for EnclosureBudgetError {}

/// Existing model and requested basis. All outputs mean price per100 face.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum EnclosureBasis {
    /// Generic prepared-bit mathematical model.
    Generic(BondPriceKind),
    /// Exact selected conventional-gilt source model.
    Gilt(BondPriceKind),
    /// Exact raw analytic Treasury coefficients and selected published AI.
    Treasury(BondPriceKind),
    /// Exact Canadian source model and its three distinct bases.
    Canadian(CanadianPriceBasis),
}
/// The stage of a bounded mathematical price operation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum EnclosureStage {
    /// Explicit working-storage placement and initialization.
    Initialization,
    /// Quote/global admission.
    QuoteAdmission,
    /// Original-order flow-domain admission.
    FlowAdmission,
    /// Exact retained source conversion.
    Source,
    /// Bounded log/exp or rational arithmetic.
    Kernel,
    /// Complete signed price reduction.
    Reduction,
    /// Source AI, clean and per100 unit adjustment.
    BasisAndUnit,
    /// Exact-point recognition or outward finite export.
    Export,
    /// Compact completion/return staging under reserved terminal work.
    Completion,
}
/// Shared compact call context, immutable and inspectable in every call outcome.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EnclosureContext {
    budget: EnclosureBudget,
    work_used: u64,
    admission_checked_flows: usize,
    reduction_completed_flows: usize,
    next_original_index: Option<usize>,
    yield_bits: u64,
    basis: EnclosureBasis,
    stage: EnclosureStage,
    admission_complete: bool,
}
impl EnclosureContext {
    /// Returns original validated configuration.
    pub const fn budget(self) -> EnclosureBudget {
        self.budget
    }
    /// Returns actually charged work, never an estimate or callback count.
    pub const fn work_used(self) -> u64 {
        self.work_used
    }
    /// Returns completed exact per-flow checks, including a completed rejection.
    /// This count is independent of reduction and does not imply admission.
    pub const fn admission_checked_flows(self) -> usize {
        self.admission_checked_flows
    }
    /// Returns completed ORIGINAL price-flow commits, never admission checks or extra AI/par steps.
    /// A split terminal coupon and principal commit the original flow only once.
    pub const fn reduction_completed_flows(self) -> usize {
        self.reduction_completed_flows
    }
    /// Returns the next unchecked or current failing/uncommitted original index.
    /// A domain rejection retains the rejecting index even though its check is
    /// counted. Global setup and final basis/export stages use None.
    pub const fn next_original_index(self) -> Option<usize> {
        self.next_original_index
    }
    /// Returns original attempted quote bits in EVERY error.
    /// Only finite and unresolved successful outcomes canonicalize signed zero.
    pub const fn yield_bits(self) -> u64 {
        self.yield_bits
    }
    /// Returns selected model and basis.
    pub const fn basis(self) -> EnclosureBasis {
        self.basis
    }
    /// Returns exact execution stage.
    pub const fn stage(self) -> EnclosureStage {
        self.stage
    }
    /// Says whether all required exact domain checks completed successfully.
    pub const fn admission_complete(self) -> bool {
        self.admission_complete
    }
}

#[derive(Clone, Copy)]
#[allow(
    dead_code,
    reason = "E3 installs controlled producer construction after the E2 reference gate"
)]
enum OriginKind<'p, 's: 'p> {
    Generic(&'p BondPricingPlan<'s>),
    Gilt(&'p GiltPricingPlan<'s>),
    Treasury(&'p TreasuryPricingPlan<'s>),
    Canadian(&'p CanadianPricingPlan<'s>),
}
/// Opaque immutable origin borrow. Borrowing an origin establishes no price proof.
#[derive(Clone, Copy)]
pub struct EnclosureOrigin<'p, 's: 'p> {
    inner: OriginKind<'p, 's>,
}
impl<'p, 's: 'p> EnclosureOrigin<'p, 's> {
    /// Borrows a real generic plan, without producing evidence or a certificate.
    #[allow(
        dead_code,
        reason = "The corresponding E3 producer constructs its own immutable origin"
    )]
    fn generic(plan: &'p BondPricingPlan<'s>) -> Self {
        Self {
            inner: OriginKind::Generic(plan),
        }
    }
    /// Borrows a real gilt plan only.
    #[allow(
        dead_code,
        reason = "The corresponding E3 producer constructs its own immutable origin"
    )]
    fn gilt(plan: &'p GiltPricingPlan<'s>) -> Self {
        Self {
            inner: OriginKind::Gilt(plan),
        }
    }
    /// Borrows a real Treasury plan only.
    #[allow(
        dead_code,
        reason = "The corresponding E3 producer constructs its own immutable origin"
    )]
    fn treasury(plan: &'p TreasuryPricingPlan<'s>) -> Self {
        Self {
            inner: OriginKind::Treasury(plan),
        }
    }
    /// Borrows a real Canadian plan only.
    #[allow(
        dead_code,
        reason = "The corresponding E3 producer constructs its own immutable origin"
    )]
    fn canadian(plan: &'p CanadianPricingPlan<'s>) -> Self {
        Self {
            inner: OriginKind::Canadian(plan),
        }
    }
    /// Returns the real generic origin when applicable.
    pub const fn generic_plan(self) -> Option<&'p BondPricingPlan<'s>> {
        match self.inner {
            OriginKind::Generic(p) => Some(p),
            _ => None,
        }
    }
    /// Returns the real gilt origin when applicable.
    pub const fn gilt_plan(self) -> Option<&'p GiltPricingPlan<'s>> {
        match self.inner {
            OriginKind::Gilt(p) => Some(p),
            _ => None,
        }
    }
    /// Returns the real Treasury origin when applicable.
    pub const fn treasury_plan(self) -> Option<&'p TreasuryPricingPlan<'s>> {
        match self.inner {
            OriginKind::Treasury(p) => Some(p),
            _ => None,
        }
    }
    /// Returns the real Canadian origin when applicable.
    pub const fn canadian_plan(self) -> Option<&'p CanadianPricingPlan<'s>> {
        match self.inner {
            OriginKind::Canadian(p) => Some(p),
            _ => None,
        }
    }
}
impl fmt::Debug for EnclosureOrigin<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let model = match self.inner {
            OriginKind::Generic(_) => "Generic",
            OriginKind::Gilt(_) => "Gilt",
            OriginKind::Treasury(_) => "Treasury",
            OriginKind::Canadian(_) => "Canadian",
        };
        f.debug_tuple("EnclosureOrigin").field(&model).finish()
    }
}

/// Representation, separately inspected from the reason it is proved.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum EnclosureEncoding {
    /// A proved canonical decimal point, not falsely described as dyadic.
    ExactDecimalPoint,
    /// A proved finite binary point.
    ExactBinary64Point,
    /// A rigorously outward finite binary interval with strict lower < upper.
    /// Known points use a preferred exact point encoding when one fits.
    OutwardBinary64Bounds,
}
/// Implementation-owned mathematical reason; enum values alone create no proof.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum EnclosureProofKind {
    /// Complete exact rational identity established within the bounded route.
    ExactRationalIdentity,
    /// Complete FINAL price bounds, after basis and units, coincide in extended
    /// space or at finite binary64 export; prefix equality does not qualify.
    /// Known-point fitting still selects Decimal then Binary; NoFittingPoint may
    /// retain strict outward binary bounds with this distinct proof fact.
    IntervalCollapse,
    /// Directed interval arithmetic/series; not native observed equality.
    DirectedArithmetic,
}
/// Controlled finite mathematical price evidence, constructed only by a producer.
/// Clone retains the same origin borrow; Copy, Eq and Hash are intentionally absent.
#[derive(Clone)]
pub struct PriceEnclosure<'p, 's: 'p> {
    origin: EnclosureOrigin<'p, 's>,
    context: EnclosureContext,
    evidence: SettlementEvidence,
    encoding: EnclosureEncoding,
    proof_kind: EnclosureProofKind,
}
impl<'p, 's: 'p> PriceEnclosure<'p, 's> {
    /// Returns the exact originating immutable plan borrow.
    pub const fn origin(&self) -> EnclosureOrigin<'p, 's> {
        self.origin
    }
    /// Returns model/basis/quote/budget/work/admission context.
    pub const fn context(&self) -> EnclosureContext {
        self.context
    }
    /// Copies the DECLARED evidence established by the originating producer.
    /// Existing cash factories do not acquire an origin binding from this copy.
    pub const fn settlement_evidence(&self) -> SettlementEvidence {
        self.evidence
    }
    /// Returns endpoint representation, distinct from proof kind.
    pub const fn encoding(&self) -> EnclosureEncoding {
        self.encoding
    }
    /// Returns the closed mathematical proof-path fact.
    pub const fn proof_kind(&self) -> EnclosureProofKind {
        self.proof_kind
    }
    /// Inspects a decimal point only when that representation was proved.
    pub fn decimal_point(&self) -> Option<DecimalAmount> {
        if self.encoding != EnclosureEncoding::ExactDecimalPoint {
            return None;
        }
        match self.evidence.decimal_bounds() {
            Some((a, b)) if a == b => Some(a),
            _ => None,
        }
    }
}
impl fmt::Debug for PriceEnclosure<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PriceEnclosure")
            .field("origin", &self.origin)
            .field("context", &self.context)
            .field("encoding", &self.encoding)
            .field("proof_kind", &self.proof_kind)
            .finish_non_exhaustive()
    }
}
/// Valid-domain unresolved result; no partial bound or source array is exposed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum EnclosureUnresolvedReason {
    /// Budget stopped arithmetic AFTER all exact admission finished.
    WorkLimitAfterAdmission,
    /// Complete extended arithmetic could not yield finite exported bounds.
    FiniteEndpointUnavailable,
}
/// Opaque unresolved output; no conversion to evidence or cash is provided.
#[derive(Clone, Debug)]
pub struct PriceEnclosureUnresolved<'p, 's: 'p> {
    origin: EnclosureOrigin<'p, 's>,
    context: EnclosureContext,
    reason: EnclosureUnresolvedReason,
    complete_extended_arithmetic: bool,
}
impl<'p, 's: 'p> PriceEnclosureUnresolved<'p, 's> {
    /// Returns original immutable plan borrow.
    pub const fn origin(&self) -> EnclosureOrigin<'p, 's> {
        self.origin
    }
    /// Returns full work/admission/model context.
    pub const fn context(&self) -> EnclosureContext {
        self.context
    }
    /// Returns the honest reason that finite proof export did not complete.
    pub const fn reason(&self) -> EnclosureUnresolvedReason {
        self.reason
    }
    /// Distinguishes completed extended arithmetic from an arithmetic work stop.
    pub const fn complete_extended_arithmetic(&self) -> bool {
        self.complete_extended_arithmetic
    }
}
/// Complete admitted-input outcome partition.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum EnclosureOutcome<'p, 's: 'p> {
    /// Complete exact admission and a controlled finite proof.
    Finite(PriceEnclosure<'p, 's>),
    /// Complete exact admission but no finite exported proof.
    Unresolved(PriceEnclosureUnresolved<'p, 's>),
}

/// Compact exact mathematical domain cause; no invented native range gate.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum EnclosureDomainCause {
    /// y must exceed minus the explicit periodic frequency, globally.
    YieldNotAboveFloor {
        /// Explicit positive annual event count.
        payments_per_year: u16,
    },
    /// Active generic Simple denominator1+y*t is not strictly positive.
    NonpositiveSimpleDenominator {
        /// Exact prepared binary64 horizon bits.
        horizon_bits: u64,
    },
    /// Canadian MM denominator1+y*(T-S)/365 is not strictly positive.
    NonpositiveMoneyMarketDenominator {
        /// Original positive S-to-true-T days.
        days_to_true_maturity: u32,
    },
}

impl EnclosureDomainCause {
    /// Returns the stable mathematical-domain classification.
    #[must_use]
    pub const fn kind(self) -> &'static str {
        match self {
            Self::YieldNotAboveFloor { .. } => "enclosure.yield_not_above_floor",
            Self::NonpositiveSimpleDenominator { .. } => "enclosure.nonpositive_simple_denominator",
            Self::NonpositiveMoneyMarketDenominator { .. } => {
                "enclosure.nonpositive_money_market_denominator"
            }
        }
    }
}

impl fmt::Display for EnclosureDomainCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exact enclosure domain: {self:?}")
    }
}
impl Error for EnclosureDomainCause {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "E3 installs controlled producer construction after the E2 reference gate"
)]
enum Operation {
    Source,
    ExactFold,
    PointEncoding,
    Initialization,
    Progress,
    Normalize,
    Add,
    Multiply,
    Divide,
    Log,
    Exp,
    Export,
}
#[derive(Clone, Debug)]
#[allow(
    dead_code,
    reason = "E3 installs controlled producer construction after the E2 reference gate"
)]
enum CallFailure {
    NonFiniteYield {
        context: EnclosureContext,
        cause: ValueError,
    },
    Domain {
        context: EnclosureContext,
        cause: EnclosureDomainCause,
    },
    AdmissionIncomplete {
        context: EnclosureContext,
    },
    Allocation {
        context: EnclosureContext,
        requested_bytes: usize,
        cause: std::collections::TryReserveError,
    },
    Invariant {
        context: EnclosureContext,
        operation: Operation,
    },
}

impl PartialEq for CallFailure {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::NonFiniteYield {
                    context: a,
                    cause: ac,
                },
                Self::NonFiniteYield {
                    context: b,
                    cause: bc,
                },
            ) => a == b && ac == bc,
            (
                Self::Domain {
                    context: a,
                    cause: ac,
                },
                Self::Domain {
                    context: b,
                    cause: bc,
                },
            ) => a == b && ac == bc,
            (
                Self::AdmissionIncomplete { context: a },
                Self::AdmissionIncomplete { context: b },
            ) => a == b,
            (
                Self::Allocation {
                    context: a,
                    requested_bytes: ar,
                    ..
                },
                Self::Allocation {
                    context: b,
                    requested_bytes: br,
                    ..
                },
            ) => a == b && ar == br,
            (
                Self::Invariant {
                    context: a,
                    operation: ao,
                },
                Self::Invariant {
                    context: b,
                    operation: bo,
                },
            ) => a == b && ao == bo,
            _ => false,
        }
    }
}

/// Compact call error shape; no constructor or From discards required context.
#[derive(Clone, Debug, PartialEq)]
pub struct EnclosureError {
    kind: CallFailure,
}
impl EnclosureError {
    /// Returns all applicable model/quote/budget/stage/index/work facts.
    pub const fn context(&self) -> EnclosureContext {
        match &self.kind {
            CallFailure::NonFiniteYield { context, .. }
            | CallFailure::Domain { context, .. }
            | CallFailure::AdmissionIncomplete { context }
            | CallFailure::Allocation { context, .. }
            | CallFailure::Invariant { context, .. } => *context,
        }
    }
    /// Returns exact mathematical-domain cause when applicable.
    pub const fn domain_cause(&self) -> Option<EnclosureDomainCause> {
        match &self.kind {
            CallFailure::Domain { cause, .. } => Some(*cause),
            _ => None,
        }
    }
    /// Returns the requested bytes on the fallible arena route.
    pub const fn requested_bytes(&self) -> Option<usize> {
        match &self.kind {
            CallFailure::Allocation {
                requested_bytes, ..
            } => Some(*requested_bytes),
            _ => None,
        }
    }
    /// Returns the exact private arithmetic operation on an invariant failure.
    pub const fn operation(&self) -> Option<&'static str> {
        match &self.kind {
            CallFailure::Invariant { operation, .. } => Some(match operation {
                Operation::Source => "source",
                Operation::ExactFold => "exact_fold",
                Operation::PointEncoding => "point_encoding",
                Operation::Initialization => "initialization",
                Operation::Progress => "progress",
                Operation::Normalize => "normalize",
                Operation::Add => "add",
                Operation::Multiply => "multiply",
                Operation::Divide => "divide",
                Operation::Log => "log",
                Operation::Exp => "exp",
                Operation::Export => "export",
            }),
            _ => None,
        }
    }
    /// Stable error class, preserving resource versus invalid-yield distinctions.
    pub const fn kind(&self) -> &'static str {
        match &self.kind {
            CallFailure::NonFiniteYield { .. } => "enclosure.non_finite_yield",
            CallFailure::Domain { cause, .. } => match cause {
                EnclosureDomainCause::YieldNotAboveFloor { .. } => {
                    "enclosure.yield_not_above_floor"
                }
                EnclosureDomainCause::NonpositiveSimpleDenominator { .. } => {
                    "enclosure.nonpositive_simple_denominator"
                }
                EnclosureDomainCause::NonpositiveMoneyMarketDenominator { .. } => {
                    "enclosure.nonpositive_money_market_denominator"
                }
            },
            CallFailure::AdmissionIncomplete { .. } => "enclosure.admission_incomplete",
            CallFailure::Allocation { .. } => "enclosure.allocation_failed",
            CallFailure::Invariant { .. } => "enclosure.internal_capacity",
        }
    }
}
impl fmt::Display for EnclosureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "enclosure error: {} ({:?})", self.kind(), self.context())
    }
}
impl Error for EnclosureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            CallFailure::NonFiniteYield { cause, .. } => Some(cause),
            CallFailure::Domain { cause, .. } => Some(cause),
            CallFailure::Allocation { cause, .. } => Some(cause),
            _ => None,
        }
    }
}

/// The controlled finite-or-unresolved result of a mathematical price operation.
pub type EnclosureResult<'p, 's> = Result<EnclosureOutcome<'p, 's>, EnclosureError>;

#[cfg(test)]
pub(crate) fn error_samples() -> Vec<(crate::error::Error, &'static str)> {
    metadata_controls::error_samples()
}

#[cfg(test)]
#[path = "enclosure/metadata_controls.rs"]
mod metadata_controls;
