//! Explicit exact monetary recipes and conditional interval consequences.
//!
//! Inputs retain their meaning as supplied decimals, supplied binary64 bits,
//! or caller-declared closed real intervals. Every cash quantum and rounding
//! mode is explicit. Trade proceeds use positive buyer-to-seller cash; coupon
//! distributions use positive issuer-to-holder cash. No currency, calendar,
//! settlement lag or financial accuracy enclosure is inferred.
//!
//! A determined interval result proves the recipe consequence **conditional
//! on the declared enclosure**. It does not prove that an ideal financial
//! price lies there or certify a named market convention (FR-SETTLE-01..16).
//!
//! ```
//! use sabiquant::amounts::{AmountQuantum, AmountRounding};
//! use sabiquant::settlement::{SettlementDecision, SettlementEvidence,
//!     SettlementInstruction, SettlementRounding};
//! let instruction = SettlementInstruction::dirty_per_100(
//!     "1000".parse()?, SettlementEvidence::exact_decimal("99.125".parse()?),
//!     SettlementRounding::new(AmountQuantum::new(1, 2)?,
//!         AmountRounding::NearestTiesEven))?;
//! let SettlementDecision::Determined(receipt) = instruction.evaluate()? else {
//!     return Err("a supplied point is determined".into());
//! };
//! assert_eq!(receipt.amount().to_string(), "991.25");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod arithmetic;

use crate::amounts::{AmountError, AmountQuantum, AmountRounding, DecimalAmount};
use crate::error::{IdentifierError, NonFinite};
use crate::identifier::{Entry, Identifiers};
use std::fmt;
use std::str::FromStr;

/// Exact supplied values or a caller-declared closed interval of real values.
///
/// Declaration provenance is retained even when the endpoints are equal.
/// Binary values use their exact finite bits, not their printed decimal text.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SettlementEvidence {
    /// Original representation and declaration provenance.
    kind: EvidenceKind,
}

/// Original finite coordinates; float fields are canonical bits for Eq/Hash.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum EvidenceKind {
    /// Exact decimal coordinates.
    Decimal {
        /// Lower exact amount.
        lower: DecimalAmount,
        /// Upper exact amount.
        upper: DecimalAmount,
        /// Whether the caller supplied an enclosure rather than a point value.
        declared: bool,
    },
    /// Exact IEEE 754 coordinates, with both zeros canonicalized.
    Binary {
        /// Finite lower bits.
        lower: u64,
        /// Finite upper bits.
        upper: u64,
        /// Whether the caller supplied an enclosure rather than a point value.
        declared: bool,
    },
}

impl SettlementEvidence {
    /// Retains the exact supplied canonical decimal value.
    #[must_use]
    pub const fn exact_decimal(value: DecimalAmount) -> Self {
        Self {
            kind: EvidenceKind::Decimal {
                lower: value,
                upper: value,
                declared: false,
            },
        }
    }

    /// Retains the exact finite binary64 value, canonicalizing either zero.
    ///
    /// # Errors
    /// Returns [`SettlementError::NonFinite`] with endpoint `value` for NaN
    /// or infinity, before any zero shortcut.
    pub fn supplied_binary64(value: f64) -> Result<Self, SettlementError> {
        let bits = finite_bits(value, "value")?;
        Ok(Self {
            kind: EvidenceKind::Binary {
                lower: bits,
                upper: bits,
                declared: false,
            },
        })
    }

    /// Retains a declared closed interval containing every real value between
    /// the supplied exact decimal endpoints.
    ///
    /// # Errors
    /// Returns [`SettlementError::DecimalIntervalOrder`] when lower>upper.
    pub fn declared_decimal_interval(
        lower: DecimalAmount,
        upper: DecimalAmount,
    ) -> Result<Self, SettlementError> {
        if lower > upper {
            return Err(SettlementError::DecimalIntervalOrder { lower, upper });
        }
        Ok(Self {
            kind: EvidenceKind::Decimal {
                lower,
                upper,
                declared: true,
            },
        })
    }

