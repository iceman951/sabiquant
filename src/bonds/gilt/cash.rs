//! Exact conventional-gilt cash recipes and their compact owned provenance.
//!
//! The four recipes are a published actual dividend, the exact source accrued
//! amount, a clean-price-plus-source-accrued invoice and a dirty-price trade
//! payment. Source values remain exact rationals: the full annual coupon times
//! an unreduced civil day ratio, or a six-place published micro coefficient.
//! They never pass through a nearest float, a decimal point value or a binary
//! enclosure before final cash is decided.
//!
//! Pure source cash combines every decimal power, including the per100
//! divisor, into one eight-limb quantization ratio. Price evidence uses the
//! common basis `Dc = 2^1074 * 5^38 * dayD` in forty limbs, where the source
//! accrued denominator is `2 * dayD` (`dayD = 1` for a dirty trade). Only the
//! final cash coefficients are admitted to `DecimalAmount`. Instructions,
//! decisions and receipts are fixed-size `Copy` values: evaluation allocates no
//! heap and performs no date, calendar, callback or clock work.

use super::{
    ConventionalGilt, ExactSource, GiltError, GiltFirstPeriod, GiltPricingPlan, SOURCE_VERSION,
    capacity,
};
use crate::amounts::uint::Wide;
use crate::amounts::{AmountError, AmountQuantum, AmountRounding, DecimalAmount};
use crate::calendar::EndOfMonth;
use crate::settlement::{
    CashAggregation, SettlementEvidence, SettlementGuarantee, SettlementRounding,
};
use crate::time::Date;
use std::fmt;

/// Eight-limb (512-bit) pure exact-source magnitudes; every bound is below 405 bits.
type SourceInt = Wide<8>;

/// Forty-limb (2,560-bit) price-evidence magnitudes; every bound is below 2,460 bits.
type PriceInt = Wide<40>;

/// Explicitly requested quantized view of an exact source dividend.
const SOURCE_DIVIDEND: &str = "source dividend";
/// Explicitly requested quantized view of exact source accrued interest.
const SOURCE_ACCRUED: &str = "source accrued";
/// Final rounding of published dividend cash.
const CASH_COUPON: &str = "cash coupon";
/// Rounding of exact source accrued cash.
const CASH_ACCRUED: &str = "cash accrued";
/// Rounding of the clean-price capital component.
const CASH_CAPITAL: &str = "cash capital";
/// Final rounding of a dirty trade or a total invoice.
const CASH_TOTAL: &str = "cash total";
/// Exact sum of separately rounded invoice components.
const CASH_NET: &str = "cash net";

/// Binary exponent of the common basis `R0 = 2^1074 * 5^38 = 10^38 * 2^1036`.
const BASIS_SHIFT: u32 = 1036;
/// Decimal exponent of the common basis `R0`.
const BASIS_DECIMALS: u8 = 38;

/// A complete exact gilt cash recipe with its compact owned source snapshot.
///
/// Equality and hashing retain the recipe, the exact rational source value,
/// the original coupon, lifecycle dates, end-of-month policy, the issue and
/// settlement quasi pairs, the selected actual dividend and its cutoff, the
/// original price-evidence kind and bounds, nominal and every cash policy.
/// Equal cash outcomes therefore never erase provenance. The snapshot is not a
/// full instrument clone, and equality does not claim instrument equality.
///
/// `Debug` shows the recipe, the retained public context, the source formula
/// kind and every policy. It omits private wide source buffers and performs no
/// source quantization; [`source_value_per_100`](Self::source_value_per_100)
/// is the explicit quantized view.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct GiltCashInstruction {
    /// Retained instrument provenance shared by every recipe.
    snapshot: Snapshot,
    /// Strictly positive exact par nominal.
    nominal: DecimalAmount,
    /// Explicit final and component cash quantizer.
    cash: SettlementRounding,
    /// Recipe-specific exact source and evidence.
    recipe: Recipe,
}

impl fmt::Debug for GiltCashInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GiltCashInstruction")
            .field("recipe", &self.recipe_identifier())
            .field("source_version", &SOURCE_VERSION)
            .field("coupon_per_100", &self.snapshot.coupon_per_100)
            .field("issue_date", &self.snapshot.issue_date)
            .field("maturity_date", &self.snapshot.maturity_date)
            .field("first_dividend_date", &self.snapshot.first_dividend_date)
            .field("first_period", &self.snapshot.first_period)
            .field("end_of_month", &self.snapshot.end_of_month)
            .field("issue_quasi_dates", &self.snapshot.issue_quasi_dates)
            .field("settlement_date", &self.settlement_date())
            .field("settlement_quasi_dates", &self.settlement_quasi_dates())
            .field("source_dividend_index", &self.source_dividend_index())
            .field("source_dividend_date", &self.source_dividend_date())
            .field("ex_dividend_date", &self.ex_dividend_date())
            .field("is_ex_dividend", &self.is_ex_dividend())
            .field("source", &self.source().map(SourceFormula))
            .field("nominal", &self.nominal)
            .field("cash_rounding", &self.cash)
            .field("price_evidence", &self.price_evidence())
            .field("aggregation", &self.aggregation())
            .finish()
    }
}

/// Readable exact source formula; a published value's wide coefficient is omitted.
struct SourceFormula(ExactSource);

impl fmt::Debug for SourceFormula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ExactSource::Unrounded {
                numerator,
                denominator,
            } => f
                .debug_struct("CouponDayRatio")
                .field("numerator", &numerator)
                .field("denominator", &denominator)
                .finish(),
            ExactSource::Published { .. } => f.write_str("PublishedSixPlaceDividend"),
        }
    }
}

/// Instrument provenance retained by every cash recipe.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Snapshot {
    /// Full exact annual coupon per100.
    coupon_per_100: DecimalAmount,
    /// Original issue date.
    issue_date: Date,
    /// Original maturity date.
    maturity_date: Date,
    /// First actual dividend date.
    first_dividend_date: Date,
    /// Inferred first-period role.
    first_period: GiltFirstPeriod,
    /// Explicit end-of-month quasi-cycle policy.
    end_of_month: EndOfMonth,
    /// Validated `(Q0, Q1)` with `Q0 <= I < Q1`.
    issue_quasi_dates: (Date, Date),
}

/// Prepared settlement and its containing quasi interval `Qk <= S < Qk1`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct SettlementContext {
    /// Prepared settlement date.
    date: Date,
    /// Containing quasi pair, beginning at a settlement quasi date.
    quasi_dates: (Date, Date),
}

/// One actual dividend ordinal (excluding skipped quasi dates and redemption).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct SelectedDividend {
    /// Ordinal among actual dividends.
    index: usize,
    /// Actual dividend date.
    date: Date,
}

/// Exact source accrued interest with its next actual dividend and cutoff.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct AccruedContext {
    /// Prepared settlement provenance.
    settlement: SettlementContext,
    /// Next actual dividend, retained even when detached from the buyer.
    dividend: SelectedDividend,
    /// Supplied ex-dividend cutoff of that dividend.
    cutoff: Date,
    /// Exact unreduced source accrued amount per100.
    source: ExactSource,
}

