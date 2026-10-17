//! Immutable zero-coupon and fixed-rate bond definitions, explicit settlement and
//! ex-coupon entitlement, earned-minus-detached accrued interest, exact binary64
//! per-100 quote conversion, and two-stage direct-horizon pricing plans.
//!
//! Every date, inclusion choice, ex-coupon cutoff and boundary, yield convention,
//! and contextual work budget is supplied explicitly. There is no implicit
//! settlement lag, payment calendar, par redemption, guessed frequency, currency
//! or global evaluation date.
//!
//! | SabiQuant type or operation | Financial role |
//! |---|---|
//! | [`Bond::zero_coupon`] / [`Bond::fixed_rate`] | Immutable bullet bond terms retaining schedule and quote provenance |
//! | [`BondSettlement`] / [`ExCouponTerms`] | Explicit payment-date inclusion and ex-coupon cutoff equality |
//! | [`CouponEntitlement`] | Per-coupon ceased, buyer-entitled or ex-coupon-detached status |
//! | [`price_per_100_from_amount`] / [`amount_from_price_per_100`] | Single-rounding binary64 quote conversions per 100 face |
//! | [`BondPricingPlan`] / [`BondValuation`] | Two-stage prepared direct-horizon pricing plan and clean/dirty/accrued valuation |
//! | [`BondPricingPlan::yield_from_price`] / [`BondYieldReport`] / [`BondYieldSensitivity`] | Bounded annual decimal yield-to-maturity inversion with fixed-scale binary64 residual |
//! | [`BondPricingPlan::risk`] / [`BondRisk`] | Analytical extended-exponent yield derivatives, time-weighted PV, modified/Macaulay duration and convexity |
//!
//! The adopted local contracts are recorded in
//! [the M8 bond core contract](https://github.com/iceman951/sabiquant/blob/main/docs/M8_BOND_CORE_CONTRACT.md)
//! and [the M8 generic bond yield/risk contract](https://github.com/iceman951/sabiquant/blob/main/docs/M8_BOND_YIELD_RISK_CONTRACT.md).
//! Named market yield conventions (UK DMO gilt, US Treasury, Canadian) and
//! analytical settlement enclosures remain separate later work.

use std::error::Error as StdError;
use std::fmt;

use crate::cashflows::{
    FixedCouponRate, FixedRateCoupon, Leg, PaymentDateInclusion, SimpleCashFlow, product,
    query_work_units,
};
use crate::error::{Interval, ValueError};
use crate::math::{SumError, exact_sum};
use crate::numeric;
use crate::rates::{AccrualConvention, Compounding, RateError, present_value_at, validate_rate};
use crate::schedule::Schedule;
use crate::time::Date;

mod quote;
mod yield_risk;

pub use yield_risk::{BondRisk, BondYieldError, BondYieldReport, BondYieldSensitivity};

/// Discriminates zero-coupon and fixed-rate bond definitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "bullet bond core has exactly two coupon structures"
)]
pub enum BondKind {
    /// A bullet bond with principal redemption only and no periodic coupons.
    ZeroCoupon,
    /// A bullet bond with scheduled fixed-rate coupons and principal redemption.
    FixedRate,
}

impl BondKind {
    /// Both explicit bond kinds.
    pub const ALL: [Self; 2] = [Self::ZeroCoupon, Self::FixedRate];
}

/// Controls whether a settlement date equal to an ex-coupon cutoff detaches the coupon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "cutoff equality has exactly two boundary conventions"
)]
pub enum ExCouponBoundary {
    /// The coupon is ex-coupon when `settlement_date >= cutoff_date`.
    OnOrAfter,
    /// The coupon is ex-coupon when `settlement_date > cutoff_date`.
    After,
}

impl ExCouponBoundary {
    /// Both explicit ex-coupon cutoff boundary rules.
    pub const ALL: [Self; 2] = [Self::OnOrAfter, Self::After];
}

/// An explicit ex-coupon cutoff date and equality boundary for one coupon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExCouponTerms {
    cutoff_date: Date,
    boundary: ExCouponBoundary,
}

impl ExCouponTerms {
    /// Creates explicit ex-coupon terms without inferring a calendar lag.
    #[must_use]
    pub const fn new(cutoff_date: Date, boundary: ExCouponBoundary) -> Self {
        Self {
            cutoff_date,
            boundary,
        }
    }

    /// Returns the supplied ex-coupon cutoff date.
    #[must_use]
    pub const fn cutoff_date(self) -> Date {
        self.cutoff_date
    }

    /// Returns the equality rule at `cutoff_date`.
    #[must_use]
    pub const fn boundary(self) -> ExCouponBoundary {
        self.boundary
    }

    /// Reports whether `settlement_date` falls in the ex-coupon region.
    #[must_use]
    pub fn is_ex_coupon(self, settlement_date: Date) -> bool {
        match self.boundary {
            ExCouponBoundary::OnOrAfter => settlement_date >= self.cutoff_date,
            ExCouponBoundary::After => settlement_date > self.cutoff_date,
        }
    }
}

/// An explicit analytical settlement date and payment-date inclusion rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BondSettlement {
    date: Date,
    payment_date_inclusion: PaymentDateInclusion,
}

impl BondSettlement {
    /// Creates an explicit settlement specification with no default lag or inclusion.
    #[must_use]
    pub const fn new(date: Date, payment_date_inclusion: PaymentDateInclusion) -> Self {
        Self {
            date,
            payment_date_inclusion,
        }
    }

    /// Returns the supplied settlement date.
    #[must_use]
    pub const fn date(self) -> Date {
        self.date
    }

