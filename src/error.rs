//! The crate-level error, and the error types that every domain shares.
//!
//! # Conventions
//!
//! SabiQuant reports failure with typed errors, as decided in
//! [ADR-0007](https://github.com/iceman951/sabiquant/blob/main/docs/adr/ADR-0007-error-handling.md):
//!
//! - Every operation that can fail for some input returns [`Result`]. Failure
//!   is never signalled by a sentinel value, by NaN, or by a panic
//!   (FR-CORE-01).
//! - Each domain defines one non-exhaustive error enumeration next to its
//!   operations. Every variant carries structured context, such as the name
//!   of the rejected parameter and its value (FR-CORE-02).
//! - [`Error`] wraps the error of every domain, so code that spans several
//!   domains can propagate any failure with `?`.
//! - Every error type implements [`std::error::Error`],
//!   [`Display`](fmt::Display), [`Debug`](fmt::Debug), [`Clone`] and
//!   [`PartialEq`], and is [`Send`], [`Sync`] and `'static`.
//!
//! # Kind names are stable; messages are not
//!
//! Every error variant has a **kind name**, returned by the `kind` method of
//! its type, for example `"value.not_finite"`. Kind names have the form
//! `<domain>.<variant>`, in lowercase ASCII letters and underscores. They are
//! part of the stable API, and reference fixtures use them to state the error
//! that a scenario expects
//! ([ADR-0008](https://github.com/iceman951/sabiquant/blob/main/docs/adr/ADR-0008-validation-strategy.md)).
//! The wording of `Display` messages is **not** stable and may change in any
//! release.
//!
//! | Kind name | Variant | Reported when |
//! |---|---|---|
//! | `value.not_finite` | [`ValueError::NotFinite`] | A real-valued input is NaN or infinite |
//! | `value.out_of_domain` | [`ValueError::OutOfDomain`] | A finite real-valued input lies outside its documented domain |
//! | `value.result_not_finite` | [`ValueError::ResultNotFinite`] | A computed result overflows, or is otherwise not finite |
//! | `value.result_out_of_domain` | [`ValueError::ResultOutOfDomain`] | A computed finite result lies outside its domain |
//! | `identifier.unknown` | [`IdentifierError::Unknown`] | Text is neither a canonical identifier nor a documented alias |
//! | `identifier.ambiguous` | [`IdentifierError::Ambiguous`] | Text could name more than one convention |
//!
//! From milestone M2 on, each domain module adds its own error type, and
//! [`Error`] gains one variant for it.
//!
//! # Error sources
//!
//! [`Error`] is transparent: its message and its
//! [`source`](std::error::Error::source) are those of the domain error that it
//! wraps. A domain error that was caused by another error returns that error
//! from `source`.
//!
//! # QuantLib correspondence
//!
//! QuantLib 1.43 signals failure by throwing `QuantLib::Error` exceptions from
//! the `QL_REQUIRE`, `QL_ENSURE` and `QL_FAIL` macros (`ql/errors.hpp`), and
//! some of its numerical code returns NaN instead. SabiQuant returns typed
//! errors and never returns NaN. Where QuantLib returns a value for an input
//! that SabiQuant rejects, the difference is a documented intentional
//! divergence
//! ([ADR-0003](https://github.com/iceman951/sabiquant/blob/main/docs/adr/ADR-0003-quantlib-compatibility.md)).

use std::error::Error as StdError;
use std::fmt;
use std::ops::Bound;

/// Any error reported by SabiQuant.
///
/// `Error` wraps the error type of each domain, so that code which combines
/// several domains can propagate any failure with `?`. Match on the variant to
/// inspect the domain error, or call [`kind`](Self::kind) for its stable kind
/// name.
///
/// `Error` is transparent: its `Display` output and its
/// [`source`](std::error::Error::source) are those of the wrapped domain
/// error.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// A real number was rejected by the numeric validation policy.
    Value(ValueError),
    /// Text did not name exactly one identifier.
    Identifier(IdentifierError),
}

impl Error {
    /// Returns the stable kind name of the wrapped error, for example
    /// `"value.not_finite"`.
    ///
    /// The [module documentation](crate::error) lists every kind name.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Value(error) => error.kind(),
            Self::Identifier(error) => error.kind(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(error) => fmt::Display::fmt(error, f),
            Self::Identifier(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Value(error) => error.source(),
            Self::Identifier(error) => error.source(),
        }
    }
}

impl From<ValueError> for Error {
    fn from(error: ValueError) -> Self {
        Self::Value(error)
    }
}