/// The four adopted recipe families; no partial or mixed shape is representable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Recipe {
    /// `R(N*D/100)` for one exact actual dividend `D`.
    Dividend {
        /// Selected actual dividend.
        dividend: SelectedDividend,
        /// Exact regular or published first dividend per100.
        source: ExactSource,
    },
    /// `R(N*A/100)` for exact source accrued interest `A`.
    Accrued(AccruedContext),
    /// Clean price evidence plus exact source accrued interest.
    Invoice {
        /// Source accrued context.
        accrued: AccruedContext,
        /// Original clean price evidence per100.
        clean: SettlementEvidence,
        /// Explicit total or component aggregation.
        aggregation: CashAggregation,
    },
    /// `R(N*P/100)` for dirty price evidence `P`.
    Dirty {
        /// Prepared settlement provenance.
        settlement: SettlementContext,
        /// Original dirty price evidence per100.
        dirty: SettlementEvidence,
    },
}

/// A determined gilt cash amount or an explicitly unresolved recipe consequence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[expect(
    clippy::large_enum_variant,
    reason = "the allocation-free Copy receipt retains its complete owned instruction"
)]
pub enum GiltCashDecision {
    /// One representable result with its retained instruction and provenance.
    Determined(GiltCashReceipt),
    /// Different representable extremes; interior representability is not promised.
    #[non_exhaustive]
    QuantumAmbiguous {
        /// Least rounded final amount.
        lower_amount: DecimalAmount,
        /// Greatest rounded final amount.
        upper_amount: DecimalAmount,
    },
    /// A continuous nonpoint image meets a required `Exact` quantizer.
    #[non_exhaustive]
    ExactStageUnresolved {
        /// First required stage whose Exact domain is not satisfied throughout.
        stage: &'static str,
    },
}

/// A determined gilt cash amount and the complete instruction that defines it.
///
/// Identity adds the final amount and price-ingredient guarantee to the
/// instruction. No settlement receipt or guarantee is fabricated.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GiltCashReceipt {
    /// Final canonical exact cash.
    amount: DecimalAmount,
    /// Original complete instruction.
    instruction: GiltCashInstruction,
    /// Price-evidence ingredient label, or None for pure source recipes.
    guarantee: Option<SettlementGuarantee>,
}

impl GiltCashReceipt {
    /// Returns final canonical cash: positive is buyer-to-seller for trade,
    /// accrued and invoice recipes, and issuer-to-holder for dividends.
    #[must_use]
    pub const fn amount(self) -> DecimalAmount {
        self.amount
    }

    /// Borrows the complete original instruction.
    #[must_use]
    pub const fn instruction(&self) -> &GiltCashInstruction {
        &self.instruction
    }

    /// Returns the guarantee scoped only to the supplied price ingredient.
    ///
    /// Pure dividend and accrued recipes return None: their exact-source cash
    /// provenance is the retained recipe and snapshot. A price recipe's label
    /// does not claim that the supplied price equals or encloses the model's
    /// ideal price.
    #[must_use]
    pub const fn price_evidence_guarantee(self) -> Option<SettlementGuarantee> {
        self.guarantee
    }
}

impl ConventionalGilt {
    /// Quantizes one exact actual source dividend per100 to `quantum`/`mode`.
    ///
    /// Regular dividends are exactly `C/2`; first short/long dividends are the
    /// six-place published values. The full coupon domain is retained, so an
    /// exact decimal point need not exist and `Exact` can fail.
    ///
    /// # Errors
    /// [`GiltError::InvalidDividendIndex`] for an ordinal outside the actual
    /// dividends; [`GiltError::CashArithmetic`] at stage `source dividend`,
    /// endpoint `point`, for an Inexact or out-of-range quantized coefficient;
    /// or a defensive [`GiltError::InternalCapacity`].
    pub fn dividend_per_100(
        &self,
        index: usize,
        quantum: AmountQuantum,
        mode: AmountRounding,
    ) -> Result<DecimalAmount, GiltError> {
        let (source, _) = self.actual_dividend(index, SOURCE_DIVIDEND)?;
        let site = Site::point(SOURCE_DIVIDEND, Some(index));
        let value = SourceValue::new(self.coupon_per_100, source, site)?;
        quantize_source(value, None, SettlementRounding::new(quantum, mode), site)
    }

    /// Defines dividend cash `R(N*D/100)` for one selected actual dividend.
    ///
    /// Positive cash is issuer-to-holder. The caller's held nominal is assumed
    /// entitled; no record-date ownership is generated or certified.
    ///
    /// # Errors
    /// [`GiltError::InvalidDividendIndex`] first, then
    /// [`GiltError::NonPositiveNominal`]. No source arithmetic or allocation
    /// precedes these checks.
    pub fn dividend_cash(
        &self,
        index: usize,
        nominal: DecimalAmount,
        rounding: SettlementRounding,
    ) -> Result<GiltCashInstruction, GiltError> {
        let (source, date) = self.actual_dividend(index, CASH_COUPON)?;
        positive_nominal(nominal)?;
        Ok(GiltCashInstruction {
            snapshot: Snapshot::new(self, CASH_COUPON, Some(index))?,
            nominal,
            cash: rounding,
            recipe: Recipe::Dividend {
                dividend: SelectedDividend { index, date },
                source,
            },
        })
    }

    /// Selects one actual dividend's exact source and date, index first.
    fn actual_dividend(
        &self,
        index: usize,
        stage: &'static str,
    ) -> Result<(ExactSource, Date), GiltError> {
        let count = self.dividend_count();
        if index >= count {
            return Err(GiltError::InvalidDividendIndex { index, count });
        }
        let source = self.dividends.get(index).copied();
        let date = self.dividend_dates().get(index).copied();
        match (source, date) {
            (Some(source), Some(date)) => Ok((source, date)),
            _ => Err(capacity(Some(index), stage, "derive source")),
        }
    }
}

