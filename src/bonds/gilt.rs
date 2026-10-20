//! Conventional gilt definition, settlement preparation and exact source values.
//!
//! This private child of `bonds` implements the instrument, first-period and
//! plan-preparation sections of the frozen local
//! [conventional gilt contract](https://github.com/iceman951/sabiquant/blob/main/docs/M8_GILT_CONTRACT.md)
//! (sections S, A-C, F, H and J, with the N1/N7 exact source values). The
//! caller supplies the complete maturity-anchored semiannual quasi-date
//! vector, the explicit end-of-month policy and every ex-dividend cutoff. No
//! business calendar, day-count query, settlement lag or default is inferred.
//!
//! Source dividends and accrued interest are retained as private exact
//! rationals: an unreduced day ratio of the full annual coupon, or an exact
//! six-place published micro-unit count. Neither is first admitted to a public
//! decimal or binary value. The `numeric` child owns price, risk and yield
//! evaluation and the one-rounding binary views; the `cash` child owns the
//! quantized source views and the four exact cash recipes.

use std::error::Error as StdError;
use std::fmt;

use super::{CouponEntitlement, ExCouponBoundary};
use crate::amounts::uint::Wide;
use crate::amounts::{AmountError, DecimalAmount};
use crate::calendar::EndOfMonth;
use crate::error::ValueError;
use crate::math::SumError;
use crate::rates::RateError;
use crate::time::{Date, DateError};

mod cash;
mod numeric;

pub use cash::{GiltCashDecision, GiltCashInstruction, GiltCashReceipt};
pub use numeric::{GiltRisk, GiltValuation, GiltYieldError, GiltYieldReport};

/// The retained formula-contract identifier returned by every `source_version`.
const SOURCE_VERSION: &str = "UK-DMO-Conventional-v4-2024-12-18";

/// The smallest admitted quasi-date count and date cap: one interval.
const MIN_QUASI_DATES: usize = 2;

/// Five-word (320-bit) scalar storage for exact source rationals (N5).
type Scalar = Wide<5>;

/// One exact private source value per 100 nominal.
///
/// `Unrounded` is `coupon_per_100 * numerator / denominator` with an
/// unreduced denominator `2*dayD`, `1 <= dayD <= 184^2`: `1/2` for a regular
/// dividend and the seven signed accrued branches otherwise. `Published` is
/// the exact first-dividend publication `micro / 10^6` (`micro < 2^147`),
/// never a rounded public decimal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ExactSource {
    /// A signed day ratio of the full retained annual coupon.
    Unrounded {
        /// Signed day numerator `K`, `abs(K) < 2^17`.
        numerator: i32,
        /// Positive unreduced denominator `2*dayD`.
        denominator: u32,
    },
    /// An exact positive half-up six-place first-dividend publication.
    Published {
        /// Published value in units of `10^-6` per 100.
        micro: Scalar,
    },
}

/// The regular dividend `C/2`, with no six-place publication rounding.
const REGULAR_DIVIDEND: ExactSource = ExactSource::Unrounded {
    numerator: 1,
    denominator: 2,
};

/// A signed exact source rational in bounded five-word form.
///
/// The denominator is positive and need not be reduced; zero is canonically
/// nonnegative. Numerators and denominators stay below `2^148` and `2^144`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SourceRational {
    numerator: Scalar,
    denominator: Scalar,
    negative: bool,
}

impl SourceRational {
    /// A canonical exact zero dividend, used for a detached final dividend.
    const ZERO: Self = Self {
        numerator: Scalar::ZERO,
        denominator: Scalar::from_u128(1),
        negative: false,
    };

    /// Groups par 100 with this nonnegative source dividend in its own basis.
    ///
    /// For regular `C/2` the numerator is `Ccoef+200*10^a < 2^135`; for a
    /// published micro count it is `micro+10^8 < 2^148` (N5).
    fn with_par(self, index: Option<usize>, phase: &'static str) -> Result<Self, GiltError> {
        let numerator = self
            .denominator
            .checked_mul_small(100)
            .and_then(|par| par.checked_add(self.numerator))
            .ok_or_else(|| capacity(index, phase, "derive source"))?;
        Ok(Self {
            numerator,
            denominator: self.denominator,
            negative: false,
        })
    }
}

/// Converts an exact source value of `coupon` into a bounded rational.
///
/// Unrounded values give `Ccoef*abs(K) / (10^a*2*dayD)`, below `2^144` each;
/// published values give `micro / 10^6`. No reduction, decimal admission or
/// binary rounding takes place. Capacity failures are proof-excluded.
fn source_rational(
    coupon: DecimalAmount,
    source: ExactSource,
    index: Option<usize>,
    phase: &'static str,
) -> Result<SourceRational, GiltError> {
    let derive = || capacity(index, phase, "derive source");
    let (numerator, denominator, negative) = match source {
        ExactSource::Unrounded {
            numerator,
            denominator,
        } => {
            let coefficient = coupon.coefficient();
            let magnitude = Scalar::from_u128(coefficient.unsigned_abs())
                .checked_mul_small(numerator.unsigned_abs())
                .ok_or_else(derive)?;
            let scaled = Scalar::pow10(u16::from(coupon.scale()))
                .and_then(|power| power.checked_mul_small(denominator))
                .ok_or_else(derive)?;
            (magnitude, scaled, (coefficient < 0) != (numerator < 0))
        }
        ExactSource::Published { micro } => (micro, Scalar::pow10(6).ok_or_else(derive)?, false),
    };
    Ok(SourceRational {
        numerator,
        denominator,
        negative: negative && !numerator.is_zero(),
    })
}

/// Builds the proof-excluded capacity error for one exact operation.
const fn capacity(index: Option<usize>, phase: &'static str, operation: &'static str) -> GiltError {
    GiltError::InternalCapacity {
        index,
        phase,
        operation,
    }
}

/// Publishes the first actual dividend `Rplus(C*day_numerator/(2*s1); 1e-6)`.
///
/// `day_numerator` is `r1` (Short) or `s1+r1` (Long). Decimal powers are
/// combined before division, so `N < 2^156` and `D < 2^117`. Ties round up,
/// decided by comparing the remainder with its complement. The first actual
/// dividend has ordinal zero.
fn publish_first_dividend(
    coupon: DecimalAmount,
    day_numerator: u32,
    s1: u32,
) -> Result<Scalar, GiltError> {
    let derive = || capacity(Some(0), "source dividend", "derive source");
    let round = || capacity(Some(0), "source dividend", "round publication");
    let scale = coupon.scale();
    let base = Scalar::from_u128(coupon.coefficient().unsigned_abs())
        .checked_mul_small(day_numerator)
        .ok_or_else(derive)?;
    let (numerator, denominator) = if scale <= 6 {
        let numerator = Scalar::pow10(u16::from(6 - scale))
            .and_then(|power| base.checked_mul(power))
            .ok_or_else(derive)?;
        (numerator, Scalar::from_u128(u128::from(2 * s1)))
    } else {
        let denominator = Scalar::pow10(u16::from(scale - 6))
            .and_then(|power| power.checked_mul_small(2 * s1))
            .ok_or_else(derive)?;
        (base, denominator)
    };
    let (quotient, remainder) = numerator.div_rem(denominator).ok_or_else(round)?;
    let complement = denominator.checked_sub(remainder).ok_or_else(round)?;
    if remainder >= complement {
        quotient.checked_add(Scalar::from_u128(1)).ok_or_else(round)
    } else {
        Ok(quotient)
    }
}

/// Returns the direct maturity-anchored quasi date `steps_before` periods earlier.
///
/// The date is `maturity - 6*steps_before` months in one operation, never a
/// recursive clamp. With Preserve it is moved to calendar month-end iff the
/// original maturity is calendar month-end; otherwise ordinary clamping holds.
fn direct_quasi_date(
    maturity_date: Date,
    steps_before: usize,
    end_of_month: EndOfMonth,
) -> Result<Date, DateError> {
    // steps_before < MAX_QUASI_DATES, so the conversion and product are exact.
    let months = i64::try_from(steps_before)
        .unwrap_or(i64::MAX)
        .saturating_mul(-6);
    let date = maturity_date.checked_add_months(months)?;
    if end_of_month == EndOfMonth::Preserve && maturity_date.is_end_of_month() {
        Ok(date.last_day_of_month())
    } else {
        Ok(date)
    }
}

/// One retained buyer payment bundle at an absolute quasi index.
///
/// `l` is the exact coordinate `L = r + j*s` (`lambda = L/s`, `L < 2^22`);
/// `amount` is the exact source dividend, grouped with par 100 at redemption.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PreparedBundle {
    index: usize,
    l: u32,
    amount: SourceRational,
}

/// Owned scalar formula context for reports and cash snapshots; no vectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Metadata {
    issue_date: Date,
    maturity_date: Date,
    coupon_per_100: DecimalAmount,
    first_dividend_date: Date,
    first_period: GiltFirstPeriod,
    end_of_month: EndOfMonth,
    settlement_date: Date,
    next_quasi_date: Date,
    n: u32,
    r: u32,
    s: u32,
}

/// Inferred source role of a conventional gilt's first dividend period.
///
/// The role is metadata inferred from the issue date, first actual dividend
/// date and supplied quasi dates; there is no parser, registry, default or
/// caller-selected policy.
///
/// | Role | Issue `I` and first actual dividend `F` | First dividend |
/// |---|---|---|
/// | Regular | `I = Q0`, `F = Q1` | `C/2`, unrounded |
/// | Short | `Q0 < I < Q1`, `F = Q1` | six-place `C*r1/(2*s1)` |
/// | Long | `Q0 < I < Q1`, `F = Q2` | six-place `C*(s1+r1)/(2*s1)` |
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum GiltFirstPeriod {
    /// Issue on the cycle; the first dividend is a full regular dividend.
    Regular,
    /// Issue inside the first quasi interval, paying at its end.
    Short,
    /// Issue inside the first quasi interval, skipping `Q1` and paying at `Q2`.
    Long,
}

