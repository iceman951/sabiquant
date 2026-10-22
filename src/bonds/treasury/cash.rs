//! Exact Treasury source cash, price cash and compact owned provenance.
//!
//! Source coupons retain the five-word `X/(10^dscale*s_part)` definition per
//! $1,000, while accrued recipes retain the prepared published five-place
//! count. Neither passes through a public decimal or binary observation.
//! Source views and source cash fuse decimal powers in eight words. Price
//! evidence is decoded onto `R0 = 2^1074*5^38` in forty words; invoice totals
//! use `10*B_price + published_count*(R0/10^5)` even when accrued is zero.
//! Components stay wide until their exact sum, and only final cash is admitted.
//! No cash call allocates or queries dates, calendars, clocks or callbacks.

use super::{
    SOURCE_VERSION, Scalar, SourceFormula, SourceRational, TreasuryError, TreasuryFirstPeriod,
    TreasuryPricingPlan, TreasurySourcePolicy, UsTreasury, capacity,
};
use crate::amounts::uint::Wide;
use crate::amounts::{AmountError, AmountQuantum, AmountRounding, DecimalAmount};
use crate::calendar::EndOfMonth;
use crate::settlement::{
    CashAggregation, SettlementEvidence, SettlementGuarantee, SettlementRounding,
};
use crate::time::Date;
use std::fmt;

/// Eight-word source/view arithmetic, bounded by cash T9's 397-bit maximum.
type SourceInt = Wide<8>;
/// Forty-word price arithmetic, including the 2,434-bit component-net bound.
type PriceInt = Wide<40>;

/// Explicit quantized view of the raw analytical coupon.
const RAW_COUPON: &str = "raw coupon";
/// Explicit quantized view of the selected-policy source coupon.
const SOURCE_COUPON: &str = "source coupon";
/// Explicit quantized view of the published accrued source.
const SOURCE_ACCRUED: &str = "source accrued";
/// Final quantization of a source coupon payment.
const CASH_COUPON: &str = "cash coupon";
/// Quantization of the published accrued payment.
const CASH_ACCRUED: &str = "cash accrued";
/// First required component-invoice capital stage.
const CASH_CAPITAL: &str = "cash capital";
/// Final dirty-price or total-invoice quantizer.
const CASH_TOTAL: &str = "cash total";
/// Exact sum of the separately rounded capital and accrued components.
const CASH_NET: &str = "cash net";

/// `R0 = 10^38 << 1036`, so every supported decimal divides this basis.
const BASIS_SHIFT: u32 = 1036;

/// One complete Treasury cash recipe and its exact source/provenance snapshot.
///
/// Equality and hashing retain source policy, original dated and issue roles,
/// coupon and lifecycle dates, end-of-month policy, dated and settlement quasi
/// pairs, selected coupon and accrued ratios, exact source, nominal, rounding,
/// price-evidence provenance and aggregation. Equal cash does not erase these
/// distinctions. Unrelated quasi dates and the complete instrument are not
/// retained. Resource caps are not financial identity fields.
///
/// `Debug` reports public provenance and policy without formatting wide source
/// buffers or quantizing the source. For a requested source observation use
/// [`source_value_per_100`](Self::source_value_per_100).
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct TreasuryCashInstruction {
    /// Shared immutable instrument provenance.
    snapshot: Snapshot,
    /// Strictly positive par nominal.
    nominal: DecimalAmount,
    /// Explicit cash quantizer and signed rounding mode.
    cash: SettlementRounding,
    /// A complete recipe-specific snapshot.
    recipe: Recipe,
}

impl fmt::Debug for TreasuryCashInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TreasuryCashInstruction")
            .field("recipe", &self.recipe_identifier())
            .field("source_version", &SOURCE_VERSION)
            .field("source_policy", &self.snapshot.source_policy)
            .field("coupon_per_100", &self.snapshot.coupon_per_100)
            .field("original_dated_date", &self.snapshot.original_dated_date)
            .field("original_issue_date", &self.snapshot.original_issue_date)
            .field("first_coupon_date", &self.snapshot.first_coupon_date)
            .field("maturity_date", &self.snapshot.maturity_date)
            .field("first_period", &self.snapshot.first_period)
            .field("end_of_month", &self.snapshot.end_of_month)
            .field("dated_quasi_dates", &self.snapshot.dated_quasi_dates)
            .field("settlement_date", &self.settlement_date())
            .field("settlement_quasi_dates", &self.settlement_quasi_dates())
            .field("source_coupon_index", &self.source_coupon_index())
            .field("source_coupon_date", &self.source_coupon_date())
            .field("source_day_ratios", &self.source_day_ratios())
            .field("nominal", &self.nominal)
            .field("cash_rounding", &self.cash)
            .field("price_evidence", &self.price_evidence())
            .field("aggregation", &self.aggregation())
            .finish_non_exhaustive()
    }
}

/// Shared instrument identity retained without a vector clone.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct Snapshot {
    /// Annual coupon in exact units per100.
    coupon_per_100: DecimalAmount,
    /// Original dated date, independent of issue.
    original_dated_date: Date,
    /// Original issue date.
    original_issue_date: Date,
    /// First actual coupon payment date.
    first_coupon_date: Date,
    /// Original maturity.
    maturity_date: Date,
    /// First-period role.
    first_period: TreasuryFirstPeriod,
    /// Explicit direct-cycle policy.
    end_of_month: EndOfMonth,
    /// Explicit daily publication and source tie policy.
    source_policy: TreasurySourcePolicy,
    /// Minimal dated pair, `Q0 <= D < Q1`.
    dated_quasi_dates: (Date, Date),
}

/// Prepared settlement provenance, with no date query needed at a factory.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct SettlementContext {
    /// Prepared settlement date.
    date: Date,
    /// Containing quasi pair, including a skipped long-first boundary.
    quasi_dates: (Date, Date),
}

/// One actual coupon ordinal and its original date.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct SelectedCoupon {
    /// Actual-coupon ordinal, excluding the long-first skipped quasi date.
    index: usize,
    /// Actual payment date.
    date: Date,
}

/// Published source AI and its retained lifecycle provenance.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct AccruedContext {
    /// Prepared settlement and containing quasi dates.
    settlement: SettlementContext,
    /// First actual coupon strictly after settlement.
    coupon: SelectedCoupon,
    /// The one or two unrounded civil-day ingredients of publication.
    pieces: ((u32, u32), Option<(u32, u32)>),
    /// Exact published five-place count per $1,000, never a public amount gate.
    published_count: Scalar,
}

