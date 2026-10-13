//! Immutable analytical cash flows, plain fixed coupons and flat-rate valuation.
//!
//! Amounts are finite signed binary64 values in one caller-selected consistent
//! unit. They do not represent exact decimal settlement or currency rounding.
//! Plain coupons use `notional × decimal annual rate × raw accrual fraction`,
//! independently of the discount quote's compounding. Every date, inclusion,
//! convention and contextual work budget is supplied explicitly.
//!
//! | SabiQuant operation | QuantLib public reference concept |
//! |---|---|
//! | [`SimpleCashFlow`] | `SimpleCashFlow` amount and payment date |
//! | [`FixedRateCoupon`] | Plain fixed coupon with separate raw accrual/payment dates |
//! | [`Leg`] | Ordered cash-flow sequence, retaining duplicate payment dates |
//! | [`Leg::net_present_value`] | Constant-rate cash-flow NPV with explicit reference date |
//! | [`Leg::accrued_amount`] | Explicitly defined outstanding coupon accrual |
//!
//! These generic analytics do not implement bond settlement or ex-coupon
//! entitlement. The adopted contract and reference differences are recorded in
//! [the cash-flow contract](https://github.com/iceman951/sabiquant/blob/main/docs/M7_CASHFLOW_CONTRACT.md).
//! A private integer product rounds the three supplied coupon factors once;
//! reductions use [`exact_sum`], which sums rounded terms, not exact products.
//! Shared immutable convention ownership avoids copying a large ICMA context
//! into every coupon. Arc construction and ordinary Clone have standard system
//! allocation behavior; growing computation vectors reserve fallibly.

use std::error::Error as StdError;
use std::fmt;
use std::sync::Arc;

use crate::error::{Interval, ValueError};
use crate::math::{SumError, exact_sum};
use crate::numeric;
use crate::rates::{AccrualConvention, InterestRate, RateError};
use crate::schedule::Schedule;
use crate::time::Date;

mod product;

/// Whether an amount payable exactly on the cutoff date remains eligible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "cutoff equality has exactly two choices"
)]
pub enum PaymentDateInclusion {
    /// Retain payments on the explicitly supplied cutoff date.
    Include,
    /// Exclude payments on the explicitly supplied cutoff date.
    Exclude,
}

impl PaymentDateInclusion {
    /// Both explicit choices; there is no financial default.
    pub const ALL: [Self; 2] = [Self::Include, Self::Exclude];

    fn eligible(self, payment: Date, cutoff: Date) -> bool {
        payment > cutoff || (payment == cutoff && self == Self::Include)
    }
}

/// A predetermined finite signed amount and its explicit payment date.
///
/// Definition equality compares date and canonicalized amount; no currency or
/// calendar is inferred. Values containing binary64 implement PartialEq only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimpleCashFlow {
    payment_date: Date,
    amount: f64,
}

impl SimpleCashFlow {
    /// Constructs a flow, canonicalizing either input zero sign to positive zero.
    ///
    /// # Errors
    /// [`CashFlowError::Value`] if the amount is not finite.
    pub fn new(payment_date: Date, amount: f64) -> Result<Self, CashFlowError> {
        let amount = numeric::finite("amount", amount)
            .map_err(|cause| value_error(None, "simple amount", cause))?;
        Ok(Self {
            payment_date,
            amount: canonical_zero(amount),
        })
    }

    /// Returns the supplied payment date without calendar adjustment.
    #[must_use]
    pub const fn payment_date(self) -> Date {
        self.payment_date
    }

    /// Returns the stored finite signed amount.
    #[must_use]
    pub const fn amount(self) -> f64 {
        self.amount
    }
}

/// A finite decimal annual SIMPLE coupon rate with complete accrual context.
///
/// This quote is distinct from a compounded discount [`InterestRate`]. Clone
/// shares its immutable convention in O(1); equality compares the entire
/// convention's definition, including coverage and source calendar policy.
#[derive(Clone, Debug, PartialEq)]
pub struct FixedCouponRate {
    rate: f64,
    accrual: Arc<AccrualConvention>,
}