    /// Returns the payment-date equality inclusion rule at `date`.
    #[must_use]
    pub const fn payment_date_inclusion(self) -> PaymentDateInclusion {
        self.payment_date_inclusion
    }
}

/// A generic direct-horizon yield accrual and compounding convention.
///
/// This represents direct `YF(settlement, payment)` discounting under the
/// supplied accrual and compounding rules; it does not claim a named market
/// street, UK DMO gilt or US Treasury convention.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BondYieldConvention {
    accrual: AccrualConvention,
    compounding: Compounding,
}

impl BondYieldConvention {
    /// Combines an explicit accrual convention and compounding rule.
    #[must_use]
    pub fn new(accrual: impl Into<AccrualConvention>, compounding: Compounding) -> Self {
        Self {
            accrual: accrual.into(),
            compounding,
        }
    }

    /// Borrows the complete discount accrual convention.
    #[must_use]
    pub const fn accrual(&self) -> &AccrualConvention {
        &self.accrual
    }

    /// Returns the declared compounding and frequency.
    #[must_use]
    pub const fn compounding(&self) -> Compounding {
        self.compounding
    }
}

/// Whether a quoted bond price per 100 face is clean or dirty.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "quoted bond prices are either clean or dirty"
)]
pub enum BondPriceKind {
    /// Excludes the bond's accrued interest adjustment (`dirty - accrued`).
    Clean,
    /// Includes the bond's accrued interest adjustment (full present value).
    Dirty,
}

impl BondPriceKind {
    /// Both explicit quoted price kinds.
    pub const ALL: [Self; 2] = [Self::Clean, Self::Dirty];
}

/// A finite signed bond price quoted per 100 of face amount, with explicit kind.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BondPrice {
    value_per_100_face: f64,
    kind: BondPriceKind,
}

impl BondPrice {
    /// Constructs a quoted price per 100 face, canonicalizing `-0.0` to `+0.0`.
    ///
    /// # Errors
    /// [`BondError::Value`] if `value_per_100_face` is not finite.
    pub fn new(value_per_100_face: f64, kind: BondPriceKind) -> Result<Self, BondError> {
        let value = numeric::finite("price per 100", value_per_100_face)
            .map_err(|cause| value_error(None, "bond price", cause))?;
        Ok(Self {
            value_per_100_face: canonical_zero(value),
            kind,
        })
    }

    /// Normalizes a finite amount to a price per 100 of `face` with one binary rounding.
    ///
    /// # Errors
    /// [`BondError::Value`] if `face` is not finite and strictly positive, if
    /// `amount` is not finite, or if a nonzero exact ratio underflows or overflows.
    pub fn from_amount(amount: f64, face: f64, kind: BondPriceKind) -> Result<Self, BondError> {
        let value_per_100_face = price_per_100_from_amount(amount, face)?;
        Ok(Self {
            value_per_100_face,
            kind,
        })
    }

    /// Returns the finite signed price per 100 of face amount.
    #[must_use]
    pub const fn value_per_100_face(self) -> f64 {
        self.value_per_100_face
    }

    /// Returns whether this quote is clean or dirty.
    #[must_use]
    pub const fn kind(self) -> BondPriceKind {
        self.kind
    }

    /// Converts this price per 100 into amount units for `face` with one binary rounding.
    ///
    /// # Errors
    /// [`BondError::Value`] if `face` is not finite and strictly positive, or if
    /// a nonzero exact product divided by 100 underflows or overflows.
    pub fn to_amount(self, face: f64) -> Result<f64, BondError> {
        amount_from_price_per_100(self.value_per_100_face, face)
    }
}

/// Normalizes a finite signed amount `X` to `100 * X / face` with one binary64 rounding.
///
/// The exact IEEE 754 binary rational `(25 * M_X / M_F) * 2^(e_X - e_F + 2)` is
/// rounded once to nearest, ties to even, avoiding intermediate `100.0 * X`
/// overflow and `X / face` underflow.
///
/// # Errors
/// [`BondError::Value`] if `face` is not finite and strictly positive, if
/// `amount` is not finite, or if a nonzero exact ratio underflows to zero or
/// overflows to infinity.
pub fn price_per_100_from_amount(amount: f64, face: f64) -> Result<f64, BondError> {
    let face = validate_face(face)?;
    let amount = numeric::finite("amount", amount)
        .map_err(|cause| value_error(None, "price per 100", cause))?;
    quote::per_100_from_amount(amount, face)
        .map_err(|cause| value_error(None, "price per 100", cause))
}

/// Converts a finite signed `value_per_100_face` to `face * value / 100` with one binary64 rounding.
///
/// The exact IEEE 754 binary rational `(M_F * M_P / 25) * 2^(e_F + e_P - 2)` is
/// rounded once to nearest, ties to even, avoiding intermediate overflow or underflow.
///
/// # Errors
/// [`BondError::Value`] if `face` is not finite and strictly positive, if
/// `value_per_100_face` is not finite, or if a nonzero exact result underflows
/// to zero or overflows to infinity.
pub fn amount_from_price_per_100(value_per_100_face: f64, face: f64) -> Result<f64, BondError> {
    let face = validate_face(face)?;
    let price = numeric::finite("price per 100", value_per_100_face)
        .map_err(|cause| value_error(None, "bond price", cause))?;
    quote::amount_from_per_100(price, face).map_err(|cause| value_error(None, "bond amount", cause))
}

/// Entitlement state of a bond coupon at a given [`BondSettlement`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "a coupon at settlement is ceased, buyer-entitled or ex-coupon detached"
)]
pub enum CouponEntitlement {
    /// The coupon payment has already occurred (or falls on settlement with `Exclude`).
    Ceased,
    /// The coupon payment is eligible and belongs to the buyer.
    BuyerEntitled,
    /// The coupon payment is still future/included, but detached from the buyer by its ex-date.
    ExCouponDetached,
}