/// Complete recipe shapes; no mixed or partially initialized form is public.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum Recipe {
    /// Selected source coupon per $1,000, scaled once by nominal/1000.
    Coupon {
        /// Selected actual coupon.
        coupon: SelectedCoupon,
        /// Exact selected-policy formula per $1,000.
        source: SourceFormula,
        /// First fractional coupon's original day ratio, or None.
        day_ratio: Option<(u32, u32)>,
    },
    /// Prepared published accrued source, scaled once by nominal/1000.
    Accrued(AccruedContext),
    /// Clean price per100 combined with published accrued per $1,000.
    Invoice {
        /// Prepared exact accrued source and provenance.
        accrued: AccruedContext,
        /// Original supplied/declared clean-price evidence.
        clean: SettlementEvidence,
        /// Explicit total or component rounding.
        aggregation: CashAggregation,
    },
    /// Dirty price per100 without an accrued source.
    Dirty {
        /// Prepared settlement context.
        settlement: SettlementContext,
        /// Original dirty-price evidence.
        dirty: SettlementEvidence,
    },
}

/// A determined amount or an explicitly unresolved exact-cash consequence.
///
/// Diagnostic variants compare/hash only their listed fields; retain the
/// instruction separately when provenance is needed for such a diagnostic.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[expect(
    clippy::large_enum_variant,
    reason = "the allocation-free Copy receipt retains its complete owned instruction"
)]
pub enum TreasuryCashDecision {
    /// One representable amount with the complete original instruction.
    Determined(TreasuryCashReceipt),
    /// Different representable rounded extrema; no interior-range claim.
    #[non_exhaustive]
    QuantumAmbiguous {
        /// Least final cash amount.
        lower_amount: DecimalAmount,
        /// Greatest final cash amount.
        upper_amount: DecimalAmount,
    },
    /// Continuous nonpoint variation at a required Exact quantizer.
    #[non_exhaustive]
    ExactStageUnresolved {
        /// First required stage without universal Exact-definedness.
        stage: &'static str,
    },
}

/// One determined Treasury cash amount with its exact instruction/provenance.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TreasuryCashReceipt {
    /// Final canonical cash amount.
    amount: DecimalAmount,
    /// Complete original owned instruction.
    instruction: TreasuryCashInstruction,
    /// Label of the supplied price ingredient, absent for pure source cash.
    guarantee: Option<SettlementGuarantee>,
}

impl TreasuryCashReceipt {
    /// Returns final cash: positive is issuer-to-holder for a coupon and
    /// buyer-to-seller for accrued, dirty and invoice payments.
    #[must_use]
    pub const fn amount(self) -> DecimalAmount {
        self.amount
    }

    /// Borrows the original complete owned instruction.
    #[must_use]
    pub const fn instruction(&self) -> &TreasuryCashInstruction {
        &self.instruction
    }

    /// Returns the price ingredient's original evidence label, or None for
    /// pure source cash. It does not certify an ideal Treasury price.
    #[must_use]
    pub const fn price_evidence_guarantee(self) -> Option<SettlementGuarantee> {
        self.guarantee
    }
}

impl UsTreasury {
    /// Quantizes one exact raw analytical coupon per100, excluding principal.
    ///
    /// Source daily and publication policies do not alter this raw view. The
    /// exact rational need not have a finite decimal representation.
    ///
    /// # Errors
    /// [`TreasuryError::InvalidCouponIndex`] precedes view work. Point
    /// Inexact or out-of-range results are [`TreasuryError::CashArithmetic`]
    /// at `raw coupon`; defensive capacity failures remain contextual.
    pub fn raw_coupon_per_100(
        &self,
        index: usize,
        quantum: AmountQuantum,
        mode: AmountRounding,
    ) -> Result<DecimalAmount, TreasuryError> {
        self.check_coupon_index(index)?;
        let site = Site::point(RAW_COUPON, Some(index));
        let raw = self.raw_coupon(index)?;
        quantize_rational(raw, quantum, mode, site)
    }

    /// Quantizes one selected-policy source coupon per100, excluding principal.
    ///
    /// A first fractional coupon retains its daily-basis stage without a
    /// coupon-level quantizer; every regular coupon is exactly C/2.
    ///
    /// # Errors
    /// [`TreasuryError::InvalidCouponIndex`] precedes view work. Inexact or
    /// out-of-range point results are [`TreasuryError::CashArithmetic`] at
    /// `source coupon`, or a defensive capacity error is returned.
    pub fn source_coupon_per_100(
        &self,
        index: usize,
        quantum: AmountQuantum,
        mode: AmountRounding,
    ) -> Result<DecimalAmount, TreasuryError> {
        self.check_coupon_index(index)?;
        let source = self.source_coupon_formula(index)?;
        let site = Site::point(SOURCE_COUPON, Some(index));
        let coefficient =
            quantize_source(source, None, SettlementRounding::new(quantum, mode), site)?;
        admit(
            coefficient,
            false,
            quantum.amount().scale(),
            site,
            "quantize",
        )
    }

    /// Defines source coupon cash `R(N*K_i/1000)` for an actual coupon ordinal.
    ///
    /// Positive cash is issuer-to-holder. The caller supplies an entitled held
    /// nominal; record-date ownership, denominations and payment calendars are
    /// not verified. Non-$1,000 coupon nominal is an explicit local extension.
    ///
    /// # Errors
    /// [`TreasuryError::InvalidCouponIndex`] first, then
    /// [`TreasuryError::NonPositiveNominal`]. Defensive snapshot inconsistencies
    /// remain [`TreasuryError::InternalCapacity`].
    pub fn coupon_cash(
        &self,
        index: usize,
        nominal: DecimalAmount,
        rounding: SettlementRounding,
    ) -> Result<TreasuryCashInstruction, TreasuryError> {
        self.check_coupon_index(index)?;
        positive_nominal(nominal)?;
        let site = Site::point(CASH_COUPON, Some(index));
        let date = self
            .first_coupon_quasi_index
            .checked_add(index)
            .and_then(|i| self.quasi_dates.get(i))
            .copied()
            .ok_or_else(|| site.capacity("derive source"))?;
        let day_ratio = if index == 0 && self.first_period != TreasuryFirstPeriod::Regular {
            Some((self.d0, self.s0))
        } else {
            None
        };
        Ok(TreasuryCashInstruction {
            snapshot: Snapshot::new(self, site)?,
            nominal,
            cash: rounding,
            recipe: Recipe::Coupon {
                coupon: SelectedCoupon { index, date },
                source: self.source_coupon_formula(index)?,
                day_ratio,
            },
        })
    }
}