    /// Retains a declared closed real interval with exact binary endpoints.
    /// Adjacent unequal floats still define a continuous, nonpoint interval.
    ///
    /// # Errors
    /// Checks nonfinite lower then upper, then rejects reversed finite bounds
    /// with [`SettlementError::BinaryIntervalOrder`]. Equal bounds are valid.
    pub fn declared_binary64_interval(lower: f64, upper: f64) -> Result<Self, SettlementError> {
        check_finite(lower, "lower")?;
        check_finite(upper, "upper")?;
        let lower_bits = canonical_bits(lower);
        let upper_bits = canonical_bits(upper);
        if lower > upper {
            return Err(SettlementError::BinaryIntervalOrder {
                lower_bits,
                upper_bits,
            });
        }
        Ok(Self {
            kind: EvidenceKind::Binary {
                lower: lower_bits,
                upper: upper_bits,
                declared: true,
            },
        })
    }

    /// Returns original decimal bounds, or None for binary evidence.
    /// Supplied values appear as an equal endpoint pair.
    #[must_use]
    pub const fn decimal_bounds(self) -> Option<(DecimalAmount, DecimalAmount)> {
        match self.kind {
            EvidenceKind::Decimal { lower, upper, .. } => Some((lower, upper)),
            EvidenceKind::Binary { .. } => None,
        }
    }

    /// Returns original finite binary bounds, or None for decimal evidence.
    /// Supplied values appear as an equal endpoint pair, with canonical zero.
    #[must_use]
    pub fn binary64_bounds(self) -> Option<(f64, f64)> {
        match self.kind {
            EvidenceKind::Binary { lower, upper, .. } => {
                Some((f64::from_bits(lower), f64::from_bits(upper)))
            }
            EvidenceKind::Decimal { .. } => None,
        }
    }

    /// Reports declared-enclosure provenance, independently of endpoint equality.
    #[must_use]
    pub const fn is_declared_interval(self) -> bool {
        match self.kind {
            EvidenceKind::Decimal { declared, .. } | EvidenceKind::Binary { declared, .. } => {
                declared
            }
        }
    }
}

/// One explicitly selected positive quantum and signed rounding mode.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SettlementRounding {
    /// Validated positive quantum.
    quantum: AmountQuantum,
    /// Explicit signed lattice rule.
    mode: AmountRounding,
}

impl SettlementRounding {
    /// Retains both supplied policies, without inferring a currency or scale.
    #[must_use]
    pub const fn new(quantum: AmountQuantum, mode: AmountRounding) -> Self {
        Self { quantum, mode }
    }
    /// Returns the supplied canonical positive quantum.
    #[must_use]
    pub const fn quantum(self) -> AmountQuantum {
        self.quantum
    }
    /// Returns the supplied signed rounding rule.
    #[must_use]
    pub const fn mode(self) -> AmountRounding {
        self.mode
    }
}

/// The explicit ordering of cash rounding and two-component aggregation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CashAggregation {
    /// Combine exact unrounded components, then round the total once.
    RoundTotal,
    /// Round capital then accrued separately and exact-sum their wide results.
    RoundComponentsThenSum,
}

impl CashAggregation {
    /// Every supported aggregation in canonical registry order.
    pub const ALL: [Self; 2] = [Self::RoundTotal, Self::RoundComponentsThenSum];
    /// Returns the exact case-sensitive identifier.
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::RoundTotal => "RoundTotal",
            Self::RoundComponentsThenSum => "RoundComponentsThenSum",
        }
    }
}

/// What a determined receipt proves about its retained source evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum SettlementGuarantee {
    /// Exact recipe arithmetic and rounding of supplied decimal values.
    ForSuppliedDecimals,
    /// Exact recipe arithmetic and rounding involving supplied binary64 bits.
    ForSuppliedBinary64,
    /// One outcome for every value in the caller-declared rectangular enclosure.
    UniqueForDeclaredEnclosure,
}

impl SettlementGuarantee {
    /// Every supported guarantee label in canonical registry order.
    pub const ALL: [Self; 3] = [
        Self::ForSuppliedDecimals,
        Self::ForSuppliedBinary64,
        Self::UniqueForDeclaredEnclosure,
    ];
    /// Returns the exact case-sensitive identifier.
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::ForSuppliedDecimals => "ForSuppliedDecimals",
            Self::ForSuppliedBinary64 => "ForSuppliedBinary64",
            Self::UniqueForDeclaredEnclosure => "UniqueForDeclaredEnclosure",
        }
    }
}