impl CouponEntitlement {
    /// All three coupon entitlement states.
    pub const ALL: [Self; 3] = [Self::Ceased, Self::BuyerEntitled, Self::ExCouponDetached];
}

/// An immutable zero-coupon or fixed-rate bullet bond definition.
///
/// Definition equality compares all retained terms, including the source
/// [`Schedule`], [`FixedCouponRate`] and per-coupon [`ExCouponTerms`].
///
/// ```
/// use sabiquant::bonds::{Bond, BondSettlement, BondYieldConvention};
/// use sabiquant::cashflows::{PaymentDateInclusion, SimpleCashFlow};
/// use sabiquant::daycount::DayCount;
/// use sabiquant::rates::Compounding;
/// use sabiquant::time::Date;
///
/// let issue = Date::from_ymd(2026, 1, 15)?;
/// let maturity = Date::from_ymd(2027, 1, 15)?;
/// let redemption = SimpleCashFlow::new(maturity, 100.0)?;
/// let bond = Bond::zero_coupon(issue, maturity, 100.0, redemption)?;
/// let settlement = BondSettlement::new(issue, PaymentDateInclusion::Include);
/// let convention = BondYieldConvention::new(DayCount::Actual365Fixed, Compounding::SIMPLE);
/// let valuation = bond.evaluate(settlement, &convention, 0.0, 2)?;
/// assert_eq!(valuation.dirty_amount(), 100.0);
/// assert_eq!(valuation.clean_price_per_100()?, 100.0);
/// # Ok::<(), sabiquant::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Bond {
    kind: BondKind,
    issue_date: Date,
    contractual_maturity: Date,
    face: f64,
    redemption: SimpleCashFlow,
    schedule: Option<Schedule>,
    coupon_rate: Option<FixedCouponRate>,
    coupons: Vec<FixedRateCoupon>,
    ex_coupon: Vec<Option<ExCouponTerms>>,
}

impl Bond {
    /// Maximum total flows (coupons plus redemption) supported by resource policy.
    pub const MAX_FLOWS: u32 = Leg::MAX_FLOWS;

    /// Constructs a zero-coupon bullet bond with explicit face and redemption flow.
    ///
    /// # Errors
    /// [`BondError::InvalidIssueMaturity`] if `issue_date > contractual_maturity`,
    /// [`BondError::RedemptionBeforeIssue`] if `issue_date > redemption.payment_date()`,
    /// or [`BondError::Value`] if `face` or `redemption.amount()` is not strictly positive.
    pub fn zero_coupon(
        issue_date: Date,
        contractual_maturity: Date,
        face: f64,
        redemption: SimpleCashFlow,
    ) -> Result<Self, BondError> {
        if issue_date > contractual_maturity {
            return Err(BondError::InvalidIssueMaturity {
                issue_date,
                contractual_maturity,
            });
        }
        if issue_date > redemption.payment_date() {
            return Err(BondError::RedemptionBeforeIssue {
                issue_date,
                redemption_payment: redemption.payment_date(),
            });
        }
        let face = validate_face(face)?;
        validate_redemption_amount(redemption)?;
        Ok(Self {
            kind: BondKind::ZeroCoupon,
            issue_date,
            contractual_maturity,
            face,
            redemption,
            schedule: None,
            coupon_rate: None,
            coupons: Vec::new(),
            ex_coupon: Vec::new(),
        })
    }

    /// Constructs a fixed-rate bullet bond from an owned [`Schedule`] and explicit terms.
    ///
    /// Count, shape and ordering checks precede allocation and coupon accrual
    /// evaluation. One [`FixedRateCoupon`] is constructed per raw schedule
    /// interval, payable on its corresponding adjusted end date, with `face` as
    /// notional. The redemption flow remains separate from the final coupon.
    ///
    /// # Errors
    /// Cap/count/ex-vector/date-ordering errors, [`BondError::Value`] for
    /// nonpositive `face` or `redemption`, [`BondError::ExCouponAfterPayment`]
    /// if an ex-coupon cutoff follows its coupon payment date, or indexed
    /// [`BondError::Rate`] / [`BondError::Value`] causes during coupon construction.
    pub fn fixed_rate(
        issue_date: Date,
        schedule: Schedule,
        face: f64,
        coupon_rate: FixedCouponRate,
        ex_coupon: Vec<Option<ExCouponTerms>>,
        redemption: SimpleCashFlow,
        max_flows: u32,
    ) -> Result<Self, BondError> {
        if max_flows > Self::MAX_FLOWS {
            return Err(BondError::InvalidFlowLimit {
                requested: max_flows,
                maximum: Self::MAX_FLOWS,
            });
        }
        let raw = schedule.contractual_dates();
        let adjusted = schedule.adjusted_dates();
        let coupon_count = raw.len() - 1;
        let total_flows = coupon_count + 1;
        if total_flows > max_flows as usize {
            return Err(BondError::FlowLimitExceeded {
                maximum: max_flows,
                required: total_flows,
            });
        }
        if ex_coupon.len() != coupon_count {
            return Err(BondError::ExCouponCountMismatch {
                expected: coupon_count,
                actual: ex_coupon.len(),
            });
        }
        let contractual_maturity = raw[coupon_count];
        if issue_date > contractual_maturity {
            return Err(BondError::InvalidIssueMaturity {
                issue_date,
                contractual_maturity,
            });
        }
        if issue_date > redemption.payment_date() {
            return Err(BondError::RedemptionBeforeIssue {
                issue_date,
                redemption_payment: redemption.payment_date(),
            });
        }
        let last_coupon_payment = adjusted[coupon_count];
        if redemption.payment_date() < last_coupon_payment {
            return Err(BondError::RedemptionBeforeLastCoupon {
                last_coupon_payment,
                redemption_payment: redemption.payment_date(),
            });
        }
        let face = validate_face(face)?;
        validate_redemption_amount(redemption)?;
        for (index, terms) in ex_coupon.iter().copied().enumerate() {
            if let Some(terms) = terms {
                let payment_date = adjusted[index + 1];
                if terms.cutoff_date() > payment_date {
                    return Err(BondError::ExCouponAfterPayment {
                        index,
                        cutoff_date: terms.cutoff_date(),
                        payment_date,
                    });
                }
            }
        }
        let mut coupons = reserved(coupon_count, "fixed-rate bond coupons")?;
        for (index, pair) in raw.windows(2).enumerate() {
            let accrual_fraction = checked_horizon(
                coupon_rate.accrual(),
                pair[0],
                pair[1],
                Some(index),
                "coupon accrual",
            )?;
            let amount =
                product::product("coupon amount", face, coupon_rate.rate(), accrual_fraction)
                    .map_err(|cause| value_error(Some(index), "coupon amount", cause))?;
            coupons.push(FixedRateCoupon::from_parts(
                pair[0],
                pair[1],
                adjusted[index + 1],
                face,
                coupon_rate.clone(),
                accrual_fraction,
                amount,
            ));
        }
        Ok(Self {
            kind: BondKind::FixedRate,
            issue_date,
            contractual_maturity,
            face,
            redemption,
            schedule: Some(schedule),
            coupon_rate: Some(coupon_rate),
            coupons,
            ex_coupon,
        })
    }

