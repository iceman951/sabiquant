//! Standard Following CDS payment and accrual date roles.

use super::{CdsMaturityConvention, DateGenerationRule, ScheduleCalendar, ScheduleError};
use crate::calendar::{BusinessDayConvention, EndpointInclusion};
use crate::time::{Date, DateError};

/// The distinct raw, payment and accrual dates of one standard CDS coupon.
///
/// Ordinary accrual ends exclude the adjusted payment date. The last coupon
/// includes raw contractual maturity, regardless of its later payment date.
/// No day-counter, premium amount or legal protection period is implied.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CdsCouponDates {
    unadjusted_start: Date,
    unadjusted_end: Date,
    payment_date: Date,
    accrual_start: Date,
    accrual_end: Date,
    accrual_end_inclusion: EndpointInclusion,
    accrual_day_count: u32,
}

impl CdsCouponDates {
    /// Returns the raw quarterly twentieth preceding this coupon's raw end.
    #[must_use]
    pub const fn unadjusted_start(&self) -> Date {
        self.unadjusted_start
    }

    /// Returns this coupon's raw quarterly twentieth.
    #[must_use]
    pub const fn unadjusted_end(&self) -> Date {
        self.unadjusted_end
    }

    /// Returns the Following-adjusted raw end.
    #[must_use]
    pub const fn payment_date(&self) -> Date {
        self.payment_date
    }

    /// Returns the included, Following-adjusted preceding quarter date.
    #[must_use]
    pub const fn accrual_start(&self) -> Date {
        self.accrual_start
    }

    /// Returns the adjusted ordinary end, or raw maturity for the final coupon.
    #[must_use]
    pub const fn accrual_end(&self) -> Date {
        self.accrual_end
    }

    /// Returns Exclude ordinarily and Include for the final raw maturity.
    #[must_use]
    pub const fn accrual_end_inclusion(&self) -> EndpointInclusion {
        self.accrual_end_inclusion
    }

    /// Returns actual days, including the final raw maturity when applicable.
    #[must_use]
    pub const fn accrual_day_count(&self) -> u32 {
        self.accrual_day_count
    }
}

/// An immutable modern CDS plan with distinct raw, payment and accrual roles.
///
/// This is a date operation under the explicitly selected standard Following
/// contract, not a priced credit product. It has no implicit conversion into
/// an ordinary [`Schedule`](super::Schedule) or raw-accrual fixed-rate leg.
/// Calendar, trade, supplied maturity and explicit regime define identity;
/// the successful operation's resource cap does not.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CdsDatePlan {
    trade_date: Date,
    payment_selection_cutoff: Date,
    unadjusted_maturity: Date,
    calendar: ScheduleCalendar,
    convention: CdsMaturityConvention,
    coupons: Vec<CdsCouponDates>,
}

impl CdsDatePlan {
    /// Minimum accepted final coupon cap.
    pub const MIN_COUPONS: u32 = 1;

    /// Safe upper bound: all quarterly twentieths in years 0001 through 9999.
    ///
    /// A plan also requires a representable accrual precursor, so this bound
    /// is not a claim that every allowed count is attainable.
    pub const MAX_COUPONS: u32 = 39_996;