impl TreasuryPricingPlan<'_> {
    /// Defines published accrued cash `R(N*P1000/1000)`.
    ///
    /// Positive cash is buyer-to-seller. The prepared total-first five-place
    /// publication is scaled exactly once, including fractional nominal; no
    /// second five-place stage is inserted after nominal scaling.
    ///
    /// # Errors
    /// [`TreasuryError::NonPositiveNominal`] precedes all snapshot work; a
    /// defensive inconsistent snapshot returns [`TreasuryError::InternalCapacity`].
    pub fn accrued_cash(
        &self,
        nominal: DecimalAmount,
        rounding: SettlementRounding,
    ) -> Result<TreasuryCashInstruction, TreasuryError> {
        positive_nominal(nominal)?;
        let site = Site::point(CASH_ACCRUED, None);
        Ok(TreasuryCashInstruction {
            snapshot: Snapshot::new(self.treasury, site)?,
            nominal,
            cash: rounding,
            recipe: Recipe::Accrued(self.accrued_context(site)?),
        })
    }

    /// Defines a clean-price invoice with the prepared published accrued source.
    ///
    /// `RoundTotal` quantizes `N*C_p/100+N*P1000/1000` once.
    /// `RoundComponentsThenSum` quantizes capital first, then accrued, and
    /// exact-sums both wide results without a further cash quantizer. The
    /// evidence label describes only the price ingredient.
    ///
    /// # Errors
    /// [`TreasuryError::NonPositiveNominal`] first, then defensive
    /// [`TreasuryError::InternalCapacity`] for an inconsistent prepared snapshot.
    pub fn clean_accrued_cash(
        &self,
        nominal: DecimalAmount,
        clean: SettlementEvidence,
        aggregation: CashAggregation,
        rounding: SettlementRounding,
    ) -> Result<TreasuryCashInstruction, TreasuryError> {
        positive_nominal(nominal)?;
        let site = Site::point(first_invoice_stage(aggregation), None);
        Ok(TreasuryCashInstruction {
            snapshot: Snapshot::new(self.treasury, site)?,
            nominal,
            cash: rounding,
            recipe: Recipe::Invoice {
                accrued: self.accrued_context(site)?,
                clean,
                aggregation,
            },
        })
    }

    /// Defines dirty-price cash `R(N*P/100)` with no source-accrued shift.
    ///
    /// Signed price evidence may reverse cash direction while nominal remains
    /// strictly positive. Its supplied/declared provenance is retained.
    ///
    /// # Errors
    /// [`TreasuryError::NonPositiveNominal`] precedes snapshot work; defensive
    /// inconsistencies return [`TreasuryError::InternalCapacity`].
    pub fn dirty_cash(
        &self,
        nominal: DecimalAmount,
        dirty: SettlementEvidence,
        rounding: SettlementRounding,
    ) -> Result<TreasuryCashInstruction, TreasuryError> {
        positive_nominal(nominal)?;
        let site = Site::point(CASH_TOTAL, None);
        Ok(TreasuryCashInstruction {
            snapshot: Snapshot::new(self.treasury, site)?,
            nominal,
            cash: rounding,
            recipe: Recipe::Dirty {
                settlement: self.settlement_context(site)?,
                dirty,
            },
        })
    }

    /// Copies the prepared settlement pair, with no new date calculation.
    fn settlement_context(&self, site: Site) -> Result<SettlementContext, TreasuryError> {
        let q = &self.treasury.quasi_dates;
        let start = self.next_quasi_index.checked_sub(1).and_then(|i| q.get(i));
        match (start, q.get(self.next_quasi_index)) {
            (Some(&start), Some(&end)) => Ok(SettlementContext {
                date: self.settlement_date,
                quasi_dates: (start, end),
            }),
            _ => Err(site.capacity("derive source")),
        }
    }

    /// Copies the published count and civil pieces already prepared by the plan.
    fn accrued_context(&self, site: Site) -> Result<AccruedContext, TreasuryError> {
        let first = self.treasury.first_coupon_quasi_index;
        let quasi_index = first.max(self.next_quasi_index);
        let date = self
            .treasury
            .quasi_dates
            .get(quasi_index)
            .copied()
            .ok_or_else(|| site.capacity("derive source"))?;
        Ok(AccruedContext {
            settlement: self.settlement_context(site)?,
            coupon: SelectedCoupon {
                index: quasi_index - first,
                date,
            },
            pieces: self.accrued_source.pieces,
            published_count: self.accrued_source.published_count,
        })
    }
}

impl Snapshot {
    /// Copies only compact owned instrument provenance.
    fn new(treasury: &UsTreasury, site: Site) -> Result<Self, TreasuryError> {
        match (treasury.quasi_dates.first(), treasury.quasi_dates.get(1)) {
            (Some(&start), Some(&end)) => Ok(Self {
                coupon_per_100: treasury.coupon_per_100,
                original_dated_date: treasury.original_dated_date,
                original_issue_date: treasury.original_issue_date,
                first_coupon_date: treasury.first_coupon_date,
                maturity_date: treasury.maturity_date,
                first_period: treasury.first_period,
                end_of_month: treasury.end_of_month,
                source_policy: treasury.source_policy,
                dated_quasi_dates: (start, end),
            }),
            _ => Err(site.capacity("derive source")),
        }
    }
}

impl TreasuryCashInstruction {
    /// Evaluates every required exact stage and admits only final cash.
    ///
    /// Nonpoint continuous variation at a required Exact stage returns
    /// [`TreasuryCashDecision::ExactStageUnresolved`]. Unequal admitted final
    /// extrema return [`TreasuryCashDecision::QuantumAmbiguous`]. Capital
    /// precedes accrued in a component invoice, and earlier Exact failures
    /// cannot be rescued by a later cancellation.
    ///
    /// # Errors
    /// [`TreasuryError::CashArithmetic`] reports point Inexact or an
    /// out-of-range canonical final value, with operation `quantize` or `sum`
    /// at `cash net`; defensive capacity failures remain contextual.
    pub fn evaluate(&self) -> Result<TreasuryCashDecision, TreasuryError> {
        match self.evaluate_inner() {
            Ok(value) => Ok(value),
            Err(Failure::Unresolved(stage)) => {
                Ok(TreasuryCashDecision::ExactStageUnresolved { stage })
            }
            Err(Failure::Error(error)) => Err(error),
        }
    }