    /// Returns whether this bond is zero-coupon or fixed-rate.
    #[must_use]
    pub const fn kind(&self) -> BondKind {
        self.kind
    }

    /// Returns the bond's issue date.
    #[must_use]
    pub const fn issue_date(&self) -> Date {
        self.issue_date
    }

    /// Returns the bond's raw contractual maturity date.
    #[must_use]
    pub const fn contractual_maturity(&self) -> Date {
        self.contractual_maturity
    }

    /// Returns the strictly positive finite face amount.
    #[must_use]
    pub const fn face(&self) -> f64 {
        self.face
    }

    /// Returns the explicit principal redemption cash flow.
    #[must_use]
    pub const fn redemption(&self) -> SimpleCashFlow {
        self.redemption
    }

    /// Borrows the retained source schedule (`None` for a zero-coupon bond).
    #[must_use]
    pub const fn schedule(&self) -> Option<&Schedule> {
        self.schedule.as_ref()
    }

    /// Borrows the retained fixed coupon quote (`None` for a zero-coupon bond).
    #[must_use]
    pub const fn coupon_rate(&self) -> Option<&FixedCouponRate> {
        self.coupon_rate.as_ref()
    }

    /// Borrows the generated fixed-rate coupons in chronological raw order.
    #[must_use]
    pub fn coupons(&self) -> &[FixedRateCoupon] {
        &self.coupons
    }

    /// Borrows the per-coupon optional ex-coupon terms.
    #[must_use]
    pub fn ex_coupon_terms(&self) -> &[Option<ExCouponTerms>] {
        &self.ex_coupon
    }

    /// Normalizes `amount` to a price per 100 of this bond's `face`.
    ///
    /// # Errors
    /// [`BondError::Value`] if `amount` is not finite or a nonzero ratio
    /// underflows or overflows.
    pub fn price_per_100_from_amount(&self, amount: f64) -> Result<f64, BondError> {
        price_per_100_from_amount(amount, self.face)
    }

    /// Converts a quoted [`BondPrice`] per 100 into amount units for this bond's `face`.
    ///
    /// # Errors
    /// [`BondError::Value`] if a nonzero result underflows or overflows.
    pub fn amount_from_price_per_100(&self, price: BondPrice) -> Result<f64, BondError> {
        price.to_amount(self.face)
    }

