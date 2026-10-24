//! Explicit Canadian fixed-coupon source, dates and analytical model.
//!
//! The selected IIAC interpretation is a versioned local engineering contract.
//! Supplied nominal/actual dates, true maturity, yield routing and cash rounding
//! remain explicit; no calendar, legal eligibility or current-market certification
//! is inferred. Exact source values remain separate from optional decimal views.

use std::error::Error as StdError;
use std::fmt;

use super::CouponEntitlement;
use crate::amounts::{AmountError, AmountQuantum, AmountRounding, DecimalAmount};
use crate::calendar::EndOfMonth;
use crate::error::ValueError;
use crate::math::SumError;
use crate::rates::RateError;
use crate::time::{Date, DateError};

mod cash;
mod numeric;
mod source;
mod wide;

pub use cash::{CanadianCashDecision, CanadianCashInstruction, CanadianCashReceipt};
pub use numeric::{CanadianRisk, CanadianValuation, CanadianYieldError, CanadianYieldReport};
use source::{Coordinate, Geometry, Source};

/// Identifies the selected local interpretation, not an IIAC effective date.
const SOURCE_VERSION: &str = "CA-IIAC13-fixed-2026-10-23-local-v1";

/// The explicit annual coupon/compound frequency.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CanadianFrequency {
    /// Two coupon periods per year, six nominal months apart.
    Semiannual,
    /// Four coupon periods per year, three nominal months apart.
    Quarterly,
    /// Twelve coupon periods per year, one nominal month apart.
    Monthly,
}

impl CanadianFrequency {
    /// All supported frequencies, semiannual, quarterly and monthly.
    pub const ALL: [Self; 3] = [Self::Semiannual, Self::Quarterly, Self::Monthly];
    /// Returns the exact annual coupon/compound count.
    #[must_use]
    pub const fn payments_per_year(self) -> u32 {
        match self {
            Self::Semiannual => 2,
            Self::Quarterly => 4,
            Self::Monthly => 12,
        }
    }
    /// Returns the direct nominal-cycle month stride.
    #[must_use]
    pub const fn months(self) -> u32 {
        12 / self.payments_per_year()
    }
}

/// Actual coupon interval relationship to the supplied quasi cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CanadianPeriodKind {
    /// One complete adjacent-Q interval.
    Regular,
    /// A nonregular interval shorter than one exact quasi period.
    Short,
    /// A nonregular interval longer than one exact quasi period.
    Long,
    /// A sole actual interval whose two endpoints are both off the Q cycle.
    Composite,
}
impl CanadianPeriodKind {
    /// All period roles in Regular, Short, Long and Composite order.
    pub const ALL: [Self; 4] = [Self::Regular, Self::Short, Self::Long, Self::Composite];
}

/// Explicit selection of the versioned Canadian source reconciliation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CanadianSourcePolicy {
    /// Retain the complete locally reconciled IIAC13 formulas and extensions.
    CanonicalIiac13LocalV1,
}
impl CanadianSourcePolicy {
    /// Every selected local source policy, with no implicit default.
    pub const ALL: [Self; 1] = [Self::CanonicalIiac13LocalV1];
}

/// Explicit annual-yield pricing equation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CanadianYieldConvention {
    /// Coupon-frequency compounding on exact quasi-period coordinates.
    Compound,
    /// True-terminal actual/365 simple money-market equation.
    MoneyMarket,
}
impl CanadianYieldConvention {
    /// Compound followed by money-market quotation.
    pub const ALL: [Self; 2] = [Self::Compound, Self::MoneyMarket];
}

/// Caller-selected yield routing; no implicit tenor default exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CanadianYieldSelection {
    /// Use the supplied equation at every admitted tenor.
    Explicit(CanadianYieldConvention),
    /// Use money-market only when the sole remaining actual coupon is terminal.
    LastCouponMoneyMarket,
}

/// The equation basis carried by a finite signed Canadian price.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CanadianPriceBasis {
    /// Remove the convention-selected source accrued observation.
    Clean,
    /// The observed positive discounted source-bundle sum.
    ModelDirty,
    /// Settlement all-in price, including the separate settlement AI bridge.
    SettlementDirty,
}
impl CanadianPriceBasis {
    /// Clean, model dirty and settlement dirty, in that order.
    pub const ALL: [Self; 3] = [Self::Clean, Self::ModelDirty, Self::SettlementDirty];
}

/// A finite signed per100 price with an explicit equation basis.
///
/// Equality and hashing use canonical finite bits and basis. Both signed zeros
/// denote the same price. No implicit generic-price or money conversion exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CanadianPrice {
    bits: u64,
    basis: CanadianPriceBasis,
}
impl CanadianPrice {
    /// Retains a finite signed value and its caller-selected equation basis.
    ///
    /// # Errors
    /// Returns [`CanadianError::Value`] for NaN or either infinity.
    pub fn new(value: f64, basis: CanadianPriceBasis) -> Result<Self, CanadianError> {
        let value = crate::numeric::finite("canadian price", value).map_err(|cause| {
            CanadianError::Value {
                index: None,
                phase: "price quotation",
                cause,
            }
        })?;
        Ok(Self::observed(value, basis))
    }
    /// Stores an already admitted finite report observation without another gate.
    fn observed(value: f64, basis: CanadianPriceBasis) -> Self {
        Self {
            bits: if value == 0.0 { 0 } else { value.to_bits() },
            basis,
        }
    }
    /// Returns the retained signed finite observation, with canonical positive zero.
    #[must_use]
    pub const fn value(self) -> f64 {
        f64::from_bits(self.bits)
    }
    /// Returns the explicitly retained price equation basis.
    #[must_use]
    pub const fn basis(self) -> CanadianPriceBasis {
        self.basis
    }
}

/// Complete owned direct-cycle and actual-coupon date evidence.
///
/// Only first/last actual periods may omit virtual dates. The supplied vectors
/// are moved without cloning; caps bound lengths rather than spare capacity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CanadianCouponSchedule {
    accrual_start: Date,
    stated_maturity: Date,
    frequency: CanadianFrequency,
    quasi_dates: Vec<Date>,
    coupon_dates: Vec<Date>,
    end_of_month: EndOfMonth,
    first_period: CanadianPeriodKind,
    last_period: CanadianPeriodKind,
    first_geometry: Geometry,
    last_geometry: Geometry,
}

impl CanadianCouponSchedule {
    /// Maximum complete supplied cycle length, including both outer endpoints.
    pub const MAX_QUASI_DATES: usize = 20_000;
    /// Maximum actual coupon-date count, including zero-valued coupons.
    pub const MAX_COUPONS: usize = 20_000;