impl GiltPricingPlan<'_> {
    /// Defines accrued cash `R(N*A/100)` from the exact source accrued amount.
    ///
    /// Positive cash is buyer-to-seller; a negative ex-dividend amount
    /// reverses it.
    ///
    /// # Errors
    /// [`GiltError::NonPositiveNominal`] first, then a defensive
    /// [`GiltError::InternalCapacity`] if the prepared snapshot is inconsistent.
    pub fn accrued_cash(
        &self,
        nominal: DecimalAmount,
        rounding: SettlementRounding,
    ) -> Result<GiltCashInstruction, GiltError> {
        positive_nominal(nominal)?;
        Ok(GiltCashInstruction {
            snapshot: Snapshot::new(self.gilt, CASH_ACCRUED, None)?,
            nominal,
            cash: rounding,
            recipe: Recipe::Accrued(self.accrued_context(CASH_ACCRUED)?),
        })
    }

    /// Defines an invoice from clean price evidence per100 and exact source
    /// accrued interest under an explicit aggregation policy.
    ///
    /// `RoundTotal` rounds `N*C_p/100 + N*A/100` once. `RoundComponentsThenSum`
    /// rounds capital then accrued and exact-sums the two rounded results.
    ///
    /// # Errors
    /// [`GiltError::NonPositiveNominal`] first, then a defensive
    /// [`GiltError::InternalCapacity`] if the prepared snapshot is inconsistent.
    pub fn clean_accrued_cash(
        &self,
        nominal: DecimalAmount,
        clean: SettlementEvidence,
        aggregation: CashAggregation,
        rounding: SettlementRounding,
    ) -> Result<GiltCashInstruction, GiltError> {
        positive_nominal(nominal)?;
        let stage = first_invoice_stage(aggregation);
        Ok(GiltCashInstruction {
            snapshot: Snapshot::new(self.gilt, stage, None)?,
            nominal,
            cash: rounding,
            recipe: Recipe::Invoice {
                accrued: self.accrued_context(stage)?,
                clean,
                aggregation,
            },
        })
    }

    /// Defines a trade payment `R(N*P/100)` from dirty price evidence per100,
    /// with one final quantizer and no accrued-price shift.
    ///
    /// # Errors
    /// [`GiltError::NonPositiveNominal`] first, then a defensive
    /// [`GiltError::InternalCapacity`] if the prepared snapshot is inconsistent.
    pub fn dirty_cash(
        &self,
        nominal: DecimalAmount,
        dirty: SettlementEvidence,
        rounding: SettlementRounding,
    ) -> Result<GiltCashInstruction, GiltError> {
        positive_nominal(nominal)?;
        Ok(GiltCashInstruction {
            snapshot: Snapshot::new(self.gilt, CASH_TOTAL, None)?,
            nominal,
            cash: rounding,
            recipe: Recipe::Dirty {
                settlement: self.settlement_context(CASH_TOTAL)?,
                dirty,
            },
        })
    }

    /// Copies the settlement date and its containing quasi pair.
    fn settlement_context(&self, stage: &'static str) -> Result<SettlementContext, GiltError> {
        let quasi = &self.gilt.quasi_dates;
        let previous = self
            .next_quasi_index
            .checked_sub(1)
            .and_then(|index| quasi.get(index));
        match (previous, quasi.get(self.next_quasi_index)) {
            (Some(&start), Some(&end)) => Ok(SettlementContext {
                date: self.settlement_date,
                quasi_dates: (start, end),
            }),
            _ => Err(capacity(None, stage, "derive source")),
        }
    }

    /// Copies exact source AI, the next actual dividend and its cutoff.
    ///
    /// The ordinal needs no date work: it is
    /// `max(next_quasi_index, first_dividend_quasi_index) - first_dividend_quasi_index`.
    fn accrued_context(&self, stage: &'static str) -> Result<AccruedContext, GiltError> {
        let first = self.gilt.first_dividend_quasi_index;
        let index = self.next_quasi_index.max(first) - first;
        let date = self.gilt.dividend_dates().get(index).copied();
        let cutoff = self.gilt.ex_dividend_dates.get(index).copied();
        match (date, cutoff) {
            (Some(date), Some(cutoff)) => Ok(AccruedContext {
                settlement: self.settlement_context(stage)?,
                dividend: SelectedDividend { index, date },
                cutoff,
                source: self.accrued_source,
            }),
            _ => Err(capacity(None, stage, "derive source")),
        }
    }
}

impl Snapshot {
    /// Copies only the provenance shared by every recipe, never a full vector.
    fn new(
        gilt: &ConventionalGilt,
        stage: &'static str,
        index: Option<usize>,
    ) -> Result<Self, GiltError> {
        match (gilt.quasi_dates.first(), gilt.quasi_dates.get(1)) {
            (Some(&start), Some(&end)) => Ok(Self {
                coupon_per_100: gilt.coupon_per_100,
                issue_date: gilt.issue_date,
                maturity_date: gilt.maturity_date,
                first_dividend_date: gilt.first_dividend_date,
                first_period: gilt.first_period,
                end_of_month: gilt.end_of_month,
                issue_quasi_dates: (start, end),
            }),
            _ => Err(capacity(index, stage, "derive source")),
        }
    }
}

impl GiltCashInstruction {
    /// Evaluates every required stage exactly and admits only final cash.
    ///
    /// A continuous nonpoint image at a required Exact quantizer returns
    /// [`GiltCashDecision::ExactStageUnresolved`]. Different admitted extremes
    /// return [`GiltCashDecision::QuantumAmbiguous`]. Earlier stages, including
    /// the complete capital stage of a component invoice, precede later stages
    /// and every final range test.
    ///
    /// # Errors
    /// [`GiltError::CashArithmetic`] for point Inexact or an out-of-range final
    /// coefficient (operation `quantize`, or `sum` at `cash net`), or a
    /// defensive [`GiltError::InternalCapacity`].
    pub fn evaluate(&self) -> Result<GiltCashDecision, GiltError> {
        match self.evaluate_inner() {
            Ok(decision) => Ok(decision),
            Err(Failure::Unresolved(stage)) => Ok(GiltCashDecision::ExactStageUnresolved { stage }),
            Err(Failure::Error(error)) => Err(error),
        }
    }