    /// Prepares a [`BondPricingPlan`] and evaluates it at `yield_rate`.
    ///
    /// Preparation errors always precede trial `yield_rate` errors.
    ///
    /// # Errors
    /// Any error from [`BondPricingPlan::new`] first, then [`BondPricingPlan::evaluate`].
    pub fn evaluate(
        &self,
        settlement: BondSettlement,
        yield_convention: &BondYieldConvention,
        yield_rate: f64,
        max_accrual_work: u64,
    ) -> Result<BondValuation, BondError> {
        let plan = BondPricingPlan::new(self, settlement, yield_convention, max_accrual_work)?;
        plan.evaluate(yield_rate)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PreparedDiscountFlow {
    index: usize,
    amount: f64,
    horizon: f64,
}

/// A prepared immutable pricing plan for one bond, settlement and yield convention.
///
/// `BondPricingPlan` resolves coupon/redemption entitlement, earned-minus-detached
/// accrued interest, and direct `settlement -> payment` discount horizons once,
/// so repeated [`evaluate`](Self::evaluate) calls perform no date or calendar work.
#[derive(Clone, Debug, PartialEq)]
pub struct BondPricingPlan<'a> {
    bond: &'a Bond,
    settlement: BondSettlement,
    yield_convention: &'a BondYieldConvention,
    coupon_entitlements: Vec<CouponEntitlement>,
    redemption_entitled: bool,
    discount_flows: Vec<PreparedDiscountFlow>,
    accrued_amount: f64,
    accrual_work_used: u64,
}

impl<'a> BondPricingPlan<'a> {
    /// Prepares entitlement, accrued interest and direct discount horizons.
    ///
    /// # Errors
    /// [`BondError::SettlementBeforeIssue`] if `settlement.date() < bond.issue_date()`,
    /// [`BondError::AccrualWorkLimitExceeded`] before any query exceeding
    /// `max_accrual_work`, [`BondError::AllocationFailed`], indexed
    /// [`BondError::Rate`] / [`BondError::Value`] for partial accrual or discount
    /// horizon failures, or [`BondError::Sum`] for accrued reduction overflow.
    pub fn new(
        bond: &'a Bond,
        settlement: BondSettlement,
        yield_convention: &'a BondYieldConvention,
        max_accrual_work: u64,
    ) -> Result<Self, BondError> {
        let settlement_date = settlement.date();
        if settlement_date < bond.issue_date() {
            return Err(BondError::SettlementBeforeIssue {
                settlement_date,
                issue_date: bond.issue_date(),
            });
        }
        let mut work = AccrualWork {
            maximum: max_accrual_work,
            used: 0,
        };
        work.charge(
            yield_convention.accrual(),
            settlement_date,
            settlement_date,
            None,
            "discount anchor",
        )?;
        let _ = checked_horizon(
            yield_convention.accrual(),
            settlement_date,
            settlement_date,
            None,
            "discount anchor",
        )?;
        let coupon_count = bond.coupons().len();
        let mut coupon_entitlements = reserved(coupon_count, "pricing plan entitlements")?;
        let mut accrued_terms = reserved(coupon_count, "pricing plan accrued terms")?;
        let mut discount_flows = reserved(coupon_count + 1, "pricing plan discount flows")?;
        let inclusion = settlement.payment_date_inclusion();

        for (index, coupon) in bond.coupons().iter().enumerate() {
            if !inclusion.eligible(coupon.payment_date(), settlement_date) {
                coupon_entitlements.push(CouponEntitlement::Ceased);
                continue;
            }
            let detached = bond.ex_coupon_terms()[index]
                .is_some_and(|terms| terms.is_ex_coupon(settlement_date));
            let earned = earned_coupon_accrual(coupon, settlement_date, index, &mut work)?;
            if detached {
                coupon_entitlements.push(CouponEntitlement::ExCouponDetached);
                let ex_term =
                    exact_sum(&[earned, -coupon.amount()]).map_err(|cause| BondError::Sum {
                        phase: "ex-coupon accrual",
                        cause,
                    })?;
                accrued_terms.push(ex_term);
            } else {
                coupon_entitlements.push(CouponEntitlement::BuyerEntitled);
                accrued_terms.push(earned);
                work.charge(
                    yield_convention.accrual(),
                    settlement_date,
                    coupon.payment_date(),
                    Some(index),
                    "discount horizon",
                )?;
                let horizon = checked_horizon(
                    yield_convention.accrual(),
                    settlement_date,
                    coupon.payment_date(),
                    Some(index),
                    "discount horizon",
                )?;
                discount_flows.push(PreparedDiscountFlow {
                    index,
                    amount: coupon.amount(),
                    horizon,
                });
            }
        }

        let redemption = bond.redemption();
        let redemption_entitled = inclusion.eligible(redemption.payment_date(), settlement_date);
        if redemption_entitled {
            work.charge(
                yield_convention.accrual(),
                settlement_date,
                redemption.payment_date(),
                Some(coupon_count),
                "redemption discount horizon",
            )?;
            let horizon = checked_horizon(
                yield_convention.accrual(),
                settlement_date,
                redemption.payment_date(),
                Some(coupon_count),
                "redemption discount horizon",
            )?;
            discount_flows.push(PreparedDiscountFlow {
                index: coupon_count,
                amount: redemption.amount(),
                horizon,
            });
        }

        let accrued_amount = exact_sum(&accrued_terms).map_err(|cause| BondError::Sum {
            phase: "accrued reduction",
            cause,
        })?;

        Ok(Self {
            bond,
            settlement,
            yield_convention,
            coupon_entitlements,
            redemption_entitled,
            discount_flows,
            accrued_amount: canonical_zero(accrued_amount),
            accrual_work_used: work.used,
        })
    }

    /// Borrows the underlying bond definition.
    #[must_use]
    pub const fn bond(&self) -> &'a Bond {
        self.bond
    }

    /// Returns the settlement specification used to prepare this plan.
    #[must_use]
    pub const fn settlement(&self) -> BondSettlement {
        self.settlement
    }