    /// Builds quarterly twentieths with Following payments and standard accrual.
    ///
    /// The first payment is strictly after unadjusted trade plus one calendar
    /// day. Its preceding adjusted quarter starts accrual. The final coupon
    /// includes raw maturity; ordinary coupons exclude their adjusted ends.
    /// The supplied maturity must already be a quarterly twentieth. Both
    /// regimes accept March/September maturities; this operation does not
    /// resolve a quoted tenor or infer a regime from the trade date.
    ///
    /// # Errors
    /// Checks coupon cap, increasing range and maturity domain in that order,
    /// then original calendar coverage. Required cutoff-quarter selection
    /// precedes exact coupon-count enforcement; an additional precursor is
    /// resolved only after that cap succeeds. Returns typed date/calendar
    /// failures, no eligible payment, adjusted-date collision or allocation
    /// failure without a partial plan. All required TARGET spans and search
    /// candidates retain the supplied calendar's guards.
    ///
    /// # Examples
    /// ```
    /// use sabiquant::calendar::{Calendar, EndpointInclusion};
    /// use sabiquant::schedule::{CdsDatePlan, CdsMaturityConvention, ScheduleCalendar};
    /// let plan = CdsDatePlan::standard_following(
    ///     "2009-03-19".parse()?, "2009-06-20".parse()?,
    ///     ScheduleCalendar::Plain(Calendar::weekends_only()),
    ///     CdsMaturityConvention::Quarterly2009, 1)?;
    /// let coupon = &plan.coupon_dates()[0];
    /// assert_eq!(coupon.payment_date().to_string(), "2009-06-22");
    /// assert_eq!(coupon.accrual_day_count(), 93);
    /// assert_eq!(coupon.accrual_end_inclusion(), EndpointInclusion::Include);
    /// # Ok::<(), sabiquant::error::Error>(())
    /// ```
    pub fn standard_following(
        trade_date: Date,
        unadjusted_maturity: Date,
        calendar: ScheduleCalendar,
        convention: CdsMaturityConvention,
        max_coupons: u32,
    ) -> Result<Self, ScheduleError> {
        if !(Self::MIN_COUPONS..=Self::MAX_COUPONS).contains(&max_coupons) {
            return Err(ScheduleError::InvalidCdsCouponLimit {
                requested: max_coupons,
                minimum: Self::MIN_COUPONS,
                maximum: Self::MAX_COUPONS,
            });
        }
        if trade_date >= unadjusted_maturity {
            return Err(ScheduleError::InvalidRange {
                start: trade_date,
                end: unadjusted_maturity,
            });
        }
        if unadjusted_maturity.day() != 20 || !unadjusted_maturity.month().is_multiple_of(3) {
            return Err(ScheduleError::InvalidCdsMaturity {
                maturity: unadjusted_maturity,
                convention,
            });
        }
        let context = Context {
            trade: trade_date,
            maturity: unadjusted_maturity,
            calendar: &calendar,
        };
        context.guard(trade_date, unadjusted_maturity, "original coverage")?;
        let cutoff = trade_date
            .checked_add_days(1)
            .map_err(|cause| context.date_error("payment cutoff", cause))?;
        let cutoff_month = month_coordinate(cutoff) - i64::from(cutoff.day() < 20);
        let quarter_month = cutoff_month - (cutoff_month - 2).rem_euclid(3);
        let quarter = context.quarter(quarter_month, "cutoff quarter")?;
        context.guard(
            trade_date.min(quarter),
            unadjusted_maturity,
            "cutoff quarter coverage",
        )?;
        let quarter_payment = context.adjust(quarter, None, "cutoff quarter adjustment")?;
        let first_month = if quarter_payment <= cutoff {
            if quarter == unadjusted_maturity {
                return Err(ScheduleError::NoEligibleCdsPayment {
                    trade_date,
                    payment_selection_cutoff: cutoff,
                    unadjusted_maturity,
                    final_payment: quarter_payment,
                });
            }
            quarter_month + 3
        } else {
            quarter_month
        };
        let required = bounded_count((month_coordinate(unadjusted_maturity) - first_month) / 3 + 1);
        if required > max_coupons {
            return Err(ScheduleError::CdsCouponLimitExceeded {
                maximum: max_coupons,
                required,
            });
        }
        let mut previous_raw = if first_month > quarter_month {
            quarter
        } else {
            context.quarter(first_month - 3, "initial accrual precursor")?
        };
        let span_start = trade_date.min(previous_raw);
        context.guard(span_start, unadjusted_maturity, "precursor coverage")?;
        let mut previous_adjusted = if previous_raw == quarter {
            quarter_payment
        } else {
            context.adjust(previous_raw, None, "precursor adjustment")?
        };
        let first_raw = if first_month == quarter_month {
            quarter
        } else {
            context.quarter(first_month, "coupon quarter")?
        };
        let first_payment = if first_raw == quarter {
            quarter_payment
        } else {
            context.adjust(first_raw, Some(0), "coupon adjustment")?
        };
        // Establish the initial pair before reserving the result. A rejected
        // initial collision must not depend on allocating all future records.
        check_order(0, previous_raw, first_raw, previous_adjusted, first_payment)?;
        let count = required as usize;
        let mut coupons = Vec::new();
        coupons
            .try_reserve_exact(count)
            .map_err(|_| ScheduleError::AllocationFailed {
                phase: "CDS coupon records",
                requested: count,
            })?;
        for position in 0..required {
            let coordinate = first_month + 3 * i64::from(position);
            let raw = if position == 0 {
                first_raw
            } else {
                context.quarter(coordinate, "coupon quarter")?
            };
            let adjusted = if position == 0 {
                first_payment
            } else {
                context.adjust(raw, Some(position as usize), "coupon adjustment")?
            };
            if position > 0 {
                check_order(
                    position as usize,
                    previous_raw,
                    raw,
                    previous_adjusted,
                    adjusted,
                )?;
            }
            // Following monotonicity plus strict order proves that the initial
            // precursor is <= cutoff and that the final start is < raw maturity.
            // A closure violating either relation would have collapsed the pair.
            let final_coupon = raw == unadjusted_maturity;
            let accrual_end = if final_coupon { raw } else { adjusted };
            let day_count =
                i64::from(accrual_end.days_since(previous_adjusted)) + i64::from(final_coupon);
            coupons.push(CdsCouponDates {
                unadjusted_start: previous_raw,
                unadjusted_end: raw,
                payment_date: adjusted,
                accrual_start: previous_adjusted,
                accrual_end,
                accrual_end_inclusion: if final_coupon {
                    EndpointInclusion::Include
                } else {
                    EndpointInclusion::Exclude
                },
                accrual_day_count: bounded_count(day_count),
            });
            previous_raw = raw;
            previous_adjusted = adjusted;
        }
        context.guard(span_start, previous_adjusted, "final coverage")?;
        Ok(Self {
            trade_date,
            payment_selection_cutoff: cutoff,
            unadjusted_maturity,
            calendar,
            convention,
            coupons,
        })
    }