    /// Returns the immutable named source/model identifier.
    #[must_use]
    pub const fn source_version(&self) -> &'static str {
        SOURCE_VERSION
    }

    /// Returns the recipe's exact stable identifier.
    #[must_use]
    pub const fn recipe_identifier(&self) -> &'static str {
        match self.recipe {
            Recipe::Coupon { .. } => "SourceCouponPer1000",
            Recipe::Accrued(_) => "PublishedAccruedPer1000",
            Recipe::Invoice { .. } => "CleanPricePer100AndPublishedAccruedPer1000",
            Recipe::Dirty { .. } => "DirtyPricePer100",
        }
    }

    /// Returns the retained explicit daily-basis and source tie policy.
    #[must_use]
    pub const fn source_policy(&self) -> TreasurySourcePolicy {
        self.snapshot.source_policy
    }

    /// Returns the strictly positive original par nominal.
    #[must_use]
    pub const fn nominal(&self) -> DecimalAmount {
        self.nominal
    }

    /// Returns the explicit cash quantum and signed rounding mode.
    #[must_use]
    pub const fn cash_rounding(&self) -> SettlementRounding {
        self.cash
    }

    /// Returns the original clean/dirty evidence, or None for pure source cash.
    #[must_use]
    pub const fn price_evidence(&self) -> Option<SettlementEvidence> {
        match self.recipe {
            Recipe::Invoice { clean, .. } => Some(clean),
            Recipe::Dirty { dirty, .. } => Some(dirty),
            Recipe::Coupon { .. } | Recipe::Accrued(_) => None,
        }
    }

    /// Returns the invoice aggregation, or None for a single-source recipe.
    #[must_use]
    pub const fn aggregation(&self) -> Option<CashAggregation> {
        match self.recipe {
            Recipe::Invoice { aggregation, .. } => Some(aggregation),
            Recipe::Coupon { .. } | Recipe::Accrued(_) | Recipe::Dirty { .. } => None,
        }
    }

    /// Returns the full exact annual coupon per100.
    #[must_use]
    pub const fn coupon_per_100(&self) -> DecimalAmount {
        self.snapshot.coupon_per_100
    }

    /// Returns the original dated date used by fractional source values.
    #[must_use]
    pub const fn original_dated_date(&self) -> Date {
        self.snapshot.original_dated_date
    }

    /// Returns the distinct original issue date.
    #[must_use]
    pub const fn original_issue_date(&self) -> Date {
        self.snapshot.original_issue_date
    }

    /// Returns the original first actual coupon date.
    #[must_use]
    pub const fn first_coupon_date(&self) -> Date {
        self.snapshot.first_coupon_date
    }

    /// Returns the original maturity date.
    #[must_use]
    pub const fn maturity_date(&self) -> Date {
        self.snapshot.maturity_date
    }

    /// Returns the inferred original first-period role.
    #[must_use]
    pub const fn first_period(&self) -> TreasuryFirstPeriod {
        self.snapshot.first_period
    }

    /// Returns the explicit direct-cycle month-end policy.
    #[must_use]
    pub const fn end_of_month(&self) -> EndOfMonth {
        self.snapshot.end_of_month
    }

    /// Returns the minimal dated pair `(Q0,Q1)`, where `Q0 <= D < Q1`.
    #[must_use]
    pub const fn dated_quasi_dates(&self) -> (Date, Date) {
        self.snapshot.dated_quasi_dates
    }

    /// Returns prepared settlement, or None for a selected coupon instruction.
    #[must_use]
    pub const fn settlement_date(&self) -> Option<Date> {
        match self.settlement() {
            Some(context) => Some(context.date),
            None => None,
        }
    }

    /// Returns the containing settlement quasi pair, or None for coupon cash.
    #[must_use]
    pub const fn settlement_quasi_dates(&self) -> Option<(Date, Date)> {
        match self.settlement() {
            Some(context) => Some(context.quasi_dates),
            None => None,
        }
    }

    /// Returns the selected actual coupon ordinal, or None for dirty-only cash.
    /// Accrued and invoice recipes retain the next actual coupon after S.
    #[must_use]
    pub const fn source_coupon_index(&self) -> Option<usize> {
        match self.coupon() {
            Some(coupon) => Some(coupon.index),
            None => None,
        }
    }

    /// Returns the selected actual coupon date, or None for dirty-only cash.
    #[must_use]
    pub const fn source_coupon_date(&self) -> Option<Date> {
        match self.coupon() {
            Some(coupon) => Some(coupon.date),
            None => None,
        }
    }

    /// Returns original `(fractional_days,half_year_days)` source ingredients.
    ///
    /// Coupon cash has one ratio only for first Short/Long coupons; regular
    /// coupons and dirty-only cash return None. Accrued/invoice retain one or
    /// two pieces, including a zero piece at an admitted lifecycle boundary.
    #[must_use]
    #[expect(
        clippy::type_complexity,
        reason = "the adopted public ledger specifies the exact two-piece tuple"
    )]
    pub const fn source_day_ratios(&self) -> Option<((u32, u32), Option<(u32, u32)>)> {
        match self.recipe {
            Recipe::Coupon { day_ratio, .. } => match day_ratio {
                Some(ratio) => Some((ratio, None)),
                None => None,
            },
            Recipe::Accrued(accrued) | Recipe::Invoice { accrued, .. } => Some(accrued.pieces),
            Recipe::Dirty { .. } => None,
        }
    }

    /// Quantizes the selected exact source per100 once, without nominal scaling.
    ///
    /// This requested view is independent of evaluation and retains no policy
    /// change. Coupon views select the source coupon; accrued/invoice views use
    /// the total-first published amount, never raw AI. Dirty cash returns
    /// `Ok(None)` without running a quantizer.
    ///
    /// # Errors
    /// [`TreasuryError::CashArithmetic`] at point in `source coupon` or
    /// `source accrued` for Inexact/out-of-range, or defensive capacity errors.
    pub fn source_value_per_100(
        &self,
        quantum: AmountQuantum,
        mode: AmountRounding,
    ) -> Result<Option<DecimalAmount>, TreasuryError> {
        let (source, site) = match self.recipe {
            Recipe::Coupon { coupon, source, .. } => {
                (source, Site::point(SOURCE_COUPON, Some(coupon.index)))
            }
            Recipe::Accrued(accrued) | Recipe::Invoice { accrued, .. } => (
                published_formula(accrued.published_count),
                Site::point(SOURCE_ACCRUED, None),
            ),
            Recipe::Dirty { .. } => return Ok(None),
        };
        let coefficient =
            quantize_source(source, None, SettlementRounding::new(quantum, mode), site)?;
        admit(
            coefficient,
            false,
            quantum.amount().scale(),
            site,
            "quantize",
        )
        .map(Some)
    }

    /// The compact settlement context of any plan recipe.
    const fn settlement(&self) -> Option<SettlementContext> {
        match self.recipe {
            Recipe::Accrued(accrued) | Recipe::Invoice { accrued, .. } => Some(accrued.settlement),
            Recipe::Dirty { settlement, .. } => Some(settlement),
            Recipe::Coupon { .. } => None,
        }
    }

    /// Selected coupon metadata, absent for dirty-only cash.
    const fn coupon(&self) -> Option<SelectedCoupon> {
        match self.recipe {
            Recipe::Coupon { coupon, .. } => Some(coupon),
            Recipe::Accrued(accrued) | Recipe::Invoice { accrued, .. } => Some(accrued.coupon),
            Recipe::Dirty { .. } => None,
        }
    }

    /// Executes only the required stages, in the adopted order.
    fn evaluate_inner(&self) -> Result<TreasuryCashDecision, Failure> {
        match self.recipe {
            Recipe::Coupon { coupon, source, .. } => {
                let site = Site::point(CASH_COUPON, Some(coupon.index));
                let coefficient = quantize_source(source, Some(self.nominal), self.cash, site)?;
                let amount = admit(
                    coefficient,
                    false,
                    self.cash.quantum().amount().scale(),
                    site,
                    "quantize",
                )?;
                Ok(self.determined(amount, None))
            }
            Recipe::Accrued(accrued) => {
                let site = Site::point(CASH_ACCRUED, None);
                let coefficient = quantize_source(
                    published_formula(accrued.published_count),
                    Some(self.nominal),
                    self.cash,
                    site,
                )?;
                let amount = admit(
                    coefficient,
                    false,
                    self.cash.quantum().amount().scale(),
                    site,
                    "quantize",
                )?;
                Ok(self.determined(amount, None))
            }
            Recipe::Dirty { dirty, .. } => {
                let rounded =
                    self.round_price(&Image::decode(dirty, CASH_TOTAL)?, 2, CASH_TOTAL)?;
                self.finish(rounded, CASH_TOTAL, "quantize", dirty)
            }
            Recipe::Invoice {
                accrued,
                clean,
                aggregation,
            } => self.invoice(accrued, clean, aggregation),
        }
    }

    /// Evaluates a total or component invoice without public intermediate gates.
    fn invoice(
        &self,
        accrued: AccruedContext,
        clean: SettlementEvidence,
        aggregation: CashAggregation,
    ) -> Result<TreasuryCashDecision, Failure> {
        match aggregation {
            CashAggregation::RoundTotal => {
                // The per1000 invoice form is selected by recipe, even for AI0.
                let total = Image::decode(clean, CASH_TOTAL)?
                    .invoice_total(accrued.published_count, CASH_TOTAL)?;
                let rounded = self.round_price(&total, 3, CASH_TOTAL)?;
                self.finish(rounded, CASH_TOTAL, "quantize", clean)
            }
            CashAggregation::RoundComponentsThenSum => {
                // Complete capital Exact checks before touching accrued Exact.
                let capital =
                    self.round_price(&Image::decode(clean, CASH_CAPITAL)?, 2, CASH_CAPITAL)?;
                let accrued = quantize_source(
                    published_formula(accrued.published_count),
                    Some(self.nominal),
                    self.cash,
                    Site::point(CASH_ACCRUED, None),
                )?
                .resize::<40>()
                .ok_or_else(|| capacity(None, CASH_ACCRUED, "scale cash"))?;
                let accrued = Signed::new(accrued, false);
                let lower = capital
                    .lower
                    .add(accrued)
                    .ok_or_else(|| capacity(None, CASH_NET, "combine cash"))?;
                let upper = capital
                    .upper
                    .add(accrued)
                    .ok_or_else(|| capacity(None, CASH_NET, "combine cash"))?;
                self.finish(
                    Rounded {
                        lower,
                        upper,
                        point: lower == upper,
                    },
                    CASH_NET,
                    "sum",
                    clean,
                )
            }
        }
    }

    /// Rounds endpoint integers on R0 after one fused decimal scaling.
    /// `unit_places=2` for capital/dirty; 3 for the per1000 total invoice.
    fn round_price(
        &self,
        image: &Image,
        unit_places: i32,
        stage: &'static str,
    ) -> Result<Rounded, Failure> {
        let point = image.is_point();
        if self.cash.mode() == AmountRounding::Exact && !point {
            return Err(Failure::Unresolved(stage));
        }
        let site = Site::point(stage, None);
        let q = self.cash.quantum().amount();
        let exponent = i32::from(q.scale()) - i32::from(self.nominal.scale()) - unit_places;
        let denominator = decimal_unit(0)
            .and_then(|basis| {
                basis.checked_mul(PriceInt::from_u128(q.coefficient().unsigned_abs()))
            })
            .and_then(|value| scale_power(value, (-exponent).max(0)))
            .ok_or_else(|| site.capacity("scale cash"))?;
        let round = |value: &Signed<40>, endpoint: &'static str| {
            let site = Site { endpoint, ..site };
            let numerator = value
                .magnitude
                .checked_mul(PriceInt::from_u128(
                    self.nominal.coefficient().unsigned_abs(),
                ))
                .and_then(|value| scale_power(value, exponent.max(0)))
                .ok_or_else(|| site.capacity("scale cash"))?;
            let count = round_ratio(
                numerator,
                &denominator,
                value.negative,
                self.cash.mode(),
                site,
            )?;
            let coefficient = count
                .checked_mul(PriceInt::from_u128(q.coefficient().unsigned_abs()))
                .ok_or_else(|| site.capacity("cash rounding"))?;
            Ok::<_, TreasuryError>(Signed::new(coefficient, value.negative))
        };
        let lower = round(&image.lower, if point { "point" } else { "lower" })?;
        let upper = if point {
            lower
        } else {
            round(&image.upper, "upper")?
        };
        Ok(Rounded {
            lower,
            upper,
            point,
        })
    }

    /// Admits final lower, then upper when the reported stage image is nonpoint.
    fn finish(
        &self,
        rounded: Rounded,
        stage: &'static str,
        operation: &'static str,
        evidence: SettlementEvidence,
    ) -> Result<TreasuryCashDecision, Failure> {
        let scale = self.cash.quantum().amount().scale();
        let site = Site {
            stage,
            endpoint: if rounded.point { "point" } else { "lower" },
            index: None,
        };
        let lower = admit(
            rounded.lower.magnitude,
            rounded.lower.negative,
            scale,
            site,
            operation,
        )?;
        let upper = if rounded.point {
            lower
        } else {
            admit(
                rounded.upper.magnitude,
                rounded.upper.negative,
                scale,
                Site {
                    endpoint: "upper",
                    ..site
                },
                operation,
            )?
        };
        if lower == upper {
            Ok(self.determined(lower, Some(guarantee(evidence))))
        } else {
            Ok(TreasuryCashDecision::QuantumAmbiguous {
                lower_amount: lower,
                upper_amount: upper,
            })
        }
    }

    /// Retains this exact complete instruction in a determined receipt.
    const fn determined(
        &self,
        amount: DecimalAmount,
        guarantee: Option<SettlementGuarantee>,
    ) -> TreasuryCashDecision {
        TreasuryCashDecision::Determined(TreasuryCashReceipt {
            amount,
            instruction: *self,
            guarantee,
        })
    }
}

