//! Exact Canadian source views and cash arithmetic.
//!
//! Magnitudes use caller-owned fixed workspaces. Source views retain their
//! exact rational until the caller's one quantum rounding and final admission.

use super::{
    CanadianBond, CanadianError, CanadianFrequency, CanadianPeriodKind, CanadianPricingPlan,
    CanadianSourcePolicy, CanadianYieldConvention, CanadianYieldSelection, Geometry,
    SOURCE_VERSION, Source, capacity, wide,
};
use crate::amounts::{AmountError, AmountQuantum, AmountRounding, DecimalAmount};
use crate::calendar::EndOfMonth;
use crate::settlement::{
    CashAggregation, SettlementEvidence, SettlementGuarantee, SettlementRounding,
};
use crate::time::Date;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Definition {
    source_policy: CanadianSourcePolicy,
    coupon_per_100: DecimalAmount,
    frequency: CanadianFrequency,
    accrual_start: Date,
    original_issue_date: Date,
    stated_maturity_date: Date,
    true_maturity_date: Date,
    first_coupon_date: Date,
    last_coupon_start_date: Date,
    first_period: CanadianPeriodKind,
    last_period: CanadianPeriodKind,
    end_of_month: EndOfMonth,
    direct_cycle_anchor: Date,
}

impl Definition {
    fn capture(bond: &CanadianBond) -> Self {
        Self {
            source_policy: bond.source_policy(),
            coupon_per_100: bond.coupon_per_100(),
            frequency: bond.schedule.frequency(),
            accrual_start: bond.schedule.accrual_start(),
            original_issue_date: bond.original_issue_date(),
            stated_maturity_date: bond.schedule.stated_maturity(),
            true_maturity_date: bond.true_maturity_date(),
            first_coupon_date: bond.first_coupon_date(),
            last_coupon_start_date: bond.last_coupon_start_date(),
            first_period: bond.schedule.first_period(),
            last_period: bond.schedule.last_period(),
            end_of_month: bond.schedule.end_of_month(),
            direct_cycle_anchor: bond.schedule.direct_cycle_anchor(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct SettlementContext {
    date: Date,
    selection: CanadianYieldSelection,
    convention: CanadianYieldConvention,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct SourceContext {
    value: Source,
    geometry: Geometry,
    index: usize,
    coupon_date: Date,
    payment_date: Date,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Recipe {
    Coupon,
    Accrued,
    Dirty,
    Invoice,
}

/// Arithmetic-only result: no owned instruction or source-pair transfer.
enum PriceImage {
    Point(DecimalAmount),
    Range(DecimalAmount, DecimalAmount),
    ExactUnresolved(&'static str),
}

/// Complete owned Canadian cash ingredients, source geometry and explicit policy.
///
/// The snapshot retains reconstructible instrument identity including the last
/// actual coupon start. Source amounts stay exact even when an optional decimal
/// source view would fail. No calendar or price publication grid is inferred.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CanadianCashInstruction {
    definition: Definition,
    settlement: Option<SettlementContext>,
    source: Option<SourceContext>,
    nominal: DecimalAmount,
    rounding: SettlementRounding,
    evidence: Option<SettlementEvidence>,
    aggregation: Option<CashAggregation>,
    recipe: Recipe,
}

/// Exact outcome or unresolved continuous-domain diagnostic for one instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[expect(
    clippy::large_enum_variant,
    reason = "The adopted Copy decision owns its complete receipt without heap indirection."
)]
pub enum CanadianCashDecision {
    /// Every admitted ingredient gives this same final cash amount.
    Determined(CanadianCashReceipt),
    /// The two admitted extrema quantize to distinct final amounts.
    ///
    /// This does not certify representability of every interior decimal value.
    #[non_exhaustive]
    QuantumAmbiguous {
        /// Smallest final cash amount.
        lower_amount: DecimalAmount,
        /// Largest final cash amount.
        upper_amount: DecimalAmount,
    },
    /// Exact rounding cannot resolve a nonpoint continuous stage image.
    #[non_exhaustive]
    ExactStageUnresolved {
        /// First required quantizer with a nonpoint continuous input.
        stage: &'static str,
    },
}

/// Determined cash retaining the complete original owned instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CanadianCashReceipt {
    amount: DecimalAmount,
    instruction: CanadianCashInstruction,
    guarantee: Option<SettlementGuarantee>,
}

impl CanadianCashReceipt {
    /// Returns the canonical final cash amount.
    #[must_use]
    pub const fn amount(self) -> DecimalAmount {
        self.amount
    }

    /// Borrows the complete immutable instruction that determined this cash.
    #[must_use]
    pub const fn instruction(&self) -> &CanadianCashInstruction {
        &self.instruction
    }

    /// Labels only the original supplied price ingredient, if present.
    ///
    /// It does not certify an ideal analytical price. Pure source cash has None.
    #[must_use]
    pub const fn price_evidence_guarantee(self) -> Option<SettlementGuarantee> {
        self.guarantee
    }
}

impl CanadianBond {
    /// Captures exact payable coupon cash per100, excluding terminal principal.
    ///
    /// # Errors
    /// Validates the original coupon ordinal before positive nominal admission.
    pub fn coupon_cash(
        &self,
        index: usize,
        nominal: DecimalAmount,
        rounding: SettlementRounding,
    ) -> Result<CanadianCashInstruction, CanadianError> {
        self.check_coupon_index(index)?;
        positive_nominal(nominal)?;
        let source = SourceContext {
            value: self.payable_coupon(index)?,
            geometry: self.coupon_geometry(index)?,
            index,
            coupon_date: self.schedule.coupon_dates[index],
            payment_date: self
                .coupon_payment_date(index)
                .ok_or_else(|| capacity(Some(index), "cash coupon", "derive source"))?,
        };
        Ok(CanadianCashInstruction {
            definition: Definition::capture(self),
            settlement: None,
            source: Some(source),
            nominal,
            rounding,
            evidence: None,
            aggregation: None,
            recipe: Recipe::Coupon,
        })
    }
}

impl CanadianPricingPlan<'_> {
    fn cash_settlement(&self) -> SettlementContext {
        SettlementContext {
            date: self.metadata.settlement_date,
            selection: self.metadata.yield_selection,
            convention: self.metadata.yield_convention,
        }
    }

    fn cash_accrued_source(&self) -> Result<SourceContext, CanadianError> {
        let index = self
            .bundles
            .first()
            .map(|bundle| bundle.index)
            .ok_or_else(|| capacity(None, "cash accrued", "derive source"))?;
        let coupon_date = self
            .bond
            .schedule
            .coupon_dates
            .get(index)
            .copied()
            .ok_or_else(|| capacity(Some(index), "cash accrued", "derive source"))?;
        let payment_date = self
            .bond
            .coupon_payment_date(index)
            .ok_or_else(|| capacity(Some(index), "cash accrued", "derive source"))?;
        Ok(SourceContext {
            value: self.settlement_ai,
            geometry: self.accrued_geometry,
            index,
            coupon_date,
            payment_date,
        })
    }

    /// Captures exact settlement accrued cash per100 at the caller's quantum.
    ///
    /// # Errors
    /// Returns NonPositiveNominal before capturing source metadata.
    pub fn accrued_cash(
        &self,
        nominal: DecimalAmount,
        rounding: SettlementRounding,
    ) -> Result<CanadianCashInstruction, CanadianError> {
        positive_nominal(nominal)?;
        Ok(CanadianCashInstruction {
            definition: Definition::capture(self.bond),
            settlement: Some(self.cash_settlement()),
            source: Some(self.cash_accrued_source()?),
            nominal,
            rounding,
            evidence: None,
            aggregation: None,
            recipe: Recipe::Accrued,
        })
    }

    /// Captures supplied clean price and exact settlement accrued ingredients.
    ///
    /// # Errors
    /// Returns NonPositiveNominal before capturing source metadata. Source views
    /// and analytical binary accrued observations do not gate this snapshot.
    pub fn clean_accrued_cash(
        &self,
        nominal: DecimalAmount,
        clean: SettlementEvidence,
        aggregation: CashAggregation,
        rounding: SettlementRounding,
    ) -> Result<CanadianCashInstruction, CanadianError> {
        positive_nominal(nominal)?;
        Ok(CanadianCashInstruction {
            definition: Definition::capture(self.bond),
            settlement: Some(self.cash_settlement()),
            source: Some(self.cash_accrued_source()?),
            nominal,
            rounding,
            evidence: Some(clean),
            aggregation: Some(aggregation),
            recipe: Recipe::Invoice,
        })
    }

    /// Captures supplied all-in settlement dirty price cash per100.
    ///
    /// # Errors
    /// Returns NonPositiveNominal before capturing metadata. The supplied price
    /// does not silently mean compound model dirty or certify an ideal price.
    pub fn dirty_cash(
        &self,
        nominal: DecimalAmount,
        settlement_dirty: SettlementEvidence,
        rounding: SettlementRounding,
    ) -> Result<CanadianCashInstruction, CanadianError> {
        positive_nominal(nominal)?;
        Ok(CanadianCashInstruction {
            definition: Definition::capture(self.bond),
            settlement: Some(self.cash_settlement()),
            source: None,
            nominal,
            rounding,
            evidence: Some(settlement_dirty),
            aggregation: None,
            recipe: Recipe::Dirty,
        })
    }
}

fn positive_nominal(nominal: DecimalAmount) -> Result<(), CanadianError> {
    if nominal.coefficient() > 0 {
        Ok(())
    } else {
        Err(CanadianError::NonPositiveNominal { nominal })
    }
}

macro_rules! definition_accessors {
    ($($name:ident: $ty:ty => $doc:literal;)*) => {
        $(
            #[doc = $doc]
            #[must_use]
            pub const fn $name(&self) -> $ty { self.definition.$name }
        )*
    };
}

impl CanadianCashInstruction {
    /// Evaluates the exact retained ingredients with the explicit cash policy.
    ///
    /// # Errors
    /// Preserves required capital/accrued/net order, exact point divisibility
    /// and final lower-before-upper amount admission. No source display grid or
    /// binary analytical accrued observation enters this computation.
    pub fn evaluate(&self) -> Result<CanadianCashDecision, CanadianError> {
        match self.recipe {
            Recipe::Coupon | Recipe::Accrued => {
                let stage = if self.recipe == Recipe::Coupon {
                    "cash coupon"
                } else {
                    "cash accrued"
                };
                let index = if self.recipe == Recipe::Coupon {
                    self.source_coupon_index()
                } else {
                    None
                };
                let source = self
                    .source
                    .as_ref()
                    .ok_or_else(|| capacity(index, stage, "derive source"))?;
                let amount = {
                    let mut work = [[0_u64; 8]; 10];
                    quantize_source(
                        &source.value,
                        Some(self.nominal),
                        self.rounding.quantum(),
                        self.rounding.mode(),
                        index,
                        stage,
                        &mut work,
                    )?;
                    admit(
                        &mut work[6],
                        false,
                        self.rounding.quantum().amount().scale(),
                        stage,
                        "point",
                        "quantize",
                        index,
                    )?
                };
                // Scratch storage ends before the owned receipt is transferred.
                Ok(self.receipt(amount))
            }
            Recipe::Dirty | Recipe::Invoice => self.evaluate_price(),
        }
    }

    fn receipt(&self, amount: DecimalAmount) -> CanadianCashDecision {
        CanadianCashDecision::Determined(CanadianCashReceipt {
            amount,
            instruction: *self,
            guarantee: self.evidence.map(ingredient_guarantee),
        })
    }

    fn evaluate_price(&self) -> Result<CanadianCashDecision, CanadianError> {
        // The price helper returns only small admitted amounts. Its complete
        // arithmetic frame ends before any owned instruction/receipt transfer.
        match self.price_image()? {
            PriceImage::Point(amount) => Ok(self.receipt(amount)),
            PriceImage::Range(lower_amount, upper_amount) => {
                Ok(CanadianCashDecision::QuantumAmbiguous {
                    lower_amount,
                    upper_amount,
                })
            }
            PriceImage::ExactUnresolved(stage) => {
                Ok(CanadianCashDecision::ExactStageUnresolved { stage })
            }
        }
    }

    fn price_image(&self) -> Result<PriceImage, CanadianError> {
        let components = self.recipe == Recipe::Invoice
            && self.aggregation == Some(CashAggregation::RoundComponentsThenSum);
        let first_stage = if components {
            "cash capital"
        } else {
            "cash total"
        };
        let evidence = self
            .evidence
            .ok_or_else(|| capacity(None, first_stage, "decode evidence"))?;
        let (lower_input, upper_input, point) = price_endpoints(evidence, first_stage)?;
        if self.rounding.mode() == AmountRounding::Exact && !point {
            return Ok(PriceImage::ExactUnresolved(first_stage));
        }
        let accrued = if self.recipe == Recipe::Invoice {
            &self
                .source
                .as_ref()
                .ok_or_else(|| capacity(None, first_stage, "derive source"))?
                .value
        } else {
            &Source::ZERO
        };
        let initial_accrued = if components { &Source::ZERO } else { accrued };

        // Sixteen disjoint endpoint/scaling/division slots and eight retained
        // coefficient/combination slots: exactly24 forty-word magnitudes.
        let mut work = [[0_u64; 40]; 24];
        let (scratch, results) = work.split_at_mut(16);
        let [lower, upper, accrued_coefficient, combine, _a, _b, _c, _d] = results else {
            return Err(capacity(None, first_stage, "scale cash"));
        };
        let mut lower_negative = quantize_price_endpoint(
            lower_input,
            initial_accrued,
            self.nominal,
            self.rounding,
            first_stage,
            lower,
            scratch,
        )?;
        let mut upper_negative = quantize_price_endpoint(
            upper_input,
            initial_accrued,
            self.nominal,
            self.rounding,
            first_stage,
            upper,
            scratch,
        )?;
        let (final_stage, operation) = if components {
            // The capital Exact stage completes before any accrued quantizer.
            // Its rounded coefficient is not admitted as DecimalAmount.
            {
                let mut pure = [[0_u64; 8]; 10];
                quantize_source(
                    accrued,
                    Some(self.nominal),
                    self.rounding.quantum(),
                    self.rounding.mode(),
                    None,
                    "cash accrued",
                    &mut pure,
                )?;
                checked(
                    wide::copy_into(&pure[6], accrued_coefficient),
                    None,
                    "cash accrued",
                    "cash rounding",
                )?;
            }
            lower_negative = add_signed(
                lower,
                lower_negative,
                accrued_coefficient,
                false,
                combine,
                "cash net",
            )?;
            upper_negative = add_signed(
                upper,
                upper_negative,
                accrued_coefficient,
                false,
                combine,
                "cash net",
            )?;
            ("cash net", "sum")
        } else {
            ("cash total", "quantize")
        };
        // Pointness belongs to the final QUANTIZED image, for every recipe.
        let final_point =
            lower_negative == upper_negative && wide::cmp(lower, upper) == Ordering::Equal;
        let scale = self.rounding.quantum().amount().scale();
        let lower_amount = admit(
            lower,
            lower_negative,
            scale,
            final_stage,
            if final_point { "point" } else { "lower" },
            operation,
            None,
        )?;
        if final_point {
            return Ok(PriceImage::Point(lower_amount));
        }
        let upper_amount = admit(
            upper,
            upper_negative,
            scale,
            final_stage,
            "upper",
            operation,
            None,
        )?;
        if lower_amount == upper_amount {
            Ok(PriceImage::Point(lower_amount))
        } else {
            Ok(PriceImage::Range(lower_amount, upper_amount))
        }
    }

    /// Identifies the frozen local source interpretation, not an effective date.
    #[must_use]
    pub const fn source_version(&self) -> &'static str {
        SOURCE_VERSION
    }

    /// Returns the exact recipe identifier without an implicit prefix or grid.
    #[must_use]
    pub const fn recipe_identifier(&self) -> &'static str {
        match self.recipe {
            Recipe::Coupon => "PayableCouponPer100",
            Recipe::Accrued => "SettlementAccruedPer100",
            Recipe::Dirty => "SettlementDirtyPricePer100",
            Recipe::Invoice => "CleanPriceAndSettlementAccruedPer100",
        }
    }