    /// Returns the supplied trade date, without an inferred effective date.
    #[must_use]
    pub const fn trade_date(&self) -> Date {
        self.trade_date
    }

    /// Returns unadjusted trade plus one calendar day, used only for selection.
    #[must_use]
    pub const fn payment_selection_cutoff(&self) -> Date {
        self.payment_selection_cutoff
    }

    /// Returns the supplied quarterly-twentieth maturity.
    #[must_use]
    pub const fn unadjusted_maturity(&self) -> Date {
        self.unadjusted_maturity
    }

    /// Borrows the explicit calendar definition and any market coverage guard.
    #[must_use]
    pub const fn calendar(&self) -> &ScheduleCalendar {
        &self.calendar
    }

    /// Returns the explicitly retained modern maturity regime.
    #[must_use]
    pub const fn maturity_convention(&self) -> CdsMaturityConvention {
        self.convention
    }

    /// Returns CDS or CDS2015 according to the explicit retained regime.
    #[must_use]
    pub const fn rule(&self) -> DateGenerationRule {
        match self.convention {
            CdsMaturityConvention::Quarterly2009 => DateGenerationRule::Cds,
            CdsMaturityConvention::Semiannual2015 => DateGenerationRule::Cds2015,
        }
    }

    /// Borrows nonempty chronological coupon date records.
    #[must_use]
    pub fn coupon_dates(&self) -> &[CdsCouponDates] {
        &self.coupons
    }
}

struct Context<'a> {
    trade: Date,
    maturity: Date,
    calendar: &'a ScheduleCalendar,
}

impl Context<'_> {
    fn date_error(&self, role: &'static str, cause: DateError) -> ScheduleError {
        ScheduleError::CdsDate {
            trade_date: self.trade,
            unadjusted_maturity: self.maturity,
            role,
            input: self.trade,
            cause,
        }
    }

    fn quarter(&self, coordinate: i64, role: &'static str) -> Result<Date, ScheduleError> {
        let month = self
            .trade
            .checked_add_months(coordinate - month_coordinate(self.trade))
            .map_err(|cause| self.date_error(role, cause))?;
        Date::from_ymd(month.year(), month.month(), 20)
            .map_err(|cause| self.date_error(role, cause))
    }

    fn guard(&self, start: Date, end: Date, phase: &'static str) -> Result<(), ScheduleError> {
        if let ScheduleCalendar::Target(calendar) = self.calendar {
            calendar
                .business_days_between(
                    start,
                    end,
                    EndpointInclusion::Include,
                    EndpointInclusion::Include,
                )
                .map_err(|cause| ScheduleError::CdsCalendar {
                    trade_date: self.trade,
                    unadjusted_maturity: self.maturity,
                    phase,
                    index: None,
                    start,
                    end,
                    cause,
                })?;
        }
        Ok(())
    }

    fn adjust(
        &self,
        raw: Date,
        index: Option<usize>,
        phase: &'static str,
    ) -> Result<Date, ScheduleError> {
        let result = match self.calendar {
            ScheduleCalendar::Plain(calendar) => {
                calendar.adjust(raw, BusinessDayConvention::Following)
            }
            ScheduleCalendar::Target(calendar) => {
                calendar.adjust(raw, BusinessDayConvention::Following)
            }
        };
        result.map_err(|cause| ScheduleError::CdsCalendar {
            trade_date: self.trade,
            unadjusted_maturity: self.maturity,
            phase,
            index,
            start: raw,
            end: raw,
            cause,
        })
    }
}

fn month_coordinate(date: Date) -> i64 {
    i64::from(date.year() - 1) * 12 + i64::from(date.month() - 1)
}

fn check_order(
    index: usize,
    previous_contractual: Date,
    contractual: Date,
    previous_adjusted: Date,
    adjusted: Date,
) -> Result<(), ScheduleError> {
    if previous_adjusted >= adjusted {
        return Err(ScheduleError::CdsAdjustedOrder {
            index,
            previous_contractual,
            contractual,
            previous_adjusted,
            adjusted,
        });
    }
    Ok(())
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "positive coupon count <= 39996 or actual accrual days <= civil range + 1"
)]
fn bounded_count(value: i64) -> u32 {
    value as u32
}
