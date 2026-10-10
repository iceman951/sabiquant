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
    use crate::support::prng::{self, SplitMix64};

    /// Test-only values that mirror the ambiguity examples of FR-CORE-06.
    /// They are not SabiQuant's day-count conventions, which arrive in M3.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Convention {
        ActualActualIsda,
        ActualActualIcma,
        Actual365Fixed,
        Actual360,
        ThirtyBondBasis,
        ThirtyEurobond,
    }

    impl Convention {
        const ALL: [Self; 6] = [
            Self::ActualActualIsda,
            Self::ActualActualIcma,
            Self::Actual365Fixed,
            Self::Actual360,
            Self::ThirtyBondBasis,
            Self::ThirtyEurobond,
        ];

        fn as_str(self) -> &'static str {
            match self {
                Self::ActualActualIsda => "Actual/Actual (ISDA)",
                Self::ActualActualIcma => "Actual/Actual (ICMA)",
                Self::Actual365Fixed => "Actual/365 (Fixed)",
                Self::Actual360 => "Actual/360",
                Self::ThirtyBondBasis => "30/360 (Bond Basis)",
                Self::ThirtyEurobond => "30E/360",
            }
        }
    }

    const ACT_ACT: &[&str] = &["Actual/Actual (ISDA)", "Actual/Actual (ICMA)"];
    const THIRTY: &[&str] = &["30/360 (Bond Basis)", "30E/360"];
    const ACT_365: &[&str] = &["Actual/365 (Fixed)", "Actual/Actual (ISDA)"];

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
            Entry {
                value: Convention::ThirtyBondBasis,
                canonical: "30/360 (Bond Basis)",
                aliases: &["30/360 ISDA", "360/360"],
            },
            Entry {
                value: Convention::ThirtyEurobond,
                canonical: "30E/360",
                aliases: &["30/360 (Eurobond Basis)"],
            },
        ],
        &[
            Ambiguous {
                label: "30/360",
                candidates: THIRTY,
            },
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
                candidates: ACT_365,
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
        // The four labels that FR-CORE-06 names, each with its candidates.
        for (label, candidates) in [
            ("30/360", THIRTY),
            ("Actual/Actual", ACT_ACT),
            ("ACT/ACT", ACT_ACT),
            ("Actual/365", ACT_365),
        ] {
            assert_eq!(
                TABLE.parse(label),
                Err(IdentifierError::Ambiguous {
                    concept: "test convention",
                    text: label.to_owned(),
                    candidates,
                })
            );
            // Every candidate is itself a valid identifier.
            for candidate in candidates {
                assert!(TABLE.parse(candidate).is_ok(), "{candidate}");
            }
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
        const fn with_aliases(
            value: Convention,
            canonical: &'static str,
            aliases: &'static [&'static str],
        ) -> Entry<Convention> {
            Entry {
                value,
                canonical,
                aliases,
            }
        }
        const fn ambiguous(label: &'static str, candidates: &'static [&'static str]) -> Ambiguous {
            Ambiguous { label, candidates }
        }
        // Constants, so that the tables are `'static` like real ones. Each
        // defect comes with a fragment of the report that names it, so the
        // test also checks that the right defect is found.
        const DEFECTIVE: [(&str, Identifiers<Convention>, &str); 20] = [
            ("empty table", Identifiers::new("c", &[], &[]), "no entries"),
            (
                "empty concept",
                Identifiers::new("", &[A], &[]),
                "empty spelling",
            ),
            (
                "concept with whitespace",
                Identifiers::new(" c", &[A], &[]),
                "surrounding whitespace",
            ),
            (
                "duplicate value",
                Identifiers::new("c", &[A, entry(Convention::Actual360, "X")], &[]),
                "more than one entry",
            ),
            (
                "duplicate canonical",
                Identifiers::new(
                    "c",
                    &[A, entry(Convention::Actual365Fixed, "Actual/360")],
                    &[],
                ),
                "listed more than once",
            ),
            (
                "alias equal to another canonical",
                Identifiers::new(
                    "c",
                    &[
                        A,
                        with_aliases(
                            Convention::Actual365Fixed,
                            "Actual/365 (Fixed)",
                            &["Actual/360"],
                        ),
                    ],
                    &[],
                ),
                "listed more than once",
            ),
            (
                "alias equal to its own canonical",
                Identifiers::new(
                    "c",
                    &[with_aliases(
                        Convention::Actual360,
                        "Actual/360",
                        &["Actual/360"],
                    )],
                    &[],
                ),
                "listed more than once",
            ),
            (
                "alias repeated",
                Identifiers::new(
                    "c",
                    &[with_aliases(
                        Convention::Actual360,
                        "Actual/360",
                        &["A/360", "A/360"],
                    )],
                    &[],
                ),
                "listed more than once",
            ),
            (
                "alias with whitespace",
                Identifiers::new(
                    "c",
                    &[with_aliases(
                        Convention::Actual360,
                        "Actual/360",
                        &[" A/360"],
                    )],
                    &[],
                ),
                "surrounding whitespace",
            ),
            (
                "alias with a no-break space",
                Identifiers::new(
                    "c",
                    &[with_aliases(
                        Convention::Actual360,
                        "Actual/360",
                        &["A/360\u{a0}"],
                    )],
                    &[],
                ),
                "surrounding whitespace",
            ),
            (
                "ambiguous label equal to a canonical",
                Identifiers::new(
                    "c",
                    &[A, B],
                    &[ambiguous(
                        "Actual/360",
                        &["Actual/360", "Actual/365 (Fixed)"],
                    )],
                ),
                "listed more than once",
            ),
            (
                "ambiguous label listed twice",
                Identifiers::new(
                    "c",
                    &[A, B],
                    &[
                        ambiguous("Actual", &["Actual/360", "Actual/365 (Fixed)"]),
                        ambiguous("Actual", &["Actual/365 (Fixed)", "Actual/360"]),
                    ],
                ),
                "listed more than once",
            ),
            (
                "no candidates",
                Identifiers::new("c", &[A, B], &[ambiguous("Actual", &[])]),
                "at least two candidates",
            ),
            (
                "single candidate",
                Identifiers::new("c", &[A, B], &[ambiguous("Actual", &["Actual/360"])]),
                "at least two candidates",
            ),
            (
                "unknown candidate",
                Identifiers::new(
                    "c",
                    &[A, B],
                    &[ambiguous("Actual", &["Actual/360", "Actual/364"])],
                ),
                "is not a canonical identifier",
            ),
            (
                "candidate that is only an alias",
                Identifiers::new(
                    "c",
                    &[
                        with_aliases(Convention::Actual360, "Actual/360", &["A/360"]),
                        B,
                    ],
                    &[ambiguous("Actual", &["A/360", "Actual/365 (Fixed)"])],
                ),
                "is not a canonical identifier",
            ),
            (
                "repeated candidate",
                Identifiers::new(
                    "c",
                    &[A, B],
                    &[ambiguous("Actual", &["Actual/360", "Actual/360"])],
                ),
                "listed twice",
            ),
            (
                "label with a control character",
                Identifiers::new(
                    "c",
                    &[A, B],
                    &[ambiguous("Act\u{0}", &["Actual/360", "Actual/365 (Fixed)"])],
                ),
                "control character",
            ),
            (
                "empty spelling",
                Identifiers::new("c", &[entry(Convention::Actual360, "")], &[]),
                "empty spelling",
            ),
            (
                "tab in a spelling",
                Identifiers::new("c", &[entry(Convention::Actual360, "Actual\t360")], &[]),
                "control character",
            ),
        ];
        for (defect, table, fragment) in DEFECTIVE {
            let report = table.check().expect_err(defect);
            assert!(report.contains(fragment), "{defect}: {report}");
        }
        // Non-ASCII spellings are allowed, as long as they are well formed.
        const ACCENTED: Identifiers<Convention> = Identifiers::new(
            "c",
            &[with_aliases(
                Convention::Actual360,
                "Actual/360",
                &["Exact/360 (Café)"],
            )],
            &[],
        );
        assert_eq!(ACCENTED.check(), Ok(()));
        // The helper runs at run time too, not only in constants.
        assert_eq!(entry(Convention::Actual360, "x").canonical, "x");
    }

    /// The round-trip check catches every kind of defect that `check` does
    /// not: a wrong `Display`, a missing or surplus value, and a spelling that
    /// parses to another value because an earlier entry claims it.
    #[test]
    fn round_trip_check_catches_defects() {
        let display = |value: Convention| value.as_str().to_owned();
        let report = TABLE
            .check_round_trip(&Convention::ALL, |value| value.as_str().to_lowercase())
            .expect_err("lowercase display");
        assert!(
            report.contains("not as its canonical identifier"),
            "{report}"
        );
        let report = TABLE
            .check_round_trip(&Convention::ALL[..3], display)
            .expect_err("missing value");
        assert!(report.contains("table entries"), "{report}");

        const PARTIAL: Identifiers<Convention> = Identifiers::new(
            "c",
            &[
                Entry {
                    value: Convention::ActualActualIsda,
                    canonical: "Actual/Actual (ISDA)",
                    aliases: &[],
                },
                Entry {
                    value: Convention::Actual365Fixed,
                    canonical: "Actual/365 (Fixed)",
                    aliases: &[],
                },
            ],
            &[],
        );
        let all = [Convention::ActualActualIsda, Convention::Actual360];
        let report = PARTIAL
            .check_round_trip(&all, display)
            .expect_err("a value without an entry");
        assert!(report.contains("has no entry"), "{report}");

        const SHADOWED: Identifiers<Convention> = Identifiers::new(
            "c",
            &[
                Entry {
                    value: Convention::Actual360,
                    canonical: "Actual/360",
                    aliases: &["Actual/365 (Fixed)"],
                },
                Entry {
                    value: Convention::Actual365Fixed,
                    canonical: "Actual/365 (Fixed)",
                    aliases: &[],
                },
            ],
            &[],
        );
        let all = [Convention::Actual360, Convention::Actual365Fixed];
        let report = SHADOWED
            .check_round_trip(&all, display)
            .expect_err("a shadowed spelling");
        assert!(report.contains("expected Actual365Fixed"), "{report}");
        assert!(SHADOWED.check().is_err(), "`check` reports it too");
    }

    /// A table built at run time behaves like the same table built as a
    /// constant.
    #[test]
    fn construction_at_run_time_matches_construction_in_a_constant() {
        let table = Identifiers::new(TABLE.concept, TABLE.entries, TABLE.ambiguous);
        assert_eq!(table.check(), Ok(()));
        for text in spellings()
            .into_iter()
            .map(|(text, _)| text)
            .chain(["", "x"])
        {
            assert_eq!(table.parse(text), TABLE.parse(text), "{text:?}");
        }
    }

    /// FR-CORE-06: matching is exact over Unicode too. Look-alike characters,
    /// other normalization forms and other spaces are different text.
    #[test]
    fn matching_is_exact_for_unicode_text() {
        const ACCENTED: Identifiers<Convention> = Identifiers::new(
            "test convention",
            &[Entry {
                value: Convention::Actual360,
                canonical: "Actual/360",
                aliases: &["Exact/360 (Caf\u{e9})"],
            }],
            &[],
        );
        assert_eq!(
            ACCENTED.parse("Exact/360 (Caf\u{e9})"),
            Ok(Convention::Actual360)
        );
        for look_alike in [
            "Exact/360 (Cafe\u{301})",      // decomposed: e + combining acute accent
            "Exact/360\u{a0}(Caf\u{e9})",   // no-break space
            "Exact\u{2215}360 (Caf\u{e9})", // division slash
            "\u{ff25}xact/360 (Caf\u{e9})", // fullwidth E
            "Exact/360 (CAF\u{c9})",
        ] {
            assert_eq!(
                ACCENTED.parse(look_alike),
                Err(IdentifierError::Unknown {
                    concept: "test convention",
                    text: look_alike.to_owned(),
                }),
                "{look_alike:?}"
            );
        }
    }

    /// Every listed spelling of `TABLE` with its meaning: a value, or the
    /// candidates of an ambiguous label.
    fn spellings() -> Vec<(&'static str, Result<Convention, &'static [&'static str]>)> {
        let mut all = Vec::new();
        for entry in TABLE.entries {
            all.push((entry.canonical, Ok(entry.value)));
            for alias in entry.aliases {
                all.push((*alias, Ok(entry.value)));
            }
        }
        for ambiguous in TABLE.ambiguous {
            all.push((ambiguous.label, Err(ambiguous.candidates)));
        }
        all
    }

    /// Characters that the fuzzing draws from: those of the spellings, plus
    /// whitespace, control, non-ASCII and look-alike characters.
    const ALPHABET: [char; 24] = [
        'A',
        'a',
        'c',
        't',
        'u',
        'l',
        'C',
        'T',
        '/',
        '(',
        ')',
        ' ',
        '3',
        '0',
        '6',
        'E',
        'I',
        '\t',
        '\n',
        '\0',
        '\u{a0}',
        '\u{e9}',
        '\u{ff0f}',
        '\u{1f600}',
    ];

    /// Text to parse: a listed spelling, a spelling with a few edits, or
    /// random characters.
    fn fuzz_text(rng: &mut SplitMix64, listed: &[&str]) -> String {
        let mut chars: Vec<char> = match rng.below(3) {
            0 => return (*rng.choose(listed)).to_owned(),
            1 => rng.choose(listed).chars().collect(),
            _ => Vec::new(),
        };
        let edits = 1 + rng.below(3);
        for _ in 0..edits {
            let length = u64::try_from(chars.len()).expect("short text");
            let position = usize::try_from(rng.below(length + 1)).expect("short text");
            match rng.below(4) {
                0 => chars.insert(position, *rng.choose(&ALPHABET)),
                1 if position < chars.len() => {
                    chars.remove(position);
                }
                2 if position < chars.len() => chars[position] = *rng.choose(&ALPHABET),
                3 if position < chars.len() => {
                    let c = chars[position];
                    chars[position] = if c.is_ascii_lowercase() {
                        c.to_ascii_uppercase()
                    } else {
                        c.to_ascii_lowercase()
                    };
                }
                _ => chars.push(*rng.choose(&ALPHABET)),
            }
        }
        chars.into_iter().collect()
    }

    /// FR-CORE-06, NFR-02: parsing is total and exact. For any text, listed
    /// or not, ASCII or not, it never panics, and it returns exactly the
    /// listed meaning: the value, the ambiguity with its candidates, or
    /// "unknown", always with the text unchanged.
    #[test]
    fn parsing_is_total_and_exact() {
        let listed = spellings();
        let texts: Vec<&str> = listed.iter().map(|(text, _)| *text).collect();
        // How often each outcome occurs, so the test can show that it
        // exercises all three.
        let mut outcomes = [0_u32; 3];
        prng::check("identifier parsing", 0x1D00_0001, 30_000, |rng| {
            let text = fuzz_text(rng, &texts);
            let expected = match listed.iter().find(|(spelling, _)| *spelling == text) {
                Some((_, Ok(value))) => Ok(*value),
                Some((_, Err(candidates))) => Err(IdentifierError::Ambiguous {
                    concept: "test convention",
                    text: text.clone(),
                    candidates,
                }),
                None => Err(IdentifierError::Unknown {
                    concept: "test convention",
                    text: text.clone(),
                }),
            };
            outcomes[match expected {
                Ok(_) => 0,
                Err(IdentifierError::Ambiguous { .. }) => 1,
                Err(IdentifierError::Unknown { .. }) => 2,
            }] += 1;
            match std::panic::catch_unwind(|| TABLE.parse(&text)) {
                Ok(actual) if actual == expected => Ok(()),
                Ok(actual) => Err(format!("{text:?}: got {actual:?}, want {expected:?}")),
                Err(_) => Err(format!("parsing {text:?} panicked")),
            }
        });
        for (outcome, count) in ["values", "ambiguities", "unknown texts"]
            .iter()
            .zip(outcomes)
        {
            assert!(count >= 1_000, "only {count} cases produced {outcome}");
        }
    }

    /// NFR-03, NFR-05: tables are immutable shared values. Parsing from four
    /// threads at once gives the same results as on the calling thread.
    #[test]
    fn parsing_is_deterministic_across_threads() {
        fn assert_shareable<T: Copy + Send + Sync + 'static>() {}
        assert_shareable::<Identifiers<Convention>>();
        assert_shareable::<Entry<Convention>>();
        assert_shareable::<Ambiguous>();

        let listed = spellings();
        let texts: Vec<&str> = listed.iter().map(|(text, _)| *text).collect();
        let mut rng = SplitMix64::new(0x1D00_0002);
        let inputs: Vec<String> = (0..5_000).map(|_| fuzz_text(&mut rng, &texts)).collect();
        let run = |inputs: &[String]| -> Vec<Result<Convention, IdentifierError>> {
            inputs.iter().map(|text| TABLE.parse(text)).collect()
        };
        let sequential = run(&inputs);
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..4).map(|_| scope.spawn(|| run(&inputs))).collect();
            for handle in handles {
                assert_eq!(handle.join().expect("no thread panics"), sequential);
            }
        });
    }
}