/// Exact aggregation identifiers; no aliases, defaults or ambiguous spellings.
const AGGREGATIONS: Identifiers<CashAggregation> = Identifiers::new(
    "cash aggregation",
    &[
        Entry {
            value: CashAggregation::RoundTotal,
            canonical: "RoundTotal",
            aliases: &[],
        },
        Entry {
            value: CashAggregation::RoundComponentsThenSum,
            canonical: "RoundComponentsThenSum",
            aliases: &[],
        },
    ],
    &[],
);
/// Exact provenance identifiers; parsing one cannot construct a receipt.
const GUARANTEES: Identifiers<SettlementGuarantee> = Identifiers::new(
    "settlement guarantee",
    &[
        Entry {
            value: SettlementGuarantee::ForSuppliedDecimals,
            canonical: "ForSuppliedDecimals",
            aliases: &[],
        },
        Entry {
            value: SettlementGuarantee::ForSuppliedBinary64,
            canonical: "ForSuppliedBinary64",
            aliases: &[],
        },
        Entry {
            value: SettlementGuarantee::UniqueForDeclaredEnclosure,
            canonical: "UniqueForDeclaredEnclosure",
            aliases: &[],
        },
    ],
    &[],
);

impl fmt::Display for CashAggregation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_identifier())
    }
}
impl FromStr for CashAggregation {
    type Err = IdentifierError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        AGGREGATIONS.parse(text)
    }
}
impl fmt::Display for SettlementGuarantee {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_identifier())
    }
}
impl FromStr for SettlementGuarantee {
    type Err = IdentifierError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        GUARANTEES.parse(text)
    }
}

/// A complete explicit monetary recipe, its evidence and every required policy.
///
/// Nominal is positive par in a caller-selected common unit. The recipe defines
/// per100/per1000 coordinates; this type never uses a pricing face implicitly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SettlementInstruction {
    /// Positive supplied par nominal.
    nominal: DecimalAmount,
    /// Dirty, clean or coupon evidence according to the family.
    primary: SettlementEvidence,
    /// Explicit final/component cash policy.
    cash: SettlementRounding,
    /// Complete family-specific terms.
    recipe: Recipe,
}

/// Bounded family-specific data, avoiding partial/invalid instruction shapes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Recipe {
    /// Direct trade proceeds per100.
    Dirty,
    /// Unrounded source coupon distribution per1000.
    Coupon,
    /// Clean plus accrued, both per100.
    Clean {
        /// Accrued per100 evidence.
        accrued: SettlementEvidence,
        /// Explicit aggregation.
        aggregation: CashAggregation,
    },
    /// Source-quantized clean per100 plus accrued per1000.
    Staged {
        /// Accrued per1000 evidence.
        accrued: SettlementEvidence,
        /// Explicit aggregation.
        aggregation: CashAggregation,
        /// Fixed source-price quantum and supplied mode.
        price: SettlementRounding,
        /// Fixed source-accrued quantum and supplied mode.
        accrued_rounding: SettlementRounding,
    },
}

impl SettlementInstruction {
    /// Defines `R(nominal * dirty_per100 / 100)` without intermediate rounding.
    ///
    /// # Errors
    /// Rejects nonpositive nominal before any evaluation work.
    pub fn dirty_per_100(
        nominal: DecimalAmount,
        dirty: SettlementEvidence,
        cash_rounding: SettlementRounding,
    ) -> Result<Self, SettlementError> {
        positive_nominal(nominal)?;
        Ok(Self {
            nominal,
            primary: dirty,
            cash: cash_rounding,
            recipe: Recipe::Dirty,
        })
    }