impl From<IdentifierError> for Error {
    fn from(error: IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

/// A real number rejected by the numeric validation policy.
///
/// Operations check the real numbers they receive at the public boundary, and
/// check the real numbers they compute before returning them
/// ([NUMERICAL_CORRECTNESS §5](https://github.com/iceman951/sabiquant/blob/main/docs/NUMERICAL_CORRECTNESS.md#5-numerical-policies)):
///
/// - **NP-01**: every real-valued input must be finite (FR-CORE-03);
/// - **NP-02**: every result must be finite and inside its domain, so an
///   overflow or underflow that makes a result meaningless is an error, never
///   an infinity or a zero.
///
/// Parameter and quantity names are the names used in the documentation of
/// the operation that reports the error.
///
/// # QuantLib correspondence
///
/// QuantLib 1.43 often performs none of these checks and returns NaN or an
/// infinity instead. Rejecting such inputs and results is an intentional
/// divergence.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ValueError {
    /// A real-valued input is NaN or infinite (NP-01).
    ///
    /// Kind name: `value.not_finite`.
    #[non_exhaustive]
    NotFinite {
        /// The name of the rejected parameter.
        parameter: &'static str,
        /// The non-finite value that was given.
        value: NonFinite,
    },
    /// A finite real-valued input lies outside its documented domain.
    ///
    /// Kind name: `value.out_of_domain`.
    #[non_exhaustive]
    OutOfDomain {
        /// The name of the rejected parameter.
        parameter: &'static str,
        /// The value that was given. It is always finite.
        value: f64,
        /// The domain that the value must lie in.
        domain: Interval,
    },
    /// A computed result is not finite, for example after an overflow (NP-02).
    ///
    /// Kind name: `value.result_not_finite`.
    #[non_exhaustive]
    ResultNotFinite {
        /// The name of the computed quantity, for example `"compound factor"`.
        quantity: &'static str,
        /// The non-finite value that the computation produced.
        value: NonFinite,
    },
    /// A computed finite result lies outside its domain, for example a
    /// compound factor that underflowed to zero (NP-02).
    ///
    /// Kind name: `value.result_out_of_domain`.
    #[non_exhaustive]
    ResultOutOfDomain {
        /// The name of the computed quantity, for example `"compound factor"`.
        quantity: &'static str,
        /// The value that the computation produced. It is always finite.
        value: f64,
        /// The domain that the result must lie in.
        domain: Interval,
    },
}

impl ValueError {
    /// Returns the stable kind name of the error: `"value.not_finite"`,
    /// `"value.out_of_domain"`, `"value.result_not_finite"` or
    /// `"value.result_out_of_domain"`.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::NotFinite { .. } => "value.not_finite",
            Self::OutOfDomain { .. } => "value.out_of_domain",
            Self::ResultNotFinite { .. } => "value.result_not_finite",
            Self::ResultOutOfDomain { .. } => "value.result_out_of_domain",
        }
    }
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFinite { parameter, value } => {
                write!(f, "`{parameter}` must be finite, but is {value}")
            }
            Self::OutOfDomain {
                parameter,
                value,
                domain,
            } => write!(f, "`{parameter}` is {value:?}, outside its domain {domain}"),
            Self::ResultNotFinite { quantity, value } => {
                write!(f, "the computed {quantity} is not finite: {value}")
            }
            Self::ResultOutOfDomain {
                quantity,
                value,
                domain,
            } => write!(
                f,
                "the computed {quantity} is {value:?}, outside its domain {domain}"
            ),
        }
    }
}

impl StdError for ValueError {}

/// The non-finite values of IEEE 754 binary64 (`f64`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[expect(
    clippy::exhaustive_enums,
    reason = "IEEE 754 has exactly these three kinds of non-finite value"
)]
pub enum NonFinite {
    /// Not a number, whatever its sign and payload.
    NaN,
    /// Positive infinity.
    PositiveInfinity,
    /// Negative infinity.
    NegativeInfinity,
}

impl NonFinite {
    /// Classifies `value`, or returns `None` when it is finite.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "used by the numeric validation helpers from M2")
    )]
    pub(crate) fn classify(value: f64) -> Option<Self> {
        if value.is_nan() {
            Some(Self::NaN)
        } else if value.is_infinite() {
            if value.is_sign_positive() {
                Some(Self::PositiveInfinity)
            } else {
                Some(Self::NegativeInfinity)
            }
        } else {
            None
        }
    }
}

impl fmt::Display for NonFinite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NaN => "NaN",
            Self::PositiveInfinity => "inf",
            Self::NegativeInfinity => "-inf",
        })
    }
}

/// An interval of real numbers: the documented domain of a parameter or a
/// result.
///
/// Each end is unbounded, inclusive or exclusive, as described by [`Bound`].
/// The bounds themselves are finite. An interval contains only finite real
/// numbers: NaN and the infinities are never inside it, even at an unbounded
/// end.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    lower: Bound<f64>,
    upper: Bound<f64>,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the constructors are used by the domain modules from M2"
    )
)]
impl Interval {
    /// Creates the interval between `lower` and `upper`.
    ///
    /// The bounds must be finite, and the interval must not be empty. A
    /// violation is a defect in SabiQuant, caught by a debug assertion.
    pub(crate) const fn new(lower: Bound<f64>, upper: Bound<f64>) -> Self {
        debug_assert!(
            Self::is_valid(lower, upper),
            "interval bounds must be finite and describe a non-empty interval"
        );
        Self { lower, upper }
    }

    /// The interval `(value, +inf)`.
    pub(crate) const fn greater_than(value: f64) -> Self {
        Self::new(Bound::Excluded(value), Bound::Unbounded)
    }

    /// The interval `[value, +inf)`.
    pub(crate) const fn at_least(value: f64) -> Self {
        Self::new(Bound::Included(value), Bound::Unbounded)
    }

    /// The interval `(-inf, value)`.
    pub(crate) const fn less_than(value: f64) -> Self {
        Self::new(Bound::Unbounded, Bound::Excluded(value))
    }

    /// The interval `(-inf, value]`.
    pub(crate) const fn at_most(value: f64) -> Self {
        Self::new(Bound::Unbounded, Bound::Included(value))
    }