    /// Validates the complete direct-anchored cycle and actual coupon evidence.
    ///
    /// # Errors
    /// Returns contextual [`CanadianError`] in the fixed order: cap, lengths,
    /// D<M, Q ordering, minimal coverage, direct cycle, terminal actual boundary,
    /// actual ordering, first>D, nonterminal membership and interior adjacency.
    pub fn new(
        accrual_start: Date,
        stated_maturity: Date,
        frequency: CanadianFrequency,
        quasi_dates: Vec<Date>,
        coupon_dates: Vec<Date>,
        end_of_month: EndOfMonth,
        max_dates: usize,
    ) -> Result<Self, CanadianError> {
        if !(2..=Self::MAX_QUASI_DATES).contains(&max_dates) {
            return Err(CanadianError::InvalidDateCap {
                cap: max_dates,
                minimum: 2,
                maximum: Self::MAX_QUASI_DATES,
            });
        }
        let count = quasi_dates.len();
        if count > max_dates {
            return Err(CanadianError::TooManyQuasiDates {
                count,
                cap: max_dates,
            });
        }
        if count < 2 {
            return Err(CanadianError::TooFewQuasiDates { count, minimum: 2 });
        }
        if coupon_dates.len() > max_dates {
            return Err(CanadianError::TooManyCoupons {
                count: coupon_dates.len(),
                cap: max_dates,
            });
        }
        if coupon_dates.is_empty() {
            return Err(CanadianError::NoCoupons);
        }
        if accrual_start >= stated_maturity {
            return Err(CanadianError::AccrualNotBeforeMaturity {
                accrual_start,
                stated_maturity,
            });
        }
        for (offset, pair) in quasi_dates.windows(2).enumerate() {
            if pair[1] <= pair[0] {
                return Err(CanadianError::QuasiDateOrder {
                    index: offset + 1,
                    previous: pair[0],
                    date: pair[1],
                });
            }
        }
        let last = count - 1;
        if !(quasi_dates[0] <= accrual_start
            && accrual_start < quasi_dates[1]
            && quasi_dates[last - 1] < stated_maturity
            && stated_maturity <= quasi_dates[last])
        {
            return Err(CanadianError::QuasiCoverage {
                accrual_start,
                stated_maturity,
                first: quasi_dates[0],
                next: quasi_dates[1],
                penultimate: quasi_dates[last - 1],
                last: quasi_dates[last],
            });
        }
        let anchor = quasi_dates[last];
        for (index, &supplied) in quasi_dates.iter().enumerate() {
            let steps = i64::try_from(last - index)
                .map_err(|_| capacity(Some(index), "quasi cycle", "derive source"))?;
            // Derive the target civil components directly. The generic Date
            // shift reports its starting date/count on range failure; this
            // source recurrence instead retains the rejected target year.
            let serial = i64::from(anchor.year() - 1) * 12 + i64::from(anchor.month() - 1)
                - steps * i64::from(frequency.months());
            let year = i32::try_from(serial.div_euclid(12) + 1)
                .map_err(|_| capacity(Some(index), "quasi cycle", "derive source"))?;
            let month = u32::try_from(serial.rem_euclid(12) + 1)
                .map_err(|_| capacity(Some(index), "quasi cycle", "derive source"))?;
            let date_error = |cause| CanadianError::Date {
                index: Some(index),
                phase: "quasi cycle",
                cause,
            };
            let length = Date::days_in_month(year, month).map_err(date_error)?;
            let day = if end_of_month == EndOfMonth::Preserve && anchor.is_end_of_month() {
                length
            } else {
                anchor.day().min(length)
            };
            let expected = Date::from_ymd(year, month, day).map_err(date_error)?;
            if expected != supplied {
                return Err(CanadianError::QuasiDateMismatch {
                    index,
                    supplied,
                    expected,
                    anchor,
                    frequency,
                    end_of_month,
                });
            }
        }
        let coupon_last = coupon_dates.len() - 1;
        if coupon_dates[coupon_last] != stated_maturity {
            return Err(CanadianError::CouponBoundary {
                last_coupon: coupon_dates[coupon_last],
                stated_maturity,
            });
        }
        for (offset, pair) in coupon_dates.windows(2).enumerate() {
            if pair[1] <= pair[0] {
                return Err(CanadianError::CouponDateOrder {
                    index: offset + 1,
                    previous: pair[0],
                    date: pair[1],
                });
            }
        }
        if coupon_dates[0] <= accrual_start {
            return Err(CanadianError::CouponBeforeAccrual {
                index: 0,
                date: coupon_dates[0],
                accrual_start,
            });
        }
        // Two independent linear passes preserve membership-before-adjacency order.
        let mut cursor = 0;
        for (index, &date) in coupon_dates[..coupon_last].iter().enumerate() {
            while cursor < count && quasi_dates[cursor] < date {
                cursor += 1;
            }
            if quasi_dates.get(cursor) != Some(&date) {
                return Err(CanadianError::CouponOffCycle { index, date });
            }
        }
        cursor = 0;
        for index in 1..coupon_last {
            let previous = coupon_dates[index - 1];
            let date = coupon_dates[index];
            while quasi_dates[cursor] < previous {
                cursor += 1;
            }
            if quasi_dates[cursor + 1] != date {
                return Err(CanadianError::IrregularInteriorPeriod {
                    index,
                    previous,
                    date,
                });
            }
        }
        let first_geometry = Geometry::new(&quasi_dates, accrual_start, coupon_dates[0]);
        let last_start = if coupon_last == 0 {
            accrual_start
        } else {
            coupon_dates[coupon_last - 1]
        };
        let last_geometry = Geometry::new(&quasi_dates, last_start, stated_maturity);
        let single = coupon_last == 0;
        Ok(Self {
            accrual_start,
            stated_maturity,
            frequency,
            quasi_dates,
            coupon_dates,
            end_of_month,
            first_period: first_geometry.period_kind(single),
            last_period: last_geometry.period_kind(single),
            first_geometry,
            last_geometry,
        })
    }

    /// Returns the original contractual accrual date D.
    #[must_use]
    pub const fn accrual_start(&self) -> Date {
        self.accrual_start
    }
    /// Returns the stated contractual terminal coupon date M.
    #[must_use]
    pub const fn stated_maturity(&self) -> Date {
        self.stated_maturity
    }
    /// Returns the retained coupon/compound frequency.
    #[must_use]
    pub const fn frequency(&self) -> CanadianFrequency {
        self.frequency
    }
    /// Returns the explicit month-end policy.
    #[must_use]
    pub const fn end_of_month(&self) -> EndOfMonth {
        self.end_of_month
    }
    /// Returns the last supplied Q, the direct recurrence anchor.
    #[must_use]
    pub fn direct_cycle_anchor(&self) -> Date {
        self.quasi_dates[self.quasi_dates.len() - 1]
    }
    /// Borrows the full supplied quasi cycle, including both outer endpoints.
    #[must_use]
    pub fn quasi_dates(&self) -> &[Date] {
        &self.quasi_dates
    }
    /// Borrows the actual coupon dates, including zero-valued coupons.
    #[must_use]
    pub fn coupon_dates(&self) -> &[Date] {
        &self.coupon_dates
    }
    /// Returns the full original actual coupon count.
    #[must_use]
    pub fn coupon_count(&self) -> usize {
        self.coupon_dates.len()
    }
    /// Returns F, the first original actual coupon date.
    #[must_use]
    pub fn first_coupon_date(&self) -> Date {
        self.coupon_dates[0]
    }
    /// Returns L, the start of the last actual coupon interval; D if sole coupon.
    #[must_use]
    pub fn last_coupon_start_date(&self) -> Date {
        self.coupon_start_date(self.coupon_count() - 1)
            .unwrap_or(self.accrual_start)
    }
    /// Returns an actual interval's start using its original coupon ordinal.
    #[must_use]
    pub fn coupon_start_date(&self, index: usize) -> Option<Date> {
        if index >= self.coupon_count() {
            None
        } else if index == 0 {
            Some(self.accrual_start)
        } else {
            self.coupon_dates.get(index - 1).copied()
        }
    }
    /// Returns an actual interval's exact role using its original coupon ordinal.
    #[must_use]
    pub fn coupon_period_kind(&self, index: usize) -> Option<CanadianPeriodKind> {
        if index >= self.coupon_count() {
            None
        } else if index == 0 {
            Some(self.first_period)
        } else if index + 1 == self.coupon_count() {
            Some(self.last_period)
        } else {
            Some(CanadianPeriodKind::Regular)
        }
    }
    /// Returns the original first actual interval's role.
    #[must_use]
    pub const fn first_period(&self) -> CanadianPeriodKind {
        self.first_period
    }
    /// Returns the original last actual interval's role.
    #[must_use]
    pub const fn last_period(&self) -> CanadianPeriodKind {
        self.last_period
    }

    /// Obtains cached edge geometry or a known regular actual adjacent pair.
    fn coupon_geometry(&self, index: usize) -> Geometry {
        if index == 0 {
            self.first_geometry
        } else if index + 1 == self.coupon_count() {
            self.last_geometry
        } else {
            let start = self.coupon_dates[index - 1];
            let end = self.coupon_dates[index];
            Geometry {
                start,
                end,
                first_quasi: (start, end),
                last_quasi: (start, end),
                full_count: 1,
                first: None,
                second: None,
            }
        }
    }
}