impl FixedCouponRate {
    /// Creates a simple-accrual coupon quote with no implicit frequency/context.
    ///
    /// One shared convention allocation follows standard Arc system-OOM
    /// behavior. It is not promised as a recoverable allocation error.
    ///
    /// # Errors
    /// [`CashFlowError::Value`] if the rate is not finite. Date coverage is
    /// checked when a coupon or actual partial accrual is constructed.
    pub fn new(rate: f64, accrual: impl Into<AccrualConvention>) -> Result<Self, CashFlowError> {
        let rate = numeric::finite("coupon rate", rate)
            .map_err(|cause| value_error(None, "coupon quote", cause))?;
        Ok(Self {
            rate: canonical_zero(rate),
            accrual: Arc::new(accrual.into()),
        })
    }

    /// Returns the finite annual decimal simple-accrual rate.
    #[must_use]
    pub const fn rate(&self) -> f64 {
        self.rate
    }

    /// Borrows the entire immutable convention without exposing pointer identity.
    #[must_use]
    pub fn accrual(&self) -> &AccrualConvention {
        &self.accrual
    }
}

/// An eager plain fixed coupon with distinct contractual accrual/payment dates.
///
/// Definition equality retains all terms and complete quote context, not just
/// the computed amount. Payment may be early or delayed; no market adjustment
/// or payment/start relationship is inferred by the manual constructor.
#[derive(Clone, Debug, PartialEq)]
pub struct FixedRateCoupon {
    start: Date,
    end: Date,
    payment_date: Date,
    notional: f64,
    coupon_rate: FixedCouponRate,
    accrual_fraction: f64,
    amount: f64,
}

impl FixedRateCoupon {
    /// Constructs `N*r*YF(start,end)` from a positive raw accrual interval.
    ///
    /// The complete convention query precedes any zero scalar shortcut. The
    /// three supplied binary64 factors are multiplied exactly then rounded once
    /// nearest/ties-even; this does not make the rounded year fraction exact.
    ///
    /// # Errors
    /// [`CashFlowError::InvalidAccrualPeriod`] for start>=end, then
    /// [`CashFlowError::Value`] for nonfinite notional, invalid fraction or
    /// unrepresentable nonzero amount; [`CashFlowError::Rate`] preserves any
    /// underlying accrual/coverage cause.
    pub fn new(
        start: Date,
        end: Date,
        payment_date: Date,
        notional: f64,
        coupon_rate: FixedCouponRate,
    ) -> Result<Self, CashFlowError> {
        if start >= end {
            return Err(CashFlowError::InvalidAccrualPeriod { start, end });
        }
        let notional = numeric::finite("notional", notional)
            .map_err(|cause| value_error(None, "coupon notional", cause))?;
        let accrual_fraction = checked_fraction(&coupon_rate, start, end, None, "coupon accrual")?;
        let amount = product::product(
            "coupon amount",
            notional,
            coupon_rate.rate,
            accrual_fraction,
        )
        .map_err(|cause| value_error(None, "coupon amount", cause))?;
        Ok(Self {
            start,
            end,
            payment_date,
            notional: canonical_zero(notional),
            coupon_rate,
            accrual_fraction,
            amount,
        })
    }

    /// Returns the inclusive raw contractual accrual start.
    #[must_use]
    pub const fn start(&self) -> Date {
        self.start
    }
    /// Returns the exclusive raw contractual accrual end.
    #[must_use]
    pub const fn end(&self) -> Date {
        self.end
    }
    /// Returns the explicit payment date, independently of raw accrual endpoints.
    #[must_use]
    pub const fn payment_date(&self) -> Date {
        self.payment_date
    }
    /// Returns the finite signed notional with canonical input zero.
    #[must_use]
    pub const fn notional(&self) -> f64 {
        self.notional
    }
    /// Borrows the original quote and its entire shared convention.
    #[must_use]
    pub const fn coupon_rate(&self) -> &FixedCouponRate {
        &self.coupon_rate
    }
    /// Returns the validated finite nonnegative full raw year fraction.
    #[must_use]
    pub const fn accrual_fraction(&self) -> f64 {
        self.accrual_fraction
    }
    /// Returns the eagerly calculated finite signed coupon amount.
    #[must_use]
    pub const fn amount(&self) -> f64 {
        self.amount
    }