    /// Borrows the yield convention used to prepare this plan.
    #[must_use]
    pub const fn yield_convention(&self) -> &'a BondYieldConvention {
        self.yield_convention
    }

    /// Returns the cached yield-independent accrued interest adjustment in amount units.
    #[must_use]
    pub const fn accrued_amount(&self) -> f64 {
        self.accrued_amount
    }

    /// Normalizes the cached accrued interest adjustment per 100 of face.
    ///
    /// # Errors
    /// [`BondError::Value`] if a nonzero normalized ratio underflows or overflows.
    pub fn accrued_per_100(&self) -> Result<f64, BondError> {
        price_per_100_from_amount(self.accrued_amount, self.bond.face)
    }

    /// Returns the entitlement state of each coupon by original coupon index.
    #[must_use]
    pub fn coupon_entitlements(&self) -> &[CouponEntitlement] {
        &self.coupon_entitlements
    }

    /// Reports whether the principal redemption payment is entitled at settlement.
    #[must_use]
    pub const fn redemption_entitled(&self) -> bool {
        self.redemption_entitled
    }

    /// Returns the total number of buyer-entitled payments (coupons plus redemption).
    #[must_use]
    pub const fn entitled_flow_count(&self) -> usize {
        self.discount_flows.len()
    }

    /// Returns the cumulative accrual work units charged during plan preparation.
    #[must_use]
    pub const fn accrual_work_used(&self) -> u64 {
        self.accrual_work_used
    }

    /// Evaluates dirty and clean bond valuation at a trial decimal annual `yield_rate`.
    ///
    /// # Errors
    /// [`BondError::Rate`] for an invalid `yield_rate` or any per-flow discount
    /// factor/present-value failure, [`BondError::AllocationFailed`], or
    /// [`BondError::Sum`] for dirty or clean reduction overflow.
    pub fn evaluate(&self, yield_rate: f64) -> Result<BondValuation, BondError> {
        let compounding = self.yield_convention.compounding();
        let rate = validate_rate(yield_rate, compounding)
            .map_err(|cause| rate_error(None, "yield quote", cause))?;
        let mut pv_terms = reserved(self.discount_flows.len(), "present value terms")?;
        let dirty_amount = self.evaluate_dirty_with_validated_rate(rate, &mut pv_terms)?;
        let clean_amount =
            exact_sum(&[dirty_amount, -self.accrued_amount]).map_err(|cause| BondError::Sum {
                phase: "clean price reduction",
                cause,
            })?;
        Ok(BondValuation {
            settlement: self.settlement,
            face: self.bond.face,
            yield_rate: rate,
            dirty_amount,
            accrued_amount: self.accrued_amount,
            clean_amount: canonical_zero(clean_amount),
        })
    }

    pub(super) fn evaluate_dirty_into(
        &self,
        yield_rate: f64,
        pv_terms: &mut Vec<f64>,
    ) -> Result<(f64, f64), BondError> {
        let compounding = self.yield_convention.compounding();
        let rate = validate_rate(yield_rate, compounding)
            .map_err(|cause| rate_error(None, "yield quote", cause))?;
        let dirty_amount = self.evaluate_dirty_with_validated_rate(rate, pv_terms)?;
        Ok((rate, dirty_amount))
    }

    fn evaluate_dirty_with_validated_rate(
        &self,
        rate: f64,
        pv_terms: &mut Vec<f64>,
    ) -> Result<f64, BondError> {
        let compounding = self.yield_convention.compounding();
        pv_terms.clear();
        for flow in &self.discount_flows {
            let pv = present_value_at(rate, compounding, flow.amount, flow.horizon)
                .map_err(|cause| rate_error(Some(flow.index), "discount factor", cause))?;
            pv_terms.push(pv);
        }
        let dirty_amount = exact_sum(pv_terms).map_err(|cause| BondError::Sum {
            phase: "dirty price reduction",
            cause,
        })?;
        Ok(canonical_zero(dirty_amount))
    }
}

/// Dirty, accrued and clean bond amounts at a settlement date and trial yield.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BondValuation {
    settlement: BondSettlement,
    face: f64,
    yield_rate: f64,
    dirty_amount: f64,
    accrued_amount: f64,
    clean_amount: f64,
}

impl BondValuation {
    /// Returns the settlement specification of this valuation.
    #[must_use]
    pub const fn settlement(self) -> BondSettlement {
        self.settlement
    }

    /// Returns the bond's strictly positive finite face amount.
    #[must_use]
    pub const fn face(self) -> f64 {
        self.face
    }

    /// Returns the canonicalized decimal annual yield rate used in this valuation.
    #[must_use]
    pub const fn yield_rate(self) -> f64 {
        self.yield_rate
    }

    /// Returns the buyer's dirty present value in amount units.
    #[must_use]
    pub const fn dirty_amount(self) -> f64 {
        self.dirty_amount
    }

    /// Returns the bond's accrued interest adjustment in amount units.
    #[must_use]
    pub const fn accrued_amount(self) -> f64 {
        self.accrued_amount
    }

    /// Returns the clean value (`exact_sum([dirty_amount, -accrued_amount])`) in amount units.
    #[must_use]
    pub const fn clean_amount(self) -> f64 {
        self.clean_amount
    }

    /// Normalizes [`dirty_amount`](Self::dirty_amount) per 100 of face.
    ///
    /// # Errors
    /// [`BondError::Value`] if a nonzero normalized ratio underflows or overflows.
    pub fn dirty_price_per_100(self) -> Result<f64, BondError> {
        price_per_100_from_amount(self.dirty_amount, self.face)
    }

    /// Normalizes [`accrued_amount`](Self::accrued_amount) per 100 of face.
    ///
    /// # Errors
    /// [`BondError::Value`] if a nonzero normalized ratio underflows or overflows.
    pub fn accrued_per_100(self) -> Result<f64, BondError> {
        price_per_100_from_amount(self.accrued_amount, self.face)
    }

    /// Normalizes [`clean_amount`](Self::clean_amount) per 100 of face.
    ///
    /// # Errors
    /// [`BondError::Value`] if a nonzero normalized ratio underflows or overflows.
    pub fn clean_price_per_100(self) -> Result<f64, BondError> {
        price_per_100_from_amount(self.clean_amount, self.face)
    }

    /// Returns the normalized dirty [`BondPrice`].
    ///
    /// # Errors
    /// [`BondError::Value`] if a nonzero normalized ratio underflows or overflows.
    pub fn dirty_price(self) -> Result<BondPrice, BondError> {
        BondPrice::from_amount(self.dirty_amount, self.face, BondPriceKind::Dirty)
    }

    /// Returns the normalized clean [`BondPrice`].
    ///
    /// # Errors
    /// [`BondError::Value`] if a nonzero normalized ratio underflows or overflows.
    pub fn clean_price(self) -> Result<BondPrice, BondError> {
        BondPrice::from_amount(self.clean_amount, self.face, BondPriceKind::Clean)
    }
}

