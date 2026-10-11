//! Signed calendar periods and named periodic frequencies (FS-PER).
//!
//! A period stores its original length and unit. Equality and hashing use
//! equivalent numbers of days or months: `14D = 2W` and `12M = 1Y`.
//! Normalization chooses weeks or years when division is exact, and every
//! zero normalizes to `0D`. Months and years have no fixed duration in days.
//!
//! Ordering compares day/week periods with day/week periods, and nonzero
//! month/year periods with nonzero month/year periods. Every zero belongs to
//! the **day family for ordering**, regardless of its stored unit. Thus zero
//! compares with day/week periods, but is incomparable with a nonzero
//! month/year period. This canonical-zero rule makes ordering agree with
//! normalized equality and remain transitive.
//!
//! Arithmetic is checked. Mixed compatible units combine as days or months;
//! matching units preserve their unit. Zero acts as the additive identity
//! across both families. No conversion between nonzero day/week and
//! month/year periods is assumed.
//!
//! A [`TimeUnit::Days`] period counts calendar days in plain date arithmetic,
//! and business days in calendar advancement. The operation determines its
//! meaning; a period alone never embeds a calendar.
//!
//! # QuantLib correspondence
//!
//! The named frequency counts and normalization rules follow the specification
//! in [FS-PER](https://github.com/iceman951/sabiquant/blob/main/docs/REQUIREMENTS.md#6-fs-per--periods-and-frequencies).
//! QuantLib attempts some cross-family comparisons using possible month
//! lengths. SabiQuant intentionally provides the narrower ordering above and
//! checked arithmetic instead of panicking operators. `OtherFrequency`,
//! intraday units and composite period text are outside this module's scope.

use crate::error::IdentifierError;
use crate::identifier::{Entry, Identifiers};
use std::cmp::Ordering;
use std::error::Error as StdError;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

/// The calendar units used to express a [`Period`].
///
/// Weeks are exactly seven days; years are exactly twelve months. No fixed
/// conversion is made between days and months.
///
/// The canonical identifiers are `Days`, `Weeks`, `Months`, and `Years`.
/// Parsing also accepts the respective aliases `D`, `W`, `M`, and `Y`.
/// Matching is exact, case-sensitive, and does not trim whitespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "FS-PER defines a closed set of four calendar time units"
)]
pub enum TimeUnit {
    /// Days; calendar or business days according to the operation.
    Days,
    /// Weeks, each comprising exactly seven calendar days.
    Weeks,
    /// Calendar months, whose number of days depends on the date.
    Months,
    /// Calendar years, each comprising exactly twelve months.
    Years,
}

impl TimeUnit {
    /// Every time unit, in order from days to years.
    pub const ALL: [Self; 4] = [Self::Days, Self::Weeks, Self::Months, Self::Years];

    /// Returns the canonical identifier used by [`Display`](fmt::Display).
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::Days => "Days",
            Self::Weeks => "Weeks",
            Self::Months => "Months",
            Self::Years => "Years",
        }
    }

    const fn letter(self) -> char {
        match self {
            Self::Days => 'D',
            Self::Weeks => 'W',
            Self::Months => 'M',
            Self::Years => 'Y',
        }
    }

    const fn family(self) -> Family {
        match self {
            Self::Days | Self::Weeks => Family::Days,
            Self::Months | Self::Years => Family::Months,
        }
    }

    const fn scale(self) -> i64 {
        match self {
            Self::Days | Self::Months => 1,
            Self::Weeks => 7,
            Self::Years => 12,
        }
    }
}

const UNIT_IDENTIFIERS: Identifiers<TimeUnit> = Identifiers::new(
    "time unit",
    &[
        Entry {
            value: TimeUnit::Days,
            canonical: "Days",
            aliases: &["D"],
        },
        Entry {
            value: TimeUnit::Weeks,
            canonical: "Weeks",
            aliases: &["W"],
        },
        Entry {
            value: TimeUnit::Months,
            canonical: "Months",
            aliases: &["M"],
        },
        Entry {
            value: TimeUnit::Years,
            canonical: "Years",
            aliases: &["Y"],
        },
    ],
    &[],
);

impl fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_identifier())
    }
}

impl FromStr for TimeUnit {
    type Err = IdentifierError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        UNIT_IDENTIFIERS.parse(text)
    }
}

/// The named periodic frequencies, plus a single event ([`Once`](Self::Once)).
///
/// Counts and equivalent periods are listed in
/// [FS-PER](https://github.com/iceman951/sabiquant/blob/main/docs/REQUIREMENTS.md#6-fs-per--periods-and-frequencies).
/// A frequency count is a convention: `Weekly` has 52 events per year, while
/// `Daily` has 365, even though calendar years vary in length.
///
/// [`Display`](fmt::Display) and parsing use the exact variant names, such as
/// `Semiannual` and `EveryFourthWeek`. No aliases are accepted, and parsing is
/// case-sensitive with no trimming. Absence of a frequency is represented by
/// [`Option`], never by a sentinel variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "FS-PER defines this closed set of named frequencies"
)]
pub enum Frequency {
    /// One regular event per year; equivalent to `1Y`.
    Annual,
    /// Two regular events per year; equivalent to `6M`.
    Semiannual,
    /// Three regular events per year; equivalent to `4M`.
    EveryFourthMonth,
    /// Four regular events per year; equivalent to `3M`.
    Quarterly,
    /// Six regular events per year; equivalent to `2M`.
    Bimonthly,
    /// Twelve regular events per year; equivalent to `1M`.
    Monthly,
    /// Thirteen regular events per year; equivalent to `4W`.
    EveryFourthWeek,
    /// Twenty-six regular events per year; equivalent to `2W`.
    Biweekly,
    /// Fifty-two regular events per year; equivalent to `1W`.
    Weekly,
    /// Three hundred sixty-five regular events per year; equivalent to `1D`.
    Daily,
    /// A single event, with no equivalent recurring period or annual count.
    Once,
}

impl Frequency {
    /// Every named frequency, in the order of the FS-PER frequency table.
    pub const ALL: [Self; 11] = [
        Self::Annual,
        Self::Semiannual,
        Self::EveryFourthMonth,
        Self::Quarterly,
        Self::Bimonthly,
        Self::Monthly,
        Self::EveryFourthWeek,
        Self::Biweekly,
        Self::Weekly,
        Self::Daily,
        Self::Once,
    ];