    /// Returns clipped earned interest for an eligible outstanding coupon.
    ///
    /// Ceased/before-start coupons return positive zero. A still-eligible
    /// completed coupon returns its cached full amount. A real partial interval
    /// uses the supplied convention rather than scaling the full coupon amount.
    /// At an early payment Include can therefore return less than the amount due.
    ///
    /// # Errors
    /// [`CashFlowError::Rate`] for a partial accrual query failure, or
    /// [`CashFlowError::Value`] for an invalid fraction/unrepresentable product.
    pub fn accrued_amount(
        &self,
        as_of: Date,
        inclusion: PaymentDateInclusion,
    ) -> Result<f64, CashFlowError> {
        self.accrued(as_of, inclusion, None, None)
    }

    fn accrued(
        &self,
        as_of: Date,
        inclusion: PaymentDateInclusion,
        index: Option<usize>,
        work: Option<&mut AccrualWork>,
    ) -> Result<f64, CashFlowError> {
        if !inclusion.eligible(self.payment_date, as_of) || as_of <= self.start {
            return Ok(0.0);
        }
        if as_of >= self.end {
            return Ok(self.amount);
        }
        if let Some(work) = work {
            work.charge(
                self.coupon_rate.accrual(),
                self.start,
                as_of,
                index,
                "partial accrual",
            )?;
        }
        let fraction = checked_fraction(
            &self.coupon_rate,
            self.start,
            as_of,
            index,
            "partial accrual",
        )?;
        product::product(
            "accrued amount",
            self.notional,
            self.coupon_rate.rate,
            fraction,
        )
        .map_err(|cause| value_error(index, "accrued amount", cause))
    }
}

/// An immutable supported cash-flow kind, retaining its concrete source terms.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CashFlow {
    /// A predetermined signed amount/date.
    Simple(SimpleCashFlow),
    /// A plain fixed coupon with explicit accrual context.
    FixedRateCoupon(FixedRateCoupon),
}

impl CashFlow {
    /// Returns the concrete flow's payment date.
    #[must_use]
    pub const fn payment_date(&self) -> Date {
        match self {
            Self::Simple(flow) => flow.payment_date,
            Self::FixedRateCoupon(flow) => flow.payment_date,
        }
    }
    /// Returns the stored undiscounted signed amount.
    #[must_use]
    pub const fn amount(&self) -> f64 {
        match self {
            Self::Simple(flow) => flow.amount,
            Self::FixedRateCoupon(flow) => flow.amount,
        }
    }
    /// Returns outstanding coupon accrual; simple flows contribute positive zero.
    ///
    /// # Errors
    /// The fixed coupon's partial accrual/product errors are preserved.
    pub fn accrued_amount(
        &self,
        as_of: Date,
        inclusion: PaymentDateInclusion,
    ) -> Result<f64, CashFlowError> {
        match self {
            Self::Simple(_) => Ok(0.0),
            Self::FixedRateCoupon(flow) => flow.accrued_amount(as_of, inclusion),
        }
    }
}

impl From<SimpleCashFlow> for CashFlow {
    fn from(flow: SimpleCashFlow) -> Self {
        Self::Simple(flow)
    }
}
impl From<FixedRateCoupon> for CashFlow {
    fn from(flow: FixedRateCoupon) -> Self {
        Self::FixedRateCoupon(flow)
    }
}