impl GiltFirstPeriod {
    /// All three first-period roles, in declaration order.
    pub const ALL: [Self; 3] = [Self::Regular, Self::Short, Self::Long];
}

/// An immutable conventional gilt under the UK DMO conventional formula contract.
///
/// The definition retains the original issue and maturity dates, the full
/// nonnegative annual coupon `C` per 100 (`6` denotes 6%), the first actual
/// dividend date, the complete caller-supplied maturity-anchored semiannual
/// quasi-date vector `Q[0..=m]` with `Q[m]` the maturity, the explicit
/// end-of-month policy and one ex-dividend cutoff per actual dividend.
/// Redemption is fixed par 100 at maturity. Equality and hashing compare the
/// complete supplied definition; derived source data cannot erase it.
///
/// Every later actual dividend is exactly `C/2`. A Short or Long first
/// dividend is the exact positive half-up six-place publication. A Long
/// first period has no payment at its skipped quasi date `Q1`. Cutoffs use
/// the fixed strict [`ExCouponBoundary::After`] rule. No maintained UK
/// calendar generated or verified the supplied dates.
///
/// ```
/// use sabiquant::amounts::DecimalAmount;
/// use sabiquant::bonds::{ConventionalGilt, CouponEntitlement, GiltFirstPeriod, GiltPricingPlan};
/// use sabiquant::calendar::EndOfMonth;
/// use sabiquant::time::Date;
///
/// // DMO worked example: 6% issued 1999-05-07 with a long first dividend.
/// let maturity = Date::from_ymd(2028, 12, 7)?;
/// let quasi_dates = (0..=60)
///     .map(|k| maturity.checked_add_months(6 * (k - 60)))
///     .collect::<Result<Vec<_>, _>>()?;
/// let ex_dividend_dates = quasi_dates[2..]
///     .iter()
///     .map(|date| date.checked_sub_days(10))
///     .collect::<Result<Vec<_>, _>>()?;
/// let gilt = ConventionalGilt::new(
///     Date::from_ymd(1999, 5, 7)?,
///     maturity,
///     DecimalAmount::new(6, 0)?,
///     Date::from_ymd(1999, 12, 7)?,
///     quasi_dates,
///     ex_dividend_dates,
///     EndOfMonth::Ignore,
///     ConventionalGilt::MAX_QUASI_DATES,
/// )?;
/// assert_eq!(gilt.first_period(), GiltFirstPeriod::Long);
/// assert_eq!(gilt.dividend_count(), 59);
///
/// // Ex-dividend one day before the first dividend: source AI is -1/61.
/// let settlement = Date::from_ymd(1999, 12, 6)?;
/// let plan = GiltPricingPlan::new(&gilt, settlement, GiltPricingPlan::MAX_BUNDLES)?;
/// assert_eq!(plan.dividend_entitlement(0), Some(CouponEntitlement::ExCouponDetached));
/// assert_eq!((plan.remaining_days(), plan.quasi_period_days()), (1, 183));
/// assert_eq!(plan.bundle_quasi_index(0), Some(3));
/// assert_eq!(plan.accrued_per_100(), -1.0 / 61.0);
/// # Ok::<(), sabiquant::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConventionalGilt {
    issue_date: Date,
    maturity_date: Date,
    coupon_per_100: DecimalAmount,
    first_dividend_date: Date,
    quasi_dates: Vec<Date>,
    ex_dividend_dates: Vec<Date>,
    end_of_month: EndOfMonth,
    first_period: GiltFirstPeriod,
    first_dividend_quasi_index: usize,
    dividends: Vec<ExactSource>,
}

impl ConventionalGilt {
    /// Largest supported quasi-date count and `max_dates` cap.
    pub const MAX_QUASI_DATES: usize = 20_000;

    /// Validates and retains a complete conventional gilt definition.
    ///
    /// `quasi_dates` is the complete unadjusted cycle `Q[0..=m]`, minimal so
    /// that `Q[0] <= issue_date < Q[1]`, ending at `maturity_date`. Each
    /// `Q[k]` must equal maturity moved back `6*(m-k)` months directly, with
    /// `end_of_month` deciding calendar month-end preservation.
    /// `ex_dividend_dates` holds one cutoff per actual dividend, in order.
    /// The vectors are owned without a hidden clone; one derived source
    /// vector is reserved fallibly after validation.
    ///
    /// # Errors
    /// Checked in this order:
    /// [`GiltError::InvalidDateCap`] unless `2 <= max_dates <= MAX_QUASI_DATES`;
    /// [`GiltError::TooManyQuasiDates`] then [`GiltError::TooFewQuasiDates`];
    /// [`GiltError::NegativeCoupon`] then [`GiltError::IssueNotBeforeMaturity`];
    /// [`GiltError::MaturityBoundary`] unless the last quasi date is maturity,
    /// then [`GiltError::QuasiDateOrder`] at the first nonincrease;
    /// [`GiltError::QuasiCoverage`] unless `Q[0] <= issue_date < Q[1]`;
    /// for each index from zero, [`GiltError::Date`] (phase `quasi cycle`) for
    /// failed month arithmetic or [`GiltError::QuasiDateMismatch`];
    /// [`GiltError::FirstDividend`] for an unsupported first period;
    /// [`GiltError::ExDateCount`], then [`GiltError::ExDateWindow`] for the
    /// first cutoff outside its inclusive compatible window;
    /// [`GiltError::AllocationFailed`] (phase `source dividends`). A
    /// [`GiltError::InternalCapacity`] is proof-excluded.
    #[expect(
        clippy::too_many_arguments,
        reason = "the frozen contract fixes every explicit definition input and cap"
    )]
    pub fn new(
        issue_date: Date,
        maturity_date: Date,
        coupon_per_100: DecimalAmount,
        first_dividend_date: Date,
        quasi_dates: Vec<Date>,
        ex_dividend_dates: Vec<Date>,
        end_of_month: EndOfMonth,
        max_dates: usize,
    ) -> Result<Self, GiltError> {
        if !(MIN_QUASI_DATES..=Self::MAX_QUASI_DATES).contains(&max_dates) {
            return Err(GiltError::InvalidDateCap {
                cap: max_dates,
                minimum: MIN_QUASI_DATES,
                maximum: Self::MAX_QUASI_DATES,
            });
        }
        let count = quasi_dates.len();
        if count > max_dates {
            return Err(GiltError::TooManyQuasiDates {
                count,
                cap: max_dates,
            });
        }
        if count < MIN_QUASI_DATES {
            return Err(GiltError::TooFewQuasiDates {
                count,
                minimum: MIN_QUASI_DATES,
            });
        }
        if coupon_per_100.coefficient() < 0 {
            return Err(GiltError::NegativeCoupon { coupon_per_100 });
        }
        if issue_date >= maturity_date {
            return Err(GiltError::IssueNotBeforeMaturity {
                issue_date,
                maturity_date,
            });
        }
        let last = count - 1;
        if quasi_dates[last] != maturity_date {
            return Err(GiltError::MaturityBoundary {
                maturity_date,
                last_quasi_date: quasi_dates[last],
            });
        }
        for (offset, pair) in quasi_dates.windows(2).enumerate() {
            if pair[1] <= pair[0] {
                return Err(GiltError::QuasiDateOrder {
                    index: offset + 1,
                    previous: pair[0],
                    date: pair[1],
                });
            }
        }
        if issue_date < quasi_dates[0] || issue_date >= quasi_dates[1] {
            return Err(GiltError::QuasiCoverage {
                issue_date,
                first_quasi_date: quasi_dates[0],
                next_quasi_date: quasi_dates[1],
            });
        }
        for (index, &supplied) in quasi_dates.iter().enumerate() {
            let expected =
                direct_quasi_date(maturity_date, last - index, end_of_month).map_err(|cause| {
                    GiltError::Date {
                        index: Some(index),
                        phase: "quasi cycle",
                        cause,
                    }
                })?;
            if supplied != expected {
                return Err(GiltError::QuasiDateMismatch {
                    index,
                    supplied,
                    expected,
                    maturity_date,
                    end_of_month,
                });
            }
        }

        // Coverage gives Q0 <= I < Q1, so I is either Q0 or inside (Q0, Q1).
        let (first_period, first_dividend_quasi_index) = if first_dividend_date == quasi_dates[1] {
            if issue_date == quasi_dates[0] {
                (GiltFirstPeriod::Regular, 1)
            } else {
                (GiltFirstPeriod::Short, 1)
            }
        } else if issue_date > quasi_dates[0] && quasi_dates.get(2) == Some(&first_dividend_date) {
            (GiltFirstPeriod::Long, 2)
        } else {
            return Err(GiltError::FirstDividend {
                issue_date,
                first_dividend_date,
                next_quasi_date: quasi_dates[1],
                second_quasi_date: quasi_dates.get(2).copied(),
            });
        };

        let dividend_count = count - first_dividend_quasi_index;
        if ex_dividend_dates.len() != dividend_count {
            return Err(GiltError::ExDateCount {
                count: ex_dividend_dates.len(),
                expected: dividend_count,
            });
        }
        let dividend_dates = &quasi_dates[first_dividend_quasi_index..];
        for (dividend_index, (&cutoff, &dividend_date)) in
            ex_dividend_dates.iter().zip(dividend_dates).enumerate()
        {
            // The first window starts at I (Regular, Short) or Q1 (Long);
            // later windows start at the previous quasi date. Both equal
            // max(previous quasi date, I).
            let lower =
                quasi_dates[first_dividend_quasi_index + dividend_index - 1].max(issue_date);
            if cutoff < lower || cutoff > dividend_date {
                return Err(GiltError::ExDateWindow {
                    dividend_index,
                    cutoff,
                    lower,
                    dividend_date,
                });
            }
        }

        let mut dividends = Vec::new();
        dividends
            .try_reserve_exact(dividend_count)
            .map_err(|_| GiltError::AllocationFailed {
                phase: "source dividends",
                requested: dividend_count,
            })?;
        // Issue ratios: s1 = days(Q0, Q1), r1 = days(I, Q1), 1 <= r1 < s1 <= 184.
        let s1 = quasi_dates[1].days_since(quasi_dates[0]).unsigned_abs();
        let r1 = quasi_dates[1].days_since(issue_date).unsigned_abs();
        let first = match first_period {
            GiltFirstPeriod::Regular => REGULAR_DIVIDEND,
            GiltFirstPeriod::Short => ExactSource::Published {
                micro: publish_first_dividend(coupon_per_100, r1, s1)?,
            },
            GiltFirstPeriod::Long => ExactSource::Published {
                micro: publish_first_dividend(coupon_per_100, s1 + r1, s1)?,
            },
        };
        dividends.push(first);
        dividends.resize(dividend_count, REGULAR_DIVIDEND);

        Ok(Self {
            issue_date,
            maturity_date,
            coupon_per_100,
            first_dividend_date,
            quasi_dates,
            ex_dividend_dates,
            end_of_month,
            first_period,
            first_dividend_quasi_index,
            dividends,
        })
    }

    /// Returns the retained formula-contract identifier `UK-DMO-Conventional-v4-2024-12-18`.
    ///
    /// It identifies the adopted source formulas, not a runtime source fetch,
    /// calendar assurance or financial certification.
    #[must_use]
    pub const fn source_version(&self) -> &'static str {
        SOURCE_VERSION
    }

    /// Returns the original issue date.
    #[must_use]
    pub const fn issue_date(&self) -> Date {
        self.issue_date
    }

    /// Returns the maturity date, the final quasi date and redemption date.
    #[must_use]
    pub const fn maturity_date(&self) -> Date {
        self.maturity_date
    }

    /// Returns the full exact annual coupon per 100 (`6` denotes 6%).
    #[must_use]
    pub const fn coupon_per_100(&self) -> DecimalAmount {
        self.coupon_per_100
    }

    /// Returns the first actual dividend date.
    #[must_use]
    pub const fn first_dividend_date(&self) -> Date {
        self.first_dividend_date
    }

    /// Returns the inferred first-period role.
    #[must_use]
    pub const fn first_period(&self) -> GiltFirstPeriod {
        self.first_period
    }

    /// Returns the quasi index of the first actual dividend: 1, or 2 for Long.
    #[must_use]
    pub const fn first_dividend_quasi_index(&self) -> usize {
        self.first_dividend_quasi_index
    }

    /// Borrows the complete supplied quasi-date vector, ending at maturity.
    #[must_use]
    pub fn quasi_dates(&self) -> &[Date] {
        &self.quasi_dates
    }

    /// Borrows every actual dividend date: the quasi suffix from the first
    /// actual dividend through maturity, excluding a skipped Long `Q1`.
    #[must_use]
    pub fn dividend_dates(&self) -> &[Date] {
        &self.quasi_dates[self.first_dividend_quasi_index..]
    }

    /// Borrows the supplied ex-dividend cutoffs, one per actual dividend.
    #[must_use]
    pub fn ex_dividend_dates(&self) -> &[Date] {
        &self.ex_dividend_dates
    }

    /// Returns the explicit end-of-month policy used to validate the cycle.
    #[must_use]
    pub const fn end_of_month(&self) -> EndOfMonth {
        self.end_of_month
    }

    /// Returns the fixed strict cutoff rule, always [`ExCouponBoundary::After`].
    #[must_use]
    pub const fn ex_coupon_boundary(&self) -> ExCouponBoundary {
        ExCouponBoundary::After
    }

    /// Returns the number of actual dividends, including zero-valued ones.
    #[must_use]
    pub fn dividend_count(&self) -> usize {
        self.dividends.len()
    }

    /// Derives the exact unreduced source accrued interest per 100.
    ///
    /// `next_quasi_index` is `k+1` for `Q[k] <= S < Q[k+1]`. Ex status is the
    /// strict-after status of the next actual dividend at `Q[max(k+1, f)]`,
    /// where `f` is the first actual dividend's quasi index. AI uses the raw
    /// source ratios and the original issue date, never the published first
    /// dividend or a later quasi-period substitute.
    fn source_accrued(&self, settlement_date: Date, next_quasi_index: usize) -> ExactSource {
        let quasi = &self.quasi_dates;
        let first = self.first_dividend_quasi_index;
        let next_actual = next_quasi_index.max(first);
        let ex = settlement_date > self.ex_dividend_dates[next_actual - first];
        // After the actual first dividend every interval is regular; at F
        // itself t is zero, so AI resets. A skipped Long Q1 does not reset.
        let lifecycle = if next_quasi_index > first {
            GiltFirstPeriod::Regular
        } else {
            self.first_period
        };
        let issue_ratios = || {
            (
                quasi[1].days_since(quasi[0]),
                quasi[1].days_since(self.issue_date),
                settlement_date.days_since(self.issue_date),
            )
        };
        let (numerator, day_denominator) = match lifecycle {
            GiltFirstPeriod::Regular => {
                let start = quasi[next_quasi_index - 1];
                let t = settlement_date.days_since(start);
                let s = quasi[next_quasi_index].days_since(start);
                (if ex { t - s } else { t }, s.unsigned_abs())
            }
            GiltFirstPeriod::Short => {
                let (s1, r1, t_star) = issue_ratios();
                (if ex { t_star - r1 } else { t_star }, s1.unsigned_abs())
            }
            // I <= S < Q1: the compatible window puts the next cutoff at or
            // after Q1, so this interval is always cum-dividend.
            GiltFirstPeriod::Long if next_quasi_index < first => {
                let (s1, _, t_star) = issue_ratios();
                (t_star, s1.unsigned_abs())
            }
            GiltFirstPeriod::Long => {
                let (s1, r1, _) = issue_ratios();
                let r2 = settlement_date.days_since(quasi[1]);
                let s2 = quasi[2].days_since(quasi[1]);
                if ex {
                    (r2 - s2, s2.unsigned_abs())
                } else {
                    (r1 * s2 + r2 * s1, (s1 * s2).unsigned_abs())
                }
            }
        };
        ExactSource::Unrounded {
            numerator,
            denominator: 2 * day_denominator,
        }
    }
}

