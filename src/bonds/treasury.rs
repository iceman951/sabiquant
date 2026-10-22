//! Explicit fixed-principal US Treasury source, date and analytical model.
//!
//! The caller supplies original dated/issue dates, the complete nominal
//! maturity-anchored cycle, EOM and publication policy. No calendar, auction
//! eligibility, record date, detached coupon or financial acceptance is implied.
//! Raw analytical amounts and selected published cash values remain separate.

use std::error::Error as StdError;
use std::fmt;

use super::CouponEntitlement;
use super::yield_risk::round_wide_ratio_to_f64;
use crate::amounts::uint::Wide;
use crate::amounts::{AmountError, AmountRounding, DecimalAmount};
use crate::calendar::EndOfMonth;
use crate::error::ValueError;
use crate::math::SumError;
use crate::time::{Date, DateError};

mod cash;
mod numeric;

pub use cash::{TreasuryCashDecision, TreasuryCashInstruction, TreasuryCashReceipt};
pub use numeric::{TreasuryRisk, TreasuryValuation, TreasuryYieldError, TreasuryYieldReport};

/// Retained fixed-principal formula interpretation, not a current-law fetch.
const SOURCE_VERSION: &str = "US-Treasury-31CFR356-AppB-fixed-2026-10-16-local-v1";
/// Five-word exact source/cache magnitude (main contract E).
type Scalar = Wide<5>;

/// A nonnegative exact per100 source amount, without decimal admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct SourceRational {
    numerator: Scalar,
    denominator: Scalar,
}

impl SourceRational {
    /// Canonical exact zero; lifecycle day metadata is retained separately.
    const ZERO: Self = Self {
        numerator: Scalar::ZERO,
        denominator: Scalar::from_u128(1),
    };

    /// Keeps unreduced bounded source forms, normalizing exact zero only.
    fn new(numerator: Scalar, denominator: Scalar) -> Self {
        if numerator.is_zero() {
            Self::ZERO
        } else {
            Self {
                numerator,
                denominator,
            }
        }
    }

    /// Groups final par100 exactly with its raw coupon, before PV rounding.
    fn with_par(self, index: usize) -> Result<Self, TreasuryError> {
        let numerator = self
            .denominator
            .checked_mul_small(100)
            .and_then(|p| p.checked_add(self.numerator))
            .ok_or_else(|| capacity(Some(index), "source present value", "derive source"))?;
        Ok(Self::new(numerator, self.denominator))
    }
}

/// Normative per$1000 source form X/(10^dscale*s_part), never an amount view.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct SourceFormula {
    x: Scalar,
    dscale: u8,
    s_part: u32,
}

impl SourceFormula {
    /// Normalizes a zero rational while retaining days outside the formula.
    fn new(x: Scalar, dscale: u8, s_part: u32) -> Self {
        if x.is_zero() {
            Self {
                x,
                dscale: 0,
                s_part: 1,
            }
        } else {
            Self { x, dscale, s_part }
        }
    }
}

/// Source accrued cache and its one or two original civil-day pieces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct AccruedSource {
    raw: SourceRational,
    published_count: Scalar,
    pieces: ((u32, u32), Option<(u32, u32)>),
}

/// Raw coupon plus optional par at its absolute original quasi index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PreparedBundle {
    index: usize,
    j: u32,
    amount: SourceRational,
}

/// Owned report context; neither a whole instrument identity nor a Q clone.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Metadata {
    original_dated_date: Date,
    original_issue_date: Date,
    maturity_date: Date,
    coupon_per_100: DecimalAmount,
    first_coupon_date: Date,
    first_period: TreasuryFirstPeriod,
    end_of_month: EndOfMonth,
    source_policy: TreasurySourcePolicy,
    settlement_date: Date,
    next_quasi_date: Date,
    n: u32,
    r: u32,
    s: u32,
    raw_accrued_per_100: f64,
    accrued_per_100: f64,
}

/// Actual first-coupon relationship to the supplied nominal cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TreasuryFirstPeriod {
    /// A full coupon from the original dated date Q0 to Q1.
    Regular,
    /// A fractional coupon from D inside (Q0,Q1) to Q1.
    Short,
    /// A fractional period plus full period, paid at Q2; Q1 is skipped.
    Long,
}

impl TreasuryFirstPeriod {
    /// All first-period variants, in Regular, Short, Long order.
    pub const ALL: [Self; 3] = [Self::Regular, Self::Short, Self::Long];
}

/// Explicit daily source ingredient for fractional coupons and accrued cash.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TreasuryDailyBasis {
    /// Retain the exact daily rational 5C/s per$1000.
    ExactRatio,
    /// Publish the daily rational to nine decimal places per$1000 first.
    NineDecimal,
}

impl TreasuryDailyBasis {
    /// Both daily bases, exact then nine-decimal publication.
    pub const ALL: [Self; 2] = [Self::ExactRatio, Self::NineDecimal];
}

/// Explicit tie selection for both daily9 and total accrued5 publication.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TreasuryTieRule {
    /// Choose the even multiple at a midpoint.
    NearestTiesEven,
    /// Choose the multiple farther from zero at a midpoint.
    NearestTiesAwayFromZero,
}

impl TreasuryTieRule {
    /// Both publication tie rules, in listed order.
    pub const ALL: [Self; 2] = [Self::NearestTiesEven, Self::NearestTiesAwayFromZero];