    definition_accessors! {
        source_policy: CanadianSourcePolicy => "Returns the retained explicit source policy.";
        coupon_per_100: DecimalAmount => "Returns the exact annual coupon per100.";
        frequency: CanadianFrequency => "Returns the retained coupon frequency.";
        accrual_start: Date => "Returns the original accrual start D.";
        original_issue_date: Date => "Returns the original issue date I.";
        stated_maturity_date: Date => "Returns the contractual maturity M.";
        true_maturity_date: Date => "Returns the actual terminal payment date T.";
        first_coupon_date: Date => "Returns the complete instrument's first actual coupon F.";
        last_coupon_start_date: Date => "Returns the complete instrument's last actual coupon start L.";
        first_period: CanadianPeriodKind => "Returns the first actual period's role.";
        last_period: CanadianPeriodKind => "Returns the last actual period's role.";
        end_of_month: EndOfMonth => "Returns the supplied direct-cycle EOM policy.";
        direct_cycle_anchor: Date => "Returns the supplied last-Q direct anchor.";
    }

    /// Returns the strictly positive supplied nominal.
    #[must_use]
    pub const fn nominal(&self) -> DecimalAmount {
        self.nominal
    }
    /// Returns the caller's final cash quantum and rounding mode.
    #[must_use]
    pub const fn cash_rounding(&self) -> SettlementRounding {
        self.rounding
    }
    /// Returns the original supplied price ingredient, if this recipe uses one.
    #[must_use]
    pub const fn price_evidence(&self) -> Option<SettlementEvidence> {
        self.evidence
    }
    /// Returns the explicit invoice aggregation, only for clean/accrued cash.
    #[must_use]
    pub const fn aggregation(&self) -> Option<CashAggregation> {
        self.aggregation
    }
    /// Returns S for prepared-plan cash; coupon-only cash has no settlement.
    #[must_use]
    pub const fn settlement_date(&self) -> Option<Date> {
        match &self.settlement {
            Some(context) => Some(context.date),
            None => None,
        }
    }
    /// Returns the original plan routing, if present.
    #[must_use]
    pub const fn yield_selection(&self) -> Option<CanadianYieldSelection> {
        match &self.settlement {
            Some(context) => Some(context.selection),
            None => None,
        }
    }
    /// Returns the resolved plan equation, if present.
    #[must_use]
    pub const fn yield_convention(&self) -> Option<CanadianYieldConvention> {
        match &self.settlement {
            Some(context) => Some(context.convention),
            None => None,
        }
    }
    /// Returns the selected original coupon ordinal, including next-coupon AI.
    #[must_use]
    pub const fn source_coupon_index(&self) -> Option<usize> {
        match &self.source {
            Some(source) => Some(source.index),
            None => None,
        }
    }
    /// Returns the selected contractual coupon date.
    #[must_use]
    pub const fn source_coupon_date(&self) -> Option<Date> {
        match &self.source {
            Some(source) => Some(source.coupon_date),
            None => None,
        }
    }
    /// Returns the selected actual payment date, using T only for terminal cash.
    #[must_use]
    pub const fn source_payment_date(&self) -> Option<Date> {
        match &self.source {
            Some(source) => Some(source.payment_date),
            None => None,
        }
    }
    /// Returns the selected exact source interval; empty accrued retains `[S, S]`.
    #[must_use]
    pub const fn source_interval(&self) -> Option<(Date, Date)> {
        match &self.source {
            Some(source) => Some((source.geometry.start, source.geometry.end)),
            None => None,
        }
    }
    /// Returns independent start-right/end-left Q pairs, including full cells.
    #[must_use]
    pub const fn source_quasi_endpoints(&self) -> Option<((Date, Date), (Date, Date))> {
        match &self.source {
            Some(source) => Some((source.geometry.first_quasi, source.geometry.last_quasi)),
            None => None,
        }
    }
    /// Returns the complete-cell count independently of the source's amount.
    #[must_use]
    pub const fn source_full_period_count(&self) -> Option<u32> {
        match &self.source {
            Some(source) => Some(source.geometry.full_count),
            None => None,
        }
    }
    /// Returns at most two nonzero proper-fragment day ratios, without duplicates.
    #[must_use]
    #[expect(
        clippy::type_complexity,
        reason = "The adopted public geometry view explicitly retains two optional exact fragments."
    )]
    pub const fn source_day_ratios(&self) -> Option<((u32, u32), Option<(u32, u32)>)> {
        match &self.source {
            Some(source) => match (source.geometry.first, source.geometry.second) {
                (Some(first), Some(second)) => Some((
                    (first.days, first.period_days),
                    Some((second.days, second.period_days)),
                )),
                (Some(fragment), None) | (None, Some(fragment)) => {
                    Some(((fragment.days, fragment.period_days), None))
                }
                (None, None) => None,
            },
            None => None,
        }
    }

    /// Independently quantizes the selected exact source at the supplied policy.
    ///
    /// # Errors
    /// Returns the same exact source-view arithmetic error as the corresponding
    /// bond/plan query. Dirty-only cash returns None without quantization.
    pub fn source_value_per_100(
        &self,
        quantum: AmountQuantum,
        mode: AmountRounding,
    ) -> Result<Option<DecimalAmount>, CanadianError> {
        match &self.source {
            Some(source) => {
                let (index, stage) = if self.recipe == Recipe::Coupon {
                    (Some(source.index), "source payable coupon")
                } else {
                    (None, "source settlement accrued")
                };
                source_view(source.value, quantum, mode, index, stage).map(Some)
            }
            None => Ok(None),
        }
    }
}