    /// Returns the immutable named source/model identifier.
    #[must_use]
    pub const fn source_version(&self) -> &'static str {
        SOURCE_VERSION
    }

    /// Returns the exact recipe identifier; it does not serialize the terms.
    #[must_use]
    pub const fn recipe_identifier(&self) -> &'static str {
        match self.recipe {
            Recipe::Dividend { .. } => "PublishedDividendPer100",
            Recipe::Accrued(_) => "SourceAccruedPer100",
            Recipe::Invoice { .. } => "CleanPriceAndSourceAccruedPer100",
            Recipe::Dirty { .. } => "DirtyPricePer100",
        }
    }

    /// Returns the original strictly positive par nominal.
    #[must_use]
    pub const fn nominal(&self) -> DecimalAmount {
        self.nominal
    }

    /// Returns the explicit final and component cash quantizer.
    #[must_use]
    pub const fn cash_rounding(&self) -> SettlementRounding {
        self.cash
    }

    /// Returns the original clean or dirty price evidence, or None.
    #[must_use]
    pub const fn price_evidence(&self) -> Option<SettlementEvidence> {
        match self.recipe {
            Recipe::Invoice { clean, .. } => Some(clean),
            Recipe::Dirty { dirty, .. } => Some(dirty),
            Recipe::Dividend { .. } | Recipe::Accrued(_) => None,
        }
    }

    /// Returns the invoice aggregation, or None for a single-term recipe.
    #[must_use]
    pub const fn aggregation(&self) -> Option<CashAggregation> {
        match self.recipe {
            Recipe::Invoice { aggregation, .. } => Some(aggregation),
            Recipe::Dividend { .. } | Recipe::Accrued(_) | Recipe::Dirty { .. } => None,
        }
    }

    /// Returns the full exact annual coupon per100.
    #[must_use]
    pub const fn coupon_per_100(&self) -> DecimalAmount {
        self.snapshot.coupon_per_100
    }

    /// Returns the original issue date.
    #[must_use]
    pub const fn issue_date(&self) -> Date {
        self.snapshot.issue_date
    }

    /// Returns the original maturity date.
    #[must_use]
    pub const fn maturity_date(&self) -> Date {
        self.snapshot.maturity_date
    }

    /// Returns the first actual dividend date.
    #[must_use]
    pub const fn first_dividend_date(&self) -> Date {
        self.snapshot.first_dividend_date
    }

    /// Returns the inferred first-period role.
    #[must_use]
    pub const fn first_period(&self) -> GiltFirstPeriod {
        self.snapshot.first_period
    }

    /// Returns the explicit end-of-month quasi-cycle policy.
    #[must_use]
    pub const fn end_of_month(&self) -> EndOfMonth {
        self.snapshot.end_of_month
    }

    /// Returns the validated issue quasi pair `(Q0, Q1)`, `Q0 <= I < Q1`.
    #[must_use]
    pub const fn issue_quasi_dates(&self) -> (Date, Date) {
        self.snapshot.issue_quasi_dates
    }

    /// Returns the prepared settlement date, or None for dividend cash.
    #[must_use]
    pub const fn settlement_date(&self) -> Option<Date> {
        match self.settlement() {
            Some(settlement) => Some(settlement.date),
            None => None,
        }
    }

    /// Returns the containing settlement quasi pair `Qk <= S < Qk1`, or None.
    #[must_use]
    pub const fn settlement_quasi_dates(&self) -> Option<(Date, Date)> {
        match self.settlement() {
            Some(settlement) => Some(settlement.quasi_dates),
            None => None,
        }
    }

    /// Returns the selected actual dividend ordinal, or None for dirty cash.
    ///
    /// Accrued recipes retain the next actual dividend, even when detached.
    #[must_use]
    pub const fn source_dividend_index(&self) -> Option<usize> {
        match self.dividend() {
            Some(dividend) => Some(dividend.index),
            None => None,
        }
    }

    /// Returns the selected actual dividend date, or None for dirty cash.
    #[must_use]
    pub const fn source_dividend_date(&self) -> Option<Date> {
        match self.dividend() {
            Some(dividend) => Some(dividend.date),
            None => None,
        }
    }

    /// Returns the next actual dividend's supplied cutoff for accrued recipes.
    #[must_use]
    pub const fn ex_dividend_date(&self) -> Option<Date> {
        match self.accrued() {
            Some(accrued) => Some(accrued.cutoff),
            None => None,
        }
    }

    /// Returns whether settlement is strictly after the cutoff, for accrued
    /// recipes; equality remains cum-dividend.
    #[must_use]
    pub fn is_ex_dividend(&self) -> Option<bool> {
        self.accrued()
            .map(|accrued| accrued.settlement.date > accrued.cutoff)
    }

    /// Quantizes the exact source value per100 once, without nominal scaling.
    ///
    /// This diagnostic view is never an earlier stage of
    /// [`evaluate`](Self::evaluate), and its policy is not retained. The dirty
    /// recipe has no source value and returns `Ok(None)`.
    ///
    /// # Errors
    /// [`GiltError::CashArithmetic`] at stage `source dividend` or
    /// `source accrued`, endpoint `point`, for an Inexact or out-of-range
    /// quantized coefficient, or a defensive [`GiltError::InternalCapacity`].
    pub fn source_value_per_100(
        &self,
        quantum: AmountQuantum,
        mode: AmountRounding,
    ) -> Result<Option<DecimalAmount>, GiltError> {
        let (source, site) = match self.recipe {
            Recipe::Dividend { dividend, source } => {
                (source, Site::point(SOURCE_DIVIDEND, Some(dividend.index)))
            }
            Recipe::Accrued(accrued) | Recipe::Invoice { accrued, .. } => {
                (accrued.source, Site::point(SOURCE_ACCRUED, None))
            }
            Recipe::Dirty { .. } => return Ok(None),
        };
        let value = SourceValue::new(self.snapshot.coupon_per_100, source, site)?;
        quantize_source(value, None, SettlementRounding::new(quantum, mode), site).map(Some)
    }

    /// The settlement provenance of the three plan recipes.
    const fn settlement(&self) -> Option<SettlementContext> {
        match self.recipe {
            Recipe::Accrued(accrued) | Recipe::Invoice { accrued, .. } => Some(accrued.settlement),
            Recipe::Dirty { settlement, .. } => Some(settlement),
            Recipe::Dividend { .. } => None,
        }
    }

    /// The selected actual dividend of the three source recipes.
    const fn dividend(&self) -> Option<SelectedDividend> {
        match self.recipe {
            Recipe::Dividend { dividend, .. } => Some(dividend),
            Recipe::Accrued(accrued) | Recipe::Invoice { accrued, .. } => Some(accrued.dividend),
            Recipe::Dirty { .. } => None,
        }
    }

    /// The exact source of the three source recipes; dirty cash has none.
    const fn source(&self) -> Option<ExactSource> {
        match self.recipe {
            Recipe::Dividend { source, .. } => Some(source),
            Recipe::Accrued(accrued) | Recipe::Invoice { accrued, .. } => Some(accrued.source),
            Recipe::Dirty { .. } => None,
        }
    }

    /// The source accrued context of the two accrued recipes.
    const fn accrued(&self) -> Option<AccruedContext> {
        match self.recipe {
            Recipe::Accrued(accrued) | Recipe::Invoice { accrued, .. } => Some(accrued),
            Recipe::Dividend { .. } | Recipe::Dirty { .. } => None,
        }
    }

    /// Executes exactly the required stages in contract order.
    fn evaluate_inner(&self) -> Result<GiltCashDecision, Failure> {
        let coupon = self.snapshot.coupon_per_100;
        match self.recipe {
            Recipe::Dividend { dividend, source } => {
                let site = Site::point(CASH_COUPON, Some(dividend.index));
                let value = SourceValue::new(coupon, source, site)?;
                let amount = quantize_source(value, Some(self.nominal), self.cash, site)?;
                Ok(self.determined(amount, None))
            }
            Recipe::Accrued(accrued) => {
                let site = Site::point(CASH_ACCRUED, None);
                let value = SourceValue::new(coupon, accrued.source, site)?;
                let amount = quantize_source(value, Some(self.nominal), self.cash, site)?;
                Ok(self.determined(amount, None))
            }
            Recipe::Dirty { dirty, .. } => {
                let image = Image::decode(dirty, 1, CASH_TOTAL)?;
                let rounded = self.cash_round(image, 1, CASH_TOTAL)?;
                self.finish(rounded, CASH_TOTAL, "quantize", dirty)
            }
            Recipe::Invoice {
                accrued,
                clean,
                aggregation,
            } => self.invoice(accrued, clean, aggregation),
        }
    }

    /// Evaluates a clean-plus-accrued invoice on the common basis `R0*dayD`.
    fn invoice(
        &self,
        accrued: AccruedContext,
        clean: SettlementEvidence,
        aggregation: CashAggregation,
    ) -> Result<GiltCashDecision, Failure> {
        let first = first_invoice_stage(aggregation);
        let value = SourceValue::new(
            self.snapshot.coupon_per_100,
            accrued.source,
            Site::point(first, None),
        )?;
        let day = value.day(first)?;
        let capital = Image::decode(clean, day, first)?;
        match aggregation {
            CashAggregation::RoundTotal => {
                let source = value.on_basis(CASH_TOTAL)?;
                let total = capital.shift(source, CASH_TOTAL)?;
                let rounded = self.cash_round(total, day, CASH_TOTAL)?;
                self.finish(rounded, CASH_TOTAL, "quantize", clean)
            }
            CashAggregation::RoundComponentsThenSum => {
                // Complete capital, including its Exact test, before accrued.
                let capital = self.cash_round(capital, day, CASH_CAPITAL)?;
                let source = value.on_basis(CASH_ACCRUED)?;
                let accrued = self.cash_round(Image::point(source), day, CASH_ACCRUED)?;
                let lower = capital
                    .lower
                    .add(accrued.lower)
                    .ok_or_else(|| capacity(None, CASH_NET, "combine cash"))?;
                let upper = capital
                    .upper
                    .add(accrued.lower)
                    .ok_or_else(|| capacity(None, CASH_NET, "combine cash"))?;
                let net = Rounded {
                    lower,
                    upper,
                    point: lower == upper,
                };
                self.finish(net, CASH_NET, "sum", clean)
            }
        }
    }

    /// Rounds a price-basis image: value = coefficient * n / (R0*dayD*10^Nscale*100).
    fn cash_round(&self, image: Image, day: u32, stage: &'static str) -> Result<Rounded, Failure> {
        let point = image.is_point();
        if self.cash.mode() == AmountRounding::Exact && !point {
            return Err(Failure::Unresolved(stage));
        }
        let site = Site::point(stage, None);
        let quantum = self.cash.quantum().amount();
        let units = PriceInt::from_u128(quantum.coefficient().unsigned_abs());
        let factor = PriceInt::from_u128(self.nominal.coefficient().unsigned_abs())
            .checked_mul(
                PriceInt::pow10(u16::from(quantum.scale()))
                    .ok_or_else(|| site.capacity("scale cash"))?,
            )
            .ok_or_else(|| site.capacity("scale cash"))?;
        let denominator = basis(day, site)?
            .checked_mul(
                PriceInt::pow10(u16::from(self.nominal.scale()))
                    .ok_or_else(|| site.capacity("scale cash"))?,
            )
            .and_then(|value| value.checked_mul_small(100))
            .and_then(|value| value.checked_mul(units))
            .ok_or_else(|| site.capacity("scale cash"))?;
        let round = |value: Signed<40>, endpoint: &'static str| -> Result<Signed<40>, GiltError> {
            let site = Site { endpoint, ..site };
            let numerator = value
                .magnitude
                .checked_mul(factor)
                .ok_or_else(|| site.capacity("cash rounding"))?;
            let count = round_ratio(
                numerator,
                denominator,
                value.negative,
                self.cash.mode(),
                site,
            )?;
            let coefficient = count
                .checked_mul(units)
                .ok_or_else(|| site.capacity("cash rounding"))?;
            Ok(Signed::new(coefficient, value.negative))
        };
        let lower = round(image.lower, if point { "point" } else { "lower" })?;
        let upper = if point {
            lower
        } else {
            round(image.upper, "upper")?
        };
        Ok(Rounded {
            lower,
            upper,
            point,
        })
    }

    /// Admits the final lower amount, then upper for a nonpoint final image.
    fn finish(
        &self,
        rounded: Rounded,
        stage: &'static str,
        operation: &'static str,
        evidence: SettlementEvidence,
    ) -> Result<GiltCashDecision, Failure> {
        let scale = self.cash.quantum().amount().scale();
        let lower_site = Site {
            stage,
            endpoint: if rounded.point { "point" } else { "lower" },
            index: None,
        };
        let lower = admit(
            rounded.lower.magnitude,
            rounded.lower.negative,
            scale,
            lower_site,
            operation,
        )?;
        let upper = if rounded.point {
            lower
        } else {
            let upper_site = Site {
                endpoint: "upper",
                ..lower_site
            };
            admit(
                rounded.upper.magnitude,
                rounded.upper.negative,
                scale,
                upper_site,
                operation,
            )?
        };
        if lower == upper {
            Ok(self.determined(lower, Some(guarantee(evidence))))
        } else {
            Ok(GiltCashDecision::QuantumAmbiguous {
                lower_amount: lower,
                upper_amount: upper,
            })
        }
    }

    /// Wraps one admitted amount with this complete instruction.
    const fn determined(
        &self,
        amount: DecimalAmount,
        guarantee: Option<SettlementGuarantee>,
    ) -> GiltCashDecision {
        GiltCashDecision::Determined(GiltCashReceipt {
            amount,
            instruction: *self,
            guarantee,
        })
    }
}