/// A bounded nondecreasing payment-date sequence, including empty/duplicate dates.
///
/// Equality compares retained flow definitions; an accepted resource cap is not
/// part of the value. No sorting, coalescing or implicit principal is performed.
///
/// ```
/// use sabiquant::cashflows::{Leg, PaymentDateInclusion, SimpleCashFlow};
/// use sabiquant::daycount::DayCount;
/// use sabiquant::rates::{Compounding, InterestRate};
/// use sabiquant::time::Date;
/// let valuation = Date::from_ymd(2026, 1, 1)?;
/// let payment = Date::from_ymd(2027, 1, 1)?;
/// let flow = SimpleCashFlow::new(payment, 100.0)?;
/// let leg = Leg::new(vec![flow.into()], 1)?;
/// let quote = InterestRate::new(0.0, DayCount::Actual365Fixed, Compounding::SIMPLE)?;
/// let npv = leg.net_present_value(valuation, &quote, PaymentDateInclusion::Exclude, 2)?;
/// assert_eq!(npv.to_bits(), 100.0_f64.to_bits());
/// # Ok::<(), sabiquant::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Leg {
    flows: Vec<CashFlow>,
}

impl Leg {
    /// Largest supported flow-count budget, a resource policy including duplicates.
    pub const MAX_FLOWS: u32 = 3_652_059;

    /// Retains supplied flows after checking cap/count and original payment order.
    ///
    /// # Errors
    /// Invalid cap, too many flows or the first adjacent payment-date reversal.
    pub fn new(flows: Vec<CashFlow>, max_flows: u32) -> Result<Self, CashFlowError> {
        check_flow_count(flows.len(), max_flows)?;
        for (index, pair) in flows.windows(2).enumerate() {
            let previous = pair[0].payment_date();
            let current = pair[1].payment_date();
            if previous > current {
                return Err(CashFlowError::PaymentOrder {
                    index: index + 1,
                    previous,
                    current,
                });
            }
        }
        Ok(Self { flows })
    }

    /// Builds one coupon per raw schedule interval, payable on its adjusted end.
    ///
    /// Quote clones share the complete accrual convention. This adds no lag,
    /// redemption, inferred frequency or adjusted-accrual substitution. Keep the
    /// source Schedule separately when its calendar provenance is needed.
    ///
    /// # Errors
    /// Cap/count errors precede coupon work. Allocation failures are typed;
    /// each coupon failure retains the original interval index when applicable.
    pub fn fixed_rate(
        schedule: &Schedule,
        notional: f64,
        coupon_rate: &FixedCouponRate,
        max_flows: u32,
    ) -> Result<Self, CashFlowError> {
        let raw = schedule.contractual_dates();
        let count = raw.len() - 1; // A valid Schedule always has at least two boundaries.
        check_flow_count(count, max_flows)?;
        let mut flows = reserved(count, "fixed-rate leg")?;
        for (index, pair) in raw.windows(2).enumerate() {
            let coupon = FixedRateCoupon::new(
                pair[0],
                pair[1],
                schedule.adjusted_dates()[index + 1],
                notional,
                coupon_rate.clone(),
            )
            .map_err(|error| error.at_index(index))?;
            flows.push(coupon.into());
        }
        Ok(Self { flows }) // Valid Schedule adjusted boundaries are strictly ordered.
    }

    /// Borrows the retained chronological payment sequence.
    #[must_use]
    pub fn flows(&self) -> &[CashFlow] {
        &self.flows
    }