/// Immutable fixed-par100 Canadian coupon definition with explicit true maturity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CanadianBond {
    original_issue_date: Date,
    coupon_per_100: DecimalAmount,
    schedule: CanadianCouponSchedule,
    true_maturity_date: Date,
    source_policy: CanadianSourcePolicy,
    regular: Source,
    first_valuation: Source,
    first_payable: Source,
    last_valuation: Source,
    last_payable: Source,
}
impl CanadianBond {
    /// Validates issue, coupon and true-terminal roles, then caches exact sources.
    ///
    /// # Errors
    /// Rejects negative C, D>I, I>=F and T<M in that order. Bounded source-helper
    /// failure retains [`CanadianError::InternalCapacity`] context.
    pub fn new(
        original_issue: Date,
        coupon_per_100: DecimalAmount,
        schedule: CanadianCouponSchedule,
        true_maturity: Date,
        source_policy: CanadianSourcePolicy,
    ) -> Result<Self, CanadianError> {
        if coupon_per_100.coefficient() < 0 {
            return Err(CanadianError::NegativeCoupon { coupon_per_100 });
        }
        if schedule.accrual_start > original_issue {
            return Err(CanadianError::DatedAfterIssue {
                accrual_start: schedule.accrual_start,
                original_issue,
            });
        }
        if original_issue >= schedule.first_coupon_date() {
            return Err(CanadianError::IssueNotBeforeFirstCoupon {
                original_issue,
                first_coupon: schedule.first_coupon_date(),
            });
        }
        if true_maturity < schedule.stated_maturity {
            return Err(CanadianError::TrueMaturityBeforeStated {
                true_maturity,
                stated_maturity: schedule.stated_maturity,
            });
        }
        let regular_geometry = Geometry::cells(
            &schedule.quasi_dates,
            schedule.quasi_dates[0],
            schedule.quasi_dates[1],
            0,
            0,
        );
        let regular = Source::coupon(
            coupon_per_100,
            schedule.frequency,
            regular_geometry,
            false,
            Some(0),
            "source valuation coupon",
        )?;
        let first_valuation = Source::coupon(
            coupon_per_100,
            schedule.frequency,
            schedule.first_geometry,
            false,
            Some(0),
            "source valuation coupon",
        )?;
        let first_payable = Source::coupon(
            coupon_per_100,
            schedule.frequency,
            schedule.first_geometry,
            true,
            Some(0),
            "source payable coupon",
        )?;
        let last = schedule.coupon_count() - 1;
        let last_valuation = if last == 0 {
            first_valuation
        } else {
            Source::coupon(
                coupon_per_100,
                schedule.frequency,
                schedule.last_geometry,
                false,
                Some(last),
                "source valuation coupon",
            )?
        };
        let last_payable = if last == 0 {
            first_payable
        } else {
            Source::coupon(
                coupon_per_100,
                schedule.frequency,
                schedule.last_geometry,
                true,
                Some(last),
                "source payable coupon",
            )?
        };
        Ok(Self {
            original_issue_date: original_issue,
            coupon_per_100,
            schedule,
            true_maturity_date: true_maturity,
            source_policy,
            regular,
            first_valuation,
            first_payable,
            last_valuation,
            last_payable,
        })
    }
    /// Returns the exact retained local source interpretation identifier.
    #[must_use]
    pub const fn source_version(&self) -> &'static str {
        SOURCE_VERSION
    }
    /// Returns the explicit source reconciliation policy.
    #[must_use]
    pub const fn source_policy(&self) -> CanadianSourcePolicy {
        self.source_policy
    }
    /// Returns the original issue date I.
    #[must_use]
    pub const fn original_issue_date(&self) -> Date {
        self.original_issue_date
    }
    /// Returns exact annual coupon C per100.
    #[must_use]
    pub const fn coupon_per_100(&self) -> DecimalAmount {
        self.coupon_per_100
    }
    /// Borrows the complete retained contractual cycle and actual dates.
    #[must_use]
    pub const fn schedule(&self) -> &CanadianCouponSchedule {
        &self.schedule
    }
    /// Returns the caller-supplied true terminal payment date T.
    #[must_use]
    pub const fn true_maturity_date(&self) -> Date {
        self.true_maturity_date
    }
    /// Returns the first actual coupon date F.
    #[must_use]
    pub fn first_coupon_date(&self) -> Date {
        self.schedule.first_coupon_date()
    }
    /// Returns the last actual interval's start L.
    #[must_use]
    pub fn last_coupon_start_date(&self) -> Date {
        self.schedule.last_coupon_start_date()
    }
    /// Returns contractual payment date for nonterminal coupons and T for terminal.
    #[must_use]
    pub fn coupon_payment_date(&self, index: usize) -> Option<Date> {
        let date = self.schedule.coupon_dates.get(index).copied()?;
        Some(if index + 1 == self.schedule.coupon_count() {
            self.true_maturity_date
        } else {
            date
        })
    }
    /// Explicitly quantizes a cached valuation coupon source at the caller's q/m.
    ///
    /// # Errors
    /// Invalid original ordinal or exact quantization/admission failure.
    pub fn valuation_coupon_per_100(
        &self,
        index: usize,
        q: AmountQuantum,
        mode: AmountRounding,
    ) -> Result<DecimalAmount, CanadianError> {
        cash::source_view(
            self.valuation_coupon(index)?,
            q,
            mode,
            Some(index),
            "source valuation coupon",
        )
    }
    /// Explicitly quantizes a cached payable coupon source at the caller's q/m.
    ///
    /// # Errors
    /// Invalid original ordinal or exact quantization/admission failure.
    pub fn payable_coupon_per_100(
        &self,
        index: usize,
        q: AmountQuantum,
        mode: AmountRounding,
    ) -> Result<DecimalAmount, CanadianError> {
        cash::source_view(
            self.payable_coupon(index)?,
            q,
            mode,
            Some(index),
            "source payable coupon",
        )
    }
    fn check_coupon_index(&self, index: usize) -> Result<(), CanadianError> {
        if index < self.schedule.coupon_count() {
            Ok(())
        } else {
            Err(CanadianError::InvalidCouponIndex {
                index,
                count: self.schedule.coupon_count(),
            })
        }
    }
    fn valuation_coupon(&self, index: usize) -> Result<Source, CanadianError> {
        self.check_coupon_index(index)?;
        Ok(if index == 0 {
            self.first_valuation
        } else if index + 1 == self.schedule.coupon_count() {
            self.last_valuation
        } else {
            self.regular
        })
    }
    fn payable_coupon(&self, index: usize) -> Result<Source, CanadianError> {
        self.check_coupon_index(index)?;
        Ok(if index == 0 {
            self.first_payable
        } else if index + 1 == self.schedule.coupon_count() {
            self.last_payable
        } else {
            self.regular
        })
    }
    fn coupon_geometry(&self, index: usize) -> Result<Geometry, CanadianError> {
        self.check_coupon_index(index)?;
        Ok(self.schedule.coupon_geometry(index))
    }
}

/// One selected future source coefficient; original ordinals are never compressed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Bundle {
    index: usize,
    amount: Source,
    lambda: Coordinate,
    days_to_payment: u32,
    days_payment_to_true: u32,
}

/// Compact owned numerical context; every report can reconstruct the definition.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Metadata {
    source_policy: CanadianSourcePolicy,
    accrual_start: Date,
    original_issue_date: Date,
    stated_maturity_date: Date,
    true_maturity_date: Date,
    first_coupon_date: Date,
    last_coupon_start_date: Date,
    first_period: CanadianPeriodKind,
    last_period: CanadianPeriodKind,
    frequency: CanadianFrequency,
    coupon_per_100: DecimalAmount,
    end_of_month: EndOfMonth,
    direct_cycle_anchor: Date,
    settlement_date: Date,
    yield_selection: CanadianYieldSelection,
    yield_convention: CanadianYieldConvention,
    valuation_accrued_per_100: f64,
    settlement_accrued_per_100: f64,
    quote_accrued_per_100: f64,
    days_to_true_maturity: u32,
}