/// A prepared immutable conventional gilt pricing plan at one settlement date.
///
/// Preparation resolves the strictly next quasi date `Q[k+1]` with
/// `Q[k] <= S < Q[k+1]`, the remaining days `r`, quasi period days `s` and
/// remaining quasi periods `n = m-k-1`, every future buyer bundle and the exact
/// source accrued interest once. Retained bundles keep their absolute quasi
/// indices: a skipped Long `Q1` or a detached dividend is absent without
/// renumbering, an entitled zero-valued dividend remains, and par 100 is
/// grouped with the maturity bundle. Repeated evaluation performs no date or
/// calendar work.
#[derive(Clone, Debug, PartialEq)]
pub struct GiltPricingPlan<'a> {
    gilt: &'a ConventionalGilt,
    settlement_date: Date,
    next_quasi_index: usize,
    n: u32,
    r: u32,
    s: u32,
    accrued_source: ExactSource,
    accrued_per_100: f64,
    bundles: Vec<PreparedBundle>,
}

impl<'a> GiltPricingPlan<'a> {
    /// Largest supported `max_bundles` cap.
    pub const MAX_BUNDLES: usize = 20_000;

    /// Prepares the settlement lifecycle, buyer bundles and exact source accrued.
    ///
    /// The named trading domain is `issue_date <= settlement_date < maturity`.
    /// Bundles are counted without allocation and checked against
    /// `max_bundles` (zero is a valid cap) before the source accrued
    /// interest is derived and the bundle vector is reserved once.
    ///
    /// # Errors
    /// Checked in this order: [`GiltError::InvalidBundleCap`] if
    /// `max_bundles > MAX_BUNDLES`; [`GiltError::SettlementBeforeIssue`];
    /// [`GiltError::SettlementAtOrAfterMaturity`]; [`GiltError::TooManyBundles`]
    /// if the required bundle count exceeds `max_bundles`;
    /// [`GiltError::AllocationFailed`] (phase `pricing bundles`). A
    /// [`GiltError::InternalCapacity`] is proof-excluded.
    pub fn new(
        gilt: &'a ConventionalGilt,
        settlement_date: Date,
        max_bundles: usize,
    ) -> Result<Self, GiltError> {
        if max_bundles > Self::MAX_BUNDLES {
            return Err(GiltError::InvalidBundleCap {
                cap: max_bundles,
                maximum: Self::MAX_BUNDLES,
            });
        }
        if settlement_date < gilt.issue_date {
            return Err(GiltError::SettlementBeforeIssue {
                settlement_date,
                issue_date: gilt.issue_date,
            });
        }
        if settlement_date >= gilt.maturity_date {
            return Err(GiltError::SettlementAtOrAfterMaturity {
                settlement_date,
                maturity_date: gilt.maturity_date,
            });
        }

        // Q0 <= I <= S < M = Q[m], so 1 <= next_quasi_index <= m.
        let quasi = &gilt.quasi_dates;
        let last = quasi.len() - 1;
        let next_quasi_index = quasi.partition_point(|date| *date <= settlement_date);
        let next_date = quasi[next_quasi_index];
        let r = next_date.days_since(settlement_date).unsigned_abs();
        let s = next_date
            .days_since(quasi[next_quasi_index - 1])
            .unsigned_abs();
        // n < MAX_QUASI_DATES, so the conversion is exact.
        let n = u32::try_from(last - next_quasi_index).unwrap_or(u32::MAX);

        // Every future actual dividend is after S, so it is retained iff
        // S <= its cutoff. Redemption is always retained, alone if detached.
        let first = gilt.first_dividend_quasi_index;
        let next_actual = next_quasi_index.max(first);
        let required = gilt.ex_dividend_dates[next_actual - first..last - first]
            .iter()
            .filter(|cutoff| settlement_date <= **cutoff)
            .count()
            + 1;
        if required > max_bundles {
            return Err(GiltError::TooManyBundles {
                count: required,
                cap: max_bundles,
            });
        }

        let accrued_source = gilt.source_accrued(settlement_date, next_quasi_index);
        let accrued = source_rational(gilt.coupon_per_100, accrued_source, None, "source accrued")?;
        let accrued_per_100 =
            numeric::source_binary(accrued, None, "source accrued", "round source accrued")?;

        let mut bundles = Vec::new();
        bundles
            .try_reserve_exact(required)
            .map_err(|_| GiltError::AllocationFailed {
                phase: "pricing bundles",
                requested: required,
            })?;
        // L = r + j*s < 20_000*184 < 2^22 for j = index - next_quasi_index.
        let mut l = r;
        for index in next_quasi_index..=last {
            if index >= first {
                let ordinal = index - first;
                let entitled = settlement_date <= gilt.ex_dividend_dates[ordinal];
                if entitled || index == last {
                    let at = Some(index);
                    let dividend = if entitled {
                        source_rational(
                            gilt.coupon_per_100,
                            gilt.dividends[ordinal],
                            at,
                            "pricing bundles",
                        )?
                    } else {
                        SourceRational::ZERO
                    };
                    let amount = if index == last {
                        dividend.with_par(at, "pricing bundles")?
                    } else {
                        dividend
                    };
                    bundles.push(PreparedBundle { index, l, amount });
                }
            }
            l += s;
        }

        Ok(Self {
            gilt,
            settlement_date,
            next_quasi_index,
            n,
            r,
            s,
            accrued_source,
            accrued_per_100,
            bundles,
        })
    }