/// First required invoice stage, also used for defensive snapshot contexts.
const fn first_invoice_stage(aggregation: CashAggregation) -> &'static str {
    match aggregation {
        CashAggregation::RoundTotal => CASH_TOTAL,
        CashAggregation::RoundComponentsThenSum => CASH_CAPITAL,
    }
}

/// Positive nominal validation before all source/snapshot work.
fn positive_nominal(nominal: DecimalAmount) -> Result<(), TreasuryError> {
    if nominal.coefficient() <= 0 {
        Err(TreasuryError::NonPositiveNominal { nominal })
    } else {
        Ok(())
    }
}

/// Price-ingredient provenance only, independent of original interval width.
fn guarantee(evidence: SettlementEvidence) -> SettlementGuarantee {
    if evidence.is_declared_interval() {
        SettlementGuarantee::UniqueForDeclaredEnclosure
    } else if evidence.binary64_bounds().is_some() {
        SettlementGuarantee::ForSuppliedBinary64
    } else {
        SettlementGuarantee::ForSuppliedDecimals
    }
}

/// Accrued publication count interpreted as an exact per1000 source formula.
const fn published_formula(count: Scalar) -> SourceFormula {
    SourceFormula {
        x: count,
        dscale: 5,
        s_part: 1,
    }
}