    /// Discounts eligible flows directly from one explicit valuation date.
    ///
    /// The equal-date quote anchor is checked first, even for an empty leg.
    /// Included zero amounts validate their full factor/context. Each horizon
    /// uses existing factor-required rate semantics, without adjacent chaining.
    /// Rounded finite PV terms are reduced by exact_sum, preserving cancellation
    /// of overflowing prefixes but not rescuing an individually failed term.
    ///
    /// Each needed query charges 1 plus ICMA overlap periods, or 1 plus calendar
    /// span days for Business/252; equal queries cost 1. Every u64 budget is valid.
    /// Charges occur immediately before that query, in original flow order.
    /// Work is O(n+charged interval work), plus context searches/calendar costs;
    /// repeated ICMA prefixes can cost O(nM) when the caller permits that budget.
    ///
    /// # Errors
    /// Work-limit/allocation errors, indexed [`CashFlowError::Rate`] causes for
    /// anchor/horizon/factor/product failures, or final [`CashFlowError::Sum`].
    pub fn net_present_value(
        &self,
        valuation_date: Date,
        discount_rate: &InterestRate,
        inclusion: PaymentDateInclusion,
        max_accrual_work: u64,
    ) -> Result<f64, CashFlowError> {
        let mut work = AccrualWork {
            maximum: max_accrual_work,
            used: 0,
        };
        work.charge(
            discount_rate.accrual(),
            valuation_date,
            valuation_date,
            None,
            "valuation anchor",
        )?;
        discount_rate
            .discount_factor_between(valuation_date, valuation_date)
            .map_err(|cause| rate_error(None, "valuation anchor", cause))?;
        let first = self.first_eligible(valuation_date, inclusion);
        let mut terms = reserved(self.flows.len() - first, "NPV terms")?;
        for (index, flow) in self.flows.iter().enumerate().skip(first) {
            work.charge(
                discount_rate.accrual(),
                valuation_date,
                flow.payment_date(),
                Some(index),
                "discount horizon",
            )?;
            let pv = discount_rate
                .present_value_between(flow.amount(), valuation_date, flow.payment_date())
                .map_err(|cause| rate_error(Some(index), "discount horizon", cause))?;
            terms.push(pv);
        }
        exact_sum(&terms).map_err(|cause| CashFlowError::Sum {
            phase: "NPV reduction",
            cause,
        })
    }

    /// Sums clipped earned interest of all eligible outstanding fixed coupons.
    ///
    /// Simple, ceased, before-start and cached completed paths need no accrual
    /// query/work units. Required partial queries follow original flow order and
    /// use the same explicit charging rule as net_present_value. Rounded finite
    /// coupon results are reduced once with exact_sum and canonical zero.
    ///
    /// # Errors
    /// Work-limit/allocation, indexed partial accrual/product or final sum errors.
    pub fn accrued_amount(
        &self,
        as_of: Date,
        inclusion: PaymentDateInclusion,
        max_accrual_work: u64,
    ) -> Result<f64, CashFlowError> {
        let first = self.first_eligible(as_of, inclusion);
        let mut terms = reserved(self.flows.len() - first, "accrued terms")?;
        let mut work = AccrualWork {
            maximum: max_accrual_work,
            used: 0,
        };
        for (index, flow) in self.flows.iter().enumerate().skip(first) {
            if let CashFlow::FixedRateCoupon(coupon) = flow {
                terms.push(coupon.accrued(as_of, inclusion, Some(index), Some(&mut work))?);
            }
        }
        exact_sum(&terms).map_err(|cause| CashFlowError::Sum {
            phase: "accrued reduction",
            cause,
        })
    }

    fn first_eligible(&self, cutoff: Date, inclusion: PaymentDateInclusion) -> usize {
        self.flows
            .partition_point(|flow| !inclusion.eligible(flow.payment_date(), cutoff))
    }
}