    /// Borrows the conventional gilt definition.
    #[must_use]
    pub const fn gilt(&self) -> &'a ConventionalGilt {
        self.gilt
    }

    /// Returns the retained formula-contract identifier `UK-DMO-Conventional-v4-2024-12-18`.
    #[must_use]
    pub const fn source_version(&self) -> &'static str {
        SOURCE_VERSION
    }

    /// Returns the settlement date `S`.
    #[must_use]
    pub const fn settlement_date(&self) -> Date {
        self.settlement_date
    }

    /// Returns the absolute index `k+1` of the strictly next quasi date.
    #[must_use]
    pub const fn next_quasi_index(&self) -> usize {
        self.next_quasi_index
    }

    /// Returns `n = m-k-1`; the final quasi interval has zero.
    #[must_use]
    pub const fn remaining_quasi_periods(&self) -> u32 {
        self.n
    }

    /// Returns `r`, the days from settlement to the next quasi date.
    #[must_use]
    pub const fn remaining_days(&self) -> u32 {
        self.r
    }

    /// Returns `s`, the days of the quasi interval containing settlement.
    #[must_use]
    pub const fn quasi_period_days(&self) -> u32 {
        self.s
    }

    /// Returns the number of retained buyer bundles, including redemption.
    #[must_use]
    pub fn bundle_count(&self) -> usize {
        self.bundles.len()
    }

    /// Returns the absolute quasi index of one retained bundle, or None.
    #[must_use]
    pub fn bundle_quasi_index(&self, bundle_index: usize) -> Option<usize> {
        self.bundles.get(bundle_index).map(|bundle| bundle.index)
    }

    /// Returns the entitlement of one actual dividend at settlement, or None.
    ///
    /// In order: `Ceased` if the dividend date is on or before settlement;
    /// `ExCouponDetached` if settlement is strictly after its cutoff;
    /// otherwise `BuyerEntitled`. Settlement equal to the cutoff is cum.
    #[must_use]
    pub fn dividend_entitlement(&self, dividend_index: usize) -> Option<CouponEntitlement> {
        let cutoff = *self.gilt.ex_dividend_dates.get(dividend_index)?;
        let dividend_date =
            self.gilt.quasi_dates[self.gilt.first_dividend_quasi_index + dividend_index];
        Some(if dividend_date <= self.settlement_date {
            CouponEntitlement::Ceased
        } else if self.settlement_date > cutoff {
            CouponEntitlement::ExCouponDetached
        } else {
            CouponEntitlement::BuyerEntitled
        })
    }

    /// Returns the cached nearest binary64 view of the exact source accrued per 100.
    ///
    /// It is rounded once, ties to even, from the exact source rational and
    /// is negative in the ex-dividend period; exact zero is `+0`. Exact
    /// source cash retains the unrounded value independently.
    #[must_use]
    pub const fn accrued_per_100(&self) -> f64 {
        self.accrued_per_100
    }

    /// Copies the owned scalar formula context for reports and snapshots.
    fn metadata(&self) -> Metadata {
        let gilt = self.gilt;
        Metadata {
            issue_date: gilt.issue_date,
            maturity_date: gilt.maturity_date,
            coupon_per_100: gilt.coupon_per_100,
            first_dividend_date: gilt.first_dividend_date,
            first_period: gilt.first_period,
            end_of_month: gilt.end_of_month,
            settlement_date: self.settlement_date,
            next_quasi_date: gilt.quasi_dates[self.next_quasi_index],
            n: self.n,
            r: self.r,
            s: self.s,
        }
    }
}

/// A rejected conventional gilt definition, preparation, valuation or cash request.
///
/// [`kind`](Self::kind) is stable; Display wording is not. Numeric `index`
/// fields identify an absolute quasi index; cash and source-view capacity
/// indices identify an actual-dividend ordinal.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum GiltError {
    /// Quasi-date cap is outside the supported range; kind `gilt.invalid_date_cap`.
    #[non_exhaustive]
    InvalidDateCap {
        /// Rejected caller cap.
        cap: usize,
        /// Smallest supported cap.
        minimum: usize,
        /// Largest supported cap.
        maximum: usize,
    },
    /// Supplied quasi dates exceed the caller cap; kind `gilt.too_many_quasi_dates`.
    #[non_exhaustive]
    TooManyQuasiDates {
        /// Supplied quasi-date count.
        count: usize,
        /// Caller cap.
        cap: usize,
    },
    /// Fewer than two quasi dates were supplied; kind `gilt.too_few_quasi_dates`.
    #[non_exhaustive]
    TooFewQuasiDates {
        /// Supplied quasi-date count.
        count: usize,
        /// Smallest admitted count.
        minimum: usize,
    },
    /// Annual coupon per 100 is negative; kind `gilt.negative_coupon`.
    #[non_exhaustive]
    NegativeCoupon {
        /// Rejected annual coupon per 100.
        coupon_per_100: DecimalAmount,
    },
    /// Issue date is not strictly before maturity; kind `gilt.issue_not_before_maturity`.
    #[non_exhaustive]
    IssueNotBeforeMaturity {
        /// Supplied issue date.
        issue_date: Date,
        /// Supplied maturity date.
        maturity_date: Date,
    },
    /// Last quasi date is not the maturity date; kind `gilt.maturity_boundary`.
    #[non_exhaustive]
    MaturityBoundary {
        /// Supplied maturity date.
        maturity_date: Date,
        /// Supplied last quasi date.
        last_quasi_date: Date,
    },
    /// Quasi dates are not strictly ascending; kind `gilt.quasi_date_order`.
    #[non_exhaustive]
    QuasiDateOrder {
        /// Index of the first nonincreasing quasi date.
        index: usize,
        /// Preceding supplied quasi date.
        previous: Date,
        /// Rejected supplied quasi date.
        date: Date,
    },
    /// Issue date is outside the first quasi interval; kind `gilt.quasi_coverage`.
    #[non_exhaustive]
    QuasiCoverage {
        /// Supplied issue date.
        issue_date: Date,
        /// Supplied first quasi date `Q0`.
        first_quasi_date: Date,
        /// Supplied second quasi date `Q1`.
        next_quasi_date: Date,
    },
    /// A quasi date differs from its direct maturity-anchored date; kind `gilt.quasi_date_mismatch`.
    #[non_exhaustive]
    QuasiDateMismatch {
        /// Index of the first mismatching quasi date.
        index: usize,
        /// Supplied quasi date.
        supplied: Date,
        /// Direct six-month multiple from maturity under the policy.
        expected: Date,
        /// Supplied maturity date.
        maturity_date: Date,
        /// Explicit end-of-month policy.
        end_of_month: EndOfMonth,
    },
    /// Checked date arithmetic failed; kind `gilt.date`, source is the date error.
    #[non_exhaustive]
    Date {
        /// Quasi index whose expected date was being derived.
        index: Option<usize>,
        /// Operation phase, `quasi cycle`.
        phase: &'static str,
        /// Preserved typed date cause.
        cause: DateError,
    },
    /// First actual dividend matches no supported first period; kind `gilt.first_dividend`.
    #[non_exhaustive]
    FirstDividend {
        /// Supplied issue date.
        issue_date: Date,
        /// Supplied first actual dividend date.
        first_dividend_date: Date,
        /// Supplied quasi date `Q1`.
        next_quasi_date: Date,
        /// Supplied quasi date `Q2`, if any.
        second_quasi_date: Option<Date>,
    },
    /// Ex-dividend count differs from the actual dividend count; kind `gilt.ex_date_count`.
    #[non_exhaustive]
    ExDateCount {
        /// Supplied ex-dividend date count.
        count: usize,
        /// Actual dividend count.
        expected: usize,
    },
    /// An ex-dividend cutoff is outside its compatible window; kind `gilt.ex_date_window`.
    #[non_exhaustive]
    ExDateWindow {
        /// Actual dividend ordinal.
        dividend_index: usize,
        /// Rejected supplied cutoff.
        cutoff: Date,
        /// Inclusive lower window bound.
        lower: Date,
        /// Inclusive upper bound, the actual dividend date.
        dividend_date: Date,
    },
    /// Bundle cap exceeds the supported maximum; kind `gilt.invalid_bundle_cap`.
    #[non_exhaustive]
    InvalidBundleCap {
        /// Rejected caller cap.
        cap: usize,
        /// Largest supported cap.
        maximum: usize,
    },
    /// Required buyer bundles exceed the caller cap; kind `gilt.too_many_bundles`.
    #[non_exhaustive]
    TooManyBundles {
        /// Required retained bundle count, including redemption.
        count: usize,
        /// Caller cap.
        cap: usize,
    },
    /// Settlement precedes the issue date; kind `gilt.settlement_before_issue`.
    #[non_exhaustive]
    SettlementBeforeIssue {
        /// Rejected settlement date.
        settlement_date: Date,
        /// Gilt issue date.
        issue_date: Date,
    },
    /// Settlement is on or after maturity; kind `gilt.settlement_at_or_after_maturity`.
    #[non_exhaustive]
    SettlementAtOrAfterMaturity {
        /// Rejected settlement date.
        settlement_date: Date,
        /// Gilt maturity date.
        maturity_date: Date,
    },
    /// An actual dividend ordinal is out of range; kind `gilt.invalid_dividend_index`.
    #[non_exhaustive]
    InvalidDividendIndex {
        /// Rejected actual dividend ordinal.
        index: usize,
        /// Actual dividend count.
        count: usize,
    },
    /// Numeric input or result failure; kind `gilt.value`, source is the value error.
    #[non_exhaustive]
    Value {
        /// Absolute quasi index, or `None` for a plan-level value.
        index: Option<usize>,
        /// Numeric operation phase.
        phase: &'static str,
        /// Preserved typed numeric cause.
        cause: ValueError,
    },
    /// Quote or discount-factor failure; kind `gilt.rate`, source is the rate error.
    #[non_exhaustive]
    Rate {
        /// Absolute quasi index, or `None` for the yield quote.
        index: Option<usize>,
        /// Quote or factor phase.
        phase: &'static str,
        /// Preserved typed rate cause.
        cause: RateError,
    },
    /// Exact summation reduction failure; kind `gilt.sum`, source is the sum error.
    #[non_exhaustive]
    Sum {
        /// Reduction phase.
        phase: &'static str,
        /// Preserved typed summation cause.
        cause: SumError,
    },
    /// A required vector reservation failed; kind `gilt.allocation_failed`.
    #[non_exhaustive]
    AllocationFailed {
        /// Operation requiring storage.
        phase: &'static str,
        /// Requested element slots, not bytes.
        requested: usize,
    },
    /// Defensive private integer-capacity invariant failure; kind `gilt.internal_capacity`.
    #[non_exhaustive]
    InternalCapacity {
        /// Absolute quasi index or actual-dividend ordinal, if applicable.
        index: Option<usize>,
        /// Operation phase.
        phase: &'static str,
        /// Specific bounded operation.
        operation: &'static str,
    },
    /// Dirty-normalized risk requires a nonzero dirty price; kind `gilt.zero_dirty_price`.
    #[non_exhaustive]
    ZeroDirtyPrice {
        /// Requested normalized risk measure.
        measure: &'static str,
        /// Canonical finite annual decimal yield rate.
        yield_rate: f64,
    },
    /// Cash nominal is not strictly positive; kind `gilt.nonpositive_nominal`.
    #[non_exhaustive]
    NonPositiveNominal {
        /// Rejected exact par nominal.
        nominal: DecimalAmount,
    },
    /// Exact cash or source-view quantization failed; kind `gilt.cash_arithmetic`.
    #[non_exhaustive]
    CashArithmetic {
        /// Stable cash or source-view stage.
        stage: &'static str,
        /// Stage input endpoint: `point`, `lower` or `upper`.
        endpoint: &'static str,
        /// Preserved Inexact or OutOfRange amount cause.
        cause: AmountError,
    },
}