/// The first stage of an invoice, which labels its shared source derivation.
const fn first_invoice_stage(aggregation: CashAggregation) -> &'static str {
    match aggregation {
        CashAggregation::RoundTotal => CASH_TOTAL,
        CashAggregation::RoundComponentsThenSum => CASH_CAPITAL,
    }
}

/// Validates strictly positive par nominal before any source or cash work.
fn positive_nominal(nominal: DecimalAmount) -> Result<(), GiltError> {
    if nominal.coefficient() <= 0 {
        Err(GiltError::NonPositiveNominal { nominal })
    } else {
        Ok(())
    }
}

/// Labels only the supplied price ingredient from its original provenance.
fn guarantee(evidence: SettlementEvidence) -> SettlementGuarantee {
    if evidence.is_declared_interval() {
        SettlementGuarantee::UniqueForDeclaredEnclosure
    } else if evidence.binary64_bounds().is_some() {
        SettlementGuarantee::ForSuppliedBinary64
    } else {
        SettlementGuarantee::ForSuppliedDecimals
    }
}

/// A stage outcome: a typed error, or an Exact-domain decision.
enum Failure {
    /// Typed arithmetic, range or capacity failure.
    Error(GiltError),
    /// A continuous nonpoint image met a required Exact quantizer.
    Unresolved(&'static str),
}

impl From<GiltError> for Failure {
    fn from(error: GiltError) -> Self {
        Self::Error(error)
    }
}

/// The stage, numeric endpoint and capacity index of one arithmetic step.
#[derive(Clone, Copy)]
struct Site {
    /// Stable cash or source-view stage.
    stage: &'static str,
    /// Stable endpoint label: point, lower or upper.
    endpoint: &'static str,
    /// Actual dividend ordinal for dividend work, otherwise None.
    index: Option<usize>,
}

impl Site {
    /// A point-image site.
    const fn point(stage: &'static str, index: Option<usize>) -> Self {
        Self {
            stage,
            endpoint: "point",
            index,
        }
    }

    /// A defensive fixed-width invariant failure at this stage.
    fn capacity(self, operation: &'static str) -> GiltError {
        capacity(self.index, self.stage, operation)
    }