    /// Returns the corresponding existing exact-amount rounding mode.
    #[must_use]
    pub const fn amount_rounding(self) -> AmountRounding {
        match self {
            Self::NearestTiesEven => AmountRounding::NearestTiesEven,
            Self::NearestTiesAwayFromZero => AmountRounding::NearestTiesAwayFromZero,
        }
    }
}

/// Caller-selected source daily basis and publication tie rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TreasurySourcePolicy {
    daily_basis: TreasuryDailyBasis,
    tie_rule: TreasuryTieRule,
}

impl TreasurySourcePolicy {
    /// Retains both explicit choices without a default or regulatory inference.
    #[must_use]
    pub const fn new(daily_basis: TreasuryDailyBasis, tie_rule: TreasuryTieRule) -> Self {
        Self {
            daily_basis,
            tie_rule,
        }
    }

    /// Returns the retained fractional-daily source basis.
    #[must_use]
    pub const fn daily_basis(self) -> TreasuryDailyBasis {
        self.daily_basis
    }

    /// Returns the tie rule used at both source publication stages.
    #[must_use]
    pub const fn tie_rule(self) -> TreasuryTieRule {
        self.tie_rule
    }
}

/// Explicit fixed-principal semiannual Treasury mathematical definition.
///
/// Coupon is exact annual per100 (5 means5%); redemption is100. The complete
/// supplied cycle is owned without cloning. No calendar or auction eligibility
/// is inferred. Equality/hash retain the definition including source policy.
///
/// ```
/// use sabiquant::amounts::DecimalAmount;
/// use sabiquant::bonds::{
///     UsTreasury, TreasuryDailyBasis, TreasuryPricingPlan,
///     TreasurySourcePolicy, TreasuryTieRule,
/// };
/// use sabiquant::calendar::EndOfMonth;
///
/// let policy = TreasurySourcePolicy::new(
///     TreasuryDailyBasis::NineDecimal, TreasuryTieRule::NearestTiesEven,
/// );
/// let treasury = UsTreasury::new(
///     "2025-01-15".parse()?, "2025-01-15".parse()?,
///     "2026-01-15".parse()?, DecimalAmount::new(5, 0)?,
///     "2025-07-15".parse()?,
///     vec!["2025-01-15".parse()?, "2025-07-15".parse()?, "2026-01-15".parse()?],
///     EndOfMonth::Preserve, policy, 3,
/// )?;
/// let plan = TreasuryPricingPlan::new(&treasury, "2025-04-15".parse()?, 2)?;
/// let value = plan.evaluate(0.05)?;
/// assert_eq!(plan.bundle_count(), 2);
/// assert_eq!(value.source_policy(), policy);
/// assert!(value.dirty_per_100() > 0.0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UsTreasury {
    original_dated_date: Date,
    original_issue_date: Date,
    maturity_date: Date,
    coupon_per_100: DecimalAmount,
    first_coupon_date: Date,
    quasi_dates: Vec<Date>,
    end_of_month: EndOfMonth,
    source_policy: TreasurySourcePolicy,
    first_period: TreasuryFirstPeriod,
    first_coupon_quasi_index: usize,
    d0: u32,
    s0: u32,
    raw_first: SourceRational,
    source_first: SourceFormula,
}

impl UsTreasury {
    /// Largest supported complete nominal cycle, including Q0 and maturity.
    pub const MAX_QUASI_DATES: usize = 20_000;