    /// Whether `lower` and `upper` describe a valid interval: finite bounds,
    /// in order, enclosing at least one real number.
    pub(crate) const fn is_valid(lower: Bound<f64>, upper: Bound<f64>) -> bool {
        match (lower, upper) {
            (Bound::Unbounded, Bound::Unbounded) => true,
            (Bound::Included(end) | Bound::Excluded(end), Bound::Unbounded)
            | (Bound::Unbounded, Bound::Included(end) | Bound::Excluded(end)) => end.is_finite(),
            (Bound::Included(low), Bound::Included(high)) => {
                low.is_finite() && high.is_finite() && low <= high
            }
            (
                Bound::Included(low) | Bound::Excluded(low),
                Bound::Included(high) | Bound::Excluded(high),
            ) => low.is_finite() && high.is_finite() && low < high,
        }
    }
}

impl Interval {
    /// Returns the lower end of the interval.
    #[must_use]
    pub const fn lower(&self) -> Bound<f64> {
        self.lower
    }

    /// Returns the upper end of the interval.
    #[must_use]
    pub const fn upper(&self) -> Bound<f64> {
        self.upper
    }

    /// Returns whether `value` lies in the interval.
    ///
    /// NaN and the infinities are never contained. `-0.0` and `+0.0` are the
    /// same number here (NP-08).
    #[must_use]
    pub fn contains(&self, value: f64) -> bool {
        let above_lower = match self.lower {
            Bound::Unbounded => true,
            Bound::Included(lower) => value >= lower,
            Bound::Excluded(lower) => value > lower,
        };
        let below_upper = match self.upper {
            Bound::Unbounded => true,
            Bound::Included(upper) => value <= upper,
            Bound::Excluded(upper) => value < upper,
        };
        value.is_finite() && above_lower && below_upper
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.lower {
            Bound::Unbounded => f.write_str("(-inf")?,
            Bound::Included(lower) => write!(f, "[{lower:?}")?,
            Bound::Excluded(lower) => write!(f, "({lower:?}")?,
        }
        f.write_str(", ")?;
        match self.upper {
            Bound::Unbounded => f.write_str("+inf)"),
            Bound::Included(upper) => write!(f, "{upper:?}]"),
            Bound::Excluded(upper) => write!(f, "{upper:?})"),
        }
    }
}

/// Text that does not name exactly one identifier.
///
/// Conventions, frequencies, time units, compounding methods and calendars
/// each have exactly one canonical identifier and a documented list of
/// aliases (FR-CORE-05). Parsing accepts exactly those spellings: it is
/// case-sensitive and does not trim whitespace. A label that could name more
/// than one convention, such as `30/360` or `Actual/Actual`, is always
/// rejected rather than mapped to a default (FR-CORE-06).
///
/// # QuantLib correspondence
///
/// QuantLib has no equivalent: its convention names are display strings, and
/// it has no parser that rejects ambiguous labels.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum IdentifierError {
    /// The text is neither a canonical identifier nor a documented alias.
    ///
    /// Kind name: `identifier.unknown`.
    #[non_exhaustive]
    Unknown {
        /// What was being parsed, for example `"frequency"`.
        concept: &'static str,
        /// The rejected text, exactly as given.
        text: String,
    },
    /// The text could name more than one convention, so it is rejected.
    ///
    /// Kind name: `identifier.ambiguous`.
    #[non_exhaustive]
    Ambiguous {
        /// What was being parsed, for example `"day-count convention"`.
        concept: &'static str,
        /// The rejected text, exactly as given.
        text: String,
        /// The canonical identifiers of the conventions that the text could
        /// name.
        candidates: &'static [&'static str],
    },
}

impl IdentifierError {
    /// Returns the stable kind name of the error: `"identifier.unknown"` or
    /// `"identifier.ambiguous"`.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Unknown { .. } => "identifier.unknown",
            Self::Ambiguous { .. } => "identifier.ambiguous",
        }
    }
}

impl fmt::Display for IdentifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown { concept, text } => {
                write!(f, "unknown {concept} identifier {text:?}")
            }
            Self::Ambiguous {
                concept,
                text,
                candidates,
            } => {
                write!(
                    f,
                    "ambiguous {concept} identifier {text:?}; name one of the candidates: "
                )?;
                for (index, candidate) in candidates.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{candidate:?}")?;
                }
                Ok(())
            }
        }
    }
}