/// Borrowed immutable Canadian definition with one bounded future-bundle vector.
#[derive(Clone, Debug, PartialEq)]
pub struct CanadianPricingPlan<'a> {
    bond: &'a CanadianBond,
    metadata: Metadata,
    bundles: Vec<Bundle>,
    valuation_ai: Source,
    settlement_ai: Source,
    accrued_geometry: Geometry,
}
impl<'a> CanadianPricingPlan<'a> {
    /// Maximum number of future actual coupon/principal bundles.
    pub const MAX_BUNDLES: usize = 20_000;
    /// Prepares exact source/time metadata once, then reserves one bundle vector.
    ///
    /// # Errors
    /// Checks cap, S>=I, S<M, future count<=cap, exact source caches and observations
    /// before the one fallible allocation. Original coupon ordinals are retained.
    pub fn new(
        bond: &'a CanadianBond,
        settlement: Date,
        selection: CanadianYieldSelection,
        max_bundles: usize,
    ) -> Result<Self, CanadianError> {
        if max_bundles > Self::MAX_BUNDLES {
            return Err(CanadianError::InvalidBundleCap {
                cap: max_bundles,
                maximum: Self::MAX_BUNDLES,
            });
        }
        if settlement < bond.original_issue_date {
            return Err(CanadianError::SettlementBeforeIssue {
                settlement,
                original_issue: bond.original_issue_date,
            });
        }
        let schedule = &bond.schedule;
        if settlement >= schedule.stated_maturity {
            return Err(CanadianError::SettlementAtOrAfterMaturity {
                settlement,
                stated_maturity: schedule.stated_maturity,
            });
        }
        let next = schedule
            .coupon_dates
            .partition_point(|date| *date <= settlement);
        let count = schedule.coupon_count() - next;
        if count > max_bundles {
            return Err(CanadianError::TooManyBundles {
                count,
                cap: max_bundles,
            });
        }
        let convention = match selection {
            CanadianYieldSelection::Explicit(x) => x,
            CanadianYieldSelection::LastCouponMoneyMarket => {
                if count == 1 {
                    CanadianYieldConvention::MoneyMarket
                } else {
                    CanadianYieldConvention::Compound
                }
            }
        };
        let start = if next == 0 {
            schedule.accrual_start
        } else {
            schedule.coupon_dates[next - 1]
        };
        let accrued_geometry = Geometry::new(&schedule.quasi_dates, start, settlement);
        let valuation_ai = Source::coupon(
            bond.coupon_per_100,
            schedule.frequency,
            accrued_geometry,
            false,
            None,
            "source valuation accrued",
        )?;
        let settlement_ai = if schedule.frequency == CanadianFrequency::Monthly {
            valuation_ai
        } else {
            Source::coupon(
                bond.coupon_per_100,
                schedule.frequency,
                accrued_geometry,
                true,
                None,
                "source settlement accrued",
            )?
        };
        let valuation_observed = valuation_ai.observe(None, "source valuation accrued")?;
        let settlement_observed = settlement_ai.observe(None, "source settlement accrued")?;
        let metadata = Metadata {
            source_policy: bond.source_policy,
            accrual_start: schedule.accrual_start,
            original_issue_date: bond.original_issue_date,
            stated_maturity_date: schedule.stated_maturity,
            true_maturity_date: bond.true_maturity_date,
            first_coupon_date: schedule.first_coupon_date(),
            last_coupon_start_date: schedule.last_coupon_start_date(),
            first_period: schedule.first_period,
            last_period: schedule.last_period,
            frequency: schedule.frequency,
            coupon_per_100: bond.coupon_per_100,
            end_of_month: schedule.end_of_month,
            direct_cycle_anchor: schedule.direct_cycle_anchor(),
            settlement_date: settlement,
            yield_selection: selection,
            yield_convention: convention,
            valuation_accrued_per_100: valuation_observed,
            settlement_accrued_per_100: settlement_observed,
            quote_accrued_per_100: if convention == CanadianYieldConvention::Compound {
                valuation_observed
            } else {
                settlement_observed
            },
            days_to_true_maturity: bond
                .true_maturity_date
                .days_since(settlement)
                .unsigned_abs(),
        };
        let mut bundles = Vec::new();
        bundles
            .try_reserve_exact(count)
            .map_err(|_| CanadianError::AllocationFailed {
                phase: "pricing bundles",
                requested: count,
            })?;
        let left = schedule
            .quasi_dates
            .partition_point(|date| *date <= settlement)
            - 1;
        let mut right = left;
        for index in next..schedule.coupon_count() {
            let date = schedule.coupon_dates[index];
            while schedule.quasi_dates[right + 1] < date {
                right += 1;
            }
            let lambda =
                Geometry::cells(&schedule.quasi_dates, settlement, date, left, right).coordinate();
            let mut amount = if convention == CanadianYieldConvention::Compound {
                bond.valuation_coupon(index)?
            } else {
                bond.payable_coupon(index)?
            };
            let payment = if index + 1 == schedule.coupon_count() {
                amount = amount.with_par(index)?;
                bond.true_maturity_date
            } else {
                date
            };
            bundles.push(Bundle {
                index,
                amount,
                lambda,
                days_to_payment: payment.days_since(settlement).unsigned_abs(),
                days_payment_to_true: bond.true_maturity_date.days_since(payment).unsigned_abs(),
            });
        }
        Ok(Self {
            bond,
            metadata,
            bundles,
            valuation_ai,
            settlement_ai,
            accrued_geometry,
        })
    }
    /// Borrows the original immutable definition, without a cloned schedule.
    #[must_use]
    pub const fn bond(&self) -> &'a CanadianBond {
        self.bond
    }
    /// Returns the exact versioned local source interpretation.
    #[must_use]
    pub const fn source_version(&self) -> &'static str {
        SOURCE_VERSION
    }
    /// Returns the retained explicit source policy.
    #[must_use]
    pub const fn source_policy(&self) -> CanadianSourcePolicy {
        self.metadata.source_policy
    }
    /// Returns the admitted trading settlement date S.
    #[must_use]
    pub const fn settlement_date(&self) -> Date {
        self.metadata.settlement_date
    }
    /// Returns the caller's routing selection unchanged.
    #[must_use]
    pub const fn yield_selection(&self) -> CanadianYieldSelection {
        self.metadata.yield_selection
    }
    /// Returns the resolved equation for this prepared remaining actual count.
    #[must_use]
    pub const fn yield_convention(&self) -> CanadianYieldConvention {
        self.metadata.yield_convention
    }
    /// Returns the future-bundle count, including all zero-valued coupons.
    #[must_use]
    pub fn bundle_count(&self) -> usize {
        self.bundles.len()
    }
    /// Maps compressed future position to original actual coupon ordinal.
    #[must_use]
    pub fn bundle_coupon_index(&self, index: usize) -> Option<usize> {
        self.bundles.get(index).map(|bundle| bundle.index)
    }
    /// Reports entitlement using an original actual ordinal, including ceased ones.
    #[must_use]
    pub fn coupon_entitlement(&self, index: usize) -> Option<CouponEntitlement> {
        self.bond.schedule.coupon_dates.get(index).map(|date| {
            if *date <= self.metadata.settlement_date {
                CouponEntitlement::Ceased
            } else {
                CouponEntitlement::BuyerEntitled
            }
        })
    }
    /// Returns the independently rounded exact valuation A cache.
    #[must_use]
    pub const fn valuation_accrued_per_100(&self) -> f64 {
        self.metadata.valuation_accrued_per_100
    }
    /// Returns the independently rounded exact settlement AI cache.
    #[must_use]
    pub const fn settlement_accrued_per_100(&self) -> f64 {
        self.metadata.settlement_accrued_per_100
    }
    /// Returns Ahat for compound, AIhat for money-market clean quotation.
    #[must_use]
    pub const fn quote_accrued_per_100(&self) -> f64 {
        self.metadata.quote_accrued_per_100
    }
    /// Explicitly quantizes exact valuation A; never gates numerical pricing.
    ///
    /// # Errors
    /// Returns the exact quantum/admission failure for this independent source view.
    pub fn valuation_accrued_at_quantum(
        &self,
        q: AmountQuantum,
        mode: AmountRounding,
    ) -> Result<DecimalAmount, CanadianError> {
        cash::source_view(self.valuation_ai, q, mode, None, "source valuation accrued")
    }
    /// Explicitly quantizes exact settlement AI; never inserts a cash intermediate.
    ///
    /// # Errors
    /// Returns the exact quantum/admission failure for this independent source view.
    pub fn settlement_accrued_at_quantum(
        &self,
        q: AmountQuantum,
        mode: AmountRounding,
    ) -> Result<DecimalAmount, CanadianError> {
        cash::source_view(
            self.settlement_ai,
            q,
            mode,
            None,
            "source settlement accrued",
        )
    }
}