/// A rejected bond definition, settlement, resource request or valuation.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum BondError {
    /// Flow-count cap exceeds the supported policy; kind `bond.invalid_flow_limit`.
    #[non_exhaustive]
    InvalidFlowLimit {
        /// Rejected caller cap.
        requested: u32,
        /// Largest supported cap.
        maximum: u32,
    },
    /// Required flows (coupons plus redemption) exceed caller cap; kind `bond.flow_limit_exceeded`.
    #[non_exhaustive]
    FlowLimitExceeded {
        /// Original caller cap.
        maximum: u32,
        /// Required flow count including redemption.
        required: usize,
    },
    /// Ex-coupon vector length does not match schedule coupon count; kind `bond.ex_coupon_count_mismatch`.
    #[non_exhaustive]
    ExCouponCountMismatch {
        /// Number of schedule coupon intervals.
        expected: usize,
        /// Supplied ex-coupon vector length.
        actual: usize,
    },
    /// Issue date follows contractual maturity; kind `bond.invalid_issue_maturity`.
    #[non_exhaustive]
    InvalidIssueMaturity {
        /// Supplied issue date.
        issue_date: Date,
        /// Contractual maturity date.
        contractual_maturity: Date,
    },
    /// Issue date follows redemption payment date; kind `bond.redemption_before_issue`.
    #[non_exhaustive]
    RedemptionBeforeIssue {
        /// Supplied issue date.
        issue_date: Date,
        /// Redemption payment date.
        redemption_payment: Date,
    },
    /// Redemption payment precedes the final coupon payment; kind `bond.redemption_before_last_coupon`.
    #[non_exhaustive]
    RedemptionBeforeLastCoupon {
        /// Final adjusted coupon payment date.
        last_coupon_payment: Date,
        /// Rejected redemption payment date.
        redemption_payment: Date,
    },
    /// Ex-coupon cutoff date follows its coupon payment date; kind `bond.ex_coupon_after_payment`.
    #[non_exhaustive]
    ExCouponAfterPayment {
        /// Zero-based coupon index.
        index: usize,
        /// Rejected ex-coupon cutoff date.
        cutoff_date: Date,
        /// Coupon payment date.
        payment_date: Date,
    },
    /// Settlement date precedes the bond's issue date; kind `bond.settlement_before_issue`.
    #[non_exhaustive]
    SettlementBeforeIssue {
        /// Rejected settlement date.
        settlement_date: Date,
        /// Bond issue date.
        issue_date: Date,
    },
    /// A required vector reservation failed; kind `bond.allocation_failed`.
    #[non_exhaustive]
    AllocationFailed {
        /// Operation requiring storage.
        phase: &'static str,
        /// Requested element capacity.
        requested: usize,
    },
    /// Next accrual query exceeds the shared per-plan work budget; kind `bond.accrual_work_limit_exceeded`.
    #[non_exhaustive]
    AccrualWorkLimitExceeded {
        /// Original flow index (`coupons.len()` for redemption, `None` for discount anchor).
        index: Option<usize>,
        /// Query phase about to begin.
        phase: &'static str,
        /// Caller work budget.
        maximum: u64,
        /// Units already charged.
        used: u64,
        /// Cumulative units required including the next query.
        required_at_least: u64,
    },
    /// Numeric input or result failure; kind `bond.value`.
    #[non_exhaustive]
    Value {
        /// Original flow index, or `None` for a bond-level/quote value.
        index: Option<usize>,
        /// Numeric operation phase.
        phase: &'static str,
        /// Preserved typed numeric cause.
        cause: ValueError,
    },
    /// Accrual or discount-rate failure; kind `bond.rate`.
    #[non_exhaustive]
    Rate {
        /// Original flow index (`coupons.len()` for redemption), or `None` for anchor/quote.
        index: Option<usize>,
        /// Query or factor operation phase.
        phase: &'static str,
        /// Preserved typed rate cause and its contextual source chain.
        cause: RateError,
    },
    /// Exact summation reduction failure; kind `bond.sum`.
    #[non_exhaustive]
    Sum {
        /// Reduction phase.
        phase: &'static str,
        /// Preserved typed summation cause.
        cause: SumError,
    },
    /// Dirty-normalized risk measure requires a nonzero dirty price; kind `bond.zero_dirty_price`.
    #[non_exhaustive]
    ZeroDirtyPrice {
        /// Requested normalized risk measure (`"modified duration"`, `"macaulay duration"`, or `"convexity"`).
        measure: &'static str,
        /// Canonical finite annual decimal yield rate.
        yield_rate: f64,
    },
    /// Defensive private integer-capacity invariant failure; kind `bond.internal_capacity`.
    #[non_exhaustive]
    InternalCapacity {
        /// Original flow index, or `None` for a plan/getter operation.
        index: Option<usize>,
        /// Operation phase.
        phase: &'static str,
        /// Specific bounded kernel operation.
        operation: &'static str,
    },
}