    /// Returns the canonical identifier used by [`Display`](fmt::Display).
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::Annual => "Annual",
            Self::Semiannual => "Semiannual",
            Self::EveryFourthMonth => "EveryFourthMonth",
            Self::Quarterly => "Quarterly",
            Self::Bimonthly => "Bimonthly",
            Self::Monthly => "Monthly",
            Self::EveryFourthWeek => "EveryFourthWeek",
            Self::Biweekly => "Biweekly",
            Self::Weekly => "Weekly",
            Self::Daily => "Daily",
            Self::Once => "Once",
        }
    }

    /// Returns the conventional count of regular events per year.
    ///
    /// [`Once`](Self::Once) is a single event and returns `None`, rather than
    /// an annual recurrence count.
    #[must_use]
    pub const fn events_per_year(self) -> Option<u32> {
        match self {
            Self::Annual => Some(1),
            Self::Semiannual => Some(2),
            Self::EveryFourthMonth => Some(3),
            Self::Quarterly => Some(4),
            Self::Bimonthly => Some(6),
            Self::Monthly => Some(12),
            Self::EveryFourthWeek => Some(13),
            Self::Biweekly => Some(26),
            Self::Weekly => Some(52),
            Self::Daily => Some(365),
            Self::Once => None,
        }
    }

    /// Returns the equivalent recurring period, or `None` for
    /// [`Once`](Self::Once).
    ///
    /// Every periodic frequency round-trips through [`Period::frequency`].
    #[must_use]
    pub const fn period(self) -> Option<Period> {
        let (length, unit) = match self {
            Self::Annual => (1, TimeUnit::Years),
            Self::Semiannual => (6, TimeUnit::Months),
            Self::EveryFourthMonth => (4, TimeUnit::Months),
            Self::Quarterly => (3, TimeUnit::Months),
            Self::Bimonthly => (2, TimeUnit::Months),
            Self::Monthly => (1, TimeUnit::Months),
            Self::EveryFourthWeek => (4, TimeUnit::Weeks),
            Self::Biweekly => (2, TimeUnit::Weeks),
            Self::Weekly => (1, TimeUnit::Weeks),
            Self::Daily => (1, TimeUnit::Days),
            Self::Once => return None,
        };
        Some(Period::new(length, unit))
    }
}

const FREQUENCY_IDENTIFIERS: Identifiers<Frequency> = Identifiers::new(
    "frequency",
    &[
        Entry {
            value: Frequency::Annual,
            canonical: "Annual",
            aliases: &[],
        },
        Entry {
            value: Frequency::Semiannual,
            canonical: "Semiannual",
            aliases: &[],
        },
        Entry {
            value: Frequency::EveryFourthMonth,
            canonical: "EveryFourthMonth",
            aliases: &[],
        },
        Entry {
            value: Frequency::Quarterly,
            canonical: "Quarterly",
            aliases: &[],
        },
        Entry {
            value: Frequency::Bimonthly,
            canonical: "Bimonthly",
            aliases: &[],
        },
        Entry {
            value: Frequency::Monthly,
            canonical: "Monthly",
            aliases: &[],
        },
        Entry {
            value: Frequency::EveryFourthWeek,
            canonical: "EveryFourthWeek",
            aliases: &[],
        },
        Entry {
            value: Frequency::Biweekly,
            canonical: "Biweekly",
            aliases: &[],
        },
        Entry {
            value: Frequency::Weekly,
            canonical: "Weekly",
            aliases: &[],
        },
        Entry {
            value: Frequency::Daily,
            canonical: "Daily",
            aliases: &[],
        },
        Entry {
            value: Frequency::Once,
            canonical: "Once",
            aliases: &[],
        },
    ],
    &[],
);

impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_identifier())
    }
}

impl FromStr for Frequency {
    type Err = IdentifierError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        FREQUENCY_IDENTIFIERS.parse(text)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Family {
    Days,
    Months,
}

/// A signed integer length in calendar [`TimeUnit`]s.
///
/// Every `i32` length is valid, including zero and negative lengths. The
/// stored representation is preserved by accessors and formatting. Use
/// [`normalized`](Self::normalized) to prefer years over exact multiples of
/// twelve months, weeks over exact multiples of seven days, and `0D` for all
/// zeros. Equality and hashing already account for these equivalences.
///
/// Only periods within the same day/week or month/year family are ordered.
/// For ordering, all zeros have the canonical day family, so they are
/// incomparable with nonzero month/year periods. See the module documentation
/// for the reason for this rule.
///
/// # Text format
///
/// Formatting writes an unpadded signed decimal length followed by `D`, `W`,
/// `M`, or `Y`. A positive length has no `+` sign. Parsing accepts an optional
/// single ASCII `+` or `-` sign, one or more ASCII decimal digits, and one
/// upper- or lowercase unit letter. Leading zeros are accepted; whitespace,
/// composite periods (`1Y6M`), and non-ASCII digits are rejected. The length
/// must fit `i32`.
///
/// ```
/// use sabiquant::time::{Period, TimeUnit};
///
/// let year = Period::new(1, TimeUnit::Years);
/// let months: Period = "+0012m".parse()?;
/// assert_eq!(year, months);
/// assert_eq!(months.to_string(), "12M");
/// assert_eq!(months.normalized().to_string(), "1Y");
/// # Ok::<(), sabiquant::time::PeriodError>(())
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Period {
    length: i32,
    unit: TimeUnit,
}

impl Period {
    /// Constructs a period, preserving its length and unit.
    ///
    /// Every `i32` length and [`TimeUnit`] is valid; construction cannot fail.
    #[must_use]
    pub const fn new(length: i32, unit: TimeUnit) -> Self {
        Self { length, unit }
    }

    /// Returns the stored signed length, measured in [`unit`](Self::unit).
    #[must_use]
    pub const fn length(self) -> i32 {
        self.length
    }

    /// Returns the stored time unit.
    #[must_use]
    pub const fn unit(self) -> TimeUnit {
        self.unit
    }

    /// Returns an equivalent canonical representation.
    ///
    /// Exact multiples of seven days become weeks; exact multiples of twelve
    /// months become years. Every zero becomes `0D`. Other representations are
    /// unchanged. This operation is idempotent and cannot overflow.
    #[must_use]
    pub const fn normalized(self) -> Self {
        if self.length == 0 {
            return Self::new(0, TimeUnit::Days);
        }
        match self.unit {
            TimeUnit::Days if self.length % 7 == 0 => Self::new(self.length / 7, TimeUnit::Weeks),
            TimeUnit::Months if self.length % 12 == 0 => {
                Self::new(self.length / 12, TimeUnit::Years)
            }
            _ => self,
        }
    }

    /// Negates the stored length, preserving its unit.
    ///
    /// # Errors
    ///
    /// [`PeriodError::Overflow`] when the length is `i32::MIN`, whose
    /// positive counterpart cannot be represented.
    pub fn checked_neg(self) -> Result<Self, PeriodError> {
        Self::from_wide(-i64::from(self.length), self.unit, "negate")
    }