/// A Canadian date/source, observation, resource or exact-cash failure.
///
/// Contextual variants retain original actual coupon ordinals; only quasi-cycle
/// date validation uses an original supplied Q ordinal. Typed wrapped causes are
/// exposed immediately through [`StdError::source`]. Resource failures do not
/// stand in for ordinary numerical or decimal range failures.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CanadianError {
    /// The supplied schedule cap is outside its admitted inclusive range.
    #[non_exhaustive]
    InvalidDateCap {
        /// Supplied cap.
        cap: usize,
        /// Smallest admitted cap.
        minimum: usize,
        /// Largest admitted cap.
        maximum: usize,
    },
    /// The supplied full quasi cycle exceeds the caller's length cap.
    #[non_exhaustive]
    TooManyQuasiDates {
        /// Supplied quasi-date count.
        count: usize,
        /// Caller-selected count cap.
        cap: usize,
    },
    /// A full quasi cycle needs at least two outer dates.
    #[non_exhaustive]
    TooFewQuasiDates {
        /// Supplied quasi-date count.
        count: usize,
        /// Minimum full-cycle count.
        minimum: usize,
    },
    /// The supplied actual coupon dates exceed the caller's count cap.
    #[non_exhaustive]
    TooManyCoupons {
        /// Supplied actual coupon count.
        count: usize,
        /// Caller-selected count cap.
        cap: usize,
    },
    /// At least one actual terminal coupon date is required, even for C0.
    NoCoupons,
    /// The accrual start D is not strictly before stated maturity M.
    #[non_exhaustive]
    AccrualNotBeforeMaturity {
        /// Supplied accrual start D.
        accrual_start: Date,
        /// Supplied stated maturity M.
        stated_maturity: Date,
    },
    /// Consecutive supplied quasi dates are equal or decreasing.
    #[non_exhaustive]
    QuasiDateOrder {
        /// Original supplied Q ordinal of the latter date.
        index: usize,
        /// Previous supplied date.
        previous: Date,
        /// Equal or earlier latter date.
        date: Date,
    },
    /// The supplied full cycle does not minimally cover D and M.
    #[non_exhaustive]
    QuasiCoverage {
        /// Supplied accrual start D.
        accrual_start: Date,
        /// Supplied stated maturity M.
        stated_maturity: Date,
        /// First supplied outer quasi date.
        first: Date,
        /// Second supplied quasi date.
        next: Date,
        /// Penultimate supplied quasi date.
        penultimate: Date,
        /// Last supplied outer quasi date.
        last: Date,
    },
    /// A supplied Q date differs from the direct last-anchor recurrence.
    #[non_exhaustive]
    QuasiDateMismatch {
        /// Original supplied Q ordinal.
        index: usize,
        /// Supplied date at this ordinal.
        supplied: Date,
        /// Date obtained directly from the last anchor.
        expected: Date,
        /// Last supplied Q, retained as the direct anchor.
        anchor: Date,
        /// Selected nominal-cycle frequency.
        frequency: CanadianFrequency,
        /// Explicit month-end policy.
        end_of_month: EndOfMonth,
    },
    /// Consecutive actual coupon dates are equal or decreasing.
    #[non_exhaustive]
    CouponDateOrder {
        /// Original actual ordinal of the latter date.
        index: usize,
        /// Previous actual coupon date.
        previous: Date,
        /// Equal or earlier latter actual date.
        date: Date,
    },
    /// The last actual coupon date differs from stated maturity M.
    #[non_exhaustive]
    CouponBoundary {
        /// Last supplied actual coupon date.
        last_coupon: Date,
        /// Supplied stated maturity M.
        stated_maturity: Date,
    },
    /// The first actual coupon is on or before accrual start D.
    #[non_exhaustive]
    CouponBeforeAccrual {
        /// Original actual coupon ordinal.
        index: usize,
        /// Rejected actual coupon date.
        date: Date,
        /// Retained accrual start D.
        accrual_start: Date,
    },
    /// A nonterminal actual coupon date is absent from the supplied cycle.
    #[non_exhaustive]
    CouponOffCycle {
        /// Original actual coupon ordinal.
        index: usize,
        /// Rejected nonterminal actual date.
        date: Date,
    },
    /// An interior actual interval skips a nominal cycle boundary.
    #[non_exhaustive]
    IrregularInteriorPeriod {
        /// Original actual ordinal at the interval end.
        index: usize,
        /// Previous actual coupon date.
        previous: Date,
        /// Current actual coupon date.
        date: Date,
    },
    /// A checked civil-date operation failed in the named source phase.
    #[non_exhaustive]
    Date {
        /// Original supplied Q or actual coupon ordinal, where applicable.
        index: Option<usize>,
        /// Exact operation phase.
        phase: &'static str,
        /// Immediate typed civil-date cause.
        cause: DateError,
    },
    /// Annual coupon C per100 must be nonnegative.
    #[non_exhaustive]
    NegativeCoupon {
        /// Rejected exact annual coupon C.
        coupon_per_100: DecimalAmount,
    },
    /// Accrual start D lies after original issue I.
    #[non_exhaustive]
    DatedAfterIssue {
        /// Retained accrual start D.
        accrual_start: Date,
        /// Rejected original issue I.
        original_issue: Date,
    },
    /// Original issue I is on or after first actual coupon F.
    #[non_exhaustive]
    IssueNotBeforeFirstCoupon {
        /// Rejected original issue I.
        original_issue: Date,
        /// First actual coupon F.
        first_coupon: Date,
    },
    /// True terminal payment T precedes stated maturity M.
    #[non_exhaustive]
    TrueMaturityBeforeStated {
        /// Rejected true terminal payment T.
        true_maturity: Date,
        /// Retained stated maturity M.
        stated_maturity: Date,
    },
    /// The future-bundle cap exceeds its maximum; zero is an admitted cap.
    #[non_exhaustive]
    InvalidBundleCap {
        /// Supplied bundle cap.
        cap: usize,
        /// Maximum admitted cap.
        maximum: usize,
    },
    /// All future actual coupons, including zeros, exceed the supplied cap.
    #[non_exhaustive]
    TooManyBundles {
        /// Required future bundle count.
        count: usize,
        /// Caller-selected cap.
        cap: usize,
    },
    /// Settlement S precedes original issue I.
    #[non_exhaustive]
    SettlementBeforeIssue {
        /// Rejected settlement S.
        settlement: Date,
        /// Retained original issue I.
        original_issue: Date,
    },
    /// Settlement S is on or after stated maturity M, even if T is later.
    #[non_exhaustive]
    SettlementAtOrAfterMaturity {
        /// Rejected settlement S.
        settlement: Date,
        /// Retained stated maturity M.
        stated_maturity: Date,
    },
    /// A selected original actual coupon ordinal is outside the full schedule.
    #[non_exhaustive]
    InvalidCouponIndex {
        /// Supplied original actual coupon ordinal.
        index: usize,
        /// Full original actual coupon count.
        count: usize,
    },
    /// Exact money-market affine denominator is not strictly positive.
    #[non_exhaustive]
    MoneyMarketDomain {
        /// Rejected finite annual quote.
        yield_rate: f64,
        /// Actual days S to true terminal payment T.
        days_to_true_maturity: u32,
    },
    /// A finite/domain/observed-range check failed in its exact named phase.
    #[non_exhaustive]
    Value {
        /// Original actual coupon ordinal, or None for shared/report work.
        index: Option<usize>,
        /// Exact observation phase.
        phase: &'static str,
        /// Immediate typed value cause.
        cause: ValueError,
    },
    /// A compound observation or closed numerical guard rejected the quote.
    #[non_exhaustive]
    Rate {
        /// Original actual coupon ordinal, or None for shared-log work.
        index: Option<usize>,
        /// Exact observation phase.
        phase: &'static str,
        /// Immediate typed rate/profile cause.
        cause: RateError,
    },
    /// One exact IEEE reduction could not return its prescribed observation.
    #[non_exhaustive]
    Sum {
        /// Exact reduction phase.
        phase: &'static str,
        /// Immediate typed exact-sum cause.
        cause: SumError,
    },
    /// One bounded fallible vector reservation failed.
    #[non_exhaustive]
    AllocationFailed {
        /// Exact allocation phase.
        phase: &'static str,
        /// Requested element count, not bytes or spare capacity.
        requested: usize,
    },
    /// A proved fixed-capacity invariant failed in a bounded helper.
    #[non_exhaustive]
    InternalCapacity {
        /// Original actual ordinal, or None for shared/report work.
        index: Option<usize>,
        /// Exact operation phase.
        phase: &'static str,
        /// Exact bounded-helper operation.
        operation: &'static str,
    },
    /// Defensive normalized-view rejection of an observed zero dirty denominator.
    #[non_exhaustive]
    ZeroDirtyPrice {
        /// Selected normalized measure.
        measure: &'static str,
        /// Original annual yield quotation.
        yield_rate: f64,
    },
    /// Cash nominal must be strictly positive, including for zero source cash.
    #[non_exhaustive]
    NonPositiveNominal {
        /// Rejected exact nominal.
        nominal: DecimalAmount,
    },
    /// One exact-money stage failed divisibility or final amount admission.
    #[non_exhaustive]
    CashArithmetic {
        /// Exact source-view or cash stage.
        stage: &'static str,
        /// Current image endpoint: point, lower or upper.
        endpoint: &'static str,
        /// Immediate typed amount cause.
        cause: AmountError,
    },
}