/// A rejected cash-flow definition, resource request or analytical calculation.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CashFlowError {
    /// Raw coupon dates are equal/reversed; kind `cashflow.invalid_accrual_period`.
    #[non_exhaustive]
    InvalidAccrualPeriod {
        /// Requested raw start.
        start: Date,
        /// Requested raw end.
        end: Date,
    },
    /// Count budget exceeds the supported policy; kind `cashflow.invalid_flow_limit`.
    #[non_exhaustive]
    InvalidFlowLimit {
        /// Rejected caller cap.
        requested: u32,
        /// Largest supported cap.
        maximum: u32,
    },
    /// Known flow count exceeds caller cap; kind `cashflow.flow_limit_exceeded`.
    #[non_exhaustive]
    FlowLimitExceeded {
        /// Original caller cap.
        maximum: u32,
        /// Known final flow count.
        required: usize,
    },
    /// Original adjacent payment dates reverse; kind `cashflow.payment_order`.
    #[non_exhaustive]
    PaymentOrder {
        /// Zero-based position of the later input flow.
        index: usize,
        /// Preceding flow's payment date.
        previous: Date,
        /// Rejected current flow's payment date.
        current: Date,
    },
    /// A required vector reserve failed; kind `cashflow.allocation_failed`.
    #[non_exhaustive]
    AllocationFailed {
        /// Operation requiring storage.
        phase: &'static str,
        /// Requested final element capacity.
        requested: usize,
    },
    /// Next query exceeds per-call work cap; kind `cashflow.accrual_work_limit_exceeded`.
    #[non_exhaustive]
    AccrualWorkLimitExceeded {
        /// Original flow position, or None for the valuation anchor.
        index: Option<usize>,
        /// Query about to begin.
        phase: &'static str,
        /// Original caller work cap.
        maximum: u64,
        /// Units already charged for completed/attempted prior queries.
        used: u64,
        /// Cumulative units needed including the next query.
        required_at_least: u64,
    },
    /// Numeric input/result failure; kind `cashflow.value`.
    #[non_exhaustive]
    Value {
        /// Original flow position, or None for a standalone value.
        index: Option<usize>,
        /// Numeric operation.
        phase: &'static str,
        /// Preserved typed value cause.
        cause: ValueError,
    },
    /// Accrual/discount failure; kind `cashflow.rate`.
    #[non_exhaustive]
    Rate {
        /// Original flow position, or None for a standalone coupon/anchor.
        index: Option<usize>,
        /// Required query/factor operation.
        phase: &'static str,
        /// Preserved typed rate cause and its contextual source chain.
        cause: RateError,
    },
    /// Final rounded-term reduction failure; kind `cashflow.sum`.
    #[non_exhaustive]
    Sum {
        /// Reduction operation.
        phase: &'static str,
        /// Preserved typed summation cause.
        cause: SumError,
    },
}

impl CashFlowError {
    /// Returns the stable cash-flow domain kind.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::InvalidAccrualPeriod { .. } => "cashflow.invalid_accrual_period",
            Self::InvalidFlowLimit { .. } => "cashflow.invalid_flow_limit",
            Self::FlowLimitExceeded { .. } => "cashflow.flow_limit_exceeded",
            Self::PaymentOrder { .. } => "cashflow.payment_order",
            Self::AllocationFailed { .. } => "cashflow.allocation_failed",
            Self::AccrualWorkLimitExceeded { .. } => "cashflow.accrual_work_limit_exceeded",
            Self::Value { .. } => "cashflow.value",
            Self::Rate { .. } => "cashflow.rate",
            Self::Sum { .. } => "cashflow.sum",
        }
    }

    fn at_index(self, index: usize) -> Self {
        match self {
            Self::Value { phase, cause, .. } => Self::Value {
                index: Some(index),
                phase,
                cause,
            },
            Self::Rate { phase, cause, .. } => Self::Rate {
                index: Some(index),
                phase,
                cause,
            },
            other => other,
        }
    }
}