#[derive(Clone, Copy)]
enum Endpoint {
    Decimal(DecimalAmount),
    Binary(u64),
}

fn price_endpoints(
    evidence: SettlementEvidence,
    stage: &'static str,
) -> Result<(Endpoint, Endpoint, bool), CanadianError> {
    if let Some((lower, upper)) = evidence.decimal_bounds() {
        Ok((
            Endpoint::Decimal(lower),
            Endpoint::Decimal(upper),
            lower == upper,
        ))
    } else if let Some((lower, upper)) = evidence.binary64_bounds() {
        Ok((
            Endpoint::Binary(lower.to_bits()),
            Endpoint::Binary(upper.to_bits()),
            lower.to_bits() == upper.to_bits(),
        ))
    } else {
        Err(capacity(None, stage, "decode evidence"))
    }
}

fn ingredient_guarantee(evidence: SettlementEvidence) -> SettlementGuarantee {
    if evidence.is_declared_interval() {
        SettlementGuarantee::UniqueForDeclaredEnclosure
    } else if evidence.decimal_bounds().is_some() {
        SettlementGuarantee::ForSuppliedDecimals
    } else {
        SettlementGuarantee::ForSuppliedBinary64
    }
}

/// Branch-specific p/P avoids multiplying the old common decimal/binary grid
/// by the complete Canadian source denominator.
fn decode_endpoint(
    endpoint: Endpoint,
    numerator: &mut [u64],
    denominator: &mut [u64],
    stage: &'static str,
) -> Result<bool, CanadianError> {
    let negative = match endpoint {
        Endpoint::Decimal(value) => {
            checked(
                wide::set_u128(numerator, value.coefficient().unsigned_abs()),
                None,
                stage,
                "decode evidence",
            )?;
            checked(
                wide::pow10_into(u16::from(value.scale()), denominator),
                None,
                stage,
                "decode evidence",
            )?;
            value.coefficient() < 0
        }
        Endpoint::Binary(bits) => {
            let field = (bits >> 52) & 2047;
            let significand =
                (bits & ((1_u64 << 52) - 1)) | if field == 0 { 0 } else { 1_u64 << 52 };
            let shift = if field == 0 { 0 } else { field - 1 };
            let shift =
                u32::try_from(shift).map_err(|_| capacity(None, stage, "decode evidence"))?;
            checked(
                wide::set_u128(numerator, u128::from(significand)),
                None,
                stage,
                "decode evidence",
            )?;
            checked(
                wide::shl_assign(numerator, shift),
                None,
                stage,
                "decode evidence",
            )?;
            checked(
                wide::set_u128(denominator, 1),
                None,
                stage,
                "decode evidence",
            )?;
            checked(
                wide::shl_assign(denominator, 1074),
                None,
                stage,
                "decode evidence",
            )?;
            bits >> 63 != 0
        }
    };
    Ok(negative && !wide::is_zero(numerator))
}

