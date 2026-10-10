//! Canonical identifiers and their aliases (internal).
//!
//! Every enumeration with a text form (frequencies, time units, business-day
//! conventions, day-count conventions, compounding methods, calendars)
//! describes its spellings in one static [`Identifiers`] table
//! ([FS-CORE](https://github.com/iceman951/sabiquant/blob/main/docs/REQUIREMENTS.md#4-fs-core--core-types)):
//!
//! - **FR-CORE-05**: each value has exactly one canonical identifier, which
//!   its `Display` implementation writes, and parsing the canonical identifier
//!   returns the same value.
//! - **FR-CORE-06**: parsing also accepts a documented list of aliases. A label
//!   that names more than one convention, such as `30/360` or
//!   `Actual/Actual`, is listed as ambiguous and is always rejected with its
//!   candidates. It is never mapped to a default.
//!
//! Matching is exact: case-sensitive, with no trimming and no normalization.
//! Every accepted spelling is written in the table, so the documentation of
//! each type can list them all.
//!
//! Each type writes its canonical identifier with an exhaustive `match`, so
//! the compiler checks that every value has one. Its tests then check the
//! table with the test-only methods `Identifiers::check` and
//! `Identifiers::check_round_trip` (TS-CORE-02).
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the enumerations of M2 (time units, frequencies) and later milestones use these tables"
    )
)]

use crate::error::IdentifierError;

/// The spellings of one value.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Entry<T> {
    /// The value.
    pub(crate) value: T,
    /// Its canonical identifier.
    pub(crate) canonical: &'static str,
    /// Its documented aliases.
    pub(crate) aliases: &'static [&'static str],
}

/// A label that could name more than one value, which parsing rejects.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ambiguous {
    /// The ambiguous label.
    pub(crate) label: &'static str,
    /// The canonical identifiers of the values that the label could name.
    pub(crate) candidates: &'static [&'static str],
}

/// The identifier table of one enumeration.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Identifiers<T: 'static> {
    concept: &'static str,
    entries: &'static [Entry<T>],
    ambiguous: &'static [Ambiguous],
}

impl<T: Copy + PartialEq> Identifiers<T> {
    /// Creates the table for `concept`, for example `"frequency"`.
    pub(crate) const fn new(
        concept: &'static str,
        entries: &'static [Entry<T>],
        ambiguous: &'static [Ambiguous],
    ) -> Self {
        Self {
            concept,
            entries,
            ambiguous,
        }
    }

    /// Parses a canonical identifier or a documented alias.
    ///
    /// # Errors
    ///
    /// - [`IdentifierError::Ambiguous`] if `text` is listed as ambiguous;
    /// - [`IdentifierError::Unknown`] if it is not listed at all.
    pub(crate) fn parse(&self, text: &str) -> Result<T, IdentifierError> {
        for entry in self.entries {
            if entry.canonical == text || entry.aliases.contains(&text) {
                return Ok(entry.value);
            }
        }
        match self
            .ambiguous
            .iter()
            .find(|ambiguous| ambiguous.label == text)
        {
            Some(ambiguous) => Err(IdentifierError::Ambiguous {
                concept: self.concept,
                text: text.to_owned(),
                candidates: ambiguous.candidates,
            }),
            None => Err(IdentifierError::Unknown {
                concept: self.concept,
                text: text.to_owned(),
            }),
        }
    }

    /// Returns the canonical identifier of `value`, if the table lists it.
    pub(crate) fn canonical(&self, value: T) -> Option<&'static str> {
        self.entries
            .iter()
            .find(|entry| entry.value == value)
            .map(|entry| entry.canonical)
    }
}

