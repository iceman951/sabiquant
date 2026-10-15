//! Explicit quoted-maturity conventions for modern single-name CDS.

use super::ScheduleError;
use crate::error::IdentifierError;
use crate::identifier::{Entry, Identifiers};
use crate::time::{Date, Period, TimeUnit};
use std::fmt;
use std::str::FromStr;

/// The explicitly selected quoted-maturity convention for single-name CDS.
///
/// These analytical definitions produce unadjusted maturities. Their names
/// neither select a regime from the trade date nor define protection,
/// settlement, a payment calendar or premium accrual.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "closed two-regime quoted CDS maturity contract"
)]
pub enum CdsMaturityConvention {
    /// Add the quoted tenor to the first quarter-twentieth strictly after trade.
    Quarterly2009,
    /// Add the tenor to three months after the last March/September twentieth.
    ///
    /// The March/September anchor is on or before trade. A zero quote whose
    /// resulting June/December twentieth is on or before trade is matured.
    Semiannual2015,
}

impl CdsMaturityConvention {
    /// Every supported convention in canonical order.
    pub const ALL: [Self; 2] = [Self::Quarterly2009, Self::Semiannual2015];

    /// Returns the exact canonical identifier, with no aliases or trimming.
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::Quarterly2009 => "Quarterly2009",
            Self::Semiannual2015 => "Semiannual2015",
        }
    }
}

const CONVENTION_IDS: Identifiers<CdsMaturityConvention> = Identifiers::new(
    "CDS maturity convention",
    &[
        Entry {
            value: CdsMaturityConvention::Quarterly2009,
            canonical: "Quarterly2009",
            aliases: &[],
        },
        Entry {
            value: CdsMaturityConvention::Semiannual2015,
            canonical: "Semiannual2015",
            aliases: &[],
        },
    ],
    &[],
);

impl fmt::Display for CdsMaturityConvention {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_identifier())
    }
}

impl FromStr for CdsMaturityConvention {
    type Err = IdentifierError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        CONVENTION_IDS.parse(text)
    }
}

/// An unadjusted maturity or the already-matured zero-month quoted contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "closed successful quoted-maturity outcome contract"
)]
pub enum CdsMaturityOutcome {
    /// The resolved unadjusted maturity, strictly after the trade date.
    Date(Date),
    /// The selected zero-month contract has matured on or before trade.
    ///
    /// This has no sentinel or historical anchor Date: its abstract base can
    /// precede the supported civil range even for a valid trade date.
    Matured,
}

/// Immutable maturity outcome and the complete caller-supplied definition.
///
/// Equality and hashing retain the stored quote length/unit. Thus 1Y and
/// 12M, or 0Y and 0M, remain distinct definitions even when their outcomes
/// coincide and the corresponding [`Period`] values compare equal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CdsMaturityResolution {
    trade_date: Date,
    quoted_length: i32,
    quoted_unit: TimeUnit,
    convention: CdsMaturityConvention,
    outcome: CdsMaturityOutcome,
}

impl CdsMaturityResolution {
    /// Returns the original explicit trade date.
    #[must_use]
    pub const fn trade_date(&self) -> Date {
        self.trade_date
    }

    /// Returns the original stored quoted contract tenor without normalization.
    #[must_use]
    pub const fn quoted_tenor(&self) -> Period {
        Period::new(self.quoted_length, self.quoted_unit)
    }

    /// Returns the caller-selected maturity convention.
    #[must_use]
    pub const fn convention(&self) -> CdsMaturityConvention {
        self.convention
    }

    /// Returns the unadjusted maturity or typed matured outcome.
    #[must_use]
    pub const fn outcome(&self) -> CdsMaturityOutcome {
        self.outcome
    }
}

/// Resolves a quoted contract tenor under an explicit CDS maturity convention.
///
/// Accepted quotes are nonnegative stored Months divisible by three and
/// nonnegative stored Years. Zero remains a contract quote, not a zero-period
/// schedule. Calendars and business-day adjustments do not apply.
///
/// The conventions are local analytical definitions for modern single-name
/// corporate/sovereign CDS. Extending their arithmetic over the supported
/// proleptic civil range does not assert historical market applicability.
///
/// # Errors
///
/// Returns [`ScheduleError::InvalidCdsTenor`] for a rejected stored tenor,
/// before maturity or range resolution. Returns [`ScheduleError::CdsMaturityDate`]
/// if the required final maturity is outside the supported civil range.
/// Its date cause retains the trade date and the displacement to that final
/// month; an unused or abstract out-of-range anchor never causes a failure.
pub fn resolve_cds_maturity(
    trade_date: Date,
    quoted_tenor: Period,
    convention: CdsMaturityConvention,
) -> Result<CdsMaturityResolution, ScheduleError> {
    let quoted_length = quoted_tenor.length();
    let quoted_unit = quoted_tenor.unit();
    let quoted_months = match (quoted_length, quoted_unit) {
        (length, TimeUnit::Months) if length >= 0 && length % 3 == 0 => i64::from(length),
        (length, TimeUnit::Years) if length >= 0 => i64::from(length) * 12,
        _ => {
            return Err(ScheduleError::InvalidCdsTenor {
                trade_date,
                tenor: quoted_tenor,
                convention,
            });
        }
    };

    let (year, month, day) = trade_date.to_ymd();
    let trade_month = i64::from(year - 1) * 12 + i64::from(month - 1);
    let cutoff = trade_month - i64::from(day < 20);
    let step = match convention {
        CdsMaturityConvention::Quarterly2009 => 3,
        CdsMaturityConvention::Semiannual2015 => 6,
    };
    // March is coordinate 2. The preceding eligible twentieth is inclusive;
    // adding three months yields either the next quarter or semiannual base.
    // Euclidean remainder also handles abstract year-zero anchors at MIN.
    let base = cutoff - (cutoff - 2).rem_euclid(step) + 3;
    let outcome = if quoted_months == 0 && (base, 20) <= (trade_month, day) {
        CdsMaturityOutcome::Matured
    } else {
        // All operands are bounded by the civil range and 12 * i32::MAX.
        // Construct only the required target month, then its twentieth.
        let target = base + quoted_months;
        let resolved = trade_date
            .checked_add_months(target - trade_month)
            .and_then(|date| {
                let (target_year, target_month, _) = date.to_ymd();
                Date::from_ymd(target_year, target_month, 20)
            })
            .map_err(|cause| ScheduleError::CdsMaturityDate {
                trade_date,
                tenor: quoted_tenor,
                convention,
                cause,
            })?;
        CdsMaturityOutcome::Date(resolved)
    };

    Ok(CdsMaturityResolution {
        trade_date,
        quoted_length,
        quoted_unit,
        convention,
        outcome,
    })
}