fn quantize_price_endpoint(
    endpoint: Endpoint,
    accrued: &Source,
    nominal: DecimalAmount,
    rounding: SettlementRounding,
    stage: &'static str,
    coefficient: &mut [u64],
    scratch: &mut [[u64; 40]],
) -> Result<bool, CanadianError> {
    let [
        price_num,
        price_den,
        price_product,
        ai_product,
        combined_num,
        combined_den,
        num,
        den,
        power,
        product,
        quotient,
        remainder,
        aligned,
        complement,
        _a,
        _b,
    ] = scratch
    else {
        return Err(capacity(None, stage, "scale cash"));
    };
    let price_negative = decode_endpoint(endpoint, price_num, price_den, stage)?;
    checked(
        wide::mul_into(price_num, &accrued.denominator, price_product),
        None,
        stage,
        "combine cash",
    )?;
    checked(
        wide::mul_into(price_den, &accrued.numerator, ai_product),
        None,
        stage,
        "combine cash",
    )?;
    checked(
        wide::mul_into(price_den, &accrued.denominator, combined_den),
        None,
        stage,
        "combine cash",
    )?;
    let negative = if price_negative {
        if wide::cmp(price_product, ai_product) != Ordering::Less {
            checked(
                wide::copy_into(price_product, combined_num),
                None,
                stage,
                "combine cash",
            )?;
            checked(
                wide::sub_assign(combined_num, ai_product),
                None,
                stage,
                "combine cash",
            )?;
            !wide::is_zero(combined_num)
        } else {
            checked(
                wide::copy_into(ai_product, combined_num),
                None,
                stage,
                "combine cash",
            )?;
            checked(
                wide::sub_assign(combined_num, price_product),
                None,
                stage,
                "combine cash",
            )?;
            false
        }
    } else {
        checked(
            wide::copy_into(price_product, combined_num),
            None,
            stage,
            "combine cash",
        )?;
        checked(
            wide::add_assign(combined_num, ai_product),
            None,
            stage,
            "combine cash",
        )?;
        false
    };
    checked(
        wide::copy_into(combined_num, num),
        None,
        stage,
        "scale cash",
    )?;
    checked(
        wide::copy_into(combined_den, den),
        None,
        stage,
        "scale cash",
    )?;
    let quantum = rounding.quantum().amount();
    checked(
        wide::set_u128(power, nominal.coefficient().unsigned_abs()),
        None,
        stage,
        "scale cash",
    )?;
    multiply_assign(num, power, product, None, stage)?;
    checked(
        wide::pow10_into(u16::from(quantum.scale()), power),
        None,
        stage,
        "scale cash",
    )?;
    multiply_assign(num, power, product, None, stage)?;
    checked(
        wide::pow10_into(u16::from(nominal.scale()), power),
        None,
        stage,
        "scale cash",
    )?;
    multiply_assign(den, power, product, None, stage)?;
    checked(wide::mul_small_assign(den, 100), None, stage, "scale cash")?;
    checked(
        wide::set_u128(power, quantum.coefficient().unsigned_abs()),
        None,
        stage,
        "scale cash",
    )?;
    multiply_assign(den, power, product, None, stage)?;
    checked(
        wide::div_rem_into(num, den, quotient, remainder, aligned),
        None,
        stage,
        "cash rounding",
    )?;
    if round_increment(
        quotient,
        remainder,
        den,
        complement,
        negative,
        rounding.mode(),
        None,
        stage,
    )? {
        checked(
            wide::add_small_assign(quotient, 1),
            None,
            stage,
            "cash rounding",
        )?;
    }
    // The actual price/count coupling proves this product <2^1273.
    checked(
        wide::mul_into(quotient, power, coefficient),
        None,
        stage,
        "cash rounding",
    )?;
    Ok(negative && !wide::is_zero(coefficient))
}