impl CanadianError {
    /// Returns the stable machine-readable Canadian domain kind.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::InvalidDateCap { .. } => "canadian.invalid_date_cap",
            Self::TooManyQuasiDates { .. } => "canadian.too_many_quasi_dates",
            Self::TooFewQuasiDates { .. } => "canadian.too_few_quasi_dates",
            Self::TooManyCoupons { .. } => "canadian.too_many_coupons",
            Self::NoCoupons => "canadian.no_coupons",
            Self::AccrualNotBeforeMaturity { .. } => "canadian.accrual_not_before_maturity",
            Self::QuasiDateOrder { .. } => "canadian.quasi_date_order",
            Self::QuasiCoverage { .. } => "canadian.quasi_coverage",
            Self::QuasiDateMismatch { .. } => "canadian.quasi_date_mismatch",
            Self::CouponDateOrder { .. } => "canadian.coupon_date_order",
            Self::CouponBoundary { .. } => "canadian.coupon_boundary",
            Self::CouponBeforeAccrual { .. } => "canadian.coupon_before_accrual",
            Self::CouponOffCycle { .. } => "canadian.coupon_off_cycle",
            Self::IrregularInteriorPeriod { .. } => "canadian.irregular_interior_period",
            Self::Date { .. } => "canadian.date",
            Self::NegativeCoupon { .. } => "canadian.negative_coupon",
            Self::DatedAfterIssue { .. } => "canadian.dated_after_issue",
            Self::IssueNotBeforeFirstCoupon { .. } => "canadian.issue_not_before_first_coupon",
            Self::TrueMaturityBeforeStated { .. } => "canadian.true_maturity_before_stated",
            Self::InvalidBundleCap { .. } => "canadian.invalid_bundle_cap",
            Self::TooManyBundles { .. } => "canadian.too_many_bundles",
            Self::SettlementBeforeIssue { .. } => "canadian.settlement_before_issue",
            Self::SettlementAtOrAfterMaturity { .. } => "canadian.settlement_at_or_after_maturity",
            Self::InvalidCouponIndex { .. } => "canadian.invalid_coupon_index",
            Self::MoneyMarketDomain { .. } => "canadian.money_market_domain",
            Self::Value { .. } => "canadian.value",
            Self::Rate { .. } => "canadian.rate",
            Self::Sum { .. } => "canadian.sum",
            Self::AllocationFailed { .. } => "canadian.allocation_failed",
            Self::InternalCapacity { .. } => "canadian.internal_capacity",
            Self::ZeroDirtyPrice { .. } => "canadian.zero_dirty_price",
            Self::NonPositiveNominal { .. } => "canadian.nonpositive_nominal",
            Self::CashArithmetic { .. } => "canadian.cash_arithmetic",
        }
    }
}