/// Typed error or continuous Exact-domain outcome, preserving stage order.
enum Failure {
    /// A public Treasury error.
    Error(TreasuryError),
    /// First required quantizer with nonpoint continuous variation under Exact.
    Unresolved(&'static str),
}

impl From<TreasuryError> for Failure {
    fn from(error: TreasuryError) -> Self {
        Self::Error(error)
    }
}

/// Stable monetary phase, endpoint and optional actual-coupon capacity index.
#[derive(Clone, Copy)]
struct Site {
    /// Stable public cash/source-view stage.
    stage: &'static str,
    /// Point/lower/upper numeric image role.
    endpoint: &'static str,
    /// Actual coupon ordinal for coupon work; None otherwise.
    index: Option<usize>,
}

impl Site {
    /// One exact source point or a later-refined price endpoint site.
    const fn point(stage: &'static str, index: Option<usize>) -> Self {
        Self {
            stage,
            endpoint: "point",
            index,
        }
    }

    /// Defensive proof-excluded fixed-width failure.
    fn capacity(self, operation: &'static str) -> TreasuryError {
        capacity(self.index, self.stage, operation)
    }

    /// CashArithmetic preserves the immediate typed AmountError cause.
    const fn arithmetic(self, cause: AmountError) -> TreasuryError {
        TreasuryError::CashArithmetic {
            stage: self.stage,
            endpoint: self.endpoint,
            cause,
        }
    }
}

/// Multiplies by a checked decimal power, with no intermediate division.
fn scale_power<const N: usize>(value: Wide<N>, exponent: i32) -> Option<Wide<N>> {
    let exponent = u16::try_from(exponent).ok()?;
    value.checked_mul(Wide::pow10(exponent)?)
}

/// One raw per100 rational view, independent of selected source publication.
fn quantize_rational(
    value: SourceRational,
    quantum: AmountQuantum,
    mode: AmountRounding,
    site: Site,
) -> Result<DecimalAmount, TreasuryError> {
    let q = quantum.amount();
    let numerator = value
        .numerator
        .resize::<8>()
        .and_then(|value| scale_power(value, i32::from(q.scale())))
        .ok_or_else(|| site.capacity("scale cash"))?;
    let denominator = value
        .denominator
        .resize::<8>()
        .and_then(|value| value.checked_mul(SourceInt::from_u128(q.coefficient().unsigned_abs())))
        .ok_or_else(|| site.capacity("scale cash"))?;
    let count = round_ratio(numerator, &denominator, false, mode, site)?;
    let coefficient = count
        .checked_mul(SourceInt::from_u128(q.coefficient().unsigned_abs()))
        .ok_or_else(|| site.capacity("cash rounding"))?;
    admit(coefficient, false, q.scale(), site, "quantize")
}

/// Fuses all powers for a per1000 source, returning a wide cash-quantum coefficient.
///
/// With nominal the exponent is `Qscale-Nscale-3-dscale`; a source view uses
/// `Qscale-1-dscale`. The returned coefficient is deliberately not admitted:
/// component cancellation may rescue an otherwise out-of-range result.
fn quantize_source(
    source: SourceFormula,
    nominal: Option<DecimalAmount>,
    rounding: SettlementRounding,
    site: Site,
) -> Result<SourceInt, TreasuryError> {
    let q = rounding.quantum().amount();
    let (factor, nominal_scale) = match nominal {
        Some(n) => (n.coefficient().unsigned_abs(), i32::from(n.scale()) + 3),
        None => (1, 1),
    };
    let exponent = i32::from(q.scale()) - nominal_scale - i32::from(source.dscale);
    let numerator = source
        .x
        .resize::<8>()
        .and_then(|value| value.checked_mul(SourceInt::from_u128(factor)))
        .and_then(|value| scale_power(value, exponent.max(0)))
        .ok_or_else(|| site.capacity("scale cash"))?;
    let denominator = SourceInt::from_u128(u128::from(source.s_part))
        .checked_mul(SourceInt::from_u128(q.coefficient().unsigned_abs()))
        .and_then(|value| scale_power(value, (-exponent).max(0)))
        .ok_or_else(|| site.capacity("scale cash"))?;
    let count = round_ratio(numerator, &denominator, false, rounding.mode(), site)?;
    count
        .checked_mul(SourceInt::from_u128(q.coefficient().unsigned_abs()))
        .ok_or_else(|| site.capacity("cash rounding"))
}

/// Returns the integer `R0/10^scale`; the supported scale is at most38.
fn decimal_unit(scale: u8) -> Option<PriceInt> {
    38_u8
        .checked_sub(scale)
        .and_then(|e| PriceInt::pow10(u16::from(e)))
        .and_then(|value| value.checked_shl(BASIS_SHIFT))
}

/// A signed wide integer, with a canonical positive zero.
#[derive(Clone, Copy, Eq, PartialEq)]
struct Signed<const N: usize> {
    /// Nonnegative magnitude.
    magnitude: Wide<N>,
    /// Negative iff the magnitude is nonzero and signed negative.
    negative: bool,
}

impl<const N: usize> Signed<N> {
    /// Canonicalizes the sign of zero.
    fn new(magnitude: Wide<N>, negative: bool) -> Self {
        Self {
            magnitude,
            negative: negative && !magnitude.is_zero(),
        }
    }