    /// Validates and owns the complete explicit fixed-principal definition.
    ///
    /// Each Q is derived directly from maturity by a six-month multiple. EOM
    /// preservation applies only when maturity itself is a month end. Dated
    /// date and original issue date are separate roles; D<=I<F is required.
    /// A long first period pays Q2 and skips Q1 without renumbering it.
    ///
    /// # Errors
    /// Checks cap, length, coupon sign, D<=I, I<M, maturity boundary, strict
    /// order, minimal Q0<=D<Q1 coverage, direct cycle and then first-coupon
    /// role in that order. Returns the corresponding contextual
    /// [`TreasuryError`]. Checked date errors preserve the original index.
    /// A private capacity failure is proof-excluded by the source width ledger.
    #[expect(
        clippy::too_many_arguments,
        reason = "the contract fixes each explicit date, policy and cap"
    )]
    pub fn new(
        original_dated_date: Date,
        original_issue_date: Date,
        maturity_date: Date,
        coupon_per_100: DecimalAmount,
        first_coupon_date: Date,
        quasi_dates: Vec<Date>,
        end_of_month: EndOfMonth,
        source_policy: TreasurySourcePolicy,
        max_dates: usize,
    ) -> Result<Self, TreasuryError> {
        if !(2..=Self::MAX_QUASI_DATES).contains(&max_dates) {
            return Err(TreasuryError::InvalidDateCap {
                cap: max_dates,
                minimum: 2,
                maximum: Self::MAX_QUASI_DATES,
            });
        }
        let count = quasi_dates.len();
        if count > max_dates {
            return Err(TreasuryError::TooManyQuasiDates {
                count,
                cap: max_dates,
            });
        }
        if count < 2 {
            return Err(TreasuryError::TooFewQuasiDates { count, minimum: 2 });
        }
        if coupon_per_100.coefficient() < 0 {
            return Err(TreasuryError::NegativeCoupon { coupon_per_100 });
        }
        if original_dated_date > original_issue_date {
            return Err(TreasuryError::DatedAfterIssue {
                original_dated_date,
                original_issue_date,
            });
        }
        if original_issue_date >= maturity_date {
            return Err(TreasuryError::IssueNotBeforeMaturity {
                original_issue_date,
                maturity_date,
            });
        }
        let last = count - 1;
        if quasi_dates[last] != maturity_date {
            return Err(TreasuryError::MaturityBoundary {
                maturity_date,
                last_quasi_date: quasi_dates[last],
            });
        }
        for (offset, pair) in quasi_dates.windows(2).enumerate() {
            if pair[1] <= pair[0] {
                return Err(TreasuryError::QuasiDateOrder {
                    index: offset + 1,
                    previous: pair[0],
                    date: pair[1],
                });
            }
        }
        if original_dated_date < quasi_dates[0] || original_dated_date >= quasi_dates[1] {
            return Err(TreasuryError::QuasiCoverage {
                original_dated_date,
                first_quasi_date: quasi_dates[0],
                next_quasi_date: quasi_dates[1],
            });
        }
        for (index, &supplied) in quasi_dates.iter().enumerate() {
            let steps = i64::try_from(last - index).unwrap_or(i64::MAX);
            let months = steps.saturating_mul(-6);
            let expected =
                maturity_date
                    .checked_add_months(months)
                    .map_err(|cause| TreasuryError::Date {
                        index: Some(index),
                        phase: "quasi cycle",
                        cause,
                    })?;
            let expected =
                if end_of_month == EndOfMonth::Preserve && maturity_date.is_end_of_month() {
                    expected.last_day_of_month()
                } else {
                    expected
                };
            if expected != supplied {
                return Err(TreasuryError::QuasiDateMismatch {
                    index,
                    supplied,
                    expected,
                    maturity_date,
                    end_of_month,
                });
            }
        }
        let bad_first = || TreasuryError::FirstCoupon {
            original_dated_date,
            original_issue_date,
            first_coupon_date,
            next_quasi_date: quasi_dates[1],
            second_quasi_date: quasi_dates.get(2).copied(),
        };
        if original_issue_date >= first_coupon_date {
            return Err(bad_first());
        }
        let (first_period, first_coupon_quasi_index) = if first_coupon_date == quasi_dates[1] {
            (
                if original_dated_date == quasi_dates[0] {
                    TreasuryFirstPeriod::Regular
                } else {
                    TreasuryFirstPeriod::Short
                },
                1,
            )
        } else if original_dated_date > quasi_dates[0]
            && quasi_dates.get(2) == Some(&first_coupon_date)
        {
            (TreasuryFirstPeriod::Long, 2)
        } else {
            return Err(bad_first());
        };
        let d0 = quasi_dates[1]
            .days_since(original_dated_date)
            .unsigned_abs();
        let s0 = quasi_dates[1].days_since(quasi_dates[0]).unsigned_abs();
        let raw_first = raw_coupon_amount(coupon_per_100, first_period, d0, s0, 0)?;
        let source_first =
            source_coupon_amount(coupon_per_100, first_period, d0, s0, source_policy, 0)?;
        Ok(Self {
            original_dated_date,
            original_issue_date,
            maturity_date,
            coupon_per_100,
            first_coupon_date,
            quasi_dates,
            end_of_month,
            source_policy,
            first_period,
            first_coupon_quasi_index,
            d0,
            s0,
            raw_first,
            source_first,
        })
    }

    /// Returns the retained local source/formula identifier.
    #[must_use]
    pub const fn source_version(&self) -> &'static str {
        SOURCE_VERSION
    }
    /// Returns the original dated date from which source accrual begins.
    #[must_use]
    pub const fn original_dated_date(&self) -> Date {
        self.original_dated_date
    }
    /// Returns the original issue date bounding valid settlements.
    #[must_use]
    pub const fn original_issue_date(&self) -> Date {
        self.original_issue_date
    }
    /// Returns the supplied nominal maturity date.
    #[must_use]
    pub const fn maturity_date(&self) -> Date {
        self.maturity_date
    }
    /// Returns the canonical exact annual coupon per100.
    #[must_use]
    pub const fn coupon_per_100(&self) -> DecimalAmount {
        self.coupon_per_100
    }
    /// Returns the first actual coupon date.
    #[must_use]
    pub const fn first_coupon_date(&self) -> Date {
        self.first_coupon_date
    }
    /// Returns the validated first-period relationship.
    #[must_use]
    pub const fn first_period(&self) -> TreasuryFirstPeriod {
        self.first_period
    }
    /// Returns the absolute Q index of the first actual coupon.
    #[must_use]
    pub const fn first_coupon_quasi_index(&self) -> usize {
        self.first_coupon_quasi_index
    }
    /// Borrows the complete caller-supplied nominal cycle, including Q0.
    #[must_use]
    pub fn quasi_dates(&self) -> &[Date] {
        &self.quasi_dates
    }
    /// Borrows the actual coupon-date suffix; a long period omits Q1.
    #[must_use]
    pub fn coupon_dates(&self) -> &[Date] {
        &self.quasi_dates[self.first_coupon_quasi_index..]
    }
    /// Returns the explicit month-end policy.
    #[must_use]
    pub const fn end_of_month(&self) -> EndOfMonth {
        self.end_of_month
    }
    /// Returns the retained daily-basis/publication-tie policy.
    #[must_use]
    pub const fn source_policy(&self) -> TreasurySourcePolicy {
        self.source_policy
    }
    /// Returns the number of actual coupons, including zero-valued coupons.
    #[must_use]
    pub fn coupon_count(&self) -> usize {
        self.quasi_dates.len() - self.first_coupon_quasi_index
    }

    /// Checks an actual coupon ordinal before a selector or cash factory.
    fn check_coupon_index(&self, index: usize) -> Result<(), TreasuryError> {
        if index < self.coupon_count() {
            Ok(())
        } else {
            Err(TreasuryError::InvalidCouponIndex {
                index,
                count: self.coupon_count(),
            })
        }
    }

    /// Retrieves an exact raw analytical coupon without principal or rounding.
    fn raw_coupon(&self, index: usize) -> Result<SourceRational, TreasuryError> {
        self.check_coupon_index(index)?;
        if index == 0 {
            Ok(self.raw_first)
        } else {
            raw_coupon_amount(
                self.coupon_per_100,
                TreasuryFirstPeriod::Regular,
                0,
                1,
                index,
            )
        }
    }

    /// Retrieves the selected unrounded per1000 source-cash formula.
    fn source_coupon_formula(&self, index: usize) -> Result<SourceFormula, TreasuryError> {
        self.check_coupon_index(index)?;
        if index == 0 {
            Ok(self.source_first)
        } else {
            source_coupon_amount(
                self.coupon_per_100,
                TreasuryFirstPeriod::Regular,
                0,
                1,
                self.source_policy,
                index,
            )
        }
    }
}