fn add_signed(
    value: &mut [u64],
    negative: bool,
    addend: &[u64],
    addend_negative: bool,
    scratch: &mut [u64],
    stage: &'static str,
) -> Result<bool, CanadianError> {
    let result_negative = if negative == addend_negative {
        checked(wide::add_assign(value, addend), None, stage, "combine cash")?;
        negative
    } else if wide::cmp(value, addend) != Ordering::Less {
        checked(wide::sub_assign(value, addend), None, stage, "combine cash")?;
        negative
    } else {
        checked(
            wide::copy_into(addend, scratch),
            None,
            stage,
            "combine cash",
        )?;
        checked(
            wide::sub_assign(scratch, value),
            None,
            stage,
            "combine cash",
        )?;
        checked(wide::copy_into(scratch, value), None, stage, "combine cash")?;
        addend_negative
    };
    Ok(result_negative && !wide::is_zero(value))
}

/// Quantizes a cached source without imposing a nominal or publication grid.
pub(super) fn source_view(
    source: Source,
    quantum: AmountQuantum,
    mode: AmountRounding,
    index: Option<usize>,
    stage: &'static str,
) -> Result<DecimalAmount, CanadianError> {
    // Ten eight-word magnitudes, with no callee-owned wide temporaries.
    let mut work = [[0_u64; 8]; 10];
    quantize_source(&source, None, quantum, mode, index, stage, &mut work)?;
    admit(
        &mut work[6],
        false,
        quantum.amount().scale(),
        stage,
        "point",
        "quantize",
        index,
    )
}