impl fmt::Display for CanadianError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDateCap {
                cap,
                minimum,
                maximum,
            } => write!(
                f,
                "Canadian date cap {cap} is outside {minimum}..={maximum}"
            ),
            Self::TooManyQuasiDates { count, cap } => {
                write!(f, "Canadian supplied Q count {count} exceeds cap {cap}")
            }
            Self::TooFewQuasiDates { count, minimum } => {
                write!(f, "Canadian supplied Q count {count} is below {minimum}")
            }
            Self::TooManyCoupons { count, cap } => {
                write!(f, "Canadian actual coupon count {count} exceeds cap {cap}")
            }
            Self::NoCoupons => f.write_str("Canadian actual coupon dates are empty"),
            Self::AccrualNotBeforeMaturity {
                accrual_start,
                stated_maturity,
            } => write!(
                f,
                "Canadian accrual start {accrual_start} is not before stated maturity {stated_maturity}"
            ),
            Self::QuasiDateOrder {
                index,
                previous,
                date,
            } => write!(f, "Canadian Q[{index}]={date} is not after {previous}"),
            Self::QuasiCoverage {
                accrual_start,
                stated_maturity,
                first,
                next,
                penultimate,
                last,
            } => write!(
                f,
                "Canadian Q outer pairs [{first},{next}] and [{penultimate},{last}] do not minimally cover D={accrual_start}, M={stated_maturity}"
            ),
            Self::QuasiDateMismatch {
                index,
                supplied,
                expected,
                anchor,
                frequency,
                end_of_month,
            } => write!(
                f,
                "Canadian Q[{index}]={supplied} differs from {expected} at direct anchor {anchor}, frequency {frequency:?}, EOM {end_of_month:?}"
            ),
            Self::CouponDateOrder {
                index,
                previous,
                date,
            } => write!(
                f,
                "Canadian actual coupon[{index}]={date} is not after {previous}"
            ),
            Self::CouponBoundary {
                last_coupon,
                stated_maturity,
            } => write!(
                f,
                "Canadian final actual coupon {last_coupon} differs from M={stated_maturity}"
            ),
            Self::CouponBeforeAccrual {
                index,
                date,
                accrual_start,
            } => write!(
                f,
                "Canadian actual coupon[{index}]={date} is not after D={accrual_start}"
            ),
            Self::CouponOffCycle { index, date } => write!(
                f,
                "Canadian nonterminal actual coupon[{index}]={date} is off the supplied Q cycle"
            ),
            Self::IrregularInteriorPeriod {
                index,
                previous,
                date,
            } => write!(
                f,
                "Canadian interior actual coupon[{index}] interval [{previous},{date}] is not one adjacent Q period"
            ),
            Self::Date {
                index,
                phase,
                cause,
            } => write!(f, "Canadian {phase} at index {index:?}: {cause}"),
            Self::NegativeCoupon { coupon_per_100 } => write!(
                f,
                "Canadian annual coupon {coupon_per_100} per100 is negative"
            ),
            Self::DatedAfterIssue {
                accrual_start,
                original_issue,
            } => write!(f, "Canadian D={accrual_start} is after I={original_issue}"),
            Self::IssueNotBeforeFirstCoupon {
                original_issue,
                first_coupon,
            } => write!(
                f,
                "Canadian I={original_issue} is not before F={first_coupon}"
            ),
            Self::TrueMaturityBeforeStated {
                true_maturity,
                stated_maturity,
            } => write!(
                f,
                "Canadian T={true_maturity} is before M={stated_maturity}"
            ),
            Self::InvalidBundleCap { cap, maximum } => {
                write!(f, "Canadian bundle cap {cap} exceeds {maximum}")
            }
            Self::TooManyBundles { count, cap } => {
                write!(f, "Canadian future bundle count {count} exceeds cap {cap}")
            }
            Self::SettlementBeforeIssue {
                settlement,
                original_issue,
            } => write!(f, "Canadian S={settlement} is before I={original_issue}"),
            Self::SettlementAtOrAfterMaturity {
                settlement,
                stated_maturity,
            } => write!(
                f,
                "Canadian S={settlement} is on or after M={stated_maturity}"
            ),
            Self::InvalidCouponIndex { index, count } => write!(
                f,
                "Canadian original coupon ordinal {index} is outside count {count}"
            ),
            Self::MoneyMarketDomain {
                yield_rate,
                days_to_true_maturity,
            } => write!(
                f,
                "Canadian money-market denominator is nonpositive for y={yield_rate}, S-to-T days={days_to_true_maturity}"
            ),
            Self::Value {
                index,
                phase,
                cause,
            } => write!(f, "Canadian {phase} at actual index {index:?}: {cause}"),
            Self::Rate {
                index,
                phase,
                cause,
            } => write!(f, "Canadian {phase} at actual index {index:?}: {cause}"),
            Self::Sum { phase, cause } => write!(f, "Canadian {phase}: {cause}"),
            Self::AllocationFailed { phase, requested } => {
                write!(f, "Canadian {phase} could not reserve {requested} elements")
            }
            Self::InternalCapacity {
                index,
                phase,
                operation,
            } => write!(
                f,
                "Canadian {phase} bounded operation {operation} exceeded capacity at index {index:?}"
            ),
            Self::ZeroDirtyPrice {
                measure,
                yield_rate,
            } => write!(
                f,
                "Canadian {measure} has zero observed dirty denominator at y={yield_rate}"
            ),
            Self::NonPositiveNominal { nominal } => {
                write!(f, "Canadian cash nominal {nominal} is not positive")
            }
            Self::CashArithmetic {
                stage,
                endpoint,
                cause,
            } => write!(f, "Canadian {stage}/{endpoint}: {cause}"),
        }
    }
}

impl StdError for CanadianError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Date { cause, .. } => Some(cause),
            Self::Value { cause, .. } => Some(cause),
            Self::Rate { cause, .. } => Some(cause),
            Self::Sum { cause, .. } => Some(cause),
            Self::CashArithmetic { cause, .. } => Some(cause),
            _ => None,
        }
    }
}