impl GiltError {
    /// Returns the stable gilt-domain error kind.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::InvalidDateCap { .. } => "gilt.invalid_date_cap",
            Self::TooManyQuasiDates { .. } => "gilt.too_many_quasi_dates",
            Self::TooFewQuasiDates { .. } => "gilt.too_few_quasi_dates",
            Self::NegativeCoupon { .. } => "gilt.negative_coupon",
            Self::IssueNotBeforeMaturity { .. } => "gilt.issue_not_before_maturity",
            Self::MaturityBoundary { .. } => "gilt.maturity_boundary",
            Self::QuasiDateOrder { .. } => "gilt.quasi_date_order",
            Self::QuasiCoverage { .. } => "gilt.quasi_coverage",
            Self::QuasiDateMismatch { .. } => "gilt.quasi_date_mismatch",
            Self::Date { .. } => "gilt.date",
            Self::FirstDividend { .. } => "gilt.first_dividend",
            Self::ExDateCount { .. } => "gilt.ex_date_count",
            Self::ExDateWindow { .. } => "gilt.ex_date_window",
            Self::InvalidBundleCap { .. } => "gilt.invalid_bundle_cap",
            Self::TooManyBundles { .. } => "gilt.too_many_bundles",
            Self::SettlementBeforeIssue { .. } => "gilt.settlement_before_issue",
            Self::SettlementAtOrAfterMaturity { .. } => "gilt.settlement_at_or_after_maturity",
            Self::InvalidDividendIndex { .. } => "gilt.invalid_dividend_index",
            Self::Value { .. } => "gilt.value",
            Self::Rate { .. } => "gilt.rate",
            Self::Sum { .. } => "gilt.sum",
            Self::AllocationFailed { .. } => "gilt.allocation_failed",
            Self::InternalCapacity { .. } => "gilt.internal_capacity",
            Self::ZeroDirtyPrice { .. } => "gilt.zero_dirty_price",
            Self::NonPositiveNominal { .. } => "gilt.nonpositive_nominal",
            Self::CashArithmetic { .. } => "gilt.cash_arithmetic",
        }
    }
}

impl fmt::Display for GiltError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDateCap {
                cap,
                minimum,
                maximum,
            } => write!(f, "quasi-date cap {cap} is outside {minimum}..={maximum}"),
            Self::TooManyQuasiDates { count, cap } => {
                write!(f, "{count} quasi dates exceed caller cap {cap}")
            }
            Self::TooFewQuasiDates { count, minimum } => {
                write!(
                    f,
                    "{count} quasi dates are fewer than the minimum {minimum}"
                )
            }
            Self::NegativeCoupon { coupon_per_100 } => {
                write!(f, "annual coupon per 100 {coupon_per_100} is negative")
            }
            Self::IssueNotBeforeMaturity {
                issue_date,
                maturity_date,
            } => write!(
                f,
                "issue date {issue_date} is not before maturity {maturity_date}"
            ),
            Self::MaturityBoundary {
                maturity_date,
                last_quasi_date,
            } => write!(
                f,
                "last quasi date {last_quasi_date} differs from maturity {maturity_date}"
            ),
            Self::QuasiDateOrder {
                index,
                previous,
                date,
            } => write!(f, "quasi date {index} ({date}) does not follow {previous}"),
            Self::QuasiCoverage {
                issue_date,
                first_quasi_date,
                next_quasi_date,
            } => write!(
                f,
                "issue date {issue_date} is outside the first quasi interval [{first_quasi_date}, {next_quasi_date})"
            ),
            Self::QuasiDateMismatch {
                index,
                supplied,
                expected,
                maturity_date,
                end_of_month,
            } => write!(
                f,
                "quasi date {index} is {supplied}; maturity {maturity_date} with end of month {end_of_month} requires {expected}"
            ),
            Self::Date {
                index,
                phase,
                cause,
            } => write!(f, "{phase} at quasi index {index:?}: {cause}"),
            Self::FirstDividend {
                issue_date,
                first_dividend_date,
                next_quasi_date,
                second_quasi_date,
            } => write!(
                f,
                "first dividend {first_dividend_date} after issue {issue_date} is neither {next_quasi_date} nor a supported long first date {second_quasi_date:?}"
            ),
            Self::ExDateCount { count, expected } => write!(
                f,
                "{count} ex-dividend dates do not match {expected} actual dividends"
            ),
            Self::ExDateWindow {
                dividend_index,
                cutoff,
                lower,
                dividend_date,
            } => write!(
                f,
                "dividend {dividend_index} ex-dividend cutoff {cutoff} is outside [{lower}, {dividend_date}]"
            ),
            Self::InvalidBundleCap { cap, maximum } => {
                write!(f, "bundle cap {cap} exceeds supported maximum {maximum}")
            }
            Self::TooManyBundles { count, cap } => {
                write!(f, "{count} pricing bundles exceed caller cap {cap}")
            }
            Self::SettlementBeforeIssue {
                settlement_date,
                issue_date,
            } => write!(
                f,
                "settlement date {settlement_date} precedes issue date {issue_date}"
            ),
            Self::SettlementAtOrAfterMaturity {
                settlement_date,
                maturity_date,
            } => write!(
                f,
                "settlement date {settlement_date} is not before maturity {maturity_date}"
            ),
            Self::InvalidDividendIndex { index, count } => {
                write!(f, "dividend index {index} is outside 0..{count}")
            }
            Self::Value {
                index,
                phase,
                cause,
            } => write!(f, "{phase} at quasi index {index:?}: {cause}"),
            Self::Rate {
                index,
                phase,
                cause,
            } => write!(f, "{phase} at quasi index {index:?}: {cause}"),
            Self::Sum { phase, cause } => write!(f, "{phase}: {cause}"),
            Self::AllocationFailed { phase, requested } => {
                write!(f, "{phase}: cannot reserve {requested} elements")
            }
            Self::InternalCapacity {
                index,
                phase,
                operation,
            } => write!(
                f,
                "{phase} at index {index:?}: internal capacity invariant failed during {operation}"
            ),
            Self::ZeroDirtyPrice {
                measure,
                yield_rate,
            } => write!(
                f,
                "cannot normalize {measure} at yield {yield_rate:?}: dirty price is zero"
            ),
            Self::NonPositiveNominal { nominal } => {
                write!(f, "nominal {nominal} is not strictly positive")
            }
            Self::CashArithmetic {
                stage,
                endpoint,
                cause,
            } => write!(f, "{stage} at {endpoint}: {cause}"),
        }
    }
}

impl StdError for GiltError {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::NonFinite;
    use std::hash::{DefaultHasher, Hash, Hasher};

    fn date(year: i32, month: u32, day: u32) -> Date {
        Date::from_ymd(year, month, day).expect("valid test date")
    }

    fn amount(coefficient: i128, scale: u8) -> DecimalAmount {
        DecimalAmount::new(coefficient, scale).expect("valid test amount")
    }

    fn scalar(value: u128) -> Scalar {
        Scalar::from_u128(value)
    }