/// Leaves the nonnegative rounded coefficient at the quantum scale in slot6.
/// An optional nominal selects N*source/100; None selects the source view.
fn quantize_source(
    source: &Source,
    nominal: Option<DecimalAmount>,
    quantum: AmountQuantum,
    mode: AmountRounding,
    index: Option<usize>,
    stage: &'static str,
    work: &mut [[u64; 8]; 10],
) -> Result<(), CanadianError> {
    let [
        num,
        den,
        quotient,
        remainder,
        aligned,
        complement,
        coefficient,
        power,
        product,
        _spare,
    ] = work;
    checked(
        wide::copy_into(&source.numerator, num),
        index,
        stage,
        "scale cash",
    )?;
    checked(
        wide::copy_into(&source.denominator, den),
        index,
        stage,
        "scale cash",
    )?;
    let q = quantum.amount();
    checked(
        wide::pow10_into(u16::from(q.scale()), power),
        index,
        stage,
        "scale cash",
    )?;
    multiply_assign(num, power, product, index, stage)?;
    if let Some(nominal) = nominal {
        checked(
            wide::set_u128(power, nominal.coefficient().unsigned_abs()),
            index,
            stage,
            "scale cash",
        )?;
        multiply_assign(num, power, product, index, stage)?;
        checked(
            wide::pow10_into(u16::from(nominal.scale()), power),
            index,
            stage,
            "scale cash",
        )?;
        multiply_assign(den, power, product, index, stage)?;
        checked(wide::mul_small_assign(den, 100), index, stage, "scale cash")?;
    }
    checked(
        wide::set_u128(power, q.coefficient().unsigned_abs()),
        index,
        stage,
        "scale cash",
    )?;
    multiply_assign(den, power, product, index, stage)?;
    checked(
        wide::div_rem_into(num, den, quotient, remainder, aligned),
        index,
        stage,
        "cash rounding",
    )?;
    if round_increment(
        quotient, remainder, den, complement, false, mode, index, stage,
    )? {
        checked(
            wide::add_small_assign(quotient, 1),
            index,
            stage,
            "cash rounding",
        )?;
    }
    // Coupled source/nominal/quantum bounds prove this product <2^390.
    checked(
        wide::mul_into(quotient, power, coefficient),
        index,
        stage,
        "cash rounding",
    )
}