/// Constructs a proof-excluded bounded-helper failure with retained stage context.
fn capacity(index: Option<usize>, phase: &'static str, operation: &'static str) -> CanadianError {
    CanadianError::InternalCapacity {
        index,
        phase,
        operation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Checked = Result<(), Box<dyn StdError>>;

    /// FR-CAD-09/15/23/29/60: all frequencies retain zero coupons and true T.
    #[test]
    fn regular_sources_and_zero_coupon_bundles_keep_original_geometry() -> Checked {
        let start = Date::from_ymd(2025, 1, 1)?;
        for (frequency, month, amount) in [
            (CanadianFrequency::Semiannual, 7, 6_u128),
            (CanadianFrequency::Quarterly, 4, 3),
            (CanadianFrequency::Monthly, 2, 1),
        ] {
            let maturity = Date::from_ymd(2025, month, 1)?;
            let payment = Date::from_ymd(2025, month, 3)?;
            for coupon in [12, 0] {
                let schedule = CanadianCouponSchedule::new(
                    start,
                    maturity,
                    frequency,
                    vec![start, maturity],
                    vec![maturity],
                    EndOfMonth::Ignore,
                    2,
                )?;
                assert_eq!(schedule.first_period(), CanadianPeriodKind::Regular);
                assert_eq!(schedule.last_period(), CanadianPeriodKind::Regular);
                let bond = CanadianBond::new(
                    start,
                    DecimalAmount::new(coupon, 0)?,
                    schedule,
                    payment,
                    CanadianSourcePolicy::CanonicalIiac13LocalV1,
                )?;
                let expected = if coupon == 0 { 0 } else { amount };
                assert_eq!(
                    wide::to_u128(&bond.valuation_coupon(0)?.numerator),
                    Some(expected)
                );
                assert_eq!(
                    wide::to_u128(&bond.payable_coupon(0)?.numerator),
                    Some(expected)
                );
                let plan = CanadianPricingPlan::new(
                    &bond,
                    start,
                    CanadianYieldSelection::LastCouponMoneyMarket,
                    1,
                )?;
                assert_eq!(plan.bundle_count(), 1);
                assert_eq!(plan.bundle_coupon_index(0), Some(0));
                assert_eq!(
                    plan.yield_convention(),
                    CanadianYieldConvention::MoneyMarket
                );
                assert_eq!(plan.accrued_geometry.start, start);
                assert_eq!(plan.accrued_geometry.end, start);
                assert_eq!(plan.accrued_geometry.first_quasi, (start, maturity));
                assert_eq!(plan.accrued_geometry.last_quasi, (start, maturity));
                assert_eq!(plan.settlement_ai, Source::ZERO);
                assert_eq!(
                    plan.bundles[0].lambda,
                    Coordinate {
                        numerator: 1,
                        denominator: 1
                    }
                );
                assert_eq!(
                    wide::to_u128(&plan.bundles[0].amount.numerator),
                    Some(100 + expected)
                );
                assert_eq!(
                    plan.bundles[0].days_to_payment,
                    payment.days_since(start).unsigned_abs()
                );
                assert_eq!(plan.bundles[0].days_payment_to_true, 0);
            }
        }
        Ok(())
    }

    /// FR-CAD-11/12/22/24/71: validation gates precede later invalid evidence.
    #[test]
    fn fixed_schedule_bond_and_plan_validation_order() -> Checked {
        let d = Date::from_ymd(2025, 1, 1)?;
        let m = Date::from_ymd(2025, 7, 1)?;
        let later = Date::from_ymd(2026, 1, 1)?;
        let new = |q, coupons, cap, start, end| {
            CanadianCouponSchedule::new(
                start,
                end,
                CanadianFrequency::Semiannual,
                q,
                coupons,
                EndOfMonth::Ignore,
                cap,
            )
        };
        assert!(matches!(
            new(vec![], vec![], 1, m, m),
            Err(CanadianError::InvalidDateCap { cap: 1, .. })
        ));
        assert!(matches!(
            new(vec![d, m, later], vec![], 2, m, m),
            Err(CanadianError::TooManyQuasiDates { count: 3, cap: 2 })
        ));
        assert!(matches!(
            new(vec![d], vec![d, m, later], 2, m, m),
            Err(CanadianError::TooFewQuasiDates { count: 1, .. })
        ));
        assert!(matches!(
            new(vec![d, m], vec![d, m, later], 2, m, m),
            Err(CanadianError::TooManyCoupons { count: 3, cap: 2 })
        ));
        assert!(matches!(
            new(vec![d, m], vec![], 2, m, m),
            Err(CanadianError::NoCoupons)
        ));
        assert!(matches!(
            new(vec![m, d], vec![m], 2, m, m),
            Err(CanadianError::AccrualNotBeforeMaturity { .. })
        ));
        assert!(matches!(
            new(vec![m, d], vec![m], 2, d, m),
            Err(CanadianError::QuasiDateOrder { index: 1, .. })
        ));

        let schedule = new(vec![d, m], vec![m], 2, d, m)?;
        assert!(matches!(
            CanadianBond::new(
                m,
                DecimalAmount::new(-1, 0)?,
                schedule.clone(),
                d,
                CanadianSourcePolicy::CanonicalIiac13LocalV1
            ),
            Err(CanadianError::NegativeCoupon { .. })
        ));
        assert!(matches!(
            CanadianBond::new(
                d,
                DecimalAmount::new(0, 0)?,
                new(vec![d, m], vec![m], 2, Date::from_ymd(2025, 1, 2)?, m)?,
                d,
                CanadianSourcePolicy::CanonicalIiac13LocalV1
            ),
            Err(CanadianError::DatedAfterIssue { .. })
        ));
        assert!(matches!(
            CanadianBond::new(
                m,
                DecimalAmount::new(0, 0)?,
                schedule.clone(),
                d,
                CanadianSourcePolicy::CanonicalIiac13LocalV1
            ),
            Err(CanadianError::IssueNotBeforeFirstCoupon { .. })
        ));
        assert!(matches!(
            CanadianBond::new(
                d,
                DecimalAmount::new(0, 0)?,
                schedule.clone(),
                d,
                CanadianSourcePolicy::CanonicalIiac13LocalV1
            ),
            Err(CanadianError::TrueMaturityBeforeStated { .. })
        ));
        let bond = CanadianBond::new(
            d,
            DecimalAmount::new(0, 0)?,
            schedule,
            later,
            CanadianSourcePolicy::CanonicalIiac13LocalV1,
        )?;
        let selection = CanadianYieldSelection::Explicit(CanadianYieldConvention::Compound);
        assert!(matches!(
            CanadianPricingPlan::new(&bond, Date::from_ymd(2024, 12, 31)?, selection, 20_001),
            Err(CanadianError::InvalidBundleCap { cap: 20_001, .. })
        ));
        assert!(matches!(
            CanadianPricingPlan::new(&bond, Date::from_ymd(2024, 12, 31)?, selection, 0),
            Err(CanadianError::SettlementBeforeIssue { .. })
        ));
        assert!(matches!(
            CanadianPricingPlan::new(&bond, m, selection, 0),
            Err(CanadianError::SettlementAtOrAfterMaturity { .. })
        ));
        assert!(matches!(
            CanadianPricingPlan::new(&bond, d, selection, 0),
            Err(CanadianError::TooManyBundles { count: 1, cap: 0 })
        ));
        Ok(())
    }

    /// FR-CAD-12/13/14/71: direct anchoring and every off-cycle membership
    /// check precede interior adjacency; Date causes keep their immediate type.
    #[test]
    fn direct_anchor_and_membership_precede_adjacency_with_typed_dates() -> Checked {
        let d = Date::from_ymd(2025, 3, 1)?;
        let m = Date::from_ymd(2025, 5, 31)?;
        let wrong = CanadianCouponSchedule::new(
            d,
            m,
            CanadianFrequency::Monthly,
            vec![
                Date::from_ymd(2025, 2, 28)?,
                Date::from_ymd(2025, 3, 30)?,
                Date::from_ymd(2025, 4, 30)?,
                m,
            ],
            vec![m],
            EndOfMonth::Ignore,
            4,
        );
        assert!(matches!(
            wrong,
            Err(CanadianError::QuasiDateMismatch { index: 1, .. })
        ));
        let q = vec![
            Date::from_ymd(2025, 2, 28)?,
            Date::from_ymd(2025, 3, 31)?,
            Date::from_ymd(2025, 4, 30)?,
            m,
        ];
        let ignore = CanadianCouponSchedule::new(
            d,
            m,
            CanadianFrequency::Monthly,
            q.clone(),
            vec![m],
            EndOfMonth::Ignore,
            4,
        )?;
        let preserve = CanadianCouponSchedule::new(
            d,
            m,
            CanadianFrequency::Monthly,
            q,
            vec![m],
            EndOfMonth::Preserve,
            20_000,
        )?;
        assert_eq!(ignore.quasi_dates(), preserve.quasi_dates());
        assert_ne!(ignore, preserve);

        let d = Date::from_ymd(2025, 1, 1)?;
        let m = Date::from_ymd(2027, 1, 1)?;
        let q = vec![
            d,
            Date::from_ymd(2025, 7, 1)?,
            Date::from_ymd(2026, 1, 1)?,
            Date::from_ymd(2026, 7, 1)?,
            m,
        ];
        let off_cycle = CanadianCouponSchedule::new(
            d,
            m,
            CanadianFrequency::Semiannual,
            q.clone(),
            vec![Date::from_ymd(2025, 7, 1)?, Date::from_ymd(2026, 8, 1)?, m],
            EndOfMonth::Ignore,
            5,
        );
        assert!(matches!(
            off_cycle,
            Err(CanadianError::CouponOffCycle { index: 1, .. })
        ));
        let skipped_interior = CanadianCouponSchedule::new(
            d,
            m,
            CanadianFrequency::Semiannual,
            q,
            vec![Date::from_ymd(2025, 7, 1)?, Date::from_ymd(2026, 7, 1)?, m],
            EndOfMonth::Ignore,
            5,
        );
        assert!(matches!(
            skipped_interior,
            Err(CanadianError::IrregularInteriorPeriod { index: 1, .. })
        ));

        let first = Date::from_ymd(1, 1, 1)?;
        let last = Date::from_ymd(1, 5, 1)?;
        let date_error = CanadianCouponSchedule::new(
            first,
            last,
            CanadianFrequency::Semiannual,
            vec![first, last],
            vec![last],
            EndOfMonth::Ignore,
            2,
        );
        match date_error {
            Err(
                ref error @ CanadianError::Date {
                    index: Some(0),
                    phase: "quasi cycle",
                    ref cause,
                },
            ) => {
                assert!(matches!(cause, DateError::YearOutOfRange { year: 0, .. }));
                assert!(
                    error
                        .source()
                        .and_then(|source| source.downcast_ref::<DateError>())
                        .is_some_and(|source| std::ptr::eq(source, cause))
                );
            }
            other => {
                return Err(
                    format!("expected original-index typed Date failure, got {other:?}").into(),
                );
            }
        }
        // Frozen S CA020971: a full20000-date input reaches direct-target
        // civil-year failure before any later cycle mismatch or allocation.
        let quasi = (0..20_000_i64)
            .map(|days| first.checked_add_days(days))
            .collect::<Result<Vec<_>, _>>()?;
        let maturity = *quasi.last().ok_or("full cycle")?;
        let large = CanadianCouponSchedule::new(
            first,
            maturity,
            CanadianFrequency::Monthly,
            quasi,
            vec![maturity],
            EndOfMonth::Ignore,
            20_000,
        );
        assert!(matches!(
            large,
            Err(CanadianError::Date {
                index: Some(0),
                phase: "quasi cycle",
                cause: DateError::YearOutOfRange { year: -1611, .. },
                ..
            })
        ));
        Ok(())
    }
}