    /// Multiplies the stored length by `multiplier`, preserving its unit.
    ///
    /// # Errors
    ///
    /// [`PeriodError::Overflow`] if the resulting length cannot fit `i32`.
    /// Multiplication by zero always succeeds and yields a zero period.
    pub fn checked_mul(self, multiplier: i32) -> Result<Self, PeriodError> {
        Self::from_wide(
            i64::from(self.length) * i64::from(multiplier),
            self.unit,
            "multiply",
        )
    }

    /// Adds `other` using exact compatible-unit arithmetic.
    ///
    /// Matching units preserve their unit. Mixed day/week periods combine as
    /// days; mixed month/year periods combine as months. A zero operand acts
    /// as the identity, regardless of its stored unit. Intermediate
    /// calculations are widened, so cancellation can produce a valid result
    /// even when one converted operand exceeds `i32`.
    ///
    /// # Errors
    ///
    /// - [`PeriodError::IncompatibleUnits`] for nonzero operands in different
    ///   unit families;
    /// - [`PeriodError::Overflow`] if the result in the chosen unit does not
    ///   fit `i32`.
    pub fn checked_add(self, other: Self) -> Result<Self, PeriodError> {
        self.combine(other, false)
    }

    /// Subtracts `other` using the same unit rules as [`checked_add`](Self::checked_add).
    ///
    /// Subtraction computes the widened difference directly. It does not
    /// first negate `other`, so subtracting `i32::MIN` can succeed when the
    /// final difference fits, for example `MIN D - MIN D = 0D`.
    ///
    /// # Errors
    ///
    /// - [`PeriodError::IncompatibleUnits`] for nonzero operands in different
    ///   unit families;
    /// - [`PeriodError::Overflow`] if the result in the chosen unit does not
    ///   fit `i32`.
    pub fn checked_sub(self, other: Self) -> Result<Self, PeriodError> {
        self.combine(other, true)
    }

    /// Returns a named frequency exactly equivalent to this period.
    ///
    /// Equivalence is normalized: `12M` maps to [`Frequency::Annual`], and
    /// `7D` maps to [`Frequency::Weekly`]. Unlisted periods, all zeros, and
    /// negative periods return `None`. [`Frequency::Once`] has no period and
    /// is never returned.
    #[must_use]
    pub fn frequency(self) -> Option<Frequency> {
        Frequency::ALL
            .into_iter()
            .find(|frequency| frequency.period().is_some_and(|period| self == period))
    }

    fn from_wide(
        length: i64,
        unit: TimeUnit,
        operation: &'static str,
    ) -> Result<Self, PeriodError> {
        match i32::try_from(length) {
            Ok(length) => Ok(Self::new(length, unit)),
            Err(_) => Err(PeriodError::Overflow {
                operation,
                length,
                unit,
            }),
        }
    }

    fn combine(self, other: Self, subtract: bool) -> Result<Self, PeriodError> {
        let operation = if subtract { "subtract" } else { "add" };
        if other.length == 0 {
            return Ok(self);
        }
        if self.length == 0 {
            let length = i64::from(other.length);
            return Self::from_wide(
                if subtract { -length } else { length },
                other.unit,
                operation,
            );
        }
        if self.unit == other.unit {
            let left = i64::from(self.length);
            let right = i64::from(other.length);
            return Self::from_wide(
                if subtract { left - right } else { left + right },
                self.unit,
                operation,
            );
        }
        if self.unit.family() != other.unit.family() {
            return Err(PeriodError::IncompatibleUnits {
                left: self.unit,
                right: other.unit,
            });
        }
        let left = i64::from(self.length) * self.unit.scale();
        let right = i64::from(other.length) * other.unit.scale();
        let unit = match self.unit.family() {
            Family::Days => TimeUnit::Days,
            Family::Months => TimeUnit::Months,
        };
        Self::from_wide(
            if subtract { left - right } else { left + right },
            unit,
            operation,
        )
    }

    // Widen before scaling: the full i32 range in years or weeks is valid,
    // even though its equivalent number of months or days may not fit i32.
    // Zero receives one canonical family for Eq, Hash and PartialOrd alike.
    fn canonical_quantity(self) -> (Family, i64) {
        if self.length == 0 {
            (Family::Days, 0)
        } else {
            (
                self.unit.family(),
                i64::from(self.length) * self.unit.scale(),
            )
        }
    }
}

impl PartialEq for Period {
    fn eq(&self, other: &Self) -> bool {
        self.canonical_quantity() == other.canonical_quantity()
    }
}

impl Eq for Period {}

impl Hash for Period {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.canonical_quantity().hash(state);
    }
}

impl PartialOrd for Period {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let (left_family, left) = self.canonical_quantity();
        let (right_family, right) = other.canonical_quantity();
        if left_family == right_family {
            Some(left.cmp(&right))
        } else {
            None
        }
    }
}

impl fmt::Display for Period {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.length, self.unit.letter())
    }
}

impl FromStr for Period {
    type Err = PeriodError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let bytes = text.as_bytes();
        let Some((&letter, length)) = bytes.split_last() else {
            return Err(PeriodError::Parse {
                text: text.to_owned(),
            });
        };
        let unit = match letter {
            b'D' | b'd' => TimeUnit::Days,
            b'W' | b'w' => TimeUnit::Weeks,
            b'M' | b'm' => TimeUnit::Months,
            b'Y' | b'y' => TimeUnit::Years,
            _ => {
                return Err(PeriodError::Parse {
                    text: text.to_owned(),
                });
            }
        };
        let digits = match length.first() {
            Some(b'+' | b'-') => &length[1..],
            _ => length,
        };
        if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
            return Err(PeriodError::Parse {
                text: text.to_owned(),
            });
        }
        // ASCII validation above guarantees the final one-byte letter starts
        // at a UTF-8 boundary; no slice is formed before this validation.
        match text[..text.len() - 1].parse::<i32>() {
            Ok(length) => Ok(Self::new(length, unit)),
            Err(_) => Err(PeriodError::Parse {
                text: text.to_owned(),
            }),
        }
    }
}

/// Failure to parse a period or complete exact checked period arithmetic.
///
/// Error kind identifiers are stable; the wording of display messages may
/// change. Overflow reports the exact widened result and its intended unit,
/// rather than wrapping or saturating the result.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PeriodError {
    /// The result's length does not fit `i32` in its intended unit.
    ///
    /// Kind name: `period.overflow`.
    #[non_exhaustive]
    Overflow {
        /// The failed operation: `negate`, `multiply`, `add`, or `subtract`.
        operation: &'static str,
        /// The exact widened result before conversion to `i32`.
        length: i64,
        /// The unit in which the resulting length was to be represented.
        unit: TimeUnit,
    },
    /// Nonzero operands belong to different unit families.
    ///
    /// Kind name: `period.incompatible_units`.
    #[non_exhaustive]
    IncompatibleUnits {
        /// The left operand's stored unit.
        left: TimeUnit,
        /// The right operand's stored unit.
        right: TimeUnit,
    },
    /// Text does not follow the documented period grammar, or its length
    /// does not fit `i32`.
    ///
    /// Kind name: `period.parse`.
    #[non_exhaustive]
    Parse {
        /// The rejected text, preserved without trimming or normalization.
        text: String,
    },
}