fn multiply_assign(
    value: &mut [u64],
    factor: &[u64],
    product: &mut [u64],
    index: Option<usize>,
    stage: &'static str,
) -> Result<(), CanadianError> {
    checked(
        wide::mul_into(value, factor, product),
        index,
        stage,
        "scale cash",
    )?;
    checked(wide::copy_into(product, value), index, stage, "scale cash")
}

#[expect(
    clippy::too_many_arguments,
    reason = "Disjoint rounding buffers and the original error context are explicit."
)]
fn round_increment(
    quotient: &[u64],
    remainder: &[u64],
    denominator: &[u64],
    complement: &mut [u64],
    negative: bool,
    mode: AmountRounding,
    index: Option<usize>,
    stage: &'static str,
) -> Result<bool, CanadianError> {
    if wide::is_zero(remainder) {
        return Ok(false);
    }
    if mode == AmountRounding::Exact {
        return Err(CanadianError::CashArithmetic {
            stage,
            endpoint: "point",
            cause: AmountError::Inexact {
                operation: "quantize",
            },
        });
    }
    checked(
        wide::copy_into(denominator, complement),
        index,
        stage,
        "cash rounding",
    )?;
    checked(
        wide::sub_assign(complement, remainder),
        index,
        stage,
        "cash rounding",
    )?;
    let midpoint = wide::cmp(remainder, complement);
    Ok(match mode {
        AmountRounding::TowardZero => false,
        AmountRounding::AwayFromZero => true,
        AmountRounding::TowardPositiveInfinity => !negative,
        AmountRounding::TowardNegativeInfinity => negative,
        AmountRounding::NearestTiesEven => {
            midpoint == Ordering::Greater || (midpoint == Ordering::Equal && wide::is_odd(quotient))
        }
        AmountRounding::NearestTiesAwayFromZero => midpoint != Ordering::Less,
        // The nonzero-remainder Exact case returned its typed error above.
        AmountRounding::Exact => false,
    })
}