    /// Defines unrounded source coupon-per1000 scaling under an explicit payment
    /// policy. Positive cash is issuer-to-holder, independently of trade parties.
    ///
    /// # Errors
    /// Rejects nonpositive nominal before any evaluation work.
    pub fn coupon_per_1000(
        nominal: DecimalAmount,
        coupon: SettlementEvidence,
        cash_rounding: SettlementRounding,
    ) -> Result<Self, SettlementError> {
        positive_nominal(nominal)?;
        Ok(Self {
            nominal,
            primary: coupon,
            cash: cash_rounding,
            recipe: Recipe::Coupon,
        })
    }

    /// Defines clean/accrued per100 scaling with explicit total/component rounding.
    ///
    /// # Errors
    /// Rejects nonpositive nominal before any evaluation work.
    pub fn clean_accrued_per_100(
        nominal: DecimalAmount,
        clean: SettlementEvidence,
        accrued: SettlementEvidence,
        aggregation: CashAggregation,
        cash_rounding: SettlementRounding,
    ) -> Result<Self, SettlementError> {
        positive_nominal(nominal)?;
        Ok(Self {
            nominal,
            primary: clean,
            cash: cash_rounding,
            recipe: Recipe::Clean {
                accrued,
                aggregation,
            },
        })
    }

    /// Defines six-place source clean-price and five-place source accrued-per1000
    /// rounding, followed by explicit nominal scaling and cash aggregation.
    /// This staging recipe makes no named Treasury invoice compliance claim.
    ///
    /// # Errors
    /// Rejects nonpositive nominal first. InternalCapacity is a defensive
    /// invariant failure if the fixed valid source quanta cannot be constructed.
    pub fn staged_price100_accrued1000(
        nominal: DecimalAmount,
        clean: SettlementEvidence,
        accrued: SettlementEvidence,
        price_mode: AmountRounding,
        accrued_mode: AmountRounding,
        aggregation: CashAggregation,
        cash_rounding: SettlementRounding,
    ) -> Result<Self, SettlementError> {
        positive_nominal(nominal)?;
        let price =
            AmountQuantum::new(1, 6).map_err(|_| capacity("source price", "source rounding"))?;
        let ai =
            AmountQuantum::new(1, 5).map_err(|_| capacity("source accrued", "source rounding"))?;
        Ok(Self {
            nominal,
            primary: clean,
            cash: cash_rounding,
            recipe: Recipe::Staged {
                accrued,
                aggregation,
                price: SettlementRounding::new(price, price_mode),
                accrued_rounding: SettlementRounding::new(ai, accrued_mode),
            },
        })
    }

    /// Returns the original positive par nominal.
    #[must_use]
    pub const fn nominal(self) -> DecimalAmount {
        self.nominal
    }
    /// Returns original dirty, clean or coupon evidence according to the family.
    #[must_use]
    pub const fn primary(self) -> SettlementEvidence {
        self.primary
    }
    /// Returns original accrued evidence for the two-component families.
    #[must_use]
    pub const fn secondary(self) -> Option<SettlementEvidence> {
        match self.recipe {
            Recipe::Clean { accrued, .. } | Recipe::Staged { accrued, .. } => Some(accrued),
            Recipe::Dirty | Recipe::Coupon => None,
        }
    }
    /// Returns the supplied two-component aggregation, or None for a single term.
    #[must_use]
    pub const fn aggregation(self) -> Option<CashAggregation> {
        match self.recipe {
            Recipe::Clean { aggregation, .. } | Recipe::Staged { aggregation, .. } => {
                Some(aggregation)
            }
            Recipe::Dirty | Recipe::Coupon => None,
        }
    }
    /// Returns the explicit cash policy.
    #[must_use]
    pub const fn cash_rounding(self) -> SettlementRounding {
        self.cash
    }
    /// Returns the staged recipe's fixed source-price quantum and supplied mode.
    #[must_use]
    pub const fn source_price_rounding(self) -> Option<SettlementRounding> {
        match self.recipe {
            Recipe::Staged { price, .. } => Some(price),
            _ => None,
        }
    }
    /// Returns the staged recipe's fixed source-accrued quantum and supplied mode.
    #[must_use]
    pub const fn source_accrued_rounding(self) -> Option<SettlementRounding> {
        match self.recipe {
            Recipe::Staged {
                accrued_rounding, ..
            } => Some(accrued_rounding),
            _ => None,
        }
    }
    /// Returns the exact family identifier; it does not serialize the terms.
    #[must_use]
    pub const fn recipe_identifier(self) -> &'static str {
        match self.recipe {
            Recipe::Dirty => "DirtyPer100",
            Recipe::Coupon => "CouponPer1000",
            Recipe::Clean { .. } => "CleanAccruedPer100",
            Recipe::Staged { .. } => "StagedPrice100Accrued1000",
        }
    }
    /// Evaluates every required source/cash stage exactly and admits final cash.
    ///
    /// # Errors
    /// Returns staged [`SettlementError::Arithmetic`] for point Inexact or final
    /// coefficient range failure, or a defensive InternalCapacity error.
    /// Earlier source and capital stages precede later stages/range admission.
    pub fn evaluate(&self) -> Result<SettlementDecision, SettlementError> {
        arithmetic::evaluate(self)
    }

    /// Chooses a guarantee from original provenance, never from numeric collapse.
    fn guarantee(self) -> SettlementGuarantee {
        let secondary = self.secondary();
        if self.primary.is_declared_interval()
            || secondary.is_some_and(SettlementEvidence::is_declared_interval)
        {
            SettlementGuarantee::UniqueForDeclaredEnclosure
        } else if matches!(self.primary.kind, EvidenceKind::Binary { .. })
            || secondary.is_some_and(|e| matches!(e.kind, EvidenceKind::Binary { .. }))
        {
            SettlementGuarantee::ForSuppliedBinary64
        } else {
            SettlementGuarantee::ForSuppliedDecimals
        }
    }
}