/// Settlement-prepared strictly future raw Treasury payment bundles.
///
/// The plan borrows its instrument and owns one fallibly reserved bundle
/// vector. Numerics reuse these exact source coefficients and coordinates;
/// they do not query dates, calendars or public decimal amount views.
#[derive(Clone, Debug, PartialEq)]
pub struct TreasuryPricingPlan<'a> {
    treasury: &'a UsTreasury,
    settlement_date: Date,
    next_quasi_index: usize,
    n: u32,
    r: u32,
    s: u32,
    accrued_source: AccruedSource,
    raw_accrued_per_100: f64,
    accrued_per_100: f64,
    bundles: Vec<PreparedBundle>,
}

impl<'a> TreasuryPricingPlan<'a> {
    /// Largest supported prepared payment count; a zero caller cap is valid.
    pub const MAX_BUNDLES: usize = 20_000;

    /// Prepares strict-future payments, exact source accrual and both AI views.
    ///
    /// Settlement must satisfy I<=S<M. A payment on S is ceased. At a skipped
    /// long-first Q1 there is no payment and accrued interest does not reset.
    ///
    /// # Errors
    /// Checks cap, settlement before issue, settlement at/after maturity and
    /// required bundle count in that order. Exact bounded source derivation
    /// precedes the single fallible bundle reservation. Allocation and
    /// proof-excluded capacity failures preserve their contextual phases.
    pub fn new(
        treasury: &'a UsTreasury,
        settlement_date: Date,
        max_bundles: usize,
    ) -> Result<Self, TreasuryError> {
        if max_bundles > Self::MAX_BUNDLES {
            return Err(TreasuryError::InvalidBundleCap {
                cap: max_bundles,
                maximum: Self::MAX_BUNDLES,
            });
        }
        if settlement_date < treasury.original_issue_date {
            return Err(TreasuryError::SettlementBeforeIssue {
                settlement_date,
                original_issue_date: treasury.original_issue_date,
            });
        }
        if settlement_date >= treasury.maturity_date {
            return Err(TreasuryError::SettlementAtOrAfterMaturity {
                settlement_date,
                maturity_date: treasury.maturity_date,
            });
        }
        let q = &treasury.quasi_dates;
        let next_quasi_index = q.partition_point(|date| *date <= settlement_date);
        let last = q.len() - 1;
        let r = q[next_quasi_index]
            .days_since(settlement_date)
            .unsigned_abs();
        let s = q[next_quasi_index]
            .days_since(q[next_quasi_index - 1])
            .unsigned_abs();
        // Both differences are below MAX_QUASI_DATES, independently of Date.
        let n = u32::try_from(last - next_quasi_index).unwrap_or(u32::MAX);
        let first = next_quasi_index.max(treasury.first_coupon_quasi_index);
        let required = q.len() - first;
        if required > max_bundles {
            return Err(TreasuryError::TooManyBundles {
                count: required,
                cap: max_bundles,
            });
        }
        let f = treasury.first_coupon_quasi_index;
        let pieces =
            if next_quasi_index > f || treasury.first_period == TreasuryFirstPeriod::Regular {
                (
                    (
                        settlement_date
                            .days_since(q[next_quasi_index - 1])
                            .unsigned_abs(),
                        s,
                    ),
                    None,
                )
            } else if next_quasi_index == 1 {
                (
                    (
                        settlement_date
                            .days_since(treasury.original_dated_date)
                            .unsigned_abs(),
                        treasury.s0,
                    ),
                    None,
                )
            } else {
                (
                    (treasury.d0, treasury.s0),
                    Some((settlement_date.days_since(q[1]).unsigned_abs(), s)),
                )
            };
        let accrued_source =
            accrued_source(treasury.coupon_per_100, pieces, treasury.source_policy)?;
        let raw_accrued_per_100 = source_binary(
            accrued_source.raw,
            "raw source accrued",
            "round source accrued",
        )?;
        let published = SourceRational::new(
            accrued_source.published_count,
            Scalar::pow10(6)
                .ok_or_else(|| capacity(None, "published accrued cache", "round source accrued"))?,
        );
        let accrued_per_100 =
            source_binary(published, "published accrued cache", "round source accrued")?;
        let mut bundles = Vec::new();
        bundles
            .try_reserve_exact(required)
            .map_err(|_| TreasuryError::AllocationFailed {
                phase: "pricing bundles",
                requested: required,
            })?;
        for index in first..=last {
            let amount = treasury.raw_coupon(index - f)?;
            let amount = if index == last {
                amount.with_par(index)?
            } else {
                amount
            };
            let j = u32::try_from(index - next_quasi_index).unwrap_or(u32::MAX);
            bundles.push(PreparedBundle { index, j, amount });
        }
        Ok(Self {
            treasury,
            settlement_date,
            next_quasi_index,
            n,
            r,
            s,
            accrued_source,
            raw_accrued_per_100,
            accrued_per_100,
            bundles,
        })
    }