impl BondError {
    /// Returns the stable bond-domain error kind.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::InvalidFlowLimit { .. } => "bond.invalid_flow_limit",
            Self::FlowLimitExceeded { .. } => "bond.flow_limit_exceeded",
            Self::ExCouponCountMismatch { .. } => "bond.ex_coupon_count_mismatch",
            Self::InvalidIssueMaturity { .. } => "bond.invalid_issue_maturity",
            Self::RedemptionBeforeIssue { .. } => "bond.redemption_before_issue",
            Self::RedemptionBeforeLastCoupon { .. } => "bond.redemption_before_last_coupon",
            Self::ExCouponAfterPayment { .. } => "bond.ex_coupon_after_payment",
            Self::SettlementBeforeIssue { .. } => "bond.settlement_before_issue",
            Self::AllocationFailed { .. } => "bond.allocation_failed",
            Self::AccrualWorkLimitExceeded { .. } => "bond.accrual_work_limit_exceeded",
            Self::Value { .. } => "bond.value",
            Self::Rate { .. } => "bond.rate",
            Self::Sum { .. } => "bond.sum",
            Self::ZeroDirtyPrice { .. } => "bond.zero_dirty_price",
            Self::InternalCapacity { .. } => "bond.internal_capacity",
        }
    }
}

impl fmt::Display for BondError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFlowLimit { requested, maximum } => write!(
                f,
                "bond flow limit {requested} exceeds supported maximum {maximum}"
            ),
            Self::FlowLimitExceeded { maximum, required } => write!(
                f,
                "bond flow count {required} exceeds caller limit {maximum}"
            ),
            Self::ExCouponCountMismatch { expected, actual } => write!(
                f,
                "ex-coupon count {actual} does not match schedule coupon count {expected}"
            ),
            Self::InvalidIssueMaturity {
                issue_date,
                contractual_maturity,
            } => write!(
                f,
                "bond issue date {issue_date} follows contractual maturity {contractual_maturity}"
            ),
            Self::RedemptionBeforeIssue {
                issue_date,
                redemption_payment,
            } => write!(
                f,
                "redemption payment {redemption_payment} precedes issue date {issue_date}"
            ),
            Self::RedemptionBeforeLastCoupon {
                last_coupon_payment,
                redemption_payment,
            } => write!(
                f,
                "redemption payment {redemption_payment} precedes final coupon payment {last_coupon_payment}"
            ),
            Self::ExCouponAfterPayment {
                index,
                cutoff_date,
                payment_date,
            } => write!(
                f,
                "coupon {index} ex-coupon cutoff {cutoff_date} follows payment date {payment_date}"
            ),
            Self::SettlementBeforeIssue {
                settlement_date,
                issue_date,
            } => write!(
                f,
                "settlement date {settlement_date} precedes bond issue date {issue_date}"
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
            Self::ZeroDirtyPrice {
                measure,
                yield_rate,
            } => write!(
                f,
                "cannot normalize {measure} at yield {yield_rate:?}: dirty price is zero"
            ),
            Self::InternalCapacity {
                index,
                phase,
                operation,
            } => write!(
                f,
                "{phase} at flow {index:?}: internal capacity invariant failed during {operation}"
            ),
        }
    }
}

impl StdError for BondError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Value { cause, .. } => Some(cause),
            Self::Rate { cause, .. } => Some(cause),
            Self::Sum { cause, .. } => Some(cause),
            _ => None,
        }
    }
}

fn validate_face(face: f64) -> Result<f64, BondError> {
    numeric::within("bond face", face, Interval::greater_than(0.0))
        .map_err(|cause| value_error(None, "bond face", cause))
}

fn validate_redemption_amount(redemption: SimpleCashFlow) -> Result<(), BondError> {
    let _ = numeric::within(
        "redemption amount",
        redemption.amount(),
        Interval::greater_than(0.0),
    )
    .map_err(|cause| value_error(None, "redemption amount", cause))?;
    Ok(())
}

fn earned_coupon_accrual(
    coupon: &FixedRateCoupon,
    settlement_date: Date,
    index: usize,
    work: &mut AccrualWork,
) -> Result<f64, BondError> {
    if settlement_date <= coupon.start() {
        return Ok(0.0);
    }
    if settlement_date >= coupon.end() {
        return Ok(coupon.amount());
    }
    work.charge(
        coupon.coupon_rate().accrual(),
        coupon.start(),
        settlement_date,
        Some(index),
        "partial accrual",
    )?;
    let fraction = checked_horizon(
        coupon.coupon_rate().accrual(),
        coupon.start(),
        settlement_date,
        Some(index),
        "partial accrual",
    )?;
    product::product(
        "accrued amount",
        coupon.notional(),
        coupon.coupon_rate().rate(),
        fraction,
    )
    .map_err(|cause| value_error(Some(index), "accrued amount", cause))
}

fn checked_horizon(
    convention: &AccrualConvention,
    start: Date,
    end: Date,
    index: Option<usize>,
    phase: &'static str,
) -> Result<f64, BondError> {
    let fraction = convention
        .year_fraction(start, end)
        .map_err(|cause| rate_error(index, phase, cause))?;
    numeric::result_within("accrual fraction", fraction, Interval::at_least(0.0))
        .map(canonical_zero)
        .map_err(|cause| value_error(index, phase, cause))
}

fn reserved<T>(count: usize, phase: &'static str) -> Result<Vec<T>, BondError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| BondError::AllocationFailed {
            phase,
            requested: count,
        })?;
    Ok(values)
}

fn canonical_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

fn value_error(index: Option<usize>, phase: &'static str, cause: ValueError) -> BondError {
    BondError::Value {
        index,
        phase,
        cause,
    }
}

fn rate_error(index: Option<usize>, phase: &'static str, cause: RateError) -> BondError {
    BondError::Rate {
        index,
        phase,
        cause,
    }
}

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
    ) -> Result<(), BondError> {
        // At most MAX_FLOWS - 1 partial coupon queries + MAX_FLOWS discount
        // queries + 1 anchor, each <= 3_652_059 units: total < 2.67e13 < u64::MAX.
        let required_at_least = self.used + query_work_units(convention, start, end);
        if required_at_least > self.maximum {
            return Err(BondError::AccrualWorkLimitExceeded {
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