/// A determined cash amount or an explicitly unresolved recipe consequence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[expect(
    clippy::large_enum_variant,
    reason = "the frozen allocation-free Copy receipt retains its complete original instruction"
)]
pub enum SettlementDecision {
    /// One representable result with retained instruction/provenance.
    Determined(SettlementReceipt),
    /// Different representable extremes; interior representability is not promised.
    #[non_exhaustive]
    QuantumAmbiguous {
        /// Least rounded final amount.
        lower_amount: DecimalAmount,
        /// Greatest rounded final amount.
        upper_amount: DecimalAmount,
    },
    /// A nonpoint image contains values undefined at a required Exact stage.
    #[non_exhaustive]
    ExactStageUnresolved {
        /// First required stage whose Exact domain is not satisfied throughout.
        stage: &'static str,
    },
}

/// A determined recipe result and the complete evidence that gives it meaning.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SettlementReceipt {
    /// Final representable exact amount.
    amount: DecimalAmount,
    /// Original complete definition.
    instruction: SettlementInstruction,
    /// Conditional/supplied-input interpretation.
    guarantee: SettlementGuarantee,
}

impl SettlementReceipt {
    /// Returns final canonical cash.
    #[must_use]
    pub const fn amount(self) -> DecimalAmount {
        self.amount
    }
    /// Borrows the complete original instruction.
    #[must_use]
    pub const fn instruction(&self) -> &SettlementInstruction {
        &self.instruction
    }
    /// Returns the exact meaning of the result, without upgrading source evidence.
    #[must_use]
    pub const fn guarantee(self) -> SettlementGuarantee {
        self.guarantee
    }
}

/// Invalid settlement terms or a failed required exact arithmetic stage.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SettlementError {
    /// Par nominal was not strictly positive.
    #[non_exhaustive]
    NonPositiveNominal {
        /// Rejected exact nominal.
        nominal: DecimalAmount,
    },
    /// Declared decimal bounds were reversed.
    #[non_exhaustive]
    DecimalIntervalOrder {
        /// Rejected lower endpoint.
        lower: DecimalAmount,
        /// Rejected upper endpoint.
        upper: DecimalAmount,
    },
    /// Declared finite binary bounds were reversed after zero canonicalization.
    #[non_exhaustive]
    BinaryIntervalOrder {
        /// Canonical finite lower bits.
        lower_bits: u64,
        /// Canonical finite upper bits.
        upper_bits: u64,
    },
    /// Binary evidence supplied NaN or infinity.
    #[non_exhaustive]
    NonFinite {
        /// Constructor coordinate: value, lower or upper.
        endpoint: &'static str,
        /// Existing typed nonfinite classification.
        value: NonFinite,
    },
    /// Point Exact rounding failed, or final normalized cash was out of range.
    #[non_exhaustive]
    Arithmetic {
        /// Required source/cash stage.
        stage: &'static str,
        /// Numeric stage input coordinate: point, lower or upper.
        endpoint: &'static str,
        /// Typed Inexact/OutOfRange amount cause.
        cause: AmountError,
    },
    /// A private fixed-capacity invariant failed, rather than a final range limit.
    #[non_exhaustive]
    InternalCapacity {
        /// Stage being computed.
        stage: &'static str,
        /// Semantic internal operation.
        operation: &'static str,
    },
}