fn admit(
    coefficient: &mut [u64],
    negative: bool,
    mut scale: u8,
    stage: &'static str,
    endpoint: &'static str,
    operation: &'static str,
    index: Option<usize>,
) -> Result<DecimalAmount, CanadianError> {
    if wide::is_zero(coefficient) {
        return DecimalAmount::new(0, 0).map_err(|cause| CanadianError::CashArithmetic {
            stage,
            endpoint,
            cause,
        });
    }
    // Test divisibility without a second coefficient-sized owned magnitude.
    while scale > 0 && divisible_by_ten(coefficient) {
        let remainder = wide::div_rem_small_assign(coefficient, 10)
            .ok_or_else(|| capacity(index, stage, "normalize cash"))?;
        checked(remainder == 0, index, stage, "normalize cash")?;
        scale -= 1;
    }
    let magnitude = wide::to_u128(coefficient);
    let signed = magnitude.and_then(|magnitude| {
        if negative && magnitude == 1_u128 << 127 {
            Some(i128::MIN)
        } else {
            i128::try_from(magnitude)
                .ok()
                .map(|value| if negative { -value } else { value })
        }
    });
    let signed = signed.ok_or(CanadianError::CashArithmetic {
        stage,
        endpoint,
        cause: AmountError::OutOfRange {
            operation,
            negative,
        },
    })?;
    DecimalAmount::new(signed, scale).map_err(|cause| CanadianError::CashArithmetic {
        stage,
        endpoint,
        cause,
    })
}

fn divisible_by_ten(value: &[u64]) -> bool {
    let mut remainder = 0_u128;
    for &word in value.iter().rev() {
        remainder = ((remainder << 64) | u128::from(word)) % 10;
    }
    remainder == 0
}

fn checked(
    success: bool,
    index: Option<usize>,
    stage: &'static str,
    operation: &'static str,
) -> Result<(), CanadianError> {
    if success {
        Ok(())
    } else {
        Err(capacity(index, stage, operation))
    }
}