    /// Exact addition, retaining cancellation before any public range gate.
    fn add(self, other: Self) -> Option<Self> {
        if self.negative == other.negative {
            Some(Self::new(
                self.magnitude.checked_add(other.magnitude)?,
                self.negative,
            ))
        } else if self.magnitude >= other.magnitude {
            Some(Self::new(
                self.magnitude.checked_sub(other.magnitude)?,
                self.negative,
            ))
        } else {
            Some(Self::new(
                other.magnitude.checked_sub(self.magnitude)?,
                other.negative,
            ))
        }
    }
}

/// Exact ordered endpoint integers on R0, including continuous interiors.
#[derive(Clone, Copy)]
struct Image {
    /// Least exact endpoint coefficient.
    lower: Signed<40>,
    /// Greatest exact endpoint coefficient.
    upper: Signed<40>,
}

impl Image {
    /// Numeric pointness is independent of evidence declaration provenance.
    fn is_point(&self) -> bool {
        self.lower == self.upper
    }

    /// Decodes validated decimal/binary evidence exactly onto R0.
    fn decode(evidence: SettlementEvidence, stage: &'static str) -> Result<Self, TreasuryError> {
        let site = Site::point(stage, None);
        if let Some((lower, upper)) = evidence.decimal_bounds() {
            Ok(Self {
                lower: decode_decimal(lower, site)?,
                upper: decode_decimal(upper, site)?,
            })
        } else if let Some((lower, upper)) = evidence.binary64_bounds() {
            Ok(Self {
                lower: decode_binary(lower.to_bits(), site)?,
                upper: decode_binary(upper.to_bits(), site)?,
            })
        } else {
            Err(site.capacity("decode evidence"))
        }
    }