    /// A typed Inexact or out-of-range cash failure at this stage and endpoint.
    const fn arithmetic(self, cause: AmountError) -> GiltError {
        GiltError::CashArithmetic {
            stage: self.stage,
            endpoint: self.endpoint,
            cause,
        }
    }
}

/// Exact signed source value per100: `sign * base / (divisor * 10^scale)`.
///
/// Unrounded sources use the full coupon coefficient times the unreduced day
/// numerator over the unreduced denominator (`2*dayD`); published dividends use
/// the six-place micro coefficient over one.
#[derive(Clone, Copy)]
struct SourceValue {
    /// Nonnegative numerator magnitude.
    base: SourceInt,
    /// Positive unreduced denominator.
    divisor: u32,
    /// Decimal scale of the numerator.
    scale: u8,
    /// Sign of a nonzero value.
    negative: bool,
}

impl SourceValue {
    /// Decodes a retained exact source without any public range admission.
    fn new(coupon: DecimalAmount, source: ExactSource, site: Site) -> Result<Self, GiltError> {
        match source {
            ExactSource::Unrounded {
                numerator,
                denominator,
            } => {
                if denominator == 0 || coupon.coefficient() < 0 {
                    return Err(site.capacity("derive source"));
                }
                let base = SourceInt::from_u128(coupon.coefficient().unsigned_abs())
                    .checked_mul(SourceInt::from_u128(u128::from(numerator.unsigned_abs())))
                    .ok_or_else(|| site.capacity("derive source"))?;
                Ok(Self {
                    negative: numerator < 0 && !base.is_zero(),
                    base,
                    divisor: denominator,
                    scale: coupon.scale(),
                })
            }
            ExactSource::Published { micro } => Ok(Self {
                base: micro
                    .resize::<8>()
                    .ok_or_else(|| site.capacity("derive source"))?,
                divisor: 1,
                scale: 6,
                negative: false,
            }),
        }
    }

    /// Returns `dayD`: half the unreduced denominator, or one for a published value.
    fn day(self, stage: &'static str) -> Result<u32, GiltError> {
        if self.divisor == 1 {
            Ok(1)
        } else if self.divisor.is_multiple_of(2) {
            Ok(self.divisor / 2)
        } else {
            Err(capacity(None, stage, "derive source"))
        }
    }

    /// Returns the exact signed coefficient on `R0*dayD`.
    ///
    /// `value * R0 * dayD = base * (10^(38-scale) << 1036) * dayD / divisor`,
    /// an integer because the divisor is `2*dayD` (or one at scale six).
    fn on_basis(self, stage: &'static str) -> Result<Signed<40>, GiltError> {
        let site = Site::point(stage, None);
        let day = self.day(stage)?;
        let unit = decimal_unit(self.scale)
            .and_then(|unit| unit.checked_mul_small(day))
            .ok_or_else(|| site.capacity("derive source"))?;
        let (unit, remainder) = unit
            .div_rem_small(self.divisor)
            .ok_or_else(|| site.capacity("derive source"))?;
        if remainder != 0 {
            return Err(site.capacity("derive source"));
        }
        let magnitude = self
            .base
            .resize::<40>()
            .and_then(|base| base.checked_mul(unit))
            .ok_or_else(|| site.capacity("derive source"))?;
        Ok(Signed::new(magnitude, self.negative))
    }
}

/// Rounds one exact source point, optionally scaled by nominal per100.
///
/// All decimal powers are combined first: with nominal `n/10^Nscale`, quantum
/// `Q/10^qscale` and exponent `e = qscale - Nscale - 2 - scale`, the count is
/// `base*n*10^max(e,0) / (divisor*Q*10^max(-e,0))`. Without nominal the
/// nominal and per100 factors are omitted.
fn quantize_source(
    value: SourceValue,
    nominal: Option<DecimalAmount>,
    rounding: SettlementRounding,
    site: Site,
) -> Result<DecimalAmount, GiltError> {
    let quantum = rounding.quantum().amount();
    let units = SourceInt::from_u128(quantum.coefficient().unsigned_abs());
    let (factor, nominal_scale) = match nominal {
        Some(nominal) => (
            nominal.coefficient().unsigned_abs(),
            i32::from(nominal.scale()) + 2,
        ),
        None => (1, 0),
    };
    let exponent = i32::from(quantum.scale()) - nominal_scale - i32::from(value.scale);
    let power = u16::try_from(exponent.unsigned_abs())
        .ok()
        .and_then(SourceInt::pow10)
        .ok_or_else(|| site.capacity("scale cash"))?;
    let mut numerator = value
        .base
        .checked_mul(SourceInt::from_u128(factor))
        .ok_or_else(|| site.capacity("scale cash"))?;
    let mut denominator = SourceInt::from_u128(u128::from(value.divisor))
        .checked_mul(units)
        .ok_or_else(|| site.capacity("scale cash"))?;
    if exponent >= 0 {
        numerator = numerator
            .checked_mul(power)
            .ok_or_else(|| site.capacity("scale cash"))?;
    } else {
        denominator = denominator
            .checked_mul(power)
            .ok_or_else(|| site.capacity("scale cash"))?;
    }
    let count = round_ratio(
        numerator,
        denominator,
        value.negative,
        rounding.mode(),
        site,
    )?;
    let coefficient = count
        .checked_mul(units)
        .ok_or_else(|| site.capacity("cash rounding"))?;
    admit(
        coefficient,
        value.negative,
        quantum.scale(),
        site,
        "quantize",
    )
}

/// Returns `R0/10^scale = 10^(38-scale) << 1036` for a scale at most 38.
fn decimal_unit(scale: u8) -> Option<PriceInt> {
    BASIS_DECIMALS
        .checked_sub(scale)
        .and_then(|exponent| PriceInt::pow10(u16::from(exponent)))
        .and_then(|power| power.checked_shl(BASIS_SHIFT))
}

/// Returns the common basis `R0*dayD`.
fn basis(day: u32, site: Site) -> Result<PriceInt, GiltError> {
    decimal_unit(0)
        .and_then(|unit| unit.checked_mul_small(day))
        .ok_or_else(|| site.capacity("scale cash"))
}

/// Signed magnitude; zero always has a positive sign.
#[derive(Clone, Copy, Eq, PartialEq)]
struct Signed<const N: usize> {
    /// Unsigned magnitude.
    magnitude: Wide<N>,
    /// Sign of a nonzero magnitude.
    negative: bool,
}

impl<const N: usize> Signed<N> {
    /// Canonicalizes zero to the positive sign.
    fn new(magnitude: Wide<N>, negative: bool) -> Self {
        Self {
            magnitude,
            negative: negative && !magnitude.is_zero(),
        }
    }

    /// Exact signed addition, retaining cancellation before any range check.
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

/// An ordered closed real image of signed coefficients on `R0*dayD`.
#[derive(Clone, Copy)]
struct Image {
    /// Least coefficient.
    lower: Signed<40>,
    /// Greatest coefficient.
    upper: Signed<40>,
}

impl Image {
    /// A single exact point.
    const fn point(value: Signed<40>) -> Self {
        Self {
            lower: value,
            upper: value,
        }
    }

    /// Equal endpoints are a point image, even for declared provenance.
    fn is_point(self) -> bool {
        self.lower == self.upper
    }

    /// Decodes validated price evidence exactly onto `R0*dayD`.
    fn decode(
        evidence: SettlementEvidence,
        day: u32,
        stage: &'static str,
    ) -> Result<Self, GiltError> {
        let site = Site::point(stage, None);
        if let Some((lower, upper)) = evidence.decimal_bounds() {
            Ok(Self {
                lower: decode_decimal(lower, day, site)?,
                upper: decode_decimal(upper, day, site)?,
            })
        } else if let Some((lower, upper)) = evidence.binary64_bounds() {
            Ok(Self {
                lower: decode_binary(lower.to_bits(), day, site)?,
                upper: decode_binary(upper.to_bits(), day, site)?,
            })
        } else {
            Err(site.capacity("decode evidence"))
        }
    }