    /// Borrows the retained complete Treasury definition.
    #[must_use]
    pub const fn treasury(&self) -> &'a UsTreasury {
        self.treasury
    }
    /// Returns the retained local source/formula identifier.
    #[must_use]
    pub const fn source_version(&self) -> &'static str {
        SOURCE_VERSION
    }
    /// Returns the source daily-basis/publication-tie policy.
    #[must_use]
    pub const fn source_policy(&self) -> TreasurySourcePolicy {
        self.treasury.source_policy
    }
    /// Returns the explicit analytical settlement date.
    #[must_use]
    pub const fn settlement_date(&self) -> Date {
        self.settlement_date
    }
    /// Returns the first original Q index strictly after settlement.
    #[must_use]
    pub const fn next_quasi_index(&self) -> usize {
        self.next_quasi_index
    }
    /// Returns the remaining full quasi periods after the next quasi date.
    #[must_use]
    pub const fn remaining_quasi_periods(&self) -> u32 {
        self.n
    }
    /// Returns days from settlement to the next quasi date.
    #[must_use]
    pub const fn remaining_days(&self) -> u32 {
        self.r
    }
    /// Returns days in the quasi period containing settlement.
    #[must_use]
    pub const fn quasi_period_days(&self) -> u32 {
        self.s
    }
    /// Returns the number of strictly future actual payment bundles.
    #[must_use]
    pub fn bundle_count(&self) -> usize {
        self.bundles.len()
    }
    /// Returns a retained bundle's absolute original Q index, or None.
    #[must_use]
    pub fn bundle_quasi_index(&self, index: usize) -> Option<usize> {
        self.bundles.get(index).map(|b| b.index)
    }
    /// Returns ceased/buyer entitlement for a valid actual coupon ordinal.
    #[must_use]
    pub fn coupon_entitlement(&self, coupon_index: usize) -> Option<CouponEntitlement> {
        self.treasury.coupon_dates().get(coupon_index).map(|date| {
            if *date <= self.settlement_date {
                CouponEntitlement::Ceased
            } else {
                CouponEntitlement::BuyerEntitled
            }
        })
    }
    /// Returns one IEEE rounding of exact raw analytical accrued per100.
    #[must_use]
    pub const fn raw_accrued_per_100(&self) -> f64 {
        self.raw_accrued_per_100
    }
    /// Returns one IEEE rounding of selected published accrued per100.
    #[must_use]
    pub const fn accrued_per_100(&self) -> f64 {
        self.accrued_per_100
    }

    /// Copies report scalars without a vector clone or date arithmetic.
    fn metadata(&self) -> Metadata {
        let t = self.treasury;
        Metadata {
            original_dated_date: t.original_dated_date,
            original_issue_date: t.original_issue_date,
            maturity_date: t.maturity_date,
            coupon_per_100: t.coupon_per_100,
            first_coupon_date: t.first_coupon_date,
            first_period: t.first_period,
            end_of_month: t.end_of_month,
            source_policy: t.source_policy,
            settlement_date: self.settlement_date,
            next_quasi_date: t.quasi_dates[self.next_quasi_index],
            n: self.n,
            r: self.r,
            s: self.s,
            raw_accrued_per_100: self.raw_accrued_per_100,
            accrued_per_100: self.accrued_per_100,
        }
    }
}

/// Produces the exact raw coupon from the original full Decimal C.
fn raw_coupon_amount(
    coupon: DecimalAmount,
    role: TreasuryFirstPeriod,
    d0: u32,
    s0: u32,
    index: usize,
) -> Result<SourceRational, TreasuryError> {
    let derive = || capacity(Some(index), "raw coupon", "derive source");
    let (days, period) = match role {
        TreasuryFirstPeriod::Regular => (1, 1),
        TreasuryFirstPeriod::Short => (d0, s0),
        TreasuryFirstPeriod::Long => (d0 + s0, s0),
    };
    let numerator = Scalar::from_u128(coupon.coefficient().unsigned_abs())
        .checked_mul_small(days)
        .ok_or_else(derive)?;
    let denominator = Scalar::pow10(u16::from(coupon.scale()))
        .and_then(|p| p.checked_mul_small(2 * period))
        .ok_or_else(derive)?;
    Ok(SourceRational::new(numerator, denominator))
}