#[cfg(test)]
impl<T: Copy + PartialEq + std::fmt::Debug> Identifiers<T> {
    /// Checks that the table is well formed:
    ///
    /// - it has at least one entry, and no value has two entries;
    /// - every spelling is non-empty, has no surrounding whitespace and no
    ///   control characters (fixture fields are tab-separated);
    /// - no spelling (canonical identifier, alias or ambiguous label) appears
    ///   twice, so every text has at most one meaning;
    /// - every ambiguous label has at least two distinct candidates, each of
    ///   them a canonical identifier of this table.
    pub(crate) fn check(&self) -> Result<(), String> {
        check_spelling(self.concept)?;
        if self.entries.is_empty() {
            return Err(format!("the {} table has no entries", self.concept));
        }
        let mut spellings = Vec::new();
        for (index, entry) in self.entries.iter().enumerate() {
            if self.entries[..index]
                .iter()
                .any(|earlier| earlier.value == entry.value)
            {
                return Err(format!("{:?} has more than one entry", entry.value));
            }
            for spelling in std::iter::once(entry.canonical).chain(entry.aliases.iter().copied()) {
                check_spelling(spelling)?;
                spellings.push(spelling);
            }
        }
        for ambiguous in self.ambiguous {
            check_spelling(ambiguous.label)?;
            spellings.push(ambiguous.label);
            if ambiguous.candidates.len() < 2 {
                return Err(format!(
                    "ambiguous label {:?} needs at least two candidates",
                    ambiguous.label
                ));
            }
            for (index, candidate) in ambiguous.candidates.iter().enumerate() {
                if !self
                    .entries
                    .iter()
                    .any(|entry| entry.canonical == *candidate)
                {
                    return Err(format!(
                        "candidate {candidate:?} of {:?} is not a canonical identifier",
                        ambiguous.label
                    ));
                }
                if ambiguous.candidates[..index].contains(candidate) {
                    return Err(format!(
                        "candidate {candidate:?} of {:?} is listed twice",
                        ambiguous.label
                    ));
                }
            }
        }
        spellings.sort_unstable();
        match spellings.windows(2).find(|pair| pair[0] == pair[1]) {
            Some(pair) => Err(format!("{:?} is listed more than once", pair[0])),
            None => Ok(()),
        }
    }