    /// Changes per100 price to the per1000 invoice total, including AI0.
    fn invoice_total(self, published: Scalar, stage: &'static str) -> Result<Self, TreasuryError> {
        let site = Site::point(stage, None);
        let source = published
            .resize::<40>()
            .and_then(|value| value.checked_mul(decimal_unit(5)?))
            .map(|value| Signed::new(value, false))
            .ok_or_else(|| site.capacity("combine cash"))?;
        let shift = |value: Signed<40>| {
            let price = value
                .magnitude
                .checked_mul_small(10)
                .map(|magnitude| Signed::new(magnitude, value.negative))
                .ok_or_else(|| site.capacity("combine cash"))?;
            price
                .add(source)
                .ok_or_else(|| site.capacity("combine cash"))
        };
        Ok(Self {
            lower: shift(self.lower)?,
            upper: shift(self.upper)?,
        })
    }
}

/// Final quantum-scale coefficients and reported-stage input pointness.
#[derive(Clone, Copy)]
struct Rounded {
    /// Least rounded coefficient, still outside the public amount domain if needed.
    lower: Signed<40>,
    /// Greatest rounded coefficient.
    upper: Signed<40>,
    /// True iff the reported final stage input is a point.
    point: bool,
}

/// Decimal coefficient/10^scale becomes coefficient*(R0/10^scale).
fn decode_decimal(value: DecimalAmount, site: Site) -> Result<Signed<40>, TreasuryError> {
    let magnitude = decimal_unit(value.scale())
        .and_then(|unit| unit.checked_mul(PriceInt::from_u128(value.coefficient().unsigned_abs())))
        .ok_or_else(|| site.capacity("decode evidence"))?;
    Ok(Signed::new(magnitude, value.coefficient() < 0))
}

/// Exact finite `M*2^e` becomes `M*5^38*2^(e+1074)`, including subnormals.
fn decode_binary(bits: u64, site: Site) -> Result<Signed<40>, TreasuryError> {
    let field = (bits >> 52) & 0x7ff;
    let fraction = bits & ((1_u64 << 52) - 1);
    let (significand, shift) = if field == 0 {
        (fraction, 0)
    } else {
        (
            fraction | (1_u64 << 52),
            u32::try_from(field - 1).map_err(|_| site.capacity("decode evidence"))?,
        )
    };
    let magnitude = PriceInt::from_u128(u128::from(significand))
        .checked_mul(PriceInt::from_u128(5_u128.pow(38)))
        .and_then(|value| value.checked_shl(shift))
        .ok_or_else(|| site.capacity("decode evidence"))?;
    Ok(Signed::new(magnitude, bits >> 63 != 0))
}

/// Signed seven-mode rational rounding; ties compare r with d-r, never2*r.
/// The denominator is borrowed so a caller-retained endpoint divisor is not
/// copied into this frame while Wide division performs its bounded work.
fn round_ratio<const N: usize>(
    numerator: Wide<N>,
    denominator: &Wide<N>,
    negative: bool,
    mode: AmountRounding,
    site: Site,
) -> Result<Wide<N>, TreasuryError> {
    let (mut count, remainder) = numerator
        .div_rem(*denominator)
        .ok_or_else(|| site.capacity("cash rounding"))?;
    if remainder.is_zero() {
        return Ok(count);
    }
    if mode == AmountRounding::Exact {
        return Err(site.arithmetic(AmountError::Inexact {
            operation: "quantize",
        }));
    }
    let complement = denominator
        .checked_sub(remainder)
        .ok_or_else(|| site.capacity("cash rounding"))?;
    let increment = match mode {
        AmountRounding::TowardZero | AmountRounding::Exact => false,
        AmountRounding::AwayFromZero => true,
        AmountRounding::TowardPositiveInfinity => !negative,
        AmountRounding::TowardNegativeInfinity => negative,
        AmountRounding::NearestTiesEven => {
            remainder > complement || (remainder == complement && count.is_odd())
        }
        AmountRounding::NearestTiesAwayFromZero => remainder >= complement,
    };
    if increment {
        count = count
            .checked_add(Wide::from_u128(1))
            .ok_or_else(|| site.capacity("cash rounding"))?;
    }
    Ok(count)
}

/// Canonicalizes fully before the asymmetric signed-i128 admission check.
fn admit<const N: usize>(
    magnitude: Wide<N>,
    negative: bool,
    scale: u8,
    site: Site,
    operation: &'static str,
) -> Result<DecimalAmount, TreasuryError> {
    let negative = negative && !magnitude.is_zero();
    let mut magnitude = magnitude;
    let mut scale = if magnitude.is_zero() { 0 } else { scale };
    while scale > 0 {
        let (quotient, remainder) = magnitude
            .div_rem_small(10)
            .ok_or_else(|| site.capacity("normalize cash"))?;
        if remainder != 0 {
            break;
        }
        magnitude = quotient;
        scale -= 1;
    }
    let range = || {
        site.arithmetic(AmountError::OutOfRange {
            operation,
            negative,
        })
    };
    let magnitude = magnitude.to_u128().ok_or_else(range)?;
    let coefficient = if negative && magnitude == 1_u128 << 127 {
        i128::MIN
    } else {
        let coefficient = i128::try_from(magnitude).map_err(|_| range())?;
        if negative { -coefficient } else { coefficient }
    };
    DecimalAmount::new(coefficient, scale).map_err(|_| site.capacity("normalize cash"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amount(text: &str) -> DecimalAmount {
        text.parse().unwrap()
    }

    fn rounding(q: i128, scale: u8, mode: AmountRounding) -> SettlementRounding {
        SettlementRounding::new(AmountQuantum::new(q, scale).unwrap(), mode)
    }

    /// FR-UST-23/24: source cents retain an unrounded coupon, while accrued
    /// nominal scaling follows the already-published per1000 count only once.
    #[test]
    fn source_coupon_anchor_and_fractional_nominal_keep_exact_stages() {
        // I.A.3: 0.227581522*182 dollars per1000, nominal20000.
        let published = SourceFormula {
            x: Scalar::from_u128(41_419_837_004),
            dscale: 9,
            s_part: 1,
        };
        let site = Site::point(CASH_COUPON, Some(0));
        let coefficient = quantize_source(
            published,
            Some(amount("20000")),
            rounding(1, 2, AmountRounding::NearestTiesEven),
            site,
        )
        .unwrap();
        assert_eq!(
            admit(coefficient, false, 2, site, "quantize").unwrap(),
            amount("828.40")
        );
        // ExactRatio source is 30485/736 per1000: no terminating 1e-9 view.
        let exact = SourceFormula {
            x: Scalar::from_u128(7_621_250),
            dscale: 3,
            s_part: 184,
        };
        let error = quantize_source(
            exact,
            None,
            rounding(1, 9, AmountRounding::Exact),
            Site::point(SOURCE_COUPON, Some(0)),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            TreasuryError::CashArithmetic {
                stage: SOURCE_COUPON,
                endpoint: "point",
                cause: AmountError::Inexact {
                    operation: "quantize"
                }
            }
        ));
        // P1000=0.00001, N300: 0.000003 cash survives a final1e-6 quantizer.
        let coefficient = quantize_source(
            published_formula(Scalar::from_u128(1)),
            Some(amount("300")),
            rounding(1, 6, AmountRounding::Exact),
            Site::point(CASH_ACCRUED, None),
        )
        .unwrap();
        assert_eq!(coefficient, SourceInt::from_u128(3));
    }

    /// FR-UST-24: total invoice always changes the price to per1000 units,
    /// including AI0; exact signed cancellation happens before admission.
    #[test]
    fn invoice_zero_accrual_and_signed_cancellation_preserve_units() {
        let price =
            Image::decode(SettlementEvidence::exact_decimal(amount("100")), CASH_TOTAL).unwrap();
        let total = price.invoice_total(Scalar::ZERO, CASH_TOTAL).unwrap();
        assert!(total.is_point());
        assert_eq!(
            total.lower.magnitude,
            decimal_unit(0).unwrap().checked_mul_small(1000).unwrap()
        );
        assert!(!total.lower.negative);
        // clean=-1e-5 per100, P_count=10 -> -1e-4+1e-4 per1000.
        let zero = Image::decode(
            SettlementEvidence::exact_decimal(amount("-0.00001")),
            CASH_TOTAL,
        )
        .unwrap()
        .invoice_total(Scalar::from_u128(10), CASH_TOTAL)
        .unwrap();
        assert!(zero.is_point() && zero.lower.magnitude.is_zero() && !zero.lower.negative);
    }

    /// FR-UST-25: denominator397 bits is an admitted coefficient/scale extreme,
    /// not an i128 intermediate; signed count ties and asymmetric final limits
    /// stay exact independently of source observations.
    #[test]
    fn widest_source_denominator_signed_ties_and_final_limits() {
        let denominator = SourceInt::from_u128(i128::MAX.unsigned_abs())
            .checked_mul_small(184)
            .unwrap()
            .checked_mul(SourceInt::pow10(79).unwrap())
            .unwrap();
        assert_eq!(denominator.bits(), 397);
        let source = SourceFormula {
            x: Scalar::from_u128(i128::MAX.unsigned_abs())
                .checked_mul_small(5 * 183)
                .unwrap(),
            dscale: 38,
            s_part: 184,
        };
        let coefficient = quantize_source(
            source,
            Some(DecimalAmount::new(1, 38).unwrap()),
            rounding(i128::MAX, 0, AmountRounding::NearestTiesEven),
            Site::point(CASH_COUPON, Some(0)),
        )
        .unwrap();
        assert!(coefficient.is_zero());
        let site = Site::point(CASH_TOTAL, None);
        let denominator = SourceInt::from_u128(2);
        for (n, negative, mode, expected) in [
            (5, true, AmountRounding::NearestTiesEven, 2),
            (7, true, AmountRounding::NearestTiesEven, 4),
            (5, true, AmountRounding::NearestTiesAwayFromZero, 3),
            (5, true, AmountRounding::TowardPositiveInfinity, 2),
            (5, true, AmountRounding::TowardNegativeInfinity, 3),
        ] {
            assert_eq!(
                round_ratio(SourceInt::from_u128(n), &denominator, negative, mode, site).unwrap(),
                SourceInt::from_u128(expected)
            );
        }
        let min = PriceInt::from_u128(1_u128 << 127)
            .checked_mul_small(10)
            .unwrap();
        assert_eq!(
            admit(min, true, 1, site, "sum").unwrap().coefficient(),
            i128::MIN
        );
        assert!(matches!(
            admit(
                PriceInt::from_u128(1_u128 << 127),
                false,
                0,
                site,
                "quantize"
            ),
            Err(TreasuryError::CashArithmetic {
                cause: AmountError::OutOfRange {
                    negative: false,
                    ..
                },
                ..
            })
        ));
    }
}