    /// Q from 1998-12-07 to 2001-06-07; s1=182, the interval after Q1 has 183 days.
    fn quasi() -> Vec<Date> {
        vec![
            date(1998, 12, 7),
            date(1999, 6, 7),
            date(1999, 12, 7),
            date(2000, 6, 7),
            date(2000, 12, 7),
            date(2001, 6, 7),
        ]
    }

    fn later_cutoffs() -> Vec<Date> {
        vec![
            date(1999, 11, 29),
            date(2000, 5, 30),
            date(2000, 11, 29),
            date(2001, 5, 30),
        ]
    }

    fn gilt(issue: Date, first: Date, ex: Vec<Date>, coupon: DecimalAmount) -> ConventionalGilt {
        ConventionalGilt::new(
            issue,
            date(2001, 6, 7),
            coupon,
            first,
            quasi(),
            ex,
            EndOfMonth::Ignore,
            6,
        )
        .expect("valid test gilt")
    }

    /// Long: I=1999-05-07 (r1=31), skipped Q1, F=1999-12-07, C=6.
    fn long_gilt() -> ConventionalGilt {
        gilt(
            date(1999, 5, 7),
            date(1999, 12, 7),
            later_cutoffs(),
            amount(6, 0),
        )
    }

    fn first_period_cutoffs() -> Vec<Date> {
        let mut cutoffs = vec![date(1999, 5, 28)];
        cutoffs.extend(later_cutoffs());
        cutoffs
    }

    /// Short: I=1999-03-07 (r1=92), F=Q1, C=6.
    fn short_gilt() -> ConventionalGilt {
        gilt(
            date(1999, 3, 7),
            date(1999, 6, 7),
            first_period_cutoffs(),
            amount(6, 0),
        )
    }

    fn regular_gilt(coupon: DecimalAmount) -> ConventionalGilt {
        gilt(
            date(1998, 12, 7),
            date(1999, 6, 7),
            first_period_cutoffs(),
            coupon,
        )
    }