impl PeriodError {
    /// Returns the stable kind name: `period.overflow`,
    /// `period.incompatible_units`, or `period.parse`.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Overflow { .. } => "period.overflow",
            Self::IncompatibleUnits { .. } => "period.incompatible_units",
            Self::Parse { .. } => "period.parse",
        }
    }
}

impl fmt::Display for PeriodError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow {
                operation,
                length,
                unit,
            } => write!(
                f,
                "period {operation} result {length} {unit} does not fit an i32 length"
            ),
            Self::IncompatibleUnits { left, right } => {
                write!(f, "cannot combine nonzero periods in {left} and {right}")
            }
            Self::Parse { text } => write!(
                f,
                "invalid period {text:?}; expected a signed i32 length and D, W, M, or Y"
            ),
        }
    }
}

impl StdError for PeriodError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::prng;
    use std::collections::{HashSet, hash_map::DefaultHasher};

    fn hash(period: Period) -> u64 {
        let mut hasher = DefaultHasher::new();
        period.hash(&mut hasher);
        hasher.finish()
    }

    fn p(length: i32, unit: TimeUnit) -> Period {
        Period::new(length, unit)
    }

    // An independent i128 formula model enumerates the four mixed-unit cases
    // explicitly. It does not call the production scaling/family helpers.
    fn reference_binary(
        left: Period,
        right: Period,
        subtract: bool,
    ) -> Result<Period, PeriodError> {
        use TimeUnit::*;
        let sign: i128 = if subtract { -1 } else { 1 };
        let a = i128::from(left.length());
        let b = i128::from(right.length());
        let (length, unit) = if b == 0 {
            (a, left.unit())
        } else if a == 0 {
            (sign * b, right.unit())
        } else {
            match (left.unit(), right.unit()) {
                (Days, Days) | (Weeks, Weeks) | (Months, Months) | (Years, Years) => {
                    (a + sign * b, left.unit())
                }
                (Days, Weeks) => (a + sign * b * 7, Days),
                (Weeks, Days) => (a * 7 + sign * b, Days),
                (Months, Years) => (a + sign * b * 12, Months),
                (Years, Months) => (a * 12 + sign * b, Months),
                _ => {
                    return Err(PeriodError::IncompatibleUnits {
                        left: left.unit(),
                        right: right.unit(),
                    });
                }
            }
        };
        match i32::try_from(length) {
            Ok(length) => Ok(p(length, unit)),
            Err(_) => Err(PeriodError::Overflow {
                operation: if subtract { "subtract" } else { "add" },
                length: i64::try_from(length).expect("sum of two scaled i32 lengths fits i64"),
                unit,
            }),
        }
    }

    fn assert_same_stored_result(
        actual: Result<Period, PeriodError>,
        expected: Result<Period, PeriodError>,
    ) {
        match (actual, expected) {
            (Ok(actual), Ok(expected)) => {
                assert_eq!(actual.length(), expected.length());
                assert_eq!(actual.unit(), expected.unit());
            }
            (actual, expected) => assert_eq!(actual, expected),
        }
    }

    fn reference_quantity(period: Period) -> (bool, i128) {
        if period.length() == 0 {
            return (false, 0);
        }
        let length = i128::from(period.length());
        match period.unit() {
            TimeUnit::Days => (false, length),
            TimeUnit::Weeks => (false, length * 7),
            TimeUnit::Months => (true, length),
            TimeUnit::Years => (true, length * 12),
        }
    }

    const EDGE_LENGTHS: [i32; 21] = [
        i32::MIN,
        i32::MIN + 1,
        i32::MIN / 12 - 1,
        i32::MIN / 12,
        i32::MIN / 7 - 1,
        i32::MIN / 7,
        -13,
        -12,
        -7,
        -1,
        0,
        1,
        7,
        12,
        13,
        i32::MAX / 12,
        i32::MAX / 12 + 1,
        i32::MAX / 7,
        i32::MAX / 7 + 1,
        i32::MAX - 1,
        i32::MAX,
    ];

    fn varied_length(rng: &mut prng::SplitMix64) -> i32 {
        match rng.below(3) {
            0 => *rng.choose(&EDGE_LENGTHS),
            1 => i32::try_from(rng.range_i64(-10_000, 10_000)).expect("small length"),
            _ => i32::try_from(rng.range_i64(i64::from(i32::MIN), i64::from(i32::MAX)))
                .expect("i32 domain"),
        }
    }

    #[test]
    fn identifiers_are_well_formed_and_exhaustively_round_trip() {
        assert_eq!(UNIT_IDENTIFIERS.check(), Ok(()));
        assert_eq!(FREQUENCY_IDENTIFIERS.check(), Ok(()));
        assert_eq!(
            UNIT_IDENTIFIERS.check_round_trip(&TimeUnit::ALL, |unit| unit.to_string()),
            Ok(())
        );
        assert_eq!(
            FREQUENCY_IDENTIFIERS
                .check_round_trip(&Frequency::ALL, |frequency| frequency.to_string()),
            Ok(())
        );
        for (unit, alias) in TimeUnit::ALL.into_iter().zip(["D", "W", "M", "Y"]) {
            assert_eq!(alias.parse::<TimeUnit>(), Ok(unit));
            assert_eq!(unit.canonical_identifier().parse::<TimeUnit>(), Ok(unit));
        }
    }

    #[test]
    fn identifiers_reject_unlisted_spelling_with_original_context() {
        for text in [
            "", "Day", "day", "days", "d", "Months ", " Years", "Weeks\n", "H", "1D",
        ] {
            assert_eq!(
                text.parse::<TimeUnit>(),
                Err(IdentifierError::Unknown {
                    concept: "time unit",
                    text: text.to_owned(),
                })
            );
        }
        for text in [
            "",
            "annual",
            "ANNUAL",
            " Annual",
            "Annual ",
            "SemiAnnual",
            "NoFrequency",
            "OtherFrequency",
            "13",
            "daily\n",
        ] {
            assert_eq!(
                text.parse::<Frequency>(),
                Err(IdentifierError::Unknown {
                    concept: "frequency",
                    text: text.to_owned(),
                })
            );
        }
    }

    #[test]
    fn every_frequency_matches_the_specified_count_and_period() {
        use Frequency::*;
        use TimeUnit::*;
        let rows = [
            (Annual, 1, p(1, Years)),
            (Semiannual, 2, p(6, Months)),
            (EveryFourthMonth, 3, p(4, Months)),
            (Quarterly, 4, p(3, Months)),
            (Bimonthly, 6, p(2, Months)),
            (Monthly, 12, p(1, Months)),
            (EveryFourthWeek, 13, p(4, Weeks)),
            (Biweekly, 26, p(2, Weeks)),
            (Weekly, 52, p(1, Weeks)),
            (Daily, 365, p(1, Days)),
        ];
        for (frequency, count, period) in rows {
            assert_eq!(frequency.events_per_year(), Some(count));
            let mapped = frequency.period().expect("periodic frequency");
            assert_eq!(mapped.length(), period.length());
            assert_eq!(mapped.unit(), period.unit());
            assert_eq!(mapped.frequency(), Some(frequency));
        }
        assert_eq!(Once.events_per_year(), None);
        assert_eq!(Once.period(), None);
    }

    #[test]
    fn frequency_mapping_uses_normalized_equivalence_and_has_no_sentinels() {
        assert_eq!(p(12, TimeUnit::Months).frequency(), Some(Frequency::Annual));
        for (days, frequency) in [
            (7, Frequency::Weekly),
            (14, Frequency::Biweekly),
            (28, Frequency::EveryFourthWeek),
        ] {
            assert_eq!(p(days, TimeUnit::Days).frequency(), Some(frequency));
        }
        for unit in TimeUnit::ALL {
            for length in [i32::MIN, -365, -52, -28, -14, -12, -7, -1, 0, i32::MAX] {
                assert_eq!(p(length, unit).frequency(), None);
            }
        }
        for period in [
            p(5, TimeUnit::Months),
            p(52, TimeUnit::Weeks),
            p(365, TimeUnit::Days),
            p(2, TimeUnit::Years),
        ] {
            assert_eq!(period.frequency(), None);
        }
    }

    #[test]
    fn construction_preserves_every_unit_and_all_extreme_lengths() {
        for unit in TimeUnit::ALL {
            for length in [i32::MIN, i32::MIN + 1, -1, 0, 1, i32::MAX - 1, i32::MAX] {
                let period = p(length, unit);
                assert_eq!(period.length(), length);
                assert_eq!(period.unit(), unit);
            }
        }
    }

    #[test]
    fn normalization_matches_divisibility_rules_at_boundaries() {
        use TimeUnit::*;
        for (input, length, unit) in [
            (p(14, Days), 2, Weeks),
            (p(-14, Days), -2, Weeks),
            (p(13, Days), 13, Days),
            (p(-13, Days), -13, Days),
            (p(12, Months), 1, Years),
            (p(-24, Months), -2, Years),
            (p(13, Months), 13, Months),
            (p(-13, Months), -13, Months),
            (p(i32::MAX - 1, Days), 306_783_378, Weeks),
            (p(i32::MIN, Days), i32::MIN, Days),
            (p(i32::MAX - 7, Months), 178_956_970, Years),
            (p(i32::MIN, Months), i32::MIN, Months),
        ] {
            let normalized = input.normalized();
            assert_eq!(normalized.length(), length);
            assert_eq!(normalized.unit(), unit);
            assert_eq!(input, normalized);
        }
        for unit in TimeUnit::ALL {
            assert_eq!(p(0, unit).normalized().length(), 0);
            assert_eq!(p(0, unit).normalized().unit(), Days);
            assert_eq!(p(i32::MIN, unit), p(i32::MIN, unit).normalized());
            assert_eq!(p(i32::MAX, unit), p(i32::MAX, unit).normalized());
        }
    }

    #[test]
    fn normalization_is_idempotent_and_preserves_value_exhaustively() {
        for unit in TimeUnit::ALL {
            for length in -10_000..=10_000 {
                let period = p(length, unit);
                let normalized = period.normalized();
                let twice = normalized.normalized();
                assert_eq!(normalized.length(), twice.length());
                assert_eq!(normalized.unit(), twice.unit());
                assert_eq!(period, normalized);
                assert_eq!(hash(period), hash(normalized));
                assert_eq!(period.partial_cmp(&normalized), Some(Ordering::Equal));
            }
        }
    }

    #[test]
    fn equality_and_hashing_use_exact_widened_family_values() {
        use TimeUnit::*;
        for (left, right) in [
            (p(1, Years), p(12, Months)),
            (p(-2, Years), p(-24, Months)),
            (p(2, Weeks), p(14, Days)),
            (p(-3, Weeks), p(-21, Days)),
            (p(i32::MAX / 12, Years), p((i32::MAX / 12) * 12, Months)),
            (p(i32::MIN / 12, Years), p((i32::MIN / 12) * 12, Months)),
            (p(i32::MAX / 7, Weeks), p((i32::MAX / 7) * 7, Days)),
            (p(i32::MIN / 7, Weeks), p((i32::MIN / 7) * 7, Days)),
        ] {
            assert_eq!(left, right);
            assert_eq!(hash(left), hash(right));
        }
        assert_ne!(p(1, Years), p(365, Days));
        assert_ne!(p(1, Months), p(30, Days));
        assert_ne!(p(i32::MAX, Years), p(i32::MAX, Months));
        assert_ne!(p(i32::MIN, Weeks), p(i32::MIN, Days));
        assert_eq!(
            TimeUnit::ALL
                .map(|unit| p(0, unit))
                .into_iter()
                .collect::<HashSet<_>>()
                .len(),
            1
        );
        assert_eq!(
            [p(1, Years), p(12, Months), p(2, Weeks), p(14, Days)]
                .into_iter()
                .collect::<HashSet<_>>()
                .len(),
            2
        );
    }

    #[test]
    fn ordering_has_two_families_with_one_canonical_zero_family() {
        use TimeUnit::*;
        for (left, right, expected) in [
            (p(7, Days), p(1, Weeks), Some(Ordering::Equal)),
            (p(-8, Days), p(-1, Weeks), Some(Ordering::Less)),
            (p(13, Months), p(1, Years), Some(Ordering::Greater)),
            (p(-13, Months), p(-1, Years), Some(Ordering::Less)),
            (
                p(i32::MAX, Years),
                p(i32::MAX, Months),
                Some(Ordering::Greater),
            ),
            (p(i32::MIN, Weeks), p(i32::MIN, Days), Some(Ordering::Less)),
            (p(1, Months), p(1, Weeks), None),
            (p(-1, Months), p(1, Weeks), None),
            (p(1, Years), p(365, Days), None),
        ] {
            assert_eq!(left.partial_cmp(&right), expected);
            assert_eq!(right.partial_cmp(&left), expected.map(Ordering::reverse));
        }
        for unit in TimeUnit::ALL {
            let zero = p(0, unit);
            for other_unit in TimeUnit::ALL {
                assert_eq!(zero.partial_cmp(&p(0, other_unit)), Some(Ordering::Equal));
            }
            for day_unit in [Days, Weeks] {
                assert_eq!(zero.partial_cmp(&p(-1, day_unit)), Some(Ordering::Greater));
                assert_eq!(zero.partial_cmp(&p(1, day_unit)), Some(Ordering::Less));
            }
            for month_unit in [Months, Years] {
                assert_eq!(zero.partial_cmp(&p(-1, month_unit)), None);
                assert_eq!(zero.partial_cmp(&p(1, month_unit)), None);
            }
        }
    }

    #[test]
    fn order_and_equality_obey_the_trait_laws_over_every_small_triple() {
        let periods: Vec<_> = TimeUnit::ALL
            .into_iter()
            .flat_map(|unit| (-4..=4).map(move |length| p(length, unit)))
            .collect();
        for &a in &periods {
            assert_eq!(a.partial_cmp(&a), Some(Ordering::Equal));
            for &b in &periods {
                let ab = a.partial_cmp(&b);
                assert_eq!(ab, b.partial_cmp(&a).map(Ordering::reverse));
                assert_eq!(ab == Some(Ordering::Equal), a == b);
                if a == b {
                    assert_eq!(hash(a), hash(b));
                }
                for &c in &periods {
                    if a == b {
                        assert_eq!(a.partial_cmp(&c), b.partial_cmp(&c));
                        assert_eq!(c.partial_cmp(&a), c.partial_cmp(&b));
                    }
                    if a == b && b == c {
                        assert_eq!(a, c);
                    }
                    if a < b && b < c {
                        assert!(a < c, "{a} < {b} < {c} must be transitive");
                    }
                    if a <= b && b <= c {
                        assert!(a <= c, "{a} <= {b} <= {c} must be transitive");
                    }
                }
            }
        }
    }

    #[test]
    fn negation_handles_the_only_unrepresentable_counterpart() {
        for unit in TimeUnit::ALL {
            for length in [i32::MIN + 1, -1, 0, 1, i32::MAX] {
                let negated = p(length, unit).checked_neg().expect("negation fits");
                assert_eq!(negated.length(), -length);
                assert_eq!(negated.unit(), unit);
                assert_eq!(negated.checked_neg(), Ok(p(length, unit)));
            }
            assert_eq!(
                p(i32::MIN, unit).checked_neg(),
                Err(PeriodError::Overflow {
                    operation: "negate",
                    length: 2_147_483_648,
                    unit,
                })
            );
        }
    }

    #[test]
    fn multiplication_accepts_exact_results_and_rejects_both_overflow_signs() {
        for unit in TimeUnit::ALL {
            for (length, multiplier, expected) in [
                (i32::MIN, 0, 0),
                (i32::MIN, 1, i32::MIN),
                (i32::MAX, -1, -i32::MAX),
                (0, i32::MIN, 0),
                (1, i32::MIN, i32::MIN),
                (-1, i32::MAX, -i32::MAX),
                (i32::MAX / 2, 2, i32::MAX - 1),
                (i32::MIN / 2, 2, i32::MIN),
                (-3, -7, 21),
            ] {
                let product = p(length, unit)
                    .checked_mul(multiplier)
                    .expect("product fits");
                assert_eq!(product.length(), expected);
                assert_eq!(product.unit(), unit);
            }
            for (length, multiplier) in [
                (i32::MIN, -1),
                (-1, i32::MIN),
                (i32::MAX, 2),
                (i32::MIN, 2),
                (i32::MAX, i32::MAX),
                (i32::MIN, i32::MIN),
                (i32::MAX, i32::MIN),
            ] {
                assert_eq!(
                    p(length, unit).checked_mul(multiplier),
                    Err(PeriodError::Overflow {
                        operation: "multiply",
                        length: i64::from(length) * i64::from(multiplier),
                        unit,
                    })
                );
            }
        }
    }

    #[test]
    fn same_unit_arithmetic_preserves_the_unit_and_uses_final_range() {
        for unit in TimeUnit::ALL {
            for (left, right, expected) in [
                (i32::MAX, -1, i32::MAX - 1),
                (i32::MIN, 1, i32::MIN + 1),
                (i32::MIN, i32::MAX, -1),
                (-17, 42, 25),
            ] {
                let sum = p(left, unit).checked_add(p(right, unit)).expect("sum fits");
                assert_eq!(sum.length(), expected);
                assert_eq!(sum.unit(), unit);
            }
            for (left, right, expected) in [
                (i32::MIN, i32::MIN, 0),
                (i32::MIN + 1, i32::MIN, 1),
                (-1, i32::MIN, i32::MAX),
                (i32::MAX, i32::MAX, 0),
                (-17, 42, -59),
            ] {
                let difference = p(left, unit)
                    .checked_sub(p(right, unit))
                    .expect("difference fits");
                assert_eq!(difference.length(), expected);
                assert_eq!(difference.unit(), unit);
            }
            for (left, right, subtract) in [
                (i32::MAX, 1, false),
                (i32::MIN, -1, false),
                (i32::MIN, 1, true),
                (i32::MAX, -1, true),
                (0, i32::MIN, true),
            ] {
                let result = if subtract {
                    p(left, unit).checked_sub(p(right, unit))
                } else {
                    p(left, unit).checked_add(p(right, unit))
                };
                assert_eq!(
                    result,
                    Err(PeriodError::Overflow {
                        operation: if subtract { "subtract" } else { "add" },
                        length: if subtract {
                            i64::from(left) - i64::from(right)
                        } else {
                            i64::from(left) + i64::from(right)
                        },
                        unit,
                    })
                );
            }
        }
    }

    #[test]
    fn mixed_units_combine_in_the_finer_unit() {
        use TimeUnit::*;
        for (left, right, sum, difference) in [
            (p(2, Years), p(3, Months), p(27, Months), p(21, Months)),
            (p(3, Months), p(2, Years), p(27, Months), p(-21, Months)),
            (p(-2, Years), p(3, Months), p(-21, Months), p(-27, Months)),
            (p(2, Weeks), p(3, Days), p(17, Days), p(11, Days)),
            (p(3, Days), p(2, Weeks), p(17, Days), p(-11, Days)),
            (p(-2, Weeks), p(-3, Days), p(-17, Days), p(-11, Days)),
        ] {
            let actual_sum = left.checked_add(right).expect("sum fits");
            let actual_difference = left.checked_sub(right).expect("difference fits");
            assert_eq!(actual_sum.length(), sum.length());
            assert_eq!(actual_sum.unit(), sum.unit());
            assert_eq!(actual_difference.length(), difference.length());
            assert_eq!(actual_difference.unit(), difference.unit());
        }
    }

    #[test]
    fn widened_conversion_allows_large_operands_to_cancel() {
        use TimeUnit::*;
        // Converted years/weeks exceed i32, but exact final lengths fit.
        for (left, right, expected) in [
            (
                p(200_000_000, Years),
                p(i32::MIN, Months),
                p(252_516_352, Months),
            ),
            (
                p(-200_000_000, Years),
                p(i32::MAX, Months),
                p(-252_516_353, Months),
            ),
            (
                p(400_000_000, Weeks),
                p(i32::MIN, Days),
                p(652_516_352, Days),
            ),
            (
                p(-400_000_000, Weeks),
                p(i32::MAX, Days),
                p(-652_516_353, Days),
            ),
        ] {
            assert_eq!(left.checked_add(right), Ok(expected));
            assert_eq!(right.checked_add(left), Ok(expected));
        }
        assert_eq!(
            p(-200_000_000, Years).checked_sub(p(i32::MIN, Months)),
            Ok(p(-252_516_352, Months))
        );
        assert_eq!(
            p(-400_000_000, Weeks).checked_sub(p(i32::MIN, Days)),
            Ok(p(-652_516_352, Days))
        );
        assert_eq!(
            p(200_000_000, Years).checked_sub(p(i32::MAX, Months)),
            Ok(p(252_516_353, Months))
        );
        assert_eq!(
            p(400_000_000, Weeks).checked_sub(p(i32::MAX, Days)),
            Ok(p(652_516_353, Days))
        );
    }

    #[test]
    fn mixed_unit_overflow_reports_the_exact_final_length() {
        use TimeUnit::*;
        for (left, right, result_unit, left_base, right_base) in [
            (
                p(i32::MAX, Years),
                p(1, Months),
                Months,
                i64::from(i32::MAX) * 12,
                1,
            ),
            (
                p(i32::MIN, Years),
                p(-1, Months),
                Months,
                i64::from(i32::MIN) * 12,
                -1,
            ),
            (
                p(i32::MAX, Weeks),
                p(1, Days),
                Days,
                i64::from(i32::MAX) * 7,
                1,
            ),
            (
                p(i32::MIN, Weeks),
                p(-1, Days),
                Days,
                i64::from(i32::MIN) * 7,
                -1,
            ),
        ] {
            assert_eq!(
                left.checked_add(right),
                Err(PeriodError::Overflow {
                    operation: "add",
                    length: left_base + right_base,
                    unit: result_unit,
                })
            );
            assert_eq!(
                left.checked_sub(right),
                Err(PeriodError::Overflow {
                    operation: "subtract",
                    length: left_base - right_base,
                    unit: result_unit,
                })
            );
        }
    }

    #[test]
    fn nonzero_cross_family_arithmetic_rejects_every_unit_pair() {
        for left in TimeUnit::ALL {
            for right in TimeUnit::ALL {
                if left.family() == right.family() {
                    continue;
                }
                for left_length in [i32::MIN, -1, 1, i32::MAX] {
                    for right_length in [i32::MIN, -1, 1, i32::MAX] {
                        let expected = Err(PeriodError::IncompatibleUnits { left, right });
                        assert_eq!(
                            p(left_length, left).checked_add(p(right_length, right)),
                            expected
                        );
                        assert_eq!(
                            p(left_length, left).checked_sub(p(right_length, right)),
                            expected
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn zero_is_the_additive_identity_independent_of_its_stored_unit() {
        for zero_unit in TimeUnit::ALL {
            let zero = p(0, zero_unit);
            for other_unit in TimeUnit::ALL {
                for length in [i32::MIN, -1, 0, 1, i32::MAX] {
                    let other = p(length, other_unit);
                    assert_eq!(zero.checked_add(other), Ok(other));
                    assert_eq!(other.checked_add(zero), Ok(other));
                    assert_eq!(other.checked_sub(zero), Ok(other));
                    assert_eq!(
                        zero.checked_sub(other),
                        other.checked_neg().map_err(|error| match error {
                            PeriodError::Overflow { length, unit, .. } => PeriodError::Overflow {
                                operation: "subtract",
                                length,
                                unit
                            },
                            _ => error,
                        })
                    );
                }
            }
        }
    }

    #[test]
    fn arithmetic_obeys_value_laws_over_every_small_length_and_unit() {
        for unit in TimeUnit::ALL {
            for length in -100..=100 {
                let period = p(length, unit);
                assert_eq!(
                    period.checked_neg().and_then(Period::checked_neg),
                    Ok(period)
                );
                assert_eq!(period.checked_mul(0), Ok(p(0, TimeUnit::Days)));
                assert_eq!(period.checked_mul(1), Ok(period));
                assert_eq!(period.checked_mul(-1), period.checked_neg());
                for multiplier in -20..=20 {
                    let product = period.checked_mul(multiplier).expect("small product fits");
                    assert_eq!(product.length(), length * multiplier);
                    assert_eq!(product.unit(), unit);
                }
                for other_unit in TimeUnit::ALL {
                    for other_length in -10..=10 {
                        let other = p(other_length, other_unit);
                        assert_eq!(
                            period.checked_add(other).is_ok(),
                            length == 0
                                || other_length == 0
                                || unit.family() == other_unit.family()
                        );
                        if let Ok(sum) = period.checked_add(other) {
                            assert_eq!(other.checked_add(period), Ok(sum));
                            assert_eq!(sum.checked_sub(other), Ok(period));
                        }
                        if let Ok(difference) = period.checked_sub(other) {
                            assert_eq!(difference.checked_add(other), Ok(period));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn every_unit_pair_and_wide_boundary_pair_matches_an_i128_formula_model() {
        for left_unit in TimeUnit::ALL {
            for right_unit in TimeUnit::ALL {
                for left_length in EDGE_LENGTHS {
                    for right_length in EDGE_LENGTHS {
                        let left = p(left_length, left_unit);
                        let right = p(right_length, right_unit);
                        assert_same_stored_result(
                            left.checked_add(right),
                            reference_binary(left, right, false),
                        );
                        assert_same_stored_result(
                            left.checked_sub(right),
                            reference_binary(left, right, true),
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn deterministic_arbitrary_i32_periods_match_widened_reference_models() {
        prng::check("periods match i128 formulas", 0x5AB1_0002, 30_000, |rng| {
            let left = p(varied_length(rng), *rng.choose(&TimeUnit::ALL));
            let right = p(varied_length(rng), *rng.choose(&TimeUnit::ALL));
            for (actual, expected) in [
                (
                    left.checked_add(right),
                    reference_binary(left, right, false),
                ),
                (left.checked_sub(right), reference_binary(left, right, true)),
            ] {
                let matches = match (&actual, &expected) {
                    (Ok(actual), Ok(expected)) => {
                        actual.length() == expected.length() && actual.unit() == expected.unit()
                    }
                    _ => actual == expected,
                };
                if !matches {
                    return Err(format!(
                        "{left} and {right}: actual {actual:?}, expected {expected:?}"
                    ));
                }
            }
            let multiplier = varied_length(rng);
            let product = i128::from(left.length()) * i128::from(multiplier);
            let expected = match i32::try_from(product) {
                Ok(length) => Ok(p(length, left.unit())),
                Err(_) => Err(PeriodError::Overflow {
                    operation: "multiply",
                    length: i64::try_from(product).expect("i32 product fits i64"),
                    unit: left.unit(),
                }),
            };
            if left.checked_mul(multiplier) != expected {
                return Err(format!("{left} * {multiplier}: expected {expected:?}"));
            }
            let (left_family, a) = reference_quantity(left);
            let (right_family, b) = reference_quantity(right);
            let expected_order = (left_family == right_family).then(|| a.cmp(&b));
            if left.partial_cmp(&right) != expected_order
                || (left == right) != (expected_order == Some(Ordering::Equal))
            {
                return Err(format!(
                    "{left} vs {right}: expected order {expected_order:?}"
                ));
            }
            let normalized = left.normalized();
            if left != normalized || hash(left) != hash(normalized) {
                return Err(format!("{left} and normalized {normalized} disagree"));
            }
            for period in [left, right] {
                let parsed = period
                    .to_string()
                    .parse::<Period>()
                    .map_err(|error| error.to_string())?;
                if parsed.length() != period.length() || parsed.unit() != period.unit() {
                    return Err(format!(
                        "stored representation {period} failed text round trip"
                    ));
                }
            }
            Ok(())
        });
    }

    #[test]
    fn formatting_and_parsing_round_trip_stored_representations() {
        for unit in TimeUnit::ALL {
            for length in (i32::MIN..=i32::MIN + 10)
                .chain(-10_000..=10_000)
                .chain(i32::MAX - 10..=i32::MAX)
            {
                let period = p(length, unit);
                let text = period.to_string();
                let parsed = text.parse::<Period>().expect("formatted period");
                assert_eq!(parsed.length(), length);
                assert_eq!(parsed.unit(), unit);
                let lower = text
                    .to_ascii_lowercase()
                    .parse::<Period>()
                    .expect("lowercase unit accepted");
                assert_eq!(lower.length(), length);
                assert_eq!(lower.unit(), unit);
            }
        }
    }

    #[test]
    fn parsing_accepts_signs_leading_zeros_and_exact_i32_boundaries() {
        for (text, length, unit) in [
            ("0D", 0, TimeUnit::Days),
            ("-0m", 0, TimeUnit::Months),
            ("+0Y", 0, TimeUnit::Years),
            ("0000000000000000000000000000000w", 0, TimeUnit::Weeks),
            ("+0012m", 12, TimeUnit::Months),
            ("-0007d", -7, TimeUnit::Days),
            ("2147483647Y", i32::MAX, TimeUnit::Years),
            ("+2147483647M", i32::MAX, TimeUnit::Months),
            ("-2147483648W", i32::MIN, TimeUnit::Weeks),
            ("0002147483647d", i32::MAX, TimeUnit::Days),
            ("-0002147483648y", i32::MIN, TimeUnit::Years),
        ] {
            let period = text.parse::<Period>().expect("valid grammar and range");
            assert_eq!(period.length(), length, "{text:?}");
            assert_eq!(period.unit(), unit, "{text:?}");
        }
    }

    #[test]
    fn parsing_rejects_all_malformed_forms_and_preserves_exact_input() {
        for text in [
            "",
            "D",
            "+D",
            "-D",
            "1",
            "+",
            "-",
            "++1D",
            "--1D",
            "+-1D",
            "-+1D",
            " 1D",
            "1D ",
            "1 D",
            "1\tD",
            "1D\n",
            "\n1D",
            "1\0D",
            "1Y6M",
            "1.5M",
            "1e3D",
            "0x1D",
            "1_000D",
            "1Days",
            "1H",
            "1q",
            "１D",
            "١D",
            "−1D",
            "１Ｄ",
            "🙂D",
            "1🙂",
            "2147483648D",
            "+2147483648W",
            "-2147483649M",
            "999999999999999999999999999999999999999Y",
            "\u{feff}1D",
            "1\u{00a0}D",
            "1\rD",
        ] {
            assert_eq!(
                text.parse::<Period>(),
                Err(PeriodError::Parse {
                    text: text.to_owned()
                }),
                "{text:?}"
            );
        }
    }

    #[test]
    fn parsing_any_two_ascii_bytes_agrees_with_the_simple_grammar() {
        // Every two-byte ASCII input, including controls, punctuation and
        // unit letters; this finds accidental trimming or permissive parsing.
        for number in 0_u8..=127 {
            for letter in 0_u8..=127 {
                let text = String::from_utf8(vec![number, letter]).expect("ASCII");
                let unit = match letter {
                    b'D' | b'd' => Some(TimeUnit::Days),
                    b'W' | b'w' => Some(TimeUnit::Weeks),
                    b'M' | b'm' => Some(TimeUnit::Months),
                    b'Y' | b'y' => Some(TimeUnit::Years),
                    _ => None,
                };
                match (number.is_ascii_digit(), unit) {
                    (true, Some(unit)) => assert_eq!(
                        text.parse::<Period>(),
                        Ok(p(i32::from(number - b'0'), unit))
                    ),
                    _ => assert_eq!(text.parse::<Period>(), Err(PeriodError::Parse { text })),
                }
            }
        }
    }

    #[test]
    fn every_error_has_a_stable_kind_and_structured_context() {
        let errors = [
            (
                "period.overflow",
                PeriodError::Overflow {
                    operation: "negate",
                    length: 2_147_483_648,
                    unit: TimeUnit::Days,
                },
            ),
            (
                "period.incompatible_units",
                PeriodError::IncompatibleUnits {
                    left: TimeUnit::Months,
                    right: TimeUnit::Days,
                },
            ),
            (
                "period.parse",
                PeriodError::Parse {
                    text: " bad 🙂".to_owned(),
                },
            ),
        ];
        for (kind, error) in errors {
            assert_eq!(error.kind(), kind);
            assert_eq!(error.clone(), error);
            assert!(!error.to_string().is_empty());
            assert!(error.source().is_none());
        }
    }

    #[test]
    fn public_values_and_errors_are_send_sync_and_static() {
        fn assert_traits<T: Send + Sync + 'static>() {}
        assert_traits::<TimeUnit>();
        assert_traits::<Frequency>();
        assert_traits::<Period>();
        assert_traits::<PeriodError>();
    }
}