impl fmt::Display for CashFlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAccrualPeriod { start, end } => {
                write!(f, "coupon accrual requires start {start} before end {end}")
            }
            Self::InvalidFlowLimit { requested, maximum } => write!(
                f,
                "flow limit {requested} exceeds supported maximum {maximum}"
            ),
            Self::FlowLimitExceeded { maximum, required } => {
                write!(f, "flow count {required} exceeds caller limit {maximum}")
            }
            Self::PaymentOrder {
                index,
                previous,
                current,
            } => write!(
                f,
                "flow {index} payment {current} precedes prior payment {previous}"
            ),
            Self::AllocationFailed { phase, requested } => {
                write!(f, "{phase}: cannot reserve {requested} elements")
            }
            Self::AccrualWorkLimitExceeded {
                index,
                phase,
                maximum,
                used,
                required_at_least,
            } => write!(
                f,
                "{phase} at flow {index:?}: work limit {maximum}, used {used}, requires at least {required_at_least}"
            ),
            Self::Value {
                index,
                phase,
                cause,
            } => write!(f, "{phase} at flow {index:?}: {cause}"),
            Self::Rate {
                index,
                phase,
                cause,
            } => write!(f, "{phase} at flow {index:?}: {cause}"),
            Self::Sum { phase, cause } => write!(f, "{phase}: {cause}"),
        }
    }
}

impl StdError for CashFlowError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Value { cause, .. } => Some(cause),
            Self::Rate { cause, .. } => Some(cause),
            Self::Sum { cause, .. } => Some(cause),
            _ => None,
        }
    }
}

fn checked_fraction(
    quote: &FixedCouponRate,
    start: Date,
    end: Date,
    index: Option<usize>,
    phase: &'static str,
) -> Result<f64, CashFlowError> {
    let fraction = quote
        .accrual
        .year_fraction(start, end)
        .map_err(|cause| rate_error(index, phase, cause))?;
    numeric::result_within("accrual fraction", fraction, Interval::at_least(0.0))
        .map(canonical_zero)
        .map_err(|cause| value_error(index, phase, cause))
}

fn check_flow_count(count: usize, maximum: u32) -> Result<(), CashFlowError> {
    if maximum > Leg::MAX_FLOWS {
        return Err(CashFlowError::InvalidFlowLimit {
            requested: maximum,
            maximum: Leg::MAX_FLOWS,
        });
    }
    if count > maximum as usize {
        return Err(CashFlowError::FlowLimitExceeded {
            maximum,
            required: count,
        });
    }
    Ok(())
}

fn reserved<T>(count: usize, phase: &'static str) -> Result<Vec<T>, CashFlowError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| CashFlowError::AllocationFailed {
            phase,
            requested: count,
        })?;
    Ok(values)
}

fn canonical_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}
fn value_error(index: Option<usize>, phase: &'static str, cause: ValueError) -> CashFlowError {
    CashFlowError::Value {
        index,
        phase,
        cause,
    }
}
fn rate_error(index: Option<usize>, phase: &'static str, cause: RateError) -> CashFlowError {
    CashFlowError::Rate {
        index,
        phase,
        cause,
    }
}

/// A local per-call ledger, never retained as mutable public configuration.
struct AccrualWork {
    maximum: u64,
    used: u64,
}

impl AccrualWork {
    fn charge(
        &mut self,
        convention: &AccrualConvention,
        start: Date,
        end: Date,
        index: Option<usize>,
        phase: &'static str,
    ) -> Result<(), CashFlowError> {
        let extra = match convention {
            AccrualConvention::Icma(context) => {
                if start == end {
                    0
                } else {
                    let lower = start.min(end);
                    let upper = start.max(end);
                    let first = context
                        .periods()
                        .partition_point(|period| period.end() <= lower);
                    let last = context
                        .periods()
                        .partition_point(|period| period.start() < upper);
                    (last - first) as u64
                }
            }
            AccrualConvention::Business252(_) => u64::from(end.days_since(start).unsigned_abs()),
            AccrualConvention::Scalar(_) | AccrualConvention::ThirtyE360Isda(_) => 0,
        };
        // At most MAX_FLOWS partial/horizon queries, each <= 3_652_059 units,
        // plus one anchor: cumulative need <1.34e13, so u64 addition cannot overflow.
        let required_at_least = self.used + 1 + extra;
        if required_at_least > self.maximum {
            return Err(CashFlowError::AccrualWorkLimitExceeded {
                index,
                phase,
                maximum: self.maximum,
                used: self.used,
                required_at_least,
            });
        }
        self.used = required_at_least;
        Ok(())
    }
}