/// Nearest publication count; complement comparison avoids doubling remainders.
fn nearest(
    numerator: Scalar,
    denominator: Scalar,
    tie: TreasuryTieRule,
    index: Option<usize>,
    phase: &'static str,
    operation: &'static str,
) -> Result<Scalar, TreasuryError> {
    let fail = || capacity(index, phase, operation);
    let (q, r) = numerator.div_rem(denominator).ok_or_else(fail)?;
    let complement = denominator.checked_sub(r).ok_or_else(fail)?;
    let increment = r > complement
        || (r == complement && (tie == TreasuryTieRule::NearestTiesAwayFromZero || q.is_odd()));
    if increment {
        q.checked_add(Scalar::from_u128(1)).ok_or_else(fail)
    } else {
        Ok(q)
    }
}

/// Daily9 count per$1000 using the contract's combined decimal powers.
fn daily_count(
    coupon: DecimalAmount,
    s: u32,
    tie: TreasuryTieRule,
    index: Option<usize>,
) -> Result<Scalar, TreasuryError> {
    let fail = || capacity(index, "daily decimal", "round daily");
    let base = Scalar::from_u128(coupon.coefficient().unsigned_abs())
        .checked_mul_small(5)
        .ok_or_else(fail)?;
    let scale = coupon.scale();
    let (numerator, denominator) = if scale <= 9 {
        (
            Scalar::pow10(u16::from(9 - scale))
                .and_then(|p| base.checked_mul(p))
                .ok_or_else(fail)?,
            Scalar::from_u128(u128::from(s)),
        )
    } else {
        (
            base,
            Scalar::pow10(u16::from(scale - 9))
                .and_then(|p| p.checked_mul_small(s))
                .ok_or_else(fail)?,
        )
    };
    nearest(
        numerator,
        denominator,
        tie,
        index,
        "daily decimal",
        "round daily",
    )
}

/// Derives main E's exact selected source formula; no coupon quantizer.
fn source_coupon_amount(
    coupon: DecimalAmount,
    role: TreasuryFirstPeriod,
    d0: u32,
    s0: u32,
    policy: TreasurySourcePolicy,
    index: usize,
) -> Result<SourceFormula, TreasuryError> {
    let fail = || capacity(Some(index), "source coupon", "derive source");
    let full = Scalar::from_u128(coupon.coefficient().unsigned_abs())
        .checked_mul_small(5)
        .ok_or_else(fail)?;
    if role == TreasuryFirstPeriod::Regular {
        return Ok(SourceFormula::new(full, coupon.scale(), 1));
    }
    if policy.daily_basis == TreasuryDailyBasis::ExactRatio {
        let days = d0
            + if role == TreasuryFirstPeriod::Long {
                s0
            } else {
                0
            };
        return Ok(SourceFormula::new(
            full.checked_mul_small(days).ok_or_else(fail)?,
            coupon.scale(),
            s0,
        ));
    }
    let k = daily_count(coupon, s0, policy.tie_rule, Some(index))?;
    let fraction = k.checked_mul_small(d0).ok_or_else(fail)?;
    if role == TreasuryFirstPeriod::Short {
        return Ok(SourceFormula::new(fraction, 9, 1));
    }
    let scale = 9.max(coupon.scale());
    let fractional = Scalar::pow10(u16::from(scale - 9))
        .and_then(|p| fraction.checked_mul(p))
        .ok_or_else(fail)?;
    let regular = Scalar::pow10(u16::from(scale - coupon.scale()))
        .and_then(|p| full.checked_mul(p))
        .ok_or_else(fail)?;
    Ok(SourceFormula::new(
        fractional.checked_add(regular).ok_or_else(fail)?,
        scale,
        1,
    ))
}

/// Accrues one/two exact civil pieces, then publishes the total once per$1000.
fn accrued_source(
    coupon: DecimalAmount,
    pieces: ((u32, u32), Option<(u32, u32)>),
    policy: TreasurySourcePolicy,
) -> Result<AccruedSource, TreasuryError> {
    let ((t0, s0), second) = pieces;
    let fail = || capacity(None, "raw source accrued", "derive source");
    let (weight, period) = if let Some((t1, s1)) = second {
        (t0 * s1 + t1 * s0, s0 * s1)
    } else {
        (t0, s0)
    };
    // End the unreduced creation operands before nested daily/publication
    // division, so the sixteen-Scalar budget does not rely on Copy-value
    // last-use optimization by the compiler.
    let raw = {
        let numerator = Scalar::from_u128(coupon.coefficient().unsigned_abs())
            .checked_mul_small(weight)
            .ok_or_else(fail)?;
        let denominator = Scalar::pow10(u16::from(coupon.scale()))
            .and_then(|p| p.checked_mul_small(2 * period))
            .ok_or_else(fail)?;
        SourceRational::new(numerator, denominator)
    };
    let publication_fail = || capacity(None, "accrued publication", "publish accrued");
    let (publication_num, publication_den) = if policy.daily_basis == TreasuryDailyBasis::ExactRatio
    {
        // Use T9's combined-power form, not rawAI*10^6 with an uncancelled
        // factor two. The published numerator then obeys the adopted
        // 162-bit bound even for maximum-C two-piece long-first accrual.
        let base = Scalar::from_u128(coupon.coefficient().unsigned_abs())
            .checked_mul_small(5)
            .and_then(|value| value.checked_mul_small(weight))
            .ok_or_else(publication_fail)?;
        if coupon.scale() <= 5 {
            (
                Scalar::pow10(u16::from(5 - coupon.scale()))
                    .and_then(|power| base.checked_mul(power))
                    .ok_or_else(publication_fail)?,
                Scalar::from_u128(u128::from(period)),
            )
        } else {
            (
                base,
                Scalar::pow10(u16::from(coupon.scale() - 5))
                    .and_then(|power| power.checked_mul_small(period))
                    .ok_or_else(publication_fail)?,
            )
        }
    } else {
        let first = daily_count(coupon, s0, policy.tie_rule, None)?
            .checked_mul_small(t0)
            .ok_or_else(publication_fail)?;
        let total = if let Some((t1, s1)) = second {
            let next = daily_count(coupon, s1, policy.tie_rule, None)?
                .checked_mul_small(t1)
                .ok_or_else(publication_fail)?;
            first.checked_add(next).ok_or_else(publication_fail)?
        } else {
            first
        };
        (total, Scalar::from_u128(10_000))
    };
    let published_count = nearest(
        publication_num,
        publication_den,
        policy.tie_rule,
        None,
        "accrued publication",
        "publish accrued",
    )?;
    Ok(AccruedSource {
        raw,
        published_count,
        pieces,
    })
}