impl SettlementError {
    /// Returns the stable domain error kind; Display wording is not stable.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::NonPositiveNominal { .. } => "settlement.nonpositive_nominal",
            Self::DecimalIntervalOrder { .. } => "settlement.decimal_interval_order",
            Self::BinaryIntervalOrder { .. } => "settlement.binary_interval_order",
            Self::NonFinite { .. } => "settlement.non_finite",
            Self::Arithmetic { .. } => "settlement.arithmetic",
            Self::InternalCapacity { .. } => "settlement.internal_capacity",
        }
    }
}

impl fmt::Display for SettlementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonPositiveNominal { nominal } => {
                write!(f, "par nominal must be positive, got {nominal}")
            }
            Self::DecimalIntervalOrder { lower, upper } => {
                write!(f, "decimal evidence bounds reversed: {lower} > {upper}")
            }
            Self::BinaryIntervalOrder {
                lower_bits,
                upper_bits,
            } => write!(
                f,
                "binary evidence bounds reversed: {lower_bits:#018x}, {upper_bits:#018x}"
            ),
            Self::NonFinite { endpoint, value } => write!(
                f,
                "settlement {endpoint} evidence must be finite, got {value}"
            ),
            Self::Arithmetic {
                stage,
                endpoint,
                cause,
            } => write!(f, "{stage} at {endpoint}: {cause}"),
            Self::InternalCapacity { stage, operation } => write!(
                f,
                "{stage}: private capacity invariant failed during {operation}"
            ),
        }
    }
}

impl std::error::Error for SettlementError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Arithmetic { cause, .. } => Some(cause),
            _ => None,
        }
    }
}

/// Validates par nominal before evaluation or source-quantum construction.
fn positive_nominal(nominal: DecimalAmount) -> Result<(), SettlementError> {
    if nominal.coefficient() <= 0 {
        Err(SettlementError::NonPositiveNominal { nominal })
    } else {
        Ok(())
    }
}

/// Validates binary evidence and canonicalizes either zero by its bits.
fn finite_bits(value: f64, endpoint: &'static str) -> Result<u64, SettlementError> {
    check_finite(value, endpoint)?;
    Ok(canonical_bits(value))
}

/// Rejects nonfinite values without canonicalizing or comparing any endpoint.
fn check_finite(value: f64, endpoint: &'static str) -> Result<(), SettlementError> {
    if let Some(value) = NonFinite::classify(value) {
        return Err(SettlementError::NonFinite { endpoint, value });
    }
    Ok(())
}

/// Canonicalizes zero only after the required finite validation has completed.
fn canonical_bits(value: f64) -> u64 {
    let bits = value.to_bits();
    if bits << 1 == 0 { 0 } else { bits }
}

/// Constructs a typed defensive capacity failure, never an input panic.
fn capacity(stage: &'static str, operation: &'static str) -> SettlementError {
    SettlementError::InternalCapacity { stage, operation }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-SETTLE-04: each text registry is complete and exact, with no aliases.
    #[test]
    fn identifier_tables_are_complete_and_exact() {
        AGGREGATIONS.check().unwrap();
        AGGREGATIONS
            .check_round_trip(&CashAggregation::ALL, |x| x.to_string())
            .unwrap();
        GUARANTEES.check().unwrap();
        GUARANTEES
            .check_round_trip(&SettlementGuarantee::ALL, |x| x.to_string())
            .unwrap();
    }
}