    /// Adds one exact point to both endpoints; positive order is preserved.
    fn shift(self, value: Signed<40>, stage: &'static str) -> Result<Self, GiltError> {
        let combine = |endpoint: Signed<40>| {
            endpoint
                .add(value)
                .ok_or_else(|| capacity(None, stage, "combine cash"))
        };
        Ok(Self {
            lower: combine(self.lower)?,
            upper: combine(self.upper)?,
        })
    }
}

/// Rounded signed coefficients at the cash quantum scale.
#[derive(Clone, Copy)]
struct Rounded {
    /// Rounded least coefficient.
    lower: Signed<40>,
    /// Rounded greatest coefficient.
    upper: Signed<40>,
    /// Whether the reported stage input image is a point.
    point: bool,
}

/// `coefficient/10^scale` becomes `|coefficient| * (R0/10^scale) * dayD`.
fn decode_decimal(value: DecimalAmount, day: u32, site: Site) -> Result<Signed<40>, GiltError> {
    let magnitude = decimal_unit(value.scale())
        .and_then(|unit| unit.checked_mul_small(day))
        .and_then(|unit| unit.checked_mul(PriceInt::from_u128(value.coefficient().unsigned_abs())))
        .ok_or_else(|| site.capacity("decode evidence"))?;
    Ok(Signed::new(magnitude, value.coefficient() < 0))
}

/// Finite bits `M*2^e` become `M * 5^38 * 2^(e+1074) * dayD`, including subnormals.
fn decode_binary(bits: u64, day: u32, site: Site) -> Result<Signed<40>, GiltError> {
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
        .and_then(|value| value.checked_mul_small(day))
        .ok_or_else(|| site.capacity("decode evidence"))?;
    Ok(Signed::new(magnitude, bits >> 63 != 0))
}

/// Rounds `numerator/denominator` to a nonnegative count under the signed mode.
///
/// Ties compare the remainder with `denominator - remainder`; no doubled
/// remainder is formed. `Exact` reports a nonzero remainder as Inexact.
fn round_ratio<const N: usize>(
    numerator: Wide<N>,
    denominator: Wide<N>,
    negative: bool,
    mode: AmountRounding,
    site: Site,
) -> Result<Wide<N>, GiltError> {
    let (mut count, remainder) = numerator
        .div_rem(denominator)
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

/// Canonicalizes completely, then admits only the signed i128 result.
///
/// The asymmetric limit `-2^127` is handled by unsigned magnitude and sign.
fn admit<const N: usize>(
    magnitude: Wide<N>,
    negative: bool,
    scale: u8,
    site: Site,
    operation: &'static str,
) -> Result<DecimalAmount, GiltError> {
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

    const fn site() -> Site {
        Site::point(CASH_TOTAL, None)
    }

    fn rounding(coefficient: i128, scale: u8, mode: AmountRounding) -> SettlementRounding {
        SettlementRounding::new(AmountQuantum::new(coefficient, scale).unwrap(), mode)
    }

    fn amount(text: &str) -> DecimalAmount {
        text.parse().unwrap()
    }

    /// FR-GILT cash rounding: every signed mode on halves, thirds and exact values.
    #[test]
    fn signed_seven_mode_ratio_rounding_uses_exact_remainders() {
        use AmountRounding::*;
        let n = |value: u128| Wide::<8>::from_u128(value);
        let cases: [(u128, u128, bool, AmountRounding, u128); 18] = [
            (5, 2, false, TowardZero, 2),
            (5, 2, false, AwayFromZero, 3),
            (5, 2, true, TowardPositiveInfinity, 2),
            (5, 2, true, TowardNegativeInfinity, 3),
            (5, 2, false, NearestTiesEven, 2),
            (7, 2, false, NearestTiesEven, 4),
            (5, 2, true, NearestTiesAwayFromZero, 3),
            (4, 3, false, NearestTiesEven, 1),
            (5, 3, false, NearestTiesEven, 2),
            (4, 3, true, NearestTiesAwayFromZero, 1),
            (5, 3, true, TowardZero, 1),
            (5, 3, false, TowardPositiveInfinity, 2),
            (6, 3, false, Exact, 2),
            (6, 3, true, AwayFromZero, 2),
            (0, 7, true, AwayFromZero, 0),
            (1, 7, false, TowardZero, 0),
            (1, 7, false, AwayFromZero, 1),
            (13, 7, true, NearestTiesEven, 2),
        ];
        for (numerator, denominator, negative, mode, expected) in cases {
            let count = round_ratio(n(numerator), n(denominator), negative, mode, site()).unwrap();
            assert_eq!(count, n(expected), "{numerator}/{denominator} {mode:?}");
        }
        let inexact = round_ratio(n(5), n(3), false, Exact, site()).unwrap_err();
        assert_eq!(
            inexact,
            GiltError::CashArithmetic {
                stage: CASH_TOTAL,
                endpoint: "point",
                cause: AmountError::Inexact {
                    operation: "quantize"
                },
            }
        );
    }

    /// Admission normalizes before checking both asymmetric i128 limits.
    #[test]
    fn admission_normalizes_then_checks_signed_limits() {
        let max = Wide::<40>::from_u128(i128::MAX.unsigned_abs());
        let min = Wide::<40>::from_u128(1_u128 << 127);
        assert_eq!(
            admit(max, false, 0, site(), "quantize")
                .unwrap()
                .coefficient(),
            i128::MAX
        );
        assert_eq!(
            admit(min, true, 0, site(), "sum").unwrap().coefficient(),
            i128::MIN
        );
        let over = admit(min, false, 0, site(), "quantize").unwrap_err();
        assert_eq!(
            over,
            GiltError::CashArithmetic {
                stage: CASH_TOTAL,
                endpoint: "point",
                cause: AmountError::OutOfRange {
                    operation: "quantize",
                    negative: false
                },
            }
        );
        // 2^127*10 at scale one normalizes back to the admissible -2^127.
        let scaled = min.checked_mul_small(10).unwrap();
        let value = admit(scaled, true, 1, site(), "sum").unwrap();
        assert_eq!((value.coefficient(), value.scale()), (i128::MIN, 0));
        let zero = admit(Wide::<8>::ZERO, true, 38, site(), "quantize").unwrap();
        assert_eq!((zero.coefficient(), zero.scale()), (0, 0));
    }

    /// The common basis decodes decimal and binary extremes exactly in 40 limbs.
    #[test]
    fn common_basis_decodes_extreme_evidence_exactly() {
        let day = 184 * 184;
        // MAX = (2^53-1)*2^971 becomes (2^53-1)*5^38*2^2045*dayD.
        let max = decode_binary(f64::MAX.to_bits(), day, site()).unwrap();
        assert!(max.magnitude.bits() < 2204 && !max.negative);
        let tiny = decode_binary((-f64::from_bits(1)).to_bits(), 1, site()).unwrap();
        assert_eq!(tiny.magnitude, Wide::from_u128(5_u128.pow(38)));
        assert!(tiny.negative);
        // 1e-38 is exactly R0/10^38 = 2^1036 on the dirty basis.
        let unit = decode_decimal(
            amount("0.00000000000000000000000000000000000001"),
            1,
            site(),
        )
        .unwrap();
        assert_eq!(
            unit.magnitude,
            Wide::from_u128(1).checked_shl(1036).unwrap()
        );
        // The same decimal and binary value share one coefficient.
        let decimal = decode_decimal(amount("-0.5"), 3, site()).unwrap();
        let binary = decode_binary((-0.5_f64).to_bits(), 3, site()).unwrap();
        assert!(decimal == binary);
        let largest = decode_decimal(
            DecimalAmount::new(i128::MAX, 0).unwrap(),
            u32::from(u16::MAX),
            site(),
        )
        .unwrap();
        // |coefficient| * R0 * dayD < 2^127 * 2^1163 * 2^16.
        assert!(largest.magnitude.bits() <= 1306);
    }

    /// An unreduced source `C*K/(2*dayD)` lands exactly on `R0*dayD`.
    #[test]
    fn source_accrued_coefficient_is_exact_on_the_invoice_basis() {
        let value = SourceValue::new(
            amount("6"),
            ExactSource::Unrounded {
                numerator: -3,
                denominator: 2 * 182,
            },
            site(),
        )
        .unwrap();
        assert_eq!(value.day(CASH_TOTAL).unwrap(), 182);
        let on_basis = value.on_basis(CASH_TOTAL).unwrap();
        // A = -18/364; A*R0*182 = -9*R0 = -9*10^38*2^1036.
        let expected = Wide::<40>::pow10(38)
            .unwrap()
            .checked_mul_small(9)
            .unwrap()
            .checked_shl(1036)
            .unwrap();
        assert!(on_basis.negative && on_basis.magnitude == expected);
        let odd = SourceValue {
            divisor: 3,
            ..value
        };
        assert_eq!(
            odd.day(CASH_TOTAL).unwrap_err(),
            capacity(None, CASH_TOTAL, "derive source")
        );
    }

    /// Pure source cash combines decimal powers and rounds the exact rational.
    #[test]
    fn pure_source_quantization_rounds_the_exact_rational_once() {
        // A = 6*(-1)/(2*61) per100: nominal 100 gives -6/122 = -0.0491803...
        let value = SourceValue::new(
            amount("6"),
            ExactSource::Unrounded {
                numerator: -1,
                denominator: 122,
            },
            site(),
        )
        .unwrap();
        let cases = [
            (AmountRounding::TowardZero, "-0.04"),
            (AmountRounding::AwayFromZero, "-0.05"),
            (AmountRounding::NearestTiesEven, "-0.05"),
            (AmountRounding::TowardPositiveInfinity, "-0.04"),
        ];
        for (mode, expected) in cases {
            let cash =
                quantize_source(value, Some(amount("100")), rounding(1, 2, mode), site()).unwrap();
            assert_eq!(cash, amount(expected), "{mode:?}");
        }
        let exact = quantize_source(
            value,
            Some(amount("100")),
            rounding(1, 2, AmountRounding::Exact),
            site(),
        );
        assert!(matches!(
            exact,
            Err(GiltError::CashArithmetic {
                cause: AmountError::Inexact { .. },
                ..
            })
        ));
        // A regular dividend C/2 of a scale-38 coupon is 5e-39, an exact tie
        // at quantum 1e-38 that no DecimalAmount view could represent.
        let tiny = SourceValue::new(
            amount("0.00000000000000000000000000000000000001"),
            ExactSource::Unrounded {
                numerator: 1,
                denominator: 2,
            },
            site(),
        )
        .unwrap();
        let half_quantum = rounding(1, 38, AmountRounding::NearestTiesEven);
        assert_eq!(
            quantize_source(tiny, None, half_quantum, site()).unwrap(),
            DecimalAmount::new(0, 0).unwrap()
        );
        let away = rounding(1, 38, AmountRounding::NearestTiesAwayFromZero);
        assert_eq!(
            quantize_source(tiny, None, away, site()).unwrap(),
            DecimalAmount::new(1, 38).unwrap()
        );
    }

    /// Instruction Debug shows readable context and omits private wide buffers.
    #[test]
    fn instruction_debug_is_readable_without_source_buffers() {
        let date = |y, m, d| Date::from_ymd(y, m, d).unwrap();
        // A short first period: Q0 < I < Q1 and F = Q1, so dividend 0 is published.
        let gilt = ConventionalGilt::new(
            date(2024, 4, 1),
            date(2025, 3, 7),
            amount("6"),
            date(2024, 9, 7),
            vec![date(2024, 3, 7), date(2024, 9, 7), date(2025, 3, 7)],
            vec![date(2024, 8, 28), date(2025, 2, 26)],
            EndOfMonth::Ignore,
            ConventionalGilt::MAX_QUASI_DATES,
        )
        .unwrap();
        let cash = rounding(1, 2, AmountRounding::NearestTiesEven);
        let published = format!("{:?}", gilt.dividend_cash(0, amount("100"), cash).unwrap());
        assert!(published.contains("recipe: \"PublishedDividendPer100\""));
        assert!(published.contains("source: Some(PublishedSixPlaceDividend)"));
        assert!(published.contains("source_dividend_index: Some(0)"));
        assert!(!published.contains("micro") && !published.contains("Wide"));
        let regular = format!("{:?}", gilt.dividend_cash(1, amount("100"), cash).unwrap());
        assert!(regular.contains("CouponDayRatio { numerator: 1, denominator: 2 }"));
        let plan =
            GiltPricingPlan::new(&gilt, date(2024, 10, 7), GiltPricingPlan::MAX_BUNDLES).unwrap();
        let dirty = plan
            .dirty_cash(
                amount("100"),
                SettlementEvidence::exact_decimal(amount("99")),
                cash,
            )
            .unwrap();
        let text = format!("{:?}", dirty.evaluate().unwrap());
        assert!(text.contains("recipe: \"DirtyPricePer100\"") && text.contains("source: None"));
        assert!(text.contains("aggregation: None") && !text.contains("Wide"));
    }

    /// The widest published micro coefficient and nominal stay within eight limbs.
    #[test]
    fn eight_limb_source_bounds_cover_extreme_scales() {
        let micro = Wide::<5>::from_u128(u128::MAX)
            .checked_mul(Wide::<5>::from_u128(1 << 19))
            .unwrap();
        let value =
            SourceValue::new(amount("1"), ExactSource::Published { micro }, site()).unwrap();
        let nominal = DecimalAmount::new(i128::MAX, 0).unwrap();
        let result = quantize_source(
            value,
            Some(nominal),
            rounding(1, 38, AmountRounding::TowardZero),
            site(),
        );
        // A huge result is an ordinary final range failure, not capacity.
        assert!(matches!(
            result,
            Err(GiltError::CashArithmetic {
                cause: AmountError::OutOfRange {
                    operation: "quantize",
                    negative: false
                },
                ..
            })
        ));
        let smallest = SourceValue::new(
            DecimalAmount::new(i128::MAX, 38).unwrap(),
            ExactSource::Unrounded {
                numerator: -(1 << 16),
                denominator: 2 * 184 * 184,
            },
            site(),
        )
        .unwrap();
        let tiny_nominal = DecimalAmount::new(1, 38).unwrap();
        let cash = quantize_source(
            smallest,
            Some(tiny_nominal),
            rounding(i128::MAX, 0, AmountRounding::TowardNegativeInfinity),
            site(),
        )
        .unwrap();
        assert_eq!(cash, DecimalAmount::new(i128::MIN + 1, 0).unwrap());
    }
}