    /// Checks the round trip of FR-CORE-05 over `all`, the complete list of
    /// values of the type: each value has an entry; `display` writes its
    /// canonical identifier; and parsing the canonical identifier or any alias
    /// returns the value.
    pub(crate) fn check_round_trip(
        &self,
        all: &[T],
        display: impl Fn(T) -> String,
    ) -> Result<(), String> {
        if all.len() != self.entries.len() {
            return Err(format!(
                "{} values but {} table entries",
                all.len(),
                self.entries.len()
            ));
        }
        for &value in all {
            let entry = self
                .entries
                .iter()
                .find(|entry| entry.value == value)
                .ok_or_else(|| format!("{value:?} has no entry"))?;
            let written = display(value);
            if written != entry.canonical {
                return Err(format!(
                    "{value:?} is written as {written:?}, not as its canonical identifier {:?}",
                    entry.canonical
                ));
            }
            if self.canonical(value) != Some(entry.canonical) {
                return Err(format!(
                    "canonical lookup of {value:?} disagrees with its entry"
                ));
            }
            for spelling in std::iter::once(entry.canonical).chain(entry.aliases.iter().copied()) {
                match self.parse(spelling) {
                    Ok(parsed) if parsed == value => {}
                    other => {
                        return Err(format!(
                            "parsing {spelling:?} gave {other:?}, expected {value:?}"
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

/// A spelling is non-empty, has no surrounding whitespace, and has no control
/// characters.
#[cfg(test)]
fn check_spelling(spelling: &str) -> Result<(), String> {
    if spelling.is_empty() {
        Err("empty spelling".to_owned())
    } else if spelling.trim() != spelling {
        Err(format!("{spelling:?} has surrounding whitespace"))
    } else if spelling.chars().any(char::is_control) {
        Err(format!("{spelling:?} contains a control character"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test-only values that mirror the ambiguity examples of FR-CORE-06.
    /// They are not SabiQuant's day-count conventions, which arrive in M3.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Convention {
        ActualActualIsda,
        ActualActualIcma,
        Actual365Fixed,
        Actual360,
    }

    impl Convention {
        const ALL: [Self; 4] = [
            Self::ActualActualIsda,
            Self::ActualActualIcma,
            Self::Actual365Fixed,
            Self::Actual360,
        ];

        fn as_str(self) -> &'static str {
            match self {
                Self::ActualActualIsda => "Actual/Actual (ISDA)",
                Self::ActualActualIcma => "Actual/Actual (ICMA)",
                Self::Actual365Fixed => "Actual/365 (Fixed)",
                Self::Actual360 => "Actual/360",
            }
        }
    }

    const ACT_ACT: &[&str] = &["Actual/Actual (ISDA)", "Actual/Actual (ICMA)"];

    const TABLE: Identifiers<Convention> = Identifiers::new(
        "test convention",
        &[
            Entry {
                value: Convention::ActualActualIsda,
                canonical: "Actual/Actual (ISDA)",
                aliases: &["Act/Act (ISDA)"],
            },
            Entry {
                value: Convention::ActualActualIcma,
                canonical: "Actual/Actual (ICMA)",
                aliases: &["Act/Act (ICMA)", "Actual/Actual (ISMA)"],
            },
            Entry {
                value: Convention::Actual365Fixed,
                canonical: "Actual/365 (Fixed)",
                aliases: &["A/365F"],
            },
            Entry {
                value: Convention::Actual360,
                canonical: "Actual/360",
                aliases: &["Act/360", "A/360"],
            },
        ],
        &[
            Ambiguous {
                label: "Actual/Actual",
                candidates: ACT_ACT,
            },
            Ambiguous {
                label: "Act/Act",
                candidates: ACT_ACT,
            },
            Ambiguous {
                label: "ACT/ACT",
                candidates: ACT_ACT,
            },
            Ambiguous {
                label: "Actual/365",
                candidates: &["Actual/365 (Fixed)", "Actual/Actual (ISDA)"],
            },
        ],
    );

    #[test]
    fn well_formed_table_passes_its_checks() {
        assert_eq!(TABLE.check(), Ok(()));
    }

    /// FR-CORE-05, TS-CORE-02: every value round-trips through its canonical
    /// identifier, and every alias parses to its value.
    #[test]
    fn canonical_identifiers_and_aliases_round_trip() {
        let display = |value: Convention| value.as_str().to_owned();
        assert_eq!(TABLE.check_round_trip(&Convention::ALL, display), Ok(()));
        assert_eq!(TABLE.parse("A/360"), Ok(Convention::Actual360));
        assert_eq!(
            TABLE.parse("Actual/Actual (ISMA)"),
            Ok(Convention::ActualActualIcma)
        );
    }

    /// FR-CORE-06, TS-CORE-01, TS-CORE-02: ambiguous labels are rejected with
    /// the candidates, never mapped to a default.
    #[test]
    fn ambiguous_labels_are_rejected_with_their_candidates() {
        for label in ["Actual/Actual", "Act/Act", "ACT/ACT"] {
            let error = TABLE.parse(label).expect_err("ambiguous");
            assert_eq!(error.kind(), "identifier.ambiguous");
            assert_eq!(
                error,
                IdentifierError::Ambiguous {
                    concept: "test convention",
                    text: label.to_owned(),
                    candidates: ACT_ACT,
                }
            );
        }
        let error = TABLE.parse("Actual/365").expect_err("ambiguous");
        match error {
            IdentifierError::Ambiguous { candidates, .. } => {
                assert_eq!(candidates, ["Actual/365 (Fixed)", "Actual/Actual (ISDA)"]);
            }
            IdentifierError::Unknown { .. } => panic!("expected an ambiguity error"),
        }
    }

    /// FR-CORE-06, TS-CORE-01: matching is exact, so any other spelling,
    /// including a change of case or surrounding whitespace, is unknown.
    #[test]
    fn unlisted_spellings_are_unknown() {
        for text in [
            "Actual/366",
            "actual/360",
            "ACTUAL/360",
            " Actual/360",
            "Actual/360 ",
            "Actual/360\n",
            "Actual/360\t",
            "act/act",
            "",
        ] {
            assert_eq!(
                TABLE.parse(text),
                Err(IdentifierError::Unknown {
                    concept: "test convention",
                    text: text.to_owned(),
                }),
                "{text:?}"
            );
        }
    }

    #[test]
    fn canonical_lookup() {
        assert_eq!(TABLE.canonical(Convention::Actual360), Some("Actual/360"));
        const PARTIAL: Identifiers<Convention> = Identifiers::new(
            "test convention",
            &[Entry {
                value: Convention::Actual360,
                canonical: "Actual/360",
                aliases: &[],
            }],
            &[],
        );
        assert_eq!(PARTIAL.canonical(Convention::Actual365Fixed), None);
    }

    /// The table checks catch every kind of defect.
    #[test]
    fn table_checks_catch_defects() {
        const fn entry(value: Convention, canonical: &'static str) -> Entry<Convention> {
            Entry {
                value,
                canonical,
                aliases: &[],
            }
        }
        const A: Entry<Convention> = entry(Convention::Actual360, "Actual/360");
        const B: Entry<Convention> = entry(Convention::Actual365Fixed, "Actual/365 (Fixed)");
        // A constant, so that the tables are `'static` like real ones.
        const DEFECTIVE: [(&str, Identifiers<Convention>); 12] = [
            ("empty table", Identifiers::new("c", &[], &[])),
            ("empty concept", Identifiers::new("", &[A], &[])),
            (
                "duplicate value",
                Identifiers::new("c", &[A, entry(Convention::Actual360, "X")], &[]),
            ),
            (
                "duplicate canonical",
                Identifiers::new(
                    "c",
                    &[A, entry(Convention::Actual365Fixed, "Actual/360")],
                    &[],
                ),
            ),
            (
                "alias equal to another canonical",
                Identifiers::new(
                    "c",
                    &[
                        A,
                        Entry {
                            value: Convention::Actual365Fixed,
                            canonical: "Actual/365 (Fixed)",
                            aliases: &["Actual/360"],
                        },
                    ],
                    &[],
                ),
            ),
            (
                "ambiguous label equal to a canonical",
                Identifiers::new(
                    "c",
                    &[A, B],
                    &[Ambiguous {
                        label: "Actual/360",
                        candidates: &["Actual/360", "Actual/365 (Fixed)"],
                    }],
                ),
            ),
            (
                "single candidate",
                Identifiers::new(
                    "c",
                    &[A, B],
                    &[Ambiguous {
                        label: "Actual",
                        candidates: &["Actual/360"],
                    }],
                ),
            ),
            (
                "unknown candidate",
                Identifiers::new(
                    "c",
                    &[A, B],
                    &[Ambiguous {
                        label: "Actual",
                        candidates: &["Actual/360", "Actual/364"],
                    }],
                ),
            ),
            (
                "repeated candidate",
                Identifiers::new(
                    "c",
                    &[A, B],
                    &[Ambiguous {
                        label: "Actual",
                        candidates: &["Actual/360", "Actual/360"],
                    }],
                ),
            ),
            (
                "empty spelling",
                Identifiers::new("c", &[entry(Convention::Actual360, "")], &[]),
            ),
            (
                "whitespace",
                Identifiers::new("c", &[entry(Convention::Actual360, "Actual/360 ")], &[]),
            ),
            (
                "control character",
                Identifiers::new("c", &[entry(Convention::Actual360, "Actual\t360")], &[]),
            ),
        ];
        for (defect, table) in DEFECTIVE {
            assert!(table.check().is_err(), "{defect} must be detected");
        }
    }

    /// The round-trip check catches a wrong `Display` and a missing value.
    #[test]
    fn round_trip_check_catches_defects() {
        let lowercase = |value: Convention| value.as_str().to_lowercase();
        assert!(TABLE.check_round_trip(&Convention::ALL, lowercase).is_err());
        let display = |value: Convention| value.as_str().to_owned();
        assert!(
            TABLE
                .check_round_trip(&Convention::ALL[..3], display)
                .is_err()
        );
    }
}