/// Once-rounds a bounded exact nonnegative source/cache value to IEEE binary64.
fn source_binary(
    value: SourceRational,
    phase: &'static str,
    operation: &'static str,
) -> Result<f64, TreasuryError> {
    // Keep persistent caches in five words, but charge IEEE extraction's
    // copied ratios and division work to the separate nine-word scratch
    // budget. Holding the parent caches plus a nested five-word extractor
    // would exceed the conservative sixteen-Scalar logical-copy census.
    let fail = || capacity(None, phase, operation);
    let numerator = value.numerator.resize::<9>().ok_or_else(fail)?;
    let denominator = value.denominator.resize::<9>().ok_or_else(fail)?;
    round_wide_ratio_to_f64(numerator, denominator, 0)
        .map(f64::from_bits)
        .ok_or_else(fail)
}

/// Builds a defensive bounded-integer error, unreachable for admitted inputs.
const fn capacity(
    index: Option<usize>,
    phase: &'static str,
    operation: &'static str,
) -> TreasuryError {
    TreasuryError::InternalCapacity {
        index,
        phase,
        operation,
    }
}

/// Contextual Treasury construction, source, analytical and cash failures.
///
/// Stable kind names use `treasury.`. Immediate typed causes are retained by
/// [`std::error::Error::source`]; formatting prose is not a stable interface.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum TreasuryError {
    /// Caller date cap is outside 2..=20,000.
    #[non_exhaustive]
    InvalidDateCap {
        /// Rejected cap.
        cap: usize,
        /// Minimum admitted cap.
        minimum: usize,
        /// Maximum admitted cap.
        maximum: usize,
    },
    /// Supplied cycle exceeds its caller cap.
    #[non_exhaustive]
    TooManyQuasiDates {
        /// Supplied count.
        count: usize,
        /// Caller cap.
        cap: usize,
    },
    /// Supplied cycle has fewer than two dates.
    #[non_exhaustive]
    TooFewQuasiDates {
        /// Supplied count.
        count: usize,
        /// Minimum admitted count.
        minimum: usize,
    },
    /// Annual coupon is negative.
    #[non_exhaustive]
    NegativeCoupon {
        /// Rejected exact coupon per100.
        coupon_per_100: DecimalAmount,
    },
    /// Original dated date follows original issue.
    #[non_exhaustive]
    DatedAfterIssue {
        /// Supplied dated date.
        original_dated_date: Date,
        /// Supplied original issue.
        original_issue_date: Date,
    },
    /// Original issue is on or after maturity.
    #[non_exhaustive]
    IssueNotBeforeMaturity {
        /// Supplied original issue.
        original_issue_date: Date,
        /// Supplied maturity.
        maturity_date: Date,
    },
    /// Last supplied quasi date differs from maturity.
    #[non_exhaustive]
    MaturityBoundary {
        /// Supplied maturity.
        maturity_date: Date,
        /// Last supplied quasi date.
        last_quasi_date: Date,
    },
    /// Supplied quasi dates do not increase strictly.
    #[non_exhaustive]
    QuasiDateOrder {
        /// First rejected original index.
        index: usize,
        /// Preceding supplied date.
        previous: Date,
        /// Rejected date.
        date: Date,
    },
    /// The first quasi interval does not minimally cover D.
    #[non_exhaustive]
    QuasiCoverage {
        /// Supplied dated date.
        original_dated_date: Date,
        /// Supplied Q0.
        first_quasi_date: Date,
        /// Supplied Q1.
        next_quasi_date: Date,
    },
    /// A supplied date differs from its direct maturity-anchored date.
    #[non_exhaustive]
    QuasiDateMismatch {
        /// Rejected original index.
        index: usize,
        /// Supplied date.
        supplied: Date,
        /// Expected direct date.
        expected: Date,
        /// Supplied maturity.
        maturity_date: Date,
        /// Explicit EOM policy.
        end_of_month: EndOfMonth,
    },
    /// Checked date arithmetic failed.
    #[non_exhaustive]
    Date {
        /// Original quasi index, if applicable.
        index: Option<usize>,
        /// Stable operation phase.
        phase: &'static str,
        /// Immediate typed date cause.
        cause: DateError,
    },
    /// First coupon violates I<F or has an unsupported first-period role.
    #[non_exhaustive]
    FirstCoupon {
        /// Original dated date.
        original_dated_date: Date,
        /// Original issue date.
        original_issue_date: Date,
        /// Supplied first actual coupon.
        first_coupon_date: Date,
        /// Supplied Q1.
        next_quasi_date: Date,
        /// Supplied Q2, if present.
        second_quasi_date: Option<Date>,
    },
    /// Caller bundle cap exceeds 20,000.
    #[non_exhaustive]
    InvalidBundleCap {
        /// Rejected cap.
        cap: usize,
        /// Maximum admitted cap.
        maximum: usize,
    },
    /// Strict-future actual bundle count exceeds its caller cap.
    #[non_exhaustive]
    TooManyBundles {
        /// Required count.
        count: usize,
        /// Caller cap.
        cap: usize,
    },
    /// Settlement precedes original issue.
    #[non_exhaustive]
    SettlementBeforeIssue {
        /// Supplied settlement.
        settlement_date: Date,
        /// Original issue date.
        original_issue_date: Date,
    },
    /// Settlement is on or after maturity.
    #[non_exhaustive]
    SettlementAtOrAfterMaturity {
        /// Supplied settlement.
        settlement_date: Date,
        /// Nominal maturity.
        maturity_date: Date,
    },
    /// Actual coupon ordinal is outside the coupon-date suffix.
    #[non_exhaustive]
    InvalidCouponIndex {
        /// Rejected ordinal.
        index: usize,
        /// Actual coupon count.
        count: usize,
    },
    /// Numeric input or result violates its finite/range domain.
    #[non_exhaustive]
    Value {
        /// Original Q index or None for plan/report-level values.
        index: Option<usize>,
        /// Stable operation phase.
        phase: &'static str,
        /// Immediate typed numeric cause.
        cause: ValueError,
    },
    /// Exact dirty/clean/target reduction failed.
    #[non_exhaustive]
    Sum {
        /// Stable reduction phase.
        phase: &'static str,
        /// Immediate typed sum cause.
        cause: SumError,
    },
    /// Fallible bounded vector reservation failed.
    #[non_exhaustive]
    AllocationFailed {
        /// Stable reservation phase.
        phase: &'static str,
        /// Requested element slots.
        requested: usize,
    },
    /// Defensive exact-integer capacity invariant failed.
    #[non_exhaustive]
    InternalCapacity {
        /// Original Q or actual coupon index, if applicable.
        index: Option<usize>,
        /// Stable operation phase.
        phase: &'static str,
        /// Specific checked operation.
        operation: &'static str,
    },
    /// Defensive guard for normalizing a zero dirty price.
    #[non_exhaustive]
    ZeroDirtyPrice {
        /// Requested normalized measure.
        measure: &'static str,
        /// Canonical finite annual yield.
        yield_rate: f64,
    },
    /// Cash nominal is not strictly positive.
    #[non_exhaustive]
    NonPositiveNominal {
        /// Rejected canonical par nominal.
        nominal: DecimalAmount,
    },
    /// Exact source-view/cash quantization or final amount admission failed.
    #[non_exhaustive]
    CashArithmetic {
        /// Stable source/cash stage.
        stage: &'static str,
        /// Stage input endpoint: point, lower or upper.
        endpoint: &'static str,
        /// Immediate typed exact amount cause.
        cause: AmountError,
    },
}