impl StdError for IdentifierError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::float;
    use crate::support::prng::{self, SplitMix64};

    const CANDIDATES: &[&str] = &["Actual/Actual (ISDA)", "Actual/Actual (ICMA)"];

    /// One error of every variant, with its documented kind name.
    fn every_variant() -> Vec<(Error, &'static str)> {
        vec![
            (
                ValueError::NotFinite {
                    parameter: "rate",
                    value: NonFinite::NaN,
                }
                .into(),
                "value.not_finite",
            ),
            (
                ValueError::OutOfDomain {
                    parameter: "time",
                    value: -0.5,
                    domain: Interval::at_least(0.0),
                }
                .into(),
                "value.out_of_domain",
            ),
            (
                ValueError::ResultNotFinite {
                    quantity: "compound factor",
                    value: NonFinite::PositiveInfinity,
                }
                .into(),
                "value.result_not_finite",
            ),
            (
                ValueError::ResultOutOfDomain {
                    quantity: "compound factor",
                    value: 0.0,
                    domain: Interval::greater_than(0.0),
                }
                .into(),
                "value.result_out_of_domain",
            ),
            (
                IdentifierError::Unknown {
                    concept: "frequency",
                    text: "Monthy".to_owned(),
                }
                .into(),
                "identifier.unknown",
            ),
            (
                IdentifierError::Ambiguous {
                    concept: "day-count convention",
                    text: "Actual/Actual".to_owned(),
                    candidates: CANDIDATES,
                }
                .into(),
                "identifier.ambiguous",
            ),
        ]
    }

    /// Exhaustive over the variants inside this crate: adding a variant
    /// breaks the build here until `every_variant` covers it.
    fn variant_name(error: &Error) -> &'static str {
        match error {
            Error::Value(ValueError::NotFinite { .. }) => "NotFinite",
            Error::Value(ValueError::OutOfDomain { .. }) => "OutOfDomain",
            Error::Value(ValueError::ResultNotFinite { .. }) => "ResultNotFinite",
            Error::Value(ValueError::ResultOutOfDomain { .. }) => "ResultOutOfDomain",
            Error::Identifier(IdentifierError::Unknown { .. }) => "Unknown",
            Error::Identifier(IdentifierError::Ambiguous { .. }) => "Ambiguous",
        }
    }

    fn is_well_formed_kind(kind: &str) -> bool {
        let segment = |s: &str| {
            !s.is_empty()
                && !s.starts_with('_')
                && !s.ends_with('_')
                && !s.contains("__")
                && s.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
        };
        matches!(kind.split_once('.'), Some((domain, variant)) if segment(domain) && segment(variant))
    }

    /// TS-CORE-01, FR-CORE-02, ADR-0007 item 5: every variant has its
    /// documented, well-formed and unique kind name, and `Error` reports the
    /// kind of the error it wraps.
    #[test]
    fn every_variant_has_a_stable_unique_kind_name() {
        let errors = every_variant();
        let mut names: Vec<&str> = errors.iter().map(|(e, _)| variant_name(e)).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 6, "every variant is sampled exactly once");

        let mut kinds = Vec::new();
        for (error, expected) in &errors {
            assert_eq!(error.kind(), *expected);
            assert!(
                is_well_formed_kind(expected),
                "malformed kind name {expected:?}"
            );
            kinds.push(*expected);
            let inner_kind = match error {
                Error::Value(inner) => inner.kind(),
                Error::Identifier(inner) => inner.kind(),
            };
            assert_eq!(inner_kind, *expected);
        }
        kinds.sort_unstable();
        kinds.dedup();
        assert_eq!(kinds.len(), errors.len(), "kind names are unique");
    }

    /// The kind-name format check itself rejects malformed names.
    #[test]
    fn kind_name_format_check_rejects_malformed_names() {
        for bad in [
            "",
            "value",
            "value.",
            ".not_finite",
            "Value.not_finite",
            "value.not-finite",
            "value.not_finite.extra",
            "value._x",
            "value.x__y",
        ] {
            assert!(!is_well_formed_kind(bad), "{bad:?} must be rejected");
        }
        assert!(is_well_formed_kind("value.not_finite"));
    }

    /// FR-CORE-02: messages carry the structured context. Only the presence
    /// of the context is checked, because message wording is not stable.
    #[test]
    fn messages_carry_the_context() {
        let messages: Vec<String> = every_variant().iter().map(|(e, _)| e.to_string()).collect();
        assert!(messages[0].contains("`rate`") && messages[0].contains("NaN"));
        assert!(messages[1].contains("`time`") && messages[1].contains("-0.5"));
        assert!(messages[1].contains("[0.0, +inf)"));
        assert!(messages[2].contains("compound factor") && messages[2].contains("inf"));
        assert!(messages[3].contains("compound factor") && messages[3].contains("(0.0, +inf)"));
        assert!(messages[4].contains("frequency") && messages[4].contains("\"Monthy\""));
        assert!(messages[5].contains("\"Actual/Actual\""));
        for candidate in CANDIDATES {
            assert!(
                messages[5].contains(candidate),
                "{candidate} missing from {}",
                messages[5]
            );
        }
    }

    /// FR-CORE-06: an ambiguity lists every candidate once, in table order,
    /// quoted and separated from its neighbors, so each one can be read and
    /// copied. The exact separator is wording, and is not checked.
    #[test]
    fn ambiguity_messages_list_the_candidates_distinctly() {
        for candidates in [
            &["A/360", "A/365F"][..],
            &[
                "30/360 (Bond Basis)",
                "30E/360",
                "30E/360 (ISDA)",
                "30/360 (US)",
            ][..],
        ] {
            let message = IdentifierError::Ambiguous {
                concept: "day-count convention",
                text: "x".to_owned(),
                candidates,
            }
            .to_string();
            let mut previous_end = None;
            for candidate in candidates {
                let quoted = format!("{candidate:?}");
                let start = message.find(&quoted).expect("every candidate is listed");
                assert_eq!(message.matches(&quoted).count(), 1, "{message}");
                if let Some(end) = previous_end {
                    let gap = &message[end..start];
                    assert!(
                        start > end && gap.contains(|c: char| !c.is_whitespace()),
                        "candidates run together in {message:?}"
                    );
                }
                previous_end = Some(start + quoted.len());
            }
        }
    }

    /// `Error` is transparent: same message as the wrapped error, and no
    /// source of its own. M1 errors have no underlying cause.
    #[test]
    fn crate_error_is_transparent() {
        for (error, _) in every_variant() {
            let inner_message = match &error {
                Error::Value(inner) => inner.to_string(),
                Error::Identifier(inner) => inner.to_string(),
            };
            assert_eq!(error.to_string(), inner_message);
            assert!(error.source().is_none());
        }
    }

    /// NP-01: classification of non-finite values, whatever the NaN payload
    /// or sign, and of the finite edge cases.
    #[test]
    fn non_finite_values_are_classified() {
        let nans = [
            f64::NAN,
            -f64::NAN,
            f64::from_bits(0x7FF0_0000_0000_0001), // signalling NaN
            f64::from_bits(0xFFF8_0000_0000_0000),
            f64::from_bits(u64::MAX),
        ];
        for nan in nans {
            assert_eq!(NonFinite::classify(nan), Some(NonFinite::NaN));
        }
        assert_eq!(
            NonFinite::classify(f64::INFINITY),
            Some(NonFinite::PositiveInfinity)
        );
        assert_eq!(
            NonFinite::classify(f64::NEG_INFINITY),
            Some(NonFinite::NegativeInfinity)
        );
        let finite = [
            0.0,
            -0.0,
            f64::from_bits(1), // smallest subnormal
            f64::MIN_POSITIVE,
            f64::EPSILON,
            f64::MAX,
            f64::MIN,
        ];
        for value in finite {
            assert_eq!(NonFinite::classify(value), None, "{value:?}");
        }
        assert_eq!(NonFinite::NaN.to_string(), "NaN");
        assert_eq!(NonFinite::PositiveInfinity.to_string(), "inf");
        assert_eq!(NonFinite::NegativeInfinity.to_string(), "-inf");
    }

    /// Interval membership at every kind of bound; NaN and the infinities are
    /// never members; both zeros behave alike (NP-08).
    #[test]
    fn interval_membership() {
        let non_negative = Interval::at_least(0.0);
        assert!(non_negative.contains(0.0));
        assert!(non_negative.contains(-0.0));
        assert!(non_negative.contains(f64::from_bits(1)));
        assert!(non_negative.contains(f64::MAX));
        assert!(!non_negative.contains(-f64::from_bits(1)));

        let positive = Interval::greater_than(0.0);
        assert!(!positive.contains(0.0));
        assert!(!positive.contains(-0.0));
        assert!(positive.contains(f64::from_bits(1)));

        let below_one = Interval::less_than(1.0);
        assert!(below_one.contains(1.0_f64.next_down()));
        assert!(!below_one.contains(1.0));
        assert!(below_one.contains(f64::MIN));

        let at_most_one = Interval::at_most(1.0);
        assert!(at_most_one.contains(1.0));
        assert!(!at_most_one.contains(1.0_f64.next_up()));

        let closed = Interval::new(Bound::Included(-1.0), Bound::Included(1.0));
        assert!(closed.contains(-1.0) && closed.contains(1.0) && closed.contains(0.0));
        assert!(!closed.contains(1.0_f64.next_up()));
        let point = Interval::new(Bound::Included(2.0), Bound::Included(2.0));
        assert!(point.contains(2.0) && !point.contains(2.0_f64.next_up()));

        let everything = Interval::new(Bound::Unbounded, Bound::Unbounded);
        for interval in [
            non_negative,
            positive,
            below_one,
            at_most_one,
            closed,
            everything,
        ] {
            for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                assert!(
                    !interval.contains(value),
                    "{interval} must not contain {value}"
                );
            }
        }
        assert!(everything.contains(f64::MAX) && everything.contains(f64::MIN));
    }

    #[test]
    fn interval_accessors_and_display() {
        let interval = Interval::greater_than(-2.0);
        assert_eq!(interval.lower(), Bound::Excluded(-2.0));
        assert_eq!(interval.upper(), Bound::Unbounded);
        assert_eq!(interval.to_string(), "(-2.0, +inf)");
        assert_eq!(Interval::at_least(0.0).to_string(), "[0.0, +inf)");
        assert_eq!(Interval::less_than(1.5).to_string(), "(-inf, 1.5)");
        assert_eq!(Interval::at_most(1.5).to_string(), "(-inf, 1.5]");
        let closed = Interval::new(Bound::Included(-1.0), Bound::Excluded(1e300));
        assert_eq!(closed.to_string(), "[-1.0, 1e300)");
        let everything = Interval::new(Bound::Unbounded, Bound::Unbounded);
        assert_eq!(everything.to_string(), "(-inf, +inf)");
    }

    /// Invalid bounds are a defect, caught by the debug assertion.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "interval bounds must be finite")]
    fn invalid_interval_is_a_defect() {
        let _ = Interval::new(Bound::Included(1.0), Bound::Excluded(1.0));
    }

    #[test]
    fn interval_validity_rules() {
        use Bound::{Excluded, Included, Unbounded};
        assert!(Interval::is_valid(Unbounded, Unbounded));
        assert!(Interval::is_valid(Included(0.0), Unbounded));
        assert!(Interval::is_valid(Unbounded, Excluded(0.0)));
        assert!(Interval::is_valid(Included(1.0), Included(1.0)));
        assert!(Interval::is_valid(Excluded(0.0), Excluded(1.0)));
        assert!(!Interval::is_valid(Included(1.0), Excluded(1.0)));
        assert!(!Interval::is_valid(Excluded(1.0), Included(1.0)));
        assert!(!Interval::is_valid(Included(2.0), Included(1.0)));
        assert!(!Interval::is_valid(Included(f64::NAN), Unbounded));
        assert!(!Interval::is_valid(Unbounded, Excluded(f64::INFINITY)));
        assert!(!Interval::is_valid(
            Included(f64::NEG_INFINITY),
            Included(0.0)
        ));
    }

    /// Validity at the edges: a NaN or infinite bound at either end is
    /// invalid; the two zeros are the same number (NP-08), so `[-0, +0]` is
    /// the point zero while `[-0, +0)` and `(-0, +0]` are empty.
    #[test]
    fn interval_validity_edge_cases() {
        use Bound::{Excluded, Included, Unbounded};
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for (lower, upper) in [
                (Included(bad), Unbounded),
                (Excluded(bad), Unbounded),
                (Unbounded, Included(bad)),
                (Unbounded, Excluded(bad)),
                (Included(0.0), Included(bad)),
                (Included(bad), Included(0.0)),
                (Excluded(0.0), Excluded(bad)),
                (Excluded(bad), Excluded(0.0)),
            ] {
                assert!(!Interval::is_valid(lower, upper), "{lower:?}, {upper:?}");
            }
        }
        assert!(Interval::is_valid(Included(-0.0), Included(0.0)));
        assert!(Interval::is_valid(Included(0.0), Included(-0.0)));
        assert!(!Interval::is_valid(Included(-0.0), Excluded(0.0)));
        assert!(!Interval::is_valid(Excluded(-0.0), Included(0.0)));
        assert!(!Interval::is_valid(Excluded(-0.0), Excluded(0.0)));
        assert!(Interval::is_valid(Included(f64::MIN), Included(f64::MAX)));
        assert!(!Interval::is_valid(Excluded(f64::MAX), Excluded(f64::MAX)));
        assert!(!Interval::is_valid(Included(f64::MAX), Included(f64::MIN)));
    }

    /// Every named constructor checks its bound in debug builds.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "interval bounds must be finite")]
    fn greater_than_nan_is_a_defect() {
        let _ = Interval::greater_than(f64::NAN);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "interval bounds must be finite")]
    fn at_least_infinity_is_a_defect() {
        let _ = Interval::at_least(f64::INFINITY);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "interval bounds must be finite")]
    fn less_than_negative_infinity_is_a_defect() {
        let _ = Interval::less_than(f64::NEG_INFINITY);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "interval bounds must be finite")]
    fn at_most_nan_is_a_defect() {
        let _ = Interval::at_most(f64::NAN);
    }

    #[test]
    fn interval_accessors_for_every_constructor() {
        let half_open = Interval::new(Bound::Excluded(-1.0), Bound::Included(1.0));
        let cases = [
            (
                Interval::greater_than(1.0),
                Bound::Excluded(1.0),
                Bound::Unbounded,
            ),
            (
                Interval::at_least(1.0),
                Bound::Included(1.0),
                Bound::Unbounded,
            ),
            (
                Interval::less_than(1.0),
                Bound::Unbounded,
                Bound::Excluded(1.0),
            ),
            (
                Interval::at_most(1.0),
                Bound::Unbounded,
                Bound::Included(1.0),
            ),
            (half_open, Bound::Excluded(-1.0), Bound::Included(1.0)),
        ];
        for (interval, lower, upper) in cases {
            assert_eq!(interval.lower(), lower);
            assert_eq!(interval.upper(), upper);
            assert_eq!(interval.clone(), interval);
        }
        assert_eq!(half_open.to_string(), "(-1.0, 1.0]");
        assert_eq!(Interval::at_least(-0.0).to_string(), "[-0.0, +inf)");
        assert_eq!(Interval::less_than(1e-300).to_string(), "(-inf, 1e-300)");
        assert_ne!(Interval::at_least(0.0), Interval::greater_than(0.0));
    }

    /// Draws a finite value: an ordinary magnitude, an arbitrary finite bit
    /// pattern, or one of the special finite values.
    fn finite_value(rng: &mut SplitMix64) -> f64 {
        match rng.below(3) {
            0 => (rng.next_f64() - 0.5) * 20.0,
            1 => loop {
                let x = rng.any_f64();
                if x.is_finite() {
                    break x;
                }
            },
            _ => *rng.choose(&[
                0.0,
                -0.0,
                1.0,
                -1.0,
                f64::MAX,
                f64::MIN,
                f64::MIN_POSITIVE,
                f64::from_bits(1),
                -f64::from_bits(1),
            ]),
        }
    }

    /// Draws a value near `bound`: the bound itself, a neighbor, or anything.
    fn near(rng: &mut SplitMix64, bound: f64) -> f64 {
        match rng.below(5) {
            0 => bound,
            1 => bound.next_up(),
            2 => bound.next_down(),
            3 => -bound,
            _ => finite_value(rng),
        }
    }

    /// Interval membership obeys the set algebra of real intervals, checked
    /// against relations rather than against the implementation:
    ///
    /// - complements: `(b, +inf)` is the complement of `(-inf, b]`, and
    ///   `[b, +inf)` of `(-inf, b)`;
    /// - closure: `[b, +inf)` is `(b, +inf)` plus the point `b`;
    /// - intersection: a two-sided interval is the intersection of its two
    ///   one-sided halves;
    /// - NaN and the infinities are never members.
    #[test]
    fn interval_membership_obeys_the_set_algebra() {
        prng::check("interval algebra", 0x1A7E_0001, 20_000, |rng| {
            let low = finite_value(rng);
            let value = near(rng, low);
            if !value.is_finite() {
                let any = Interval::at_least(low);
                return if any.contains(value) {
                    Err(format!("{any} contains {value:?}"))
                } else {
                    Ok(())
                };
            }
            let (above, at_most) = (Interval::greater_than(low), Interval::at_most(low));
            let (at_least, below) = (Interval::at_least(low), Interval::less_than(low));
            if above.contains(value) == at_most.contains(value) {
                return Err(format!(
                    "{above} and {at_most} overlap or leave a gap at {value:?}"
                ));
            }
            if at_least.contains(value) == below.contains(value) {
                return Err(format!(
                    "{at_least} and {below} overlap or leave a gap at {value:?}"
                ));
            }
            let equal = float::ulp_distance(value, low) == Some(0);
            if at_least.contains(value) != (above.contains(value) || equal) {
                return Err(format!(
                    "{at_least} is not {above} plus {low:?} at {value:?}"
                ));
            }

            let high = finite_value(rng);
            let (low, high) = if low <= high {
                (low, high)
            } else {
                (high, low)
            };
            for lower in [Bound::Included(low), Bound::Excluded(low)] {
                for upper in [Bound::Included(high), Bound::Excluded(high)] {
                    if !Interval::is_valid(lower, upper) {
                        continue;
                    }
                    let both = Interval::new(lower, upper);
                    let left = Interval::new(lower, Bound::Unbounded);
                    let right = Interval::new(Bound::Unbounded, upper);
                    if both.contains(value) != (left.contains(value) && right.contains(value)) {
                        return Err(format!("{both} is not {left} ∩ {right} at {value:?}"));
                    }
                }
            }
            Ok(())
        });
    }

    /// A `fmt::Write` sink that fails as soon as more than `limit` bytes
    /// would have been written to it.
    struct Truncating {
        text: String,
        limit: usize,
    }

    impl fmt::Write for Truncating {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            if self.text.len() + s.len() > self.limit {
                Err(fmt::Error)
            } else {
                self.text.push_str(s);
                Ok(())
            }
        }
    }

    /// Writes `value` into sinks of every length below its full message: the
    /// sink's error must reach the caller each time, and a sink that is long
    /// enough must receive exactly the `to_string` text. This exercises every
    /// early return of the formatting code, and shows that it never panics.
    fn check_formatting_propagates_errors(value: &dyn fmt::Display) {
        let full = value.to_string();
        for limit in 0..full.len() {
            let mut sink = Truncating {
                text: String::new(),
                limit,
            };
            assert!(
                fmt::write(&mut sink, format_args!("{value}")).is_err(),
                "a sink of {limit} bytes must fail for {full:?}"
            );
        }
        let mut sink = Truncating {
            text: String::new(),
            limit: full.len(),
        };
        assert!(fmt::write(&mut sink, format_args!("{value}")).is_ok());
        assert_eq!(sink.text, full);
    }

    #[test]
    fn formatting_propagates_writer_errors() {
        for (error, _) in every_variant() {
            check_formatting_propagates_errors(&error);
            match &error {
                Error::Value(inner) => check_formatting_propagates_errors(inner),
                Error::Identifier(inner) => check_formatting_propagates_errors(inner),
            }
        }
        let three = IdentifierError::Ambiguous {
            concept: "day-count convention",
            text: "30/360".to_owned(),
            candidates: &["30/360 (Bond Basis)", "30E/360", "30E/360 (ISDA)"],
        };
        check_formatting_propagates_errors(&three);
        for class in [
            NonFinite::NaN,
            NonFinite::PositiveInfinity,
            NonFinite::NegativeInfinity,
        ] {
            check_formatting_propagates_errors(&class);
        }
        for interval in [
            Interval::greater_than(-2.5),
            Interval::at_least(0.0),
            Interval::less_than(1e300),
            Interval::at_most(-1e-300),
            Interval::new(Bound::Included(-1.0), Bound::Excluded(1.0)),
            Interval::new(Bound::Excluded(-1.0), Bound::Included(1.0)),
            Interval::new(Bound::Unbounded, Bound::Unbounded),
        ] {
            check_formatting_propagates_errors(&interval);
        }
    }

    /// FR-CORE-02: an error stays inspectable after `?` turns it into a boxed
    /// standard error, as application code does.
    #[test]
    fn errors_stay_inspectable_through_box_dyn_error() {
        for (error, kind) in every_variant() {
            let boxed: Box<dyn StdError + Send + Sync> = error.clone().into();
            assert_eq!(boxed.to_string(), error.to_string());
            let recovered = boxed.downcast_ref::<Error>().expect("an Error inside");
            assert_eq!(recovered, &error);
            assert_eq!(recovered.kind(), kind);
        }

        fn validate(rate: f64) -> Result<f64, ValueError> {
            NonFinite::classify(rate).map_or(Ok(rate), |value| {
                Err(ValueError::NotFinite {
                    parameter: "rate",
                    value,
                })
            })
        }
        fn price(rate: f64) -> Result<f64, Error> {
            Ok(validate(rate)? * 2.0)
        }
        fn application(rate: f64) -> Result<f64, Box<dyn StdError + Send + Sync>> {
            Ok(price(rate)?)
        }
        let boxed = application(f64::NAN).expect_err("NaN is rejected");
        let error = boxed.downcast_ref::<Error>().expect("an Error inside");
        assert_eq!(error.kind(), "value.not_finite");
        assert!(matches!(
            error,
            Error::Value(ValueError::NotFinite {
                parameter: "rate",
                value: NonFinite::NaN
            })
        ));
        assert_eq!(
            application(0.25).expect("finite").to_bits(),
            0.5_f64.to_bits()
        );
    }

    /// Equality compares every field, so tests can match errors exactly.
    /// The two zeros are equal, as everywhere else (NP-08).
    #[test]
    fn equality_compares_every_field() {
        let out_of_domain = |parameter, value, domain| ValueError::OutOfDomain {
            parameter,
            value,
            domain,
        };
        let base = out_of_domain("time", -1.0, Interval::at_least(0.0));
        assert_eq!(base.clone(), base);
        assert_ne!(base, out_of_domain("rate", -1.0, Interval::at_least(0.0)));
        assert_ne!(base, out_of_domain("time", -2.0, Interval::at_least(0.0)));
        assert_ne!(
            base,
            out_of_domain("time", -1.0, Interval::greater_than(0.0))
        );
        assert_ne!(
            base,
            ValueError::ResultOutOfDomain {
                quantity: "time",
                value: -1.0,
                domain: Interval::at_least(0.0),
            }
        );
        assert_eq!(
            out_of_domain("time", -0.0, Interval::greater_than(0.0)),
            out_of_domain("time", 0.0, Interval::greater_than(0.0))
        );
        let not_finite = |value| ValueError::NotFinite {
            parameter: "rate",
            value,
        };
        assert_ne!(
            not_finite(NonFinite::PositiveInfinity),
            not_finite(NonFinite::NegativeInfinity)
        );

        let unknown = |concept, text: &str| IdentifierError::Unknown {
            concept,
            text: text.to_owned(),
        };
        assert_eq!(unknown("frequency", "X"), unknown("frequency", "X"));
        assert_ne!(unknown("frequency", "X"), unknown("frequency", "x"));
        assert_ne!(unknown("frequency", "X"), unknown("time unit", "X"));
        let ambiguous = |candidates: &'static [&'static str]| IdentifierError::Ambiguous {
            concept: "day-count convention",
            text: "Act/Act".to_owned(),
            candidates,
        };
        assert_ne!(
            ambiguous(&["A", "B"]),
            ambiguous(&["B", "A"]),
            "candidate order is significant"
        );
        assert_eq!(Error::from(base.clone()), Error::Value(base));
        assert_ne!(
            Error::from(unknown("frequency", "X")),
            Error::from(not_finite(NonFinite::NaN))
        );
    }

    #[test]
    fn messages_name_the_non_finite_class() {
        for (class, word) in [
            (NonFinite::NaN, "NaN"),
            (NonFinite::PositiveInfinity, "inf"),
            (NonFinite::NegativeInfinity, "-inf"),
        ] {
            let input = ValueError::NotFinite {
                parameter: "rate",
                value: class,
            }
            .to_string();
            let result = ValueError::ResultNotFinite {
                quantity: "compound factor",
                value: class,
            }
            .to_string();
            for message in [input, result] {
                assert!(message.contains(word), "{message}");
                if class == NonFinite::PositiveInfinity {
                    assert!(!message.contains("-inf"), "{message}");
                }
            }
        }
    }

    /// Messages stay on one line: control characters in rejected text are
    /// escaped, so logs cannot be forged or broken by input.
    #[test]
    fn messages_escape_control_characters() {
        let error = IdentifierError::Unknown {
            concept: "frequency",
            text: "Monthly\n\tX\u{0}\u{1b}".to_owned(),
        };
        let message = error.to_string();
        assert!(!message.chars().any(char::is_control), "{message:?}");
        for escape in [r"\n", r"\t", r"\0", r"\u{1b}"] {
            assert!(
                message.contains(escape),
                "{escape} missing from {message:?}"
            );
        }
    }

    /// NP-01: `classify` agrees with the IEEE 754 binary64 encoding for every
    /// sign and every exponent, with mantissas at both ends of their range,
    /// next to the quiet bit, and at random. An all-ones exponent means an
    /// infinity when the mantissa is zero and a NaN otherwise; every other
    /// exponent encodes a finite number (IEEE 754-2008 §3.4).
    #[test]
    fn classification_matches_the_ieee_754_encoding() {
        const MANTISSA: u64 = (1 << 52) - 1;
        let mut rng = SplitMix64::new(0x0754_2008);
        let mut checked = 0_u32;
        for sign in [0_u64, 1] {
            for exponent in 0..=0x7FF_u64 {
                let mut mantissas = vec![0, 1, (1 << 51) - 1, 1 << 51, MANTISSA];
                mantissas.extend((0..8).map(|_| rng.next_u64() & MANTISSA));
                for mantissa in mantissas {
                    let bits = (sign << 63) | (exponent << 52) | mantissa;
                    let expected = match (exponent, mantissa, sign) {
                        (0x7FF, 0, 0) => Some(NonFinite::PositiveInfinity),
                        (0x7FF, 0, _) => Some(NonFinite::NegativeInfinity),
                        (0x7FF, _, _) => Some(NonFinite::NaN),
                        _ => None,
                    };
                    let value = f64::from_bits(bits);
                    assert_eq!(NonFinite::classify(value), expected, "{bits:#018x}");
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, 2 * 2048 * 13);
    }
}