    fn plan(gilt: &ConventionalGilt, settlement: Date) -> GiltPricingPlan<'_> {
        GiltPricingPlan::new(gilt, settlement, GiltPricingPlan::MAX_BUNDLES).expect("valid plan")
    }

    const fn unrounded(numerator: i32, denominator: u32) -> ExactSource {
        ExactSource::Unrounded {
            numerator,
            denominator,
        }
    }

    const fn rational(numerator: u128, denominator: u128) -> SourceRational {
        SourceRational {
            numerator: Scalar::from_u128(numerator),
            denominator: Scalar::from_u128(denominator),
            negative: false,
        }
    }

    /// FR-GILT-03: positive half-up six-place publication, both decimal-power
    /// paths, ties, zero and the 147-bit maximum micro coefficient.
    #[test]
    fn first_publication_rounds_half_up_on_both_scale_paths() {
        // DMO long example 6*(182+31)/(2*182) = 639/182 publishes 3.510989.
        assert_eq!(
            publish_first_dividend(amount(6, 0), 213, 182),
            Ok(scalar(3_510_989))
        );
        // Exact half micro ties round up: 3e-6*61/366 and 1.5e-6*122/366.
        assert_eq!(publish_first_dividend(amount(3, 6), 61, 183), Ok(scalar(1)));
        assert_eq!(
            publish_first_dividend(amount(15, 7), 122, 183),
            Ok(scalar(1))
        );
        // 1.4e-6*122/366 is 7/15 micro, below the tie.
        assert_eq!(
            publish_first_dividend(amount(14, 7), 122, 183),
            Ok(scalar(0))
        );
        assert_eq!(
            publish_first_dividend(amount(1, 38), 183, 184),
            Ok(scalar(0))
        );
        assert_eq!(
            publish_first_dividend(DecimalAmount::new(0, 0).expect("zero"), 100, 183),
            Ok(scalar(0))
        );
        // (2^127-1)*367/368 per 100 needs a 147-bit micro coefficient.
        let maximum = scalar(0x7_9bd1)
            .checked_shl(128)
            .and_then(|high| high.checked_add(scalar(0x4de9_bd37_a6f4_de9b_d37a_6f4d_e9ae_0004)))
            .expect("147-bit value");
        let published = publish_first_dividend(amount(i128::MAX, 0), 367, 184);
        assert_eq!(published, Ok(maximum));
        assert_eq!(maximum.bits(), 147);
    }

    /// FR-GILT-03/09: unreduced signed source rationals, canonical zero and
    /// par grouping in the dividend's own basis.
    #[test]
    fn source_rationals_retain_unreduced_signed_and_published_values() {
        // Ex AI -1/61 is retained as 6*(-1)/366.
        let ex = source_rational(amount(6, 0), unrounded(-1, 366), None, "source accrued");
        assert_eq!(
            ex,
            Ok(SourceRational {
                negative: true,
                ..rational(6, 366)
            })
        );
        let zero = DecimalAmount::new(0, 0).expect("zero");
        assert_eq!(
            source_rational(zero, unrounded(-1, 366), None, "source accrued"),
            Ok(rational(0, 366))
        );
        let tiny = source_rational(amount(1, 38), REGULAR_DIVIDEND, Some(0), "pricing bundles")
            .expect("scale 38");
        assert_eq!(tiny, rational(1, 2 * 10_u128.pow(38)));
        let published = ExactSource::Published {
            micro: scalar(3_510_989),
        };
        let first = source_rational(amount(6, 0), published, Some(2), "pricing bundles")
            .expect("published");
        assert_eq!(first, rational(3_510_989, 1_000_000));
        assert_eq!(
            first.with_par(Some(2), "pricing bundles"),
            Ok(rational(103_510_989, 1_000_000))
        );
        let regular = source_rational(amount(6, 0), REGULAR_DIVIDEND, Some(5), "pricing bundles")
            .expect("regular");
        assert_eq!(
            regular.with_par(Some(5), "pricing bundles"),
            Ok(rational(206, 2))
        );
        assert_eq!(
            SourceRational::ZERO.with_par(Some(5), "pricing bundles"),
            Ok(rational(100, 1))
        );
    }

    /// FR-GILT-04: all seven exact AI branches, original issue ratios, the
    /// skipped-Q1 nonreset, the actual-F reset and the -1/61 discriminator.
    #[test]
    fn seven_source_accrued_branches_use_original_issue_ratios() {
        let long = long_gilt();
        for ((year, month, day), expected) in [
            // Long first interval: t*/(2*s1), s1=182.
            ((1999, 5, 7), unrounded(0, 364)),
            ((1999, 5, 20), unrounded(13, 364)),
            // At Q1 r2=0: r1*s2/(2*s1*s2); no reset at the skipped date.
            ((1999, 6, 7), unrounded(31 * 183, 2 * 182 * 183)),
            // Long second cum at the cutoff (r2=175).
            (
                (1999, 11, 29),
                unrounded(31 * 183 + 175 * 182, 2 * 182 * 183),
            ),
            // Long second ex: (r2-s2)/(2*s2) = -1/366 of C=6, i.e. -1/61.
            ((1999, 12, 6), unrounded(-1, 366)),
            // Actual F resets to a regular interval with t=0.
            ((1999, 12, 7), unrounded(0, 366)),
            ((2000, 1, 7), unrounded(31, 366)),
            ((2000, 6, 4), unrounded(-3, 366)),
        ] {
            let settlement = date(year, month, day);
            assert_eq!(
                plan(&long, settlement).accrued_source,
                expected,
                "{settlement}"
            );
        }
        let short = short_gilt();
        for ((year, month, day), expected) in [
            ((1999, 3, 7), unrounded(0, 364)),
            ((1999, 4, 7), unrounded(31, 364)),
            // Short ex: (t*-r1)/(2*s1) with r1=92, not the settlement r.
            ((1999, 6, 1), unrounded(-6, 364)),
            ((1999, 6, 7), unrounded(0, 366)),
        ] {
            let settlement = date(year, month, day);
            assert_eq!(
                plan(&short, settlement).accrued_source,
                expected,
                "{settlement}"
            );
        }
        let regular = regular_gilt(amount(6, 0));
        for ((year, month, day), expected) in [
            ((1998, 12, 7), unrounded(0, 364)),
            ((1999, 1, 7), unrounded(31, 364)),
            ((1999, 5, 28), unrounded(172, 364)),
            ((1999, 6, 1), unrounded(-6, 364)),
        ] {
            let settlement = date(year, month, day);
            assert_eq!(
                plan(&regular, settlement).accrued_source,
                expected,
                "{settlement}"
            );
        }
    }

    /// FR-GILT-06: absolute sparse indices, exact L/s coordinates, the skipped
    /// Long Q1, detached dividends, entitled zero coupons and grouped par.
    #[test]
    fn plan_bundles_keep_absolute_indices_and_grouped_par() {
        let bundles = |plan: &GiltPricingPlan<'_>| {
            plan.bundles
                .iter()
                .map(|bundle| (bundle.index, bundle.l, bundle.amount))
                .collect::<Vec<_>>()
        };
        let long = long_gilt();
        let at_issue = plan(&long, date(1999, 5, 7));
        assert_eq!(
            (
                at_issue.next_quasi_index(),
                at_issue.remaining_days(),
                at_issue.quasi_period_days(),
                at_issue.remaining_quasi_periods()
            ),
            (1, 31, 182, 4)
        );
        assert_eq!(
            bundles(&at_issue),
            vec![
                (2, 213, rational(3_510_989, 1_000_000)),
                (3, 395, rational(6, 2)),
                (4, 577, rational(6, 2)),
                (5, 759, rational(206, 2)),
            ]
        );
        assert_eq!(at_issue.bundle_count(), 4);
        assert_eq!(at_issue.bundle_quasi_index(0), Some(2));
        assert_eq!(at_issue.bundle_quasi_index(4), None);

        let ex = plan(&long, date(1999, 12, 6));
        assert_eq!(
            (
                ex.next_quasi_index(),
                ex.remaining_days(),
                ex.quasi_period_days()
            ),
            (2, 1, 183)
        );
        assert_eq!(
            bundles(&ex),
            vec![
                (3, 184, rational(6, 2)),
                (4, 367, rational(6, 2)),
                (5, 550, rational(206, 2)),
            ]
        );

        // Final interval after the final cutoff: n=0 and redemption alone.
        let last = plan(&long, date(2001, 6, 1));
        assert_eq!(
            (
                last.next_quasi_index(),
                last.remaining_days(),
                last.quasi_period_days()
            ),
            (5, 6, 182)
        );
        assert_eq!(last.remaining_quasi_periods(), 0);
        assert_eq!(bundles(&last), vec![(5, 6, rational(100, 1))]);

        // A zero coupon keeps every entitled actual dividend bundle (r=151, s=182).
        let zero = regular_gilt(DecimalAmount::new(0, 0).expect("zero"));
        let zero_plan = plan(&zero, date(1999, 1, 7));
        assert_eq!(
            bundles(&zero_plan),
            vec![
                (1, 151, rational(0, 2)),
                (2, 333, rational(0, 2)),
                (3, 515, rational(0, 2)),
                (4, 697, rational(0, 2)),
                (5, 879, rational(200, 2)),
            ]
        );
        assert_eq!(zero_plan.accrued_source, unrounded(31, 364));
    }

    /// FR-GILT-05: Ceased, ExCouponDetached and BuyerEntitled in order, with
    /// cutoff equality cum and out-of-range None.
    #[test]
    fn dividend_entitlement_orders_ceased_detached_and_entitled() {
        let long = long_gilt();
        let at_cutoff = plan(&long, date(1999, 11, 29));
        assert_eq!(
            at_cutoff.dividend_entitlement(0),
            Some(CouponEntitlement::BuyerEntitled)
        );
        assert_eq!(at_cutoff.bundle_quasi_index(0), Some(2));
        let after = plan(&long, date(1999, 11, 30));
        assert_eq!(
            after.dividend_entitlement(0),
            Some(CouponEntitlement::ExCouponDetached)
        );
        assert_eq!(after.bundle_quasi_index(0), Some(3));
        let paid = plan(&long, date(1999, 12, 7));
        assert_eq!(
            paid.dividend_entitlement(0),
            Some(CouponEntitlement::Ceased)
        );
        assert_eq!(
            paid.dividend_entitlement(1),
            Some(CouponEntitlement::BuyerEntitled)
        );
        assert_eq!(paid.dividend_entitlement(4), None);
        let last = plan(&long, date(2001, 6, 1));
        assert_eq!(
            last.dividend_entitlement(2),
            Some(CouponEntitlement::Ceased)
        );
        assert_eq!(
            last.dividend_entitlement(3),
            Some(CouponEntitlement::ExCouponDetached)
        );
    }

    /// FR-GILT-02/03/05: constructor precedence, direct maturity-anchored
    /// cycle under both policies and the supported first-period roles.
    #[test]
    fn constructor_failure_order_and_direct_cycle() {
        let build = |issue: Date,
                     maturity: Date,
                     coupon: DecimalAmount,
                     first: Date,
                     quasi: Vec<Date>,
                     ex: Vec<Date>,
                     policy: EndOfMonth,
                     cap: usize| {
            ConventionalGilt::new(issue, maturity, coupon, first, quasi, ex, policy, cap)
        };
        let issue = date(1999, 5, 7);
        let maturity = date(2001, 6, 7);
        let first = date(1999, 12, 7);
        let six = amount(6, 0);
        let negative = amount(-1, 0);
        let ignore = EndOfMonth::Ignore;
        for cap in [0, 1, ConventionalGilt::MAX_QUASI_DATES + 1] {
            assert_eq!(
                build(
                    maturity,
                    issue,
                    negative,
                    first,
                    vec![],
                    vec![],
                    ignore,
                    cap
                ),
                Err(GiltError::InvalidDateCap {
                    cap,
                    minimum: 2,
                    maximum: ConventionalGilt::MAX_QUASI_DATES,
                })
            );
        }
        assert_eq!(
            build(maturity, issue, negative, first, quasi(), vec![], ignore, 5),
            Err(GiltError::TooManyQuasiDates { count: 6, cap: 5 })
        );
        assert_eq!(
            build(
                maturity,
                issue,
                negative,
                first,
                vec![maturity],
                vec![],
                ignore,
                2
            ),
            Err(GiltError::TooFewQuasiDates {
                count: 1,
                minimum: 2,
            })
        );
        assert_eq!(
            build(maturity, issue, negative, first, quasi(), vec![], ignore, 6),
            Err(GiltError::NegativeCoupon {
                coupon_per_100: negative,
            })
        );
        assert_eq!(
            build(maturity, maturity, six, first, quasi(), vec![], ignore, 6),
            Err(GiltError::IssueNotBeforeMaturity {
                issue_date: maturity,
                maturity_date: maturity,
            })
        );
        let mut disordered = quasi();
        disordered[2] = date(1999, 6, 7);
        disordered[5] = date(2001, 6, 8);
        assert_eq!(
            build(
                issue,
                maturity,
                six,
                first,
                disordered.clone(),
                vec![],
                ignore,
                6
            ),
            Err(GiltError::MaturityBoundary {
                maturity_date: maturity,
                last_quasi_date: date(2001, 6, 8),
            })
        );
        disordered[5] = maturity;
        assert_eq!(
            build(issue, maturity, six, first, disordered, vec![], ignore, 6),
            Err(GiltError::QuasiDateOrder {
                index: 2,
                previous: date(1999, 6, 7),
                date: date(1999, 6, 7),
            })
        );
        for uncovered in [date(1998, 12, 6), date(1999, 6, 7)] {
            assert_eq!(
                build(uncovered, maturity, six, first, quasi(), vec![], ignore, 6),
                Err(GiltError::QuasiCoverage {
                    issue_date: uncovered,
                    first_quasi_date: date(1998, 12, 7),
                    next_quasi_date: date(1999, 6, 7),
                })
            );
        }
        let mut shifted = quasi();
        shifted[1] = date(1999, 6, 8);
        assert_eq!(
            build(issue, maturity, six, first, shifted, vec![], ignore, 6),
            Err(GiltError::QuasiDateMismatch {
                index: 1,
                supplied: date(1999, 6, 8),
                expected: date(1999, 6, 7),
                maturity_date: maturity,
                end_of_month: ignore,
            })
        );
        // Expected Q0 is maturity minus 12 months, before 0001-01-01.
        let early = date(1, 7, 1);
        let failed = build(
            date(1, 1, 1),
            early,
            six,
            date(1, 1, 2),
            vec![date(1, 1, 1), date(1, 1, 2), early],
            vec![],
            ignore,
            3,
        );
        assert!(matches!(
            failed,
            Err(GiltError::Date {
                index: Some(0),
                phase: "quasi cycle",
                cause: DateError::OutOfRange { count: -12, .. },
            })
        ));

        // Direct, not recursive: 2024-08-31 minus 12 months is 2023-08-31.
        let month_end = date(2024, 8, 31);
        let direct = vec![date(2023, 8, 31), date(2024, 2, 29), month_end];
        let accepted = build(
            date(2023, 8, 31),
            month_end,
            six,
            date(2024, 2, 29),
            direct,
            vec![date(2024, 2, 20), date(2024, 8, 20)],
            ignore,
            3,
        )
        .expect("direct Ignore cycle");
        assert_eq!(accepted.first_period(), GiltFirstPeriod::Regular);
        // February maturity: Preserve moves Q0 to month-end, Ignore clamps to 29.
        let february = date(2024, 2, 29);
        let preserved = vec![date(2023, 8, 31), february];
        assert!(
            build(
                date(2023, 8, 31),
                february,
                six,
                february,
                preserved.clone(),
                vec![february],
                EndOfMonth::Preserve,
                2,
            )
            .is_ok()
        );
        assert_eq!(
            build(
                date(2023, 8, 31),
                february,
                six,
                february,
                preserved,
                vec![february],
                ignore,
                2,
            ),
            Err(GiltError::QuasiDateMismatch {
                index: 0,
                supplied: date(2023, 8, 31),
                expected: date(2023, 8, 29),
                maturity_date: february,
                end_of_month: ignore,
            })
        );

        // Issue on Q0 with F=Q2 is unsupported; so is F off the cycle.
        let on_cycle = date(1998, 12, 7);
        for unsupported in [date(1999, 12, 7), date(1999, 12, 6)] {
            assert_eq!(
                build(
                    on_cycle,
                    maturity,
                    six,
                    unsupported,
                    quasi(),
                    vec![],
                    ignore,
                    6
                ),
                Err(GiltError::FirstDividend {
                    issue_date: on_cycle,
                    first_dividend_date: unsupported,
                    next_quasi_date: date(1999, 6, 7),
                    second_quasi_date: Some(date(1999, 12, 7)),
                })
            );
        }
        assert_eq!(
            build(issue, maturity, six, first, quasi(), vec![], ignore, 6),
            Err(GiltError::ExDateCount {
                count: 0,
                expected: 4,
            })
        );
        // Long first window starts at Q1; Short first at I; later at Q(k-1).
        let mut early_long = later_cutoffs();
        early_long[0] = date(1999, 6, 6);
        assert_eq!(
            build(issue, maturity, six, first, quasi(), early_long, ignore, 6),
            Err(GiltError::ExDateWindow {
                dividend_index: 0,
                cutoff: date(1999, 6, 6),
                lower: date(1999, 6, 7),
                dividend_date: first,
            })
        );
        let short_issue = date(1999, 3, 7);
        let mut early_short = first_period_cutoffs();
        early_short[0] = date(1999, 3, 6);
        assert_eq!(
            build(
                short_issue,
                maturity,
                six,
                date(1999, 6, 7),
                quasi(),
                early_short,
                ignore,
                6
            ),
            Err(GiltError::ExDateWindow {
                dividend_index: 0,
                cutoff: date(1999, 3, 6),
                lower: short_issue,
                dividend_date: date(1999, 6, 7),
            })
        );
        let mut late = later_cutoffs();
        late[2] = date(2000, 12, 8);
        assert_eq!(
            build(issue, maturity, six, first, quasi(), late, ignore, 6),
            Err(GiltError::ExDateWindow {
                dividend_index: 2,
                cutoff: date(2000, 12, 8),
                lower: date(2000, 6, 7),
                dividend_date: date(2000, 12, 7),
            })
        );
        // Window endpoints are inclusive.
        let inclusive = vec![
            date(1999, 6, 7),
            date(2000, 6, 7),
            date(2000, 6, 7),
            date(2001, 6, 7),
        ];
        assert!(build(issue, maturity, six, first, quasi(), inclusive, ignore, 6).is_ok());
    }

    /// FR-GILT-06/21: plan cap, settlement domain and bundle-count precedence.
    #[test]
    fn plan_preparation_order_and_bundle_caps() {
        let long = long_gilt();
        assert_eq!(
            GiltPricingPlan::new(&long, date(1990, 1, 1), GiltPricingPlan::MAX_BUNDLES + 1),
            Err(GiltError::InvalidBundleCap {
                cap: GiltPricingPlan::MAX_BUNDLES + 1,
                maximum: GiltPricingPlan::MAX_BUNDLES,
            })
        );
        assert_eq!(
            GiltPricingPlan::new(&long, date(1999, 5, 6), 0),
            Err(GiltError::SettlementBeforeIssue {
                settlement_date: date(1999, 5, 6),
                issue_date: date(1999, 5, 7),
            })
        );
        assert_eq!(
            GiltPricingPlan::new(&long, date(2001, 6, 7), 0),
            Err(GiltError::SettlementAtOrAfterMaturity {
                settlement_date: date(2001, 6, 7),
                maturity_date: date(2001, 6, 7),
            })
        );
        for cap in [0, 3] {
            assert_eq!(
                GiltPricingPlan::new(&long, date(1999, 5, 7), cap),
                Err(GiltError::TooManyBundles { count: 4, cap })
            );
        }
        assert!(GiltPricingPlan::new(&long, date(1999, 5, 7), 4).is_ok());
        assert_eq!(
            GiltPricingPlan::new(&long, date(1999, 12, 6), 2),
            Err(GiltError::TooManyBundles { count: 3, cap: 2 })
        );
        assert!(GiltPricingPlan::new(&long, date(2001, 6, 1), 1).is_ok());
    }

    /// FR-GILT-01: definition identity, inspectors and owned plan metadata.
    #[test]
    fn definition_identity_inspectors_and_metadata() {
        let long = long_gilt();
        let hash = |gilt: &ConventionalGilt| {
            let mut hasher = DefaultHasher::new();
            gilt.hash(&mut hasher);
            hasher.finish()
        };
        assert_eq!(long, long_gilt());
        assert_eq!(hash(&long), hash(&long_gilt()));
        let mut moved = later_cutoffs();
        moved[3] = date(2001, 5, 29);
        assert_ne!(
            long,
            gilt(date(1999, 5, 7), date(1999, 12, 7), moved, amount(6, 0))
        );
        // Canonical coupon spelling does not affect identity.
        assert_eq!(
            long,
            gilt(
                date(1999, 5, 7),
                date(1999, 12, 7),
                later_cutoffs(),
                amount(600, 2)
            )
        );
        assert_eq!(long.source_version(), "UK-DMO-Conventional-v4-2024-12-18");
        assert_eq!(long.issue_date(), date(1999, 5, 7));
        assert_eq!(long.maturity_date(), date(2001, 6, 7));
        assert_eq!(long.coupon_per_100(), amount(6, 0));
        assert_eq!(long.first_dividend_date(), date(1999, 12, 7));
        assert_eq!(long.first_period(), GiltFirstPeriod::Long);
        assert_eq!(long.first_dividend_quasi_index(), 2);
        assert_eq!(long.quasi_dates(), quasi().as_slice());
        assert_eq!(long.dividend_dates(), &quasi()[2..]);
        assert_eq!(long.ex_dividend_dates(), later_cutoffs().as_slice());
        assert_eq!(long.end_of_month(), EndOfMonth::Ignore);
        assert_eq!(long.ex_coupon_boundary(), ExCouponBoundary::After);
        assert_eq!(long.dividend_count(), 4);
        assert_eq!(
            long.dividends,
            vec![
                ExactSource::Published {
                    micro: scalar(3_510_989),
                },
                REGULAR_DIVIDEND,
                REGULAR_DIVIDEND,
                REGULAR_DIVIDEND,
            ]
        );
        assert_eq!(short_gilt().first_period(), GiltFirstPeriod::Short);
        assert_eq!(short_gilt().dividend_dates(), &quasi()[1..]);
        assert_eq!(GiltFirstPeriod::ALL.len(), 3);

        let ex = plan(&long, date(1999, 12, 6));
        assert!(std::ptr::eq(ex.gilt(), &long));
        assert_eq!(ex.source_version(), long.source_version());
        assert_eq!(ex.settlement_date(), date(1999, 12, 6));
        assert_eq!(
            ex.metadata(),
            Metadata {
                issue_date: date(1999, 5, 7),
                maturity_date: date(2001, 6, 7),
                coupon_per_100: amount(6, 0),
                first_dividend_date: date(1999, 12, 7),
                first_period: GiltFirstPeriod::Long,
                end_of_month: EndOfMonth::Ignore,
                settlement_date: date(1999, 12, 6),
                next_quasi_date: date(1999, 12, 7),
                n: 3,
                r: 1,
                s: 183,
            }
        );
    }

    /// FR-GILT-20: all 26 stable kinds and the five typed source chains.
    #[test]
    fn error_kinds_and_sources() {
        let day = date(2000, 1, 1);
        let value = ValueError::NotFinite {
            parameter: "rate",
            value: NonFinite::NaN,
        };
        let errors = [
            GiltError::InvalidDateCap {
                cap: 1,
                minimum: 2,
                maximum: 3,
            },
            GiltError::TooManyQuasiDates { count: 2, cap: 1 },
            GiltError::TooFewQuasiDates {
                count: 1,
                minimum: 2,
            },
            GiltError::NegativeCoupon {
                coupon_per_100: amount(-1, 0),
            },
            GiltError::IssueNotBeforeMaturity {
                issue_date: day,
                maturity_date: day,
            },
            GiltError::MaturityBoundary {
                maturity_date: day,
                last_quasi_date: day,
            },
            GiltError::QuasiDateOrder {
                index: 1,
                previous: day,
                date: day,
            },
            GiltError::QuasiCoverage {
                issue_date: day,
                first_quasi_date: day,
                next_quasi_date: day,
            },
            GiltError::QuasiDateMismatch {
                index: 0,
                supplied: day,
                expected: day,
                maturity_date: day,
                end_of_month: EndOfMonth::Preserve,
            },
            GiltError::Date {
                index: Some(0),
                phase: "quasi cycle",
                cause: DateError::OutOfRange {
                    date: day,
                    operation: "add months",
                    count: -6,
                },
            },
            GiltError::FirstDividend {
                issue_date: day,
                first_dividend_date: day,
                next_quasi_date: day,
                second_quasi_date: None,
            },
            GiltError::ExDateCount {
                count: 0,
                expected: 1,
            },
            GiltError::ExDateWindow {
                dividend_index: 0,
                cutoff: day,
                lower: day,
                dividend_date: day,
            },
            GiltError::InvalidBundleCap { cap: 1, maximum: 0 },
            GiltError::TooManyBundles { count: 1, cap: 0 },
            GiltError::SettlementBeforeIssue {
                settlement_date: day,
                issue_date: day,
            },
            GiltError::SettlementAtOrAfterMaturity {
                settlement_date: day,
                maturity_date: day,
            },
            GiltError::InvalidDividendIndex { index: 1, count: 1 },
            GiltError::Value {
                index: None,
                phase: "yield quote",
                cause: value.clone(),
            },
            GiltError::Rate {
                index: Some(3),
                phase: "discount factor",
                cause: RateError::Value(value.clone()),
            },
            GiltError::Sum {
                phase: "dirty price reduction",
                cause: SumError::Arithmetic {
                    index: None,
                    error: value,
                },
            },
            GiltError::AllocationFailed {
                phase: "pricing bundles",
                requested: 1,
            },
            capacity(None, "source accrued", "derive source"),
            GiltError::ZeroDirtyPrice {
                measure: "convexity",
                yield_rate: 0.0,
            },
            GiltError::NonPositiveNominal {
                nominal: amount(0, 0),
            },
            GiltError::CashArithmetic {
                stage: "cash total",
                endpoint: "point",
                cause: AmountError::Inexact {
                    operation: "quantize",
                },
            },
        ];
        let kinds = errors.iter().map(GiltError::kind).collect::<Vec<_>>();
        assert_eq!(
            kinds,
            [
                "gilt.invalid_date_cap",
                "gilt.too_many_quasi_dates",
                "gilt.too_few_quasi_dates",
                "gilt.negative_coupon",
                "gilt.issue_not_before_maturity",
                "gilt.maturity_boundary",
                "gilt.quasi_date_order",
                "gilt.quasi_coverage",
                "gilt.quasi_date_mismatch",
                "gilt.date",
                "gilt.first_dividend",
                "gilt.ex_date_count",
                "gilt.ex_date_window",
                "gilt.invalid_bundle_cap",
                "gilt.too_many_bundles",
                "gilt.settlement_before_issue",
                "gilt.settlement_at_or_after_maturity",
                "gilt.invalid_dividend_index",
                "gilt.value",
                "gilt.rate",
                "gilt.sum",
                "gilt.allocation_failed",
                "gilt.internal_capacity",
                "gilt.zero_dirty_price",
                "gilt.nonpositive_nominal",
                "gilt.cash_arithmetic",
            ]
        );
        let sourced = [
            "gilt.date",
            "gilt.value",
            "gilt.rate",
            "gilt.sum",
            "gilt.cash_arithmetic",
        ];
        for error in &errors {
            assert_eq!(
                error.source().is_some(),
                sourced.contains(&error.kind()),
                "{error}"
            );
            assert!(!error.to_string().is_empty());
        }
    }
}