impl TreasuryError {
    /// Returns the stable Treasury-domain failure kind.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::InvalidDateCap { .. } => "treasury.invalid_date_cap",
            Self::TooManyQuasiDates { .. } => "treasury.too_many_quasi_dates",
            Self::TooFewQuasiDates { .. } => "treasury.too_few_quasi_dates",
            Self::NegativeCoupon { .. } => "treasury.negative_coupon",
            Self::DatedAfterIssue { .. } => "treasury.dated_after_issue",
            Self::IssueNotBeforeMaturity { .. } => "treasury.issue_not_before_maturity",
            Self::MaturityBoundary { .. } => "treasury.maturity_boundary",
            Self::QuasiDateOrder { .. } => "treasury.quasi_date_order",
            Self::QuasiCoverage { .. } => "treasury.quasi_coverage",
            Self::QuasiDateMismatch { .. } => "treasury.quasi_date_mismatch",
            Self::Date { .. } => "treasury.date",
            Self::FirstCoupon { .. } => "treasury.first_coupon",
            Self::InvalidBundleCap { .. } => "treasury.invalid_bundle_cap",
            Self::TooManyBundles { .. } => "treasury.too_many_bundles",
            Self::SettlementBeforeIssue { .. } => "treasury.settlement_before_issue",
            Self::SettlementAtOrAfterMaturity { .. } => "treasury.settlement_at_or_after_maturity",
            Self::InvalidCouponIndex { .. } => "treasury.invalid_coupon_index",
            Self::Value { .. } => "treasury.value",
            Self::Sum { .. } => "treasury.sum",
            Self::AllocationFailed { .. } => "treasury.allocation_failed",
            Self::InternalCapacity { .. } => "treasury.internal_capacity",
            Self::ZeroDirtyPrice { .. } => "treasury.zero_dirty_price",
            Self::NonPositiveNominal { .. } => "treasury.nonpositive_nominal",
            Self::CashArithmetic { .. } => "treasury.cash_arithmetic",
        }
    }
}

impl fmt::Display for TreasuryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {self:?}", self.kind())
    }
}

impl StdError for TreasuryError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Date { cause, .. } => Some(cause),
            Self::Value { cause, .. } => Some(cause),
            Self::Sum { cause, .. } => Some(cause),
            Self::CashArithmetic { cause, .. } => Some(cause),
            _ => None,
        }
    }
}
