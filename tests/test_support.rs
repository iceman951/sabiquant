//! Self-tests of the shared test support (F-CORE-05, F-CORE-06;
//! TESTING_STRATEGY §14, M1 exit criteria): the PRNG and the ULP support are
//! tested, and the fixture reader rejects malformed input with typed errors.
//! Besides examples, the tests use known-answer vectors, independent
//! oracles, exhaustive windows, generated fixtures, and fuzzing.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers report failure by panicking, like the tests themselves"
)]

mod support;

use support::fixture::{Fixture, FixtureErrorKind as Kind, REQUIRED_KEYS};
use support::float::{Tolerance, assert_close, ulp_distance};
use support::prng::{self, SplitMix64};

// ---------------------------------------------------------------------------
// PRNG (F-CORE-05)
// ---------------------------------------------------------------------------

/// Known answers. Seed 1234567: the published SplitMix64 test vector
/// (Rosetta Code, "Pseudo-random numbers/Splitmix64"). Both sequences were
/// also recomputed from Vigna's public-domain C reference implementation.
#[test]
fn splitmix64_matches_the_reference_outputs() {
    let mut rng = SplitMix64::new(1_234_567);
    let expected = [
        6_457_827_717_110_365_317,
        3_203_168_211_198_807_973,
        9_817_491_932_198_370_423,
        4_593_380_528_125_082_431,
        16_408_922_859_458_223_821,
    ];
    for value in expected {
        assert_eq!(rng.next_u64(), value);
    }
    let mut rng = SplitMix64::new(0);
    for value in [
        0xE220_A839_7B1D_CDAF,
        0x6E78_9E6A_A1B9_65F4,
        0x06C4_5D18_8009_454F,
        0xF88B_B8A8_724C_81EC,
    ] {
        assert_eq!(rng.next_u64(), value);
    }
}

/// NFR-03: the same seed gives the same sequence, and a clone continues it.
#[test]
fn splitmix64_is_deterministic() {
    let mut first = SplitMix64::new(42);
    let mut second = SplitMix64::new(42);
    for _ in 0..1_000 {
        assert_eq!(first.next_u64(), second.next_u64());
    }
    let mut clone = first.clone();
    for _ in 0..1_000 {
        assert_eq!(first.next_u64(), clone.next_u64());
    }
}

#[test]
fn case_seed_is_the_nth_output_of_the_run_seed() {
    let mut rng = SplitMix64::new(0xDEAD_BEEF);
    for case in 0..100 {
        assert_eq!(prng::case_seed(0xDEAD_BEEF, case), rng.next_u64());
    }
}

#[test]
fn next_f64_is_in_the_unit_interval_with_53_bit_resolution() {
    let scale = 9_007_199_254_740_992.0; // 2^53
    let mut rng = SplitMix64::new(7);
    for _ in 0..100_000 {
        let x = rng.next_f64();
        assert!((0.0..1.0).contains(&x), "{x}");
        let scaled = x * scale;
        assert_eq!(
            scaled.fract().to_bits(),
            0.0_f64.to_bits(),
            "{x} is a multiple of 2^-53"
        );
    }
}

#[test]
fn below_stays_in_range_and_reaches_every_value() {
    let mut rng = SplitMix64::new(11);
    assert_eq!(rng.below(1), 0);
    let mut counts = [0_u32; 10];
    for _ in 0..100_000 {
        let value = rng.below(10);
        counts[usize::try_from(value).expect("small")] += 1;
    }
    // A loose, deterministic uniformity check: each bucket expects 10,000.
    for (value, count) in counts.into_iter().enumerate() {
        assert!(
            (9_500..=10_500).contains(&count),
            "value {value} drawn {count} times"
        );
    }
    for _ in 0..1_000 {
        assert!(rng.below(u64::MAX) < u64::MAX);
        assert!(rng.below(3) < 3);
    }
}

#[test]
#[should_panic(expected = "the bound must be positive")]
fn below_zero_is_a_test_defect() {
    let _ = SplitMix64::new(0).below(0);
}

#[test]
fn range_i64_includes_both_ends() {
    let mut rng = SplitMix64::new(13);
    let mut seen = [false; 5];
    for _ in 0..1_000 {
        let value = rng.range_i64(-2, 2);
        assert!((-2..=2).contains(&value));
        seen[usize::try_from(value + 2).expect("in range")] = true;
    }
    assert!(seen.iter().all(|&s| s), "every value in [-2, 2] appears");
    assert_eq!(rng.range_i64(5, 5), 5);
    // The full range uses every bit pattern, so it cannot fail.
    for _ in 0..1_000 {
        let _ = rng.range_i64(i64::MIN, i64::MAX);
    }
    let near_max = rng.range_i64(i64::MAX - 1, i64::MAX);
    assert!(near_max >= i64::MAX - 1);
}

#[test]
#[should_panic(expected = "empty range")]
fn reversed_range_is_a_test_defect() {
    let _ = SplitMix64::new(0).range_i64(1, 0);
}

#[test]
fn choose_reaches_every_item() {
    let items = ["a", "b", "c"];
    let mut rng = SplitMix64::new(17);
    let mut seen = [false; 3];
    for _ in 0..300 {
        let item = *rng.choose(&items);
        let index = items
            .iter()
            .position(|&i| i == item)
            .expect("an item of the slice");
        seen[index] = true;
    }
    assert_eq!(seen, [true; 3]);
}

#[test]
fn any_f64_produces_every_class_of_value() {
    // Arbitrary bit patterns give a NaN, and a subnormal, about once in 2048
    // draws. An infinity (2 patterns in 2^64) needs no sampling: it is one of
    // the bit patterns by construction.
    let mut rng = SplitMix64::new(19);
    let (mut nan, mut subnormal, mut normal, mut negative) = (false, false, false, false);
    for _ in 0..100_000 {
        let x = rng.any_f64();
        nan |= x.is_nan();
        subnormal |= x.is_subnormal();
        normal |= x.is_normal();
        negative |= x.is_sign_negative();
    }
    assert!(nan && subnormal && normal && negative);
}

/// The runner gives each case the generator seeded with its case seed.
#[test]
fn check_runs_every_case_with_its_case_seed() {
    let mut firsts = Vec::new();
    prng::check("records first outputs", 99, 25, |rng| {
        firsts.push(rng.next_u64());
        Ok(())
    });
    assert_eq!(firsts.len(), 25);
    for (case, first) in (0_u64..).zip(firsts) {
        assert_eq!(first, SplitMix64::new(prng::case_seed(99, case)).next_u64());
    }
}

/// A failure report names the property, the case and both seeds
/// (TESTING_STRATEGY §5).
#[test]
#[should_panic(
    expected = "property `fails on the third case` failed at case 2 of 5 (run seed 0x0000000000000007, case seed 0x"
)]
fn check_reports_the_failing_case() {
    let mut case = 0;
    prng::check("fails on the third case", 7, 5, |_| {
        case += 1;
        if case == 3 {
            Err("boom".to_owned())
        } else {
            Ok(())
        }
    });
}

// ---------------------------------------------------------------------------
// ULP and tolerance comparison (F-CORE-05; NUMERICAL_CORRECTNESS §7.2)
// ---------------------------------------------------------------------------

#[test]
fn ulp_distance_known_values() {
    let tiny = f64::from_bits(1);
    assert_eq!(ulp_distance(1.0, 1.0), Some(0));
    assert_eq!(ulp_distance(0.0, -0.0), Some(0));
    assert_eq!(ulp_distance(1.0, 1.0_f64.next_up()), Some(1));
    assert_eq!(ulp_distance(1.0, 1.0_f64.next_down()), Some(1));
    assert_eq!(ulp_distance(tiny, -tiny), Some(2));
    assert_eq!(ulp_distance(0.0, tiny), Some(1));
    assert_eq!(ulp_distance(f64::MAX, f64::INFINITY), Some(1));
    assert_eq!(
        ulp_distance(f64::NEG_INFINITY, f64::INFINITY),
        Some(2 * 0x7FF0_0000_0000_0000)
    );
    assert_eq!(ulp_distance(f64::NAN, 1.0), None);
    assert_eq!(ulp_distance(1.0, f64::NAN), None);
    // 0.1 + 0.2 lands one ULP above 0.3.
    assert_eq!(ulp_distance(0.1 + 0.2, 0.3), Some(1));
}

/// ULP distance is symmetric, counts neighbors as 1, and adds up along a
/// chain of neighbors, for arbitrary non-NaN values.
#[test]
fn ulp_distance_properties() {
    prng::check("ulp distance", 0x0F10_A700, 20_000, |rng| {
        let a = rng.any_f64();
        let b = rng.any_f64();
        if a.is_nan() || b.is_nan() {
            return Ok(());
        }
        if ulp_distance(a, b) != ulp_distance(b, a) {
            return Err(format!("not symmetric for {a:?}, {b:?}"));
        }
        if a.is_finite() {
            let up = a.next_up();
            let up_up = up.next_up();
            if ulp_distance(a, up) != Some(1) {
                return Err(format!("{a:?} and its successor are not 1 apart"));
            }
            if a < f64::MAX && up < f64::MAX && ulp_distance(a, up_up) != Some(2) {
                return Err(format!("{a:?} and its second successor are not 2 apart"));
            }
        }
        Ok(())
    });
}

#[test]
fn tolerance_rules() {
    let next = 1.0_f64.next_up();
    // Bitwise: the sign of zero matters; identical NaN bits are identical.
    assert!(Tolerance::Bitwise.check(1.0, 1.0).is_ok());
    assert!(Tolerance::Bitwise.check(0.0, -0.0).is_err());
    assert!(Tolerance::Bitwise.check(f64::NAN, f64::NAN).is_ok());
    // ULPs: TOL-E1 is Ulps(0), which equates the zeros.
    assert!(Tolerance::Ulps(0).check(0.0, -0.0).is_ok());
    assert!(Tolerance::Ulps(0).check(next, 1.0).is_err());
    assert!(Tolerance::Ulps(1).check(next, 1.0).is_ok());
    assert!(Tolerance::Ulps(u64::MAX).check(f64::NAN, f64::NAN).is_err());
    // Mixed: the absolute bound governs near zero, the relative one far away.
    let mixed = Tolerance::Mixed {
        absolute: 1e-12,
        relative: 1e-9,
    };
    assert!(mixed.check(1e-13, 0.0).is_ok());
    assert!(mixed.check(1e-11, 0.0).is_err());
    assert!(mixed.check(1e6 + 1e-4, 1e6).is_ok());
    assert!(mixed.check(1e6 + 1e-2, 1e6).is_err());
    assert!(mixed.check(f64::NAN, 0.0).is_err());
    assert!(mixed.check(f64::INFINITY, f64::INFINITY).is_err());
}

/// The bounds of `Mixed` are inclusive, as in the formula of
/// NUMERICAL_CORRECTNESS §7.2. These differences are exact in binary, so
/// they land exactly on the bound.
#[test]
fn mixed_bounds_are_inclusive() {
    let absolute = Tolerance::Mixed {
        absolute: 0.5,
        relative: 0.0,
    };
    assert!(
        absolute.check(1.5, 1.0).is_ok(),
        "|1.5 - 1| = 0.5 is on the bound"
    );
    assert!(absolute.check(1.5_f64.next_up(), 1.0).is_err());
    let relative = Tolerance::Mixed {
        absolute: 0.0,
        relative: 0.25,
    };
    // |2 - 1.5| = 0.5 = 0.25 * max(2, 1.5).
    assert!(relative.check(2.0, 1.5).is_ok());
    assert!(relative.check(2.0, 1.5_f64.next_down()).is_err());
    let exact = Tolerance::Mixed {
        absolute: 0.0,
        relative: 0.0,
    };
    assert!(
        exact.check(-0.0, 0.0).is_ok(),
        "zero bounds still accept equal values"
    );
    assert!(exact.check(1.0_f64.next_up(), 1.0).is_err());
}

#[test]
fn mismatch_report_has_the_diagnostics() {
    let report = Tolerance::Ulps(0)
        .check(1.0_f64.next_up(), 1.0)
        .expect_err("one ULP apart");
    for part in [
        "1.0000000000000002",
        "0x3ff0000000000001",
        "0x3ff0000000000000",
        "1 ULPs apart",
    ] {
        assert!(report.contains(part), "{part:?} missing from {report:?}");
    }
    assert!(report.contains(std::env::consts::OS));
}

#[test]
#[should_panic(expected = "values differ beyond Ulps(0)")]
fn assert_close_panics_with_the_report() {
    assert_close(0.1 + 0.2, 0.3, Tolerance::Ulps(0));
}

#[test]
#[should_panic(expected = "invalid tolerance")]
fn negative_mixed_bound_is_a_test_defect() {
    let _ = Tolerance::Mixed {
        absolute: -1.0,
        relative: 0.0,
    }
    .check(0.0, 0.0);
}

// ---------------------------------------------------------------------------
// Fixture reader (F-CORE-06; TESTING_STRATEGY §7.3)
// ---------------------------------------------------------------------------

/// A complete, valid header (12 lines with the separator).
const HEADER: &str = "schema: 1\n\
fixture-set: TEST\n\
domain: test-support\n\
description: Reader test\n\
reference: synthetic\n\
reference-version: not applicable\n\
reference-source: not applicable\n\
tolerance: TOL-E0\n\
reproduction: not applicable\n\
generated-at: 2026-10-10T00:00:00Z\n\
platform: any\n\
\n";

/// The line number of the column line after `HEADER`.
const COLUMN_LINE: usize = 13;

fn with_table(table: &str) -> String {
    format!("{HEADER}{table}")
}

const TABLE: &str = "id\tstart\tvalue\tbits\tcount\texpected_error\n\
A-1\t2024-01-01\t0.5\t0x3fe0000000000000\t7\t\n\
A-2\t2024-02-29\t-0.0\t0x8000000000000000\t-3\tvalue.not_finite\n\
A-3\t2024-03-31\t1e-07\t\t\t\n";

#[test]
fn valid_fixture_is_read() {
    let fixture = Fixture::parse(&with_table(TABLE)).expect("valid fixture");
    assert_eq!(fixture.origin(), "<inline>");
    for key in REQUIRED_KEYS {
        assert!(fixture.header(key).is_some(), "{key}");
    }
    assert_eq!(fixture.header("schema"), Some("1"));
    assert_eq!(fixture.header("tolerance"), Some("TOL-E0"));
    assert_eq!(fixture.header("notes"), None);
    assert_eq!(
        fixture.columns(),
        ["id", "start", "value", "bits", "count", "expected_error"]
    );
    assert_eq!(fixture.record_count(), 3);

    let records: Vec<_> = fixture.records().collect();
    let [first, second, third] = records.as_slice() else {
        panic!("three records");
    };
    assert_eq!(first.id(), "A-1");
    assert_eq!(first.line(), COLUMN_LINE + 1);
    assert_eq!(third.line(), COLUMN_LINE + 3);
    assert_eq!(first.text("start").expect("column"), "2024-01-01");
    assert_eq!(first.optional_text("expected_error").expect("column"), None);
    assert_eq!(
        second.optional_text("expected_error").expect("column"),
        Some("value.not_finite")
    );
    assert_eq!(first.parse::<i64>("count").expect("integer"), 7);
    assert_eq!(second.parse::<i64>("count").expect("integer"), -3);
    assert_eq!(third.parse_optional::<i64>("count").expect("empty"), None);
    assert_eq!(
        first.parse_optional::<i64>("count").expect("integer"),
        Some(7)
    );

    assert_eq!(
        first.real("value").expect("real").to_bits(),
        0.5_f64.to_bits()
    );
    assert_eq!(
        second.real("value").expect("real").to_bits(),
        (-0.0_f64).to_bits()
    );
    assert_eq!(
        third.real("value").expect("real").to_bits(),
        1e-7_f64.to_bits()
    );
    assert_eq!(
        first.real_bits("bits").expect("bits").to_bits(),
        0.5_f64.to_bits()
    );
    assert_eq!(
        second.real_bits("bits").expect("bits").to_bits(),
        0x8000_0000_0000_0000
    );
}

/// Rule 1: LF and CRLF line endings read the same, and the final line ending
/// is optional.
#[test]
fn line_endings_do_not_matter() {
    let lf = with_table(TABLE);
    let crlf = lf.replace('\n', "\r\n");
    let unterminated = lf.trim_end_matches('\n').to_owned();
    let read = |text: &str| {
        let fixture = Fixture::parse(text).expect("valid fixture");
        let records: Vec<(String, usize, String)> = fixture
            .records()
            .map(|r| {
                (
                    r.id().to_owned(),
                    r.line(),
                    r.text("expected_error").expect("column").to_owned(),
                )
            })
            .collect();
        (fixture.columns().to_vec(), records)
    };
    assert_eq!(read(&lf), read(&crlf));
    assert_eq!(read(&lf), read(&unterminated));
}

#[test]
fn extra_header_keys_are_kept() {
    let text = HEADER.replacen("platform: any\n", "platform: any\nnotes: a: b\n", 1);
    let fixture = Fixture::parse(&format!("{text}id\nX\n")).expect("valid fixture");
    assert_eq!(fixture.header("notes"), Some("a: b"));
}

/// Every rule violation gives its typed error at the offending line.
#[test]
fn malformed_fixtures_are_rejected_with_typed_errors() {
    let header_without = |key: &str| {
        HEADER
            .lines()
            .filter(|line| !line.starts_with(&format!("{key}: ")))
            .map(|line| format!("{line}\n"))
            .collect::<String>()
    };
    let cases: Vec<(&str, String, Kind, Option<usize>)> = vec![
        (
            "byte-order mark",
            format!("\u{feff}{}", with_table(TABLE)),
            Kind::ByteOrderMark,
            Some(1),
        ),
        ("empty text", String::new(), Kind::MissingSchema, Some(1)),
        (
            "empty first line",
            format!("\n{}", with_table(TABLE)),
            Kind::MissingSchema,
            Some(1),
        ),
        (
            "schema not first",
            with_table(TABLE).replacen("schema: 1\n", "", 1),
            Kind::MissingSchema,
            Some(1),
        ),
        (
            "schema without space",
            with_table(TABLE).replacen("schema: 1", "schema:1", 1),
            Kind::MissingSchema,
            Some(1),
        ),
        (
            "unsupported schema",
            with_table(TABLE).replacen("schema: 1", "schema: 2", 1),
            Kind::UnsupportedSchema("2".to_owned()),
            Some(1),
        ),
        (
            "uppercase key",
            with_table(TABLE).replacen("domain:", "Domain:", 1),
            Kind::MalformedHeaderLine,
            Some(3),
        ),
        (
            "empty value",
            with_table(TABLE).replacen("domain: test-support", "domain: ", 1),
            Kind::MalformedHeaderLine,
            Some(3),
        ),
        (
            "trailing space",
            with_table(TABLE).replacen("test-support", "test-support ", 1),
            Kind::MalformedHeaderLine,
            Some(3),
        ),
        (
            "no colon",
            with_table(TABLE).replacen("domain: test-support", "domain test-support", 1),
            Kind::MalformedHeaderLine,
            Some(3),
        ),
        (
            "tab in header",
            with_table(TABLE).replacen("domain: test", "domain: te\tst", 1),
            Kind::ControlCharacter,
            Some(3),
        ),
        (
            "duplicate key",
            with_table(TABLE).replacen("platform: any\n", "platform: any\ndomain: again\n", 1),
            Kind::DuplicateHeaderKey("domain".to_owned()),
            Some(12),
        ),
        (
            "missing key",
            format!("{}{TABLE}", header_without("platform")),
            Kind::MissingHeaderKey("platform"),
            None,
        ),
        (
            "no separator",
            HEADER.trim_end_matches('\n').to_owned(),
            Kind::MissingSeparator,
            None,
        ),
        ("no columns", HEADER.to_owned(), Kind::MissingColumns, None),
        (
            "two empty lines",
            with_table(&format!("\n{TABLE}")),
            Kind::EmptyLine,
            Some(COLUMN_LINE),
        ),
        (
            "uppercase column",
            with_table("id\tValue\nA\t1\n"),
            Kind::MalformedColumnName("Value".to_owned()),
            Some(COLUMN_LINE),
        ),
        (
            "hyphenated column",
            with_table("id\tmy-value\nA\t1\n"),
            Kind::MalformedColumnName("my-value".to_owned()),
            Some(COLUMN_LINE),
        ),
        (
            "empty column",
            with_table("id\t\tvalue\nA\t1\t2\n"),
            Kind::MalformedColumnName(String::new()),
            Some(COLUMN_LINE),
        ),
        (
            "duplicate column",
            with_table("id\tvalue\tvalue\nA\t1\t2\n"),
            Kind::DuplicateColumn("value".to_owned()),
            Some(COLUMN_LINE),
        ),
        (
            "first column not id",
            with_table("value\tid\n1\tA\n"),
            Kind::FirstColumnNotId("value".to_owned()),
            Some(COLUMN_LINE),
        ),
        (
            "empty line in table",
            with_table("id\tvalue\nA\t1\n\nB\t2\n"),
            Kind::EmptyLine,
            Some(COLUMN_LINE + 2),
        ),
        (
            "trailing empty line",
            with_table("id\tvalue\nA\t1\n\n"),
            Kind::EmptyLine,
            Some(COLUMN_LINE + 2),
        ),
        (
            "too few fields",
            with_table("id\tvalue\tother\nA\t1\n"),
            Kind::FieldCount {
                expected: 3,
                found: 2,
            },
            Some(COLUMN_LINE + 1),
        ),
        (
            "too many fields",
            with_table("id\tvalue\nA\t1\t2\n"),
            Kind::FieldCount {
                expected: 2,
                found: 3,
            },
            Some(COLUMN_LINE + 1),
        ),
        (
            "trailing tab",
            with_table("id\tvalue\nA\t1\t\n"),
            Kind::FieldCount {
                expected: 2,
                found: 3,
            },
            Some(COLUMN_LINE + 1),
        ),
        (
            "empty id",
            with_table("id\tvalue\n\t1\n"),
            Kind::EmptyId,
            Some(COLUMN_LINE + 1),
        ),
        (
            "duplicate id",
            with_table("id\tvalue\nA\t1\nB\t2\nA\t3\n"),
            Kind::DuplicateId {
                id: "A".to_owned(),
                first_line: COLUMN_LINE + 1,
            },
            Some(COLUMN_LINE + 3),
        ),
        (
            "no records",
            with_table("id\tvalue\n"),
            Kind::NoRecords,
            None,
        ),
        (
            "NUL in record",
            with_table("id\tvalue\nA\t1\u{0}\n"),
            Kind::ControlCharacter,
            Some(COLUMN_LINE + 1),
        ),
        (
            "lone carriage return",
            with_table("id\tvalue\nA\t1\rB\n"),
            Kind::ControlCharacter,
            Some(COLUMN_LINE + 1),
        ),
        (
            "control in columns",
            with_table("id\tval\u{7f}\nA\t1\n"),
            Kind::ControlCharacter,
            Some(COLUMN_LINE),
        ),
    ];
    for (name, text, kind, line) in cases {
        let error = Fixture::parse(&text).expect_err(name);
        assert_eq!(error.kind(), &kind, "{name}: {error}");
        assert_eq!(error.line(), line, "{name}: {error}");
        assert!(!error.to_string().is_empty());
    }
}

/// Field access errors are typed and name the record's line.
#[test]
fn field_errors_are_typed() {
    let fixture =
        Fixture::parse(&with_table("id\tvalue\tcount\tempty\nA\tNaN\tx\t\n")).expect("valid");
    let record = fixture.records().next().expect("one record");
    let line = Some(COLUMN_LINE + 1);

    let error = record.text("missing").expect_err("unknown column");
    assert_eq!(error.kind(), &Kind::UnknownColumn("missing".to_owned()));
    assert_eq!(error.line(), line);

    let error = record.parse::<i64>("empty").expect_err("empty field");
    assert_eq!(error.kind(), &Kind::EmptyField("empty".to_owned()));
    assert!(record.real("empty").is_err());
    assert!(record.real_bits("empty").is_err());

    let error = record.parse::<i64>("count").expect_err("not an integer");
    assert!(
        matches!(error.kind(), Kind::InvalidField { column, text, .. } if column == "count" && text == "x")
    );
    assert!(record.parse_optional::<i64>("count").is_err());

    // Non-finite values use exactly the spellings `nan`, `inf` and `-inf`.
    let error = record
        .real("value")
        .expect_err("`NaN` is not a valid spelling");
    assert!(matches!(error.kind(), Kind::InvalidField { text, .. } if text == "NaN"));
}

/// Rule 7: the accepted and rejected spellings of real numbers.
#[test]
fn real_number_spellings() {
    let table = "id\tvalue\n\
        a\t0.5\nb\t-0.0\nc\t1e-07\nd\t1.5e+16\ne\t2E3\nf\t123\ng\tnan\nh\tinf\ni\t-inf\n\
        j\tNaN\nk\tInfinity\nl\t+1\nm\t.5\nn\t5.\no\t1,5\np\t 1\nq\t1e\nr\t0x3ff0000000000000\ns\t--1\n";
    let fixture = Fixture::parse(&with_table(table)).expect("valid");
    let mut values = Vec::new();
    for record in fixture.records() {
        values.push((record.id().to_owned(), record.real("value").ok()));
    }
    let value = |id: &str| values.iter().find(|(i, _)| i == id).and_then(|(_, v)| *v);
    assert_eq!(value("a").map(f64::to_bits), Some(0.5_f64.to_bits()));
    assert_eq!(value("b").map(f64::to_bits), Some((-0.0_f64).to_bits()));
    assert_eq!(value("c").map(f64::to_bits), Some(1e-7_f64.to_bits()));
    assert_eq!(value("d").map(f64::to_bits), Some(1.5e16_f64.to_bits()));
    assert_eq!(value("e").map(f64::to_bits), Some(2000.0_f64.to_bits()));
    assert_eq!(value("f").map(f64::to_bits), Some(123.0_f64.to_bits()));
    assert!(value("g").is_some_and(f64::is_nan));
    assert_eq!(value("h"), Some(f64::INFINITY));
    assert_eq!(value("i"), Some(f64::NEG_INFINITY));
    for rejected in ["j", "k", "l", "m", "n", "o", "p", "q", "r", "s"] {
        assert_eq!(value(rejected), None, "{rejected} must be rejected");
    }
}

/// Rule 7: bit patterns are `0x` and exactly 16 hexadecimal digits.
#[test]
fn bit_pattern_spellings() {
    let table = "id\tbits\n\
        a\t0x3ff0000000000000\nb\t0x3FF0000000000000\nc\t0x7ff8000000000000\n\
        d\t3ff0000000000000\ne\t0x3ff\nf\t0x3ff00000000000000\ng\t0xzz00000000000000\nh\t0X3ff0000000000000\n";
    let fixture = Fixture::parse(&with_table(table)).expect("valid");
    let read: Vec<Option<u64>> = fixture
        .records()
        .map(|r| r.real_bits("bits").ok().map(f64::to_bits))
        .collect();
    assert_eq!(
        read,
        [
            Some(0x3FF0_0000_0000_0000),
            Some(0x3FF0_0000_0000_0000),
            Some(0x7FF8_0000_0000_0000),
            None,
            None,
            None,
            None,
            None,
        ]
    );
}

/// The committed sample fixture loads from disk, whatever line endings the
/// checkout uses.
#[test]
fn committed_sample_fixture_loads() {
    let path = Fixture::repository_path("tests/data/sample.fixture");
    let fixture = Fixture::load(&path).expect("the sample fixture is valid");
    assert_eq!(fixture.header("fixture-set"), Some("SUPPORT-SAMPLE"));
    assert_eq!(fixture.record_count(), 4);
    let records: Vec<_> = fixture.records().collect();
    assert_eq!(
        records[0].real("real").expect("real").to_bits(),
        (182.0_f64 / 360.0).to_bits()
    );
    assert_eq!(
        records[0].real_bits("real_bits").expect("bits").to_bits(),
        records[0].real("real").expect("real").to_bits()
    );
    assert_eq!(
        records[1].real("real").expect("real").to_bits(),
        (-0.0_f64).to_bits()
    );
    assert_eq!(
        records[2].optional_text("expected_error").expect("column"),
        Some("value.not_finite")
    );
    assert!(records[3].real("real").expect("real").is_nan());
    assert_eq!(
        records[3].parse_optional::<i64>("count").expect("empty"),
        None
    );
}

#[test]
fn missing_file_is_an_io_error() {
    let path = Fixture::repository_path("tests/data/no-such-file.fixture");
    let error = Fixture::load(&path).expect_err("missing file");
    assert!(
        matches!(
            error.kind(),
            Kind::Io {
                kind: std::io::ErrorKind::NotFound,
                ..
            }
        ),
        "{error}"
    );
    assert_eq!(error.line(), None);
}

// ---------------------------------------------------------------------------
// More PRNG cases
// ---------------------------------------------------------------------------

/// More known answers, recomputed from the reference implementation: the
/// largest seed, whose first step wraps around 2^64, and seed 42.
#[test]
fn splitmix64_matches_more_reference_outputs() {
    for (seed, expected) in [
        (
            u64::MAX,
            [
                0xE4D9_7177_1B65_2C20,
                0xE99F_F867_DBF6_82C9,
                0x382F_F84C_B272_81E9,
            ],
        ),
        (
            42,
            [
                0xBDD7_3226_2FEB_6E95,
                0x28EF_E333_B266_F103,
                0x4752_6757_130F_9F52,
            ],
        ),
    ] {
        let mut rng = SplitMix64::new(seed);
        for value in expected {
            assert_eq!(rng.next_u64(), value, "seed {seed:#x}");
        }
    }
    // The state after one step from seed 0 is GAMMA, so seeding with GAMMA
    // continues the sequence of seed 0 one output later.
    let mut zero = SplitMix64::new(0);
    zero.next_u64();
    let mut gamma = SplitMix64::new(SplitMix64::GAMMA);
    for _ in 0..1_000 {
        assert_eq!(zero.next_u64(), gamma.next_u64());
    }
}

/// `case_seed(s, k + 1) = case_seed(s + GAMMA, k)` with wrapping arithmetic,
/// for any run seed and case number, including the largest.
#[test]
fn case_seeds_follow_the_state_arithmetic() {
    prng::check("case seed arithmetic", 0xCA5E_0001, 10_000, |rng| {
        let seed = rng.next_u64();
        let case = if rng.below(2) == 0 {
            rng.next_u64()
        } else {
            rng.below(1_000)
        };
        let left = prng::case_seed(seed, case.wrapping_add(1));
        let right = prng::case_seed(seed.wrapping_add(SplitMix64::GAMMA), case);
        if left == right {
            Ok(())
        } else {
            Err(format!("seed {seed:#x}, case {case}"))
        }
    });
}

/// A power-of-two bound rejects nothing, so the result is the low bits of
/// the next output.
#[test]
fn power_of_two_bounds_take_the_low_bits() {
    for shift in 0_u32..64 {
        let bound = 1_u64 << shift;
        let mut by_below = SplitMix64::new(u64::from(shift));
        let mut by_bits = SplitMix64::new(u64::from(shift));
        for _ in 0..100 {
            assert_eq!(by_below.below(bound), by_bits.next_u64() & (bound - 1));
        }
    }
}

/// With the bound 2^63 + 1, almost half of all outputs must be rejected
/// (those below 2^64 mod bound = 2^63 - 1), so the rejection loop runs, and
/// every result stays below the bound.
#[test]
fn rejection_sampling_handles_large_bounds() {
    let bound = (1_u64 << 63) + 1;
    let threshold = bound.wrapping_neg() % bound;
    assert_eq!(threshold, (1 << 63) - 1);
    let mut rng = SplitMix64::new(23);
    let mut rejected = 0;
    for _ in 0..10_000 {
        let mut probe = rng.clone();
        let value = rng.below(bound);
        assert!(value < bound);
        let first = probe.next_u64();
        // The output function is a bijection of the state, so equal next
        // outputs mean that `below` consumed exactly one draw.
        let consumed_one = rng.clone().next_u64() == probe.clone().next_u64();
        if first < threshold {
            assert!(!consumed_one, "a draw below the threshold is rejected");
            rejected += 1;
        } else {
            assert!(consumed_one, "an accepted draw ends the loop");
            assert_eq!(value, first % bound, "an accepted draw is used directly");
        }
    }
    assert!(
        (4_500..=5_500).contains(&rejected),
        "{rejected} first draws rejected"
    );
}

/// Loose, deterministic balance checks: each of the 64 output bits is set
/// about half the time, and unit-interval values average about one half.
#[test]
fn outputs_are_balanced() {
    let mut rng = SplitMix64::new(29);
    let draws = 100_000_u32;
    let mut ones = [0_u32; 64];
    let mut sum = 0.0;
    for _ in 0..draws {
        let x = rng.next_u64();
        for (bit, count) in ones.iter_mut().enumerate() {
            *count += u32::from((x >> bit) & 1 == 1);
        }
        sum += rng.next_f64();
    }
    for (bit, count) in ones.into_iter().enumerate() {
        assert!(
            (48_000..=52_000).contains(&count),
            "bit {bit} set {count} times"
        );
    }
    let mean = sum / f64::from(draws);
    assert!((0.495..=0.505).contains(&mean), "mean {mean}");
}

#[test]
fn range_i64_is_uniform_and_reaches_the_extremes() {
    let mut rng = SplitMix64::new(31);
    let mut counts = [0_u32; 10];
    for _ in 0..100_000 {
        let value = rng.range_i64(-5, 4);
        counts[usize::try_from(value + 5).expect("in range")] += 1;
    }
    for (index, count) in counts.into_iter().enumerate() {
        assert!((9_500..=10_500).contains(&count), "{index}: {count}");
    }
    let (mut lowest, mut next) = (false, false);
    for _ in 0..200 {
        match rng.range_i64(i64::MIN, i64::MIN + 1) {
            i64::MIN => lowest = true,
            value => {
                assert_eq!(value, i64::MIN + 1);
                next = true;
            }
        }
    }
    assert!(lowest && next);
    let (mut negative, mut positive) = (false, false);
    for _ in 0..200 {
        let value = rng.range_i64(i64::MIN, i64::MAX);
        negative |= value < 0;
        positive |= value > 0;
    }
    assert!(negative && positive, "the full range covers both signs");
}

#[test]
#[should_panic(expected = "cannot choose from an empty slice")]
fn choosing_from_nothing_is_a_test_defect() {
    let _ = SplitMix64::new(0).choose::<u8>(&[]);
}

#[test]
fn check_with_zero_cases_runs_nothing() {
    let mut calls = 0;
    prng::check("never runs", 1, 0, |_| {
        calls += 1;
        Ok(())
    });
    assert_eq!(calls, 0);
}

// ---------------------------------------------------------------------------
// More ULP and tolerance cases
// ---------------------------------------------------------------------------

/// An independent formulation of the order of doubles: from the sign and
/// the magnitude bits, in 128-bit arithmetic. Both zeros are at position 0.
fn position(value: f64) -> i128 {
    let bits = value.to_bits();
    let magnitude = i128::from(bits & 0x7FFF_FFFF_FFFF_FFFF);
    if bits >> 63 == 1 {
        -magnitude
    } else {
        magnitude
    }
}

/// Values for distance tests: anything, tiny values of either sign, and the
/// special values.
fn distance_operand(rng: &mut SplitMix64) -> f64 {
    match rng.below(3) {
        0 => rng.any_f64(),
        1 => {
            let tiny = f64::from_bits(rng.below(4_096));
            if rng.below(2) == 0 { tiny } else { -tiny }
        }
        _ => *rng.choose(&[
            0.0,
            -0.0,
            1.0,
            -1.0,
            f64::MAX,
            f64::MIN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MIN_POSITIVE,
            -f64::MIN_POSITIVE,
        ]),
    }
}

#[test]
fn ulp_distance_matches_an_independent_formulation() {
    prng::check("ulp distance oracle", 0x0F10_A701, 50_000, |rng| {
        let (a, b) = (distance_operand(rng), distance_operand(rng));
        let expected = if a.is_nan() || b.is_nan() {
            None
        } else {
            Some(u64::try_from((position(a) - position(b)).abs()).expect("fits in u64"))
        };
        let actual = ulp_distance(a, b);
        if actual == expected {
            Ok(())
        } else {
            Err(format!("{a:?}, {b:?}: got {actual:?}, want {expected:?}"))
        }
    });
}

/// Exhaustive windows around the points where the encoding changes: zero,
/// the boundary between subnormal and normal numbers, exponent boundaries
/// at 1 and 2, and the largest finite value, next to infinity. Inside each
/// window, consecutive doubles are exactly one ULP apart, and the distance
/// from the first double counts every double in between.
#[test]
fn ulp_distance_counts_every_double_in_a_window() {
    for center in [
        0.0,
        f64::MIN_POSITIVE,
        -f64::MIN_POSITIVE,
        1.0,
        2.0,
        -1.0,
        f64::MAX,
        f64::MIN,
    ] {
        let mut below = Vec::new();
        let mut x = center;
        while below.len() < 1_000 && x.is_finite() {
            x = x.next_down();
            below.push(x);
        }
        below.reverse();
        let mut window = below;
        window.push(center);
        let mut x = center;
        for _ in 0..1_000 {
            if !x.is_finite() {
                break;
            }
            x = x.next_up();
            window.push(x);
        }
        for (steps, value) in (0_u64..).zip(&window) {
            assert_eq!(ulp_distance(window[0], *value), Some(steps), "{value:?}");
        }
        for pair in window.windows(2) {
            assert_eq!(ulp_distance(pair[0], pair[1]), Some(1), "{pair:?}");
        }
    }
    // The negative zero sits at the same position as the positive one.
    for x in [f64::from_bits(1), -f64::from_bits(1), 1.0, f64::INFINITY] {
        assert_eq!(ulp_distance(-0.0, x), ulp_distance(0.0, x));
    }
}

/// The distance is a metric along the line of doubles: zero exactly between
/// equal values, and additive for ordered triples.
#[test]
fn ulp_distance_is_additive_along_the_line() {
    prng::check("ulp additivity", 0x0F10_A702, 20_000, |rng| {
        let mut values = [
            distance_operand(rng),
            distance_operand(rng),
            distance_operand(rng),
        ];
        if values.iter().any(|value| value.is_nan()) {
            return Ok(());
        }
        values.sort_by(f64::total_cmp);
        let [a, b, c] = values;
        let distance = |x, y| ulp_distance(x, y).expect("no NaN");
        let sum = distance(a, b).checked_add(distance(b, c));
        if sum != Some(distance(a, c)) {
            return Err(format!("{a:?} {b:?} {c:?} are not additive"));
        }
        let equal = a.partial_cmp(&b) == Some(std::cmp::Ordering::Equal);
        if (distance(a, b) == 0) != equal {
            return Err(format!(
                "distance({a:?}, {b:?}) is zero only for equal values"
            ));
        }
        Ok(())
    });
}

/// Every comparison rule is symmetric; `Ulps` widens monotonically; bit
/// identity implies `Ulps(0)` for numbers; and `Ulps(k)` accepts a value
/// exactly when it is at most `k` steps away.
#[test]
fn tolerance_rules_are_symmetric_and_monotone() {
    let rules = [
        Tolerance::Bitwise,
        Tolerance::Ulps(0),
        Tolerance::Ulps(3),
        Tolerance::Mixed {
            absolute: 1e-300,
            relative: 1e-15,
        },
        Tolerance::Mixed {
            absolute: 0.0,
            relative: 1e-12,
        },
    ];
    prng::check("tolerance properties", 0x0F10_A703, 20_000, |rng| {
        let a = distance_operand(rng);
        let steps = rng.below(6);
        let mut b = a;
        for _ in 0..steps {
            b = b.next_up();
        }
        for rule in rules {
            if rule.check(a, b).is_ok() != rule.check(b, a).is_ok() {
                return Err(format!("{rule:?} is not symmetric for {a:?}, {b:?}"));
            }
        }
        for limit in 0..8 {
            if Tolerance::Ulps(limit).check(a, b).is_ok()
                && Tolerance::Ulps(limit + 1).check(a, b).is_err()
            {
                return Err(format!("Ulps is not monotone at {limit} for {a:?}, {b:?}"));
            }
        }
        if !a.is_nan()
            && Tolerance::Bitwise.check(a, b).is_ok()
            && Tolerance::Ulps(0).check(a, b).is_err()
        {
            return Err(format!("bit identity without Ulps(0) for {a:?}"));
        }
        if a.is_finite() && b.is_finite() {
            for limit in 0..8 {
                if Tolerance::Ulps(limit).check(a, b).is_ok() != (steps <= limit) {
                    return Err(format!("Ulps({limit}) at {steps} steps from {a:?}"));
                }
            }
        }
        Ok(())
    });
}

#[test]
fn mismatch_report_handles_nan() {
    let report = Tolerance::Ulps(0)
        .check(f64::NAN, 1.0)
        .expect_err("NaN never matches");
    assert!(report.contains("undefined (NaN)"), "{report}");
}

#[test]
fn assert_close_accepts_matching_values() {
    assert_close(0.1 + 0.2, 0.3, Tolerance::Ulps(1));
    assert_close(-0.0, 0.0, Tolerance::Ulps(0));
    assert_close(1.5, 1.5, Tolerance::Bitwise);
    assert_close(
        1e-13,
        0.0,
        Tolerance::Mixed {
            absolute: 1e-12,
            relative: 0.0,
        },
    );
}

#[test]
#[should_panic(expected = "invalid tolerance")]
fn nan_mixed_bound_is_a_test_defect() {
    let _ = Tolerance::Mixed {
        absolute: 0.0,
        relative: f64::NAN,
    }
    .check(0.0, 0.0);
}

#[test]
#[should_panic(expected = "invalid tolerance")]
fn infinite_mixed_bound_is_a_test_defect() {
    let _ = Tolerance::Mixed {
        absolute: f64::INFINITY,
        relative: 0.0,
    }
    .check(0.0, 0.0);
}

// ---------------------------------------------------------------------------
// More fixture-reader cases
// ---------------------------------------------------------------------------

#[test]
fn every_error_message_names_its_origin_and_line() {
    let fixture = Fixture::parse(&with_table("id\tvalue\tempty\nA\tx\t\n")).expect("valid fixture");
    let record = fixture.records().next().expect("one record");
    let prefix = format!("<inline>:{}: ", COLUMN_LINE + 1);
    let errors = [
        record.text("missing").expect_err("unknown column"),
        record.optional_text("missing").expect_err("unknown column"),
        record.parse::<i64>("empty").expect_err("empty field"),
        record.parse::<i64>("value").expect_err("not an integer"),
        record
            .parse_optional::<i64>("missing")
            .expect_err("unknown column"),
        record.real("value").expect_err("not a real number"),
        record.real_bits("value").expect_err("not a bit pattern"),
    ];
    for error in errors {
        let message = error.to_string();
        assert!(message.starts_with(&prefix), "{message}");
    }
    // Every typed accessor reports an unknown column the same way.
    let unknown = Kind::UnknownColumn("missing".to_owned());
    assert_eq!(
        record.parse::<i64>("missing").expect_err("unknown").kind(),
        &unknown
    );
    assert_eq!(
        record.real("missing").expect_err("unknown").kind(),
        &unknown
    );
    assert_eq!(
        record.real_bits("missing").expect_err("unknown").kind(),
        &unknown
    );
    // An invalid field carries the message of the parser that rejected it.
    let error = record.parse::<i64>("value").expect_err("not an integer");
    let parser_message = "x".parse::<i64>().expect_err("not an integer").to_string();
    assert!(error.to_string().ends_with(&parser_message), "{error}");

    let error = Fixture::parse(&with_table("id\tvalue\n")).expect_err("no records");
    assert!(error.to_string().starts_with("<inline>: "), "{error}");
    let path = Fixture::repository_path("tests/data/no-such-file.fixture");
    let error = Fixture::load(&path).expect_err("missing file");
    assert!(
        error
            .to_string()
            .starts_with(&format!("{}: ", path.display())),
        "{error}"
    );
}

/// A `fmt::Write` sink that fails once more than `limit` bytes would have
/// been written to it.
struct Truncating {
    text: String,
    limit: usize,
}

impl std::fmt::Write for Truncating {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if self.text.len() + s.len() > self.limit {
            Err(std::fmt::Error)
        } else {
            self.text.push_str(s);
            Ok(())
        }
    }
}

/// Error messages of the reader pass a failing writer's error back to the
/// caller at every length, with and without a line number, and never panic.
#[test]
fn error_formatting_propagates_writer_errors() {
    let on_a_line = Fixture::parse(&with_table("id\tx\nA\t1\t2\n")).expect_err("field count");
    let whole_file = Fixture::parse(&with_table("id\tx\n")).expect_err("no records");
    for error in [on_a_line, whole_file] {
        let full = error.to_string();
        for limit in 0..full.len() {
            let mut sink = Truncating {
                text: String::new(),
                limit,
            };
            assert!(
                std::fmt::write(&mut sink, format_args!("{error}")).is_err(),
                "{limit} bytes for {full:?}"
            );
        }
        let mut sink = Truncating {
            text: String::new(),
            limit: full.len(),
        };
        assert!(std::fmt::write(&mut sink, format_args!("{error}")).is_ok());
        assert_eq!(sink.text, full);
    }
}

/// Rule 1: a file that is not valid UTF-8 is rejected as unreadable.
#[test]
fn a_file_that_is_not_utf8_is_rejected() {
    let path = Fixture::repository_path("tests/data/not-utf8.fixture");
    let error = Fixture::load(&path).expect_err("not UTF-8");
    assert!(
        matches!(
            error.kind(),
            Kind::Io {
                kind: std::io::ErrorKind::InvalidData,
                ..
            }
        ),
        "{error}"
    );
}

#[test]
fn a_loaded_fixture_names_its_file() {
    let path = Fixture::repository_path("tests/data/sample.fixture");
    let fixture = Fixture::load(&path).expect("the sample fixture is valid");
    assert_eq!(fixture.origin(), path.display().to_string());
    assert_eq!(fixture.records().len(), fixture.record_count());
}

/// Rules 3 and 5: the exact grammar of header keys and column names. Values
/// may contain spaces, colons and any non-control Unicode character.
#[test]
fn header_key_and_column_name_grammar() {
    let with_line = |line: &str| {
        let header = HEADER.replacen("platform: any\n", &format!("platform: any\n{line}\n"), 1);
        format!("{header}id\nA\n")
    };
    for valid in [
        "key-2: v",
        "a1: v",
        "x-: v",
        "k: a value with spaces: and colons",
        "notes: \u{e9}t\u{e9} \u{e01}\u{e32}\u{e23}\u{e40}\u{e07}\u{e34}\u{e19} \u{1f600}",
    ] {
        let fixture =
            Fixture::parse(&with_line(valid)).unwrap_or_else(|error| panic!("{valid:?}: {error}"));
        let (key, value) = valid.split_once(": ").expect("key: value");
        assert_eq!(fixture.header(key), Some(value), "{valid:?}");
    }
    for invalid in [
        "2key: v",
        "key_x: v",
        "-key: v",
        "Key: v",
        "k\u{e9}y: v",
        "key : v",
        "key:  v",
        "key: v ",
        "key: \u{a0}v",
        ": v",
        "key:",
        "key: ",
        "key",
    ] {
        let error = Fixture::parse(&with_line(invalid)).expect_err(invalid);
        assert_eq!(error.kind(), &Kind::MalformedHeaderLine, "{invalid:?}");
        assert_eq!(error.line(), Some(12), "{invalid:?}");
    }

    for valid in ["rate_2", "a", "x9_y", "z_"] {
        let fixture = Fixture::parse(&with_table(&format!("id\t{valid}\nA\t1\n")))
            .unwrap_or_else(|error| panic!("{valid:?}: {error}"));
        assert_eq!(fixture.columns(), ["id", valid]);
    }
    for invalid in [
        "2rate", "_x", "rate-2", "Rate", "r\u{e9}", " rate", "rate ", "id ",
    ] {
        let error =
            Fixture::parse(&with_table(&format!("id\t{invalid}\nA\t1\n"))).expect_err(invalid);
        assert_eq!(
            error.kind(),
            &Kind::MalformedColumnName(invalid.to_owned()),
            "{invalid:?}"
        );
    }
}

/// Rule 1: LF and CRLF may be mixed within one file.
#[test]
fn mixed_line_endings_are_accepted() {
    let lf = with_table(TABLE);
    let mixed: String = lf
        .split_inclusive('\n')
        .enumerate()
        .map(|(index, line)| {
            if index % 2 == 0 {
                line.replace('\n', "\r\n")
            } else {
                line.to_owned()
            }
        })
        .collect();
    let summary = |text: &str| {
        let fixture = Fixture::parse(text).expect("valid fixture");
        fixture
            .records()
            .map(|record| (record.id().to_owned(), record.line()))
            .collect::<Vec<_>>()
    };
    assert_eq!(summary(&mixed), summary(&lf));
}

/// Rule 6: a record of tabs only has an empty id; a record whose other
/// fields are all empty is valid.
#[test]
fn records_of_empty_fields() {
    let error = Fixture::parse(&with_table("id\ta\tb\n\t\t\n")).expect_err("empty id");
    assert_eq!(error.kind(), &Kind::EmptyId);
    let fixture = Fixture::parse(&with_table("id\ta\tb\nA\t\t\n")).expect("valid");
    let record = fixture.records().next().expect("one record");
    assert_eq!(record.optional_text("a"), Ok(None));
    assert_eq!(record.optional_text("b"), Ok(None));
}

/// Fields are kept exactly as written: non-ASCII text, and spaces at either
/// end.
#[test]
fn fields_are_kept_exactly() {
    let thai = "\u{e01}\u{e32}\u{e23}\u{e40}\u{e07}\u{e34}\u{e19}";
    let table = format!("id\tnote\tpadded\nA\t{thai} \u{20ac}\t  two  spaces  \n");
    let fixture = Fixture::parse(&with_table(&table)).expect("valid");
    let record = fixture.records().next().expect("one record");
    assert_eq!(record.text("note"), Ok(format!("{thai} \u{20ac}").as_str()));
    assert_eq!(record.text("padded"), Ok("  two  spaces  "));
    assert_eq!(record.id(), "A");
}

/// An even integer, to show that `parse` reports the message of any
/// `FromStr` implementation.
#[derive(Debug, PartialEq)]
struct Even(i64);

impl std::str::FromStr for Even {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, String> {
        let number: i64 = text
            .parse()
            .map_err(|error: std::num::ParseIntError| error.to_string())?;
        if number % 2 == 0 {
            Ok(Self(number))
        } else {
            Err(format!("{number} is odd"))
        }
    }
}

#[test]
fn parse_uses_any_from_str_implementation() {
    let fixture = Fixture::parse(&with_table("id\tn\nA\t4\nB\t3\n")).expect("valid");
    let records: Vec<_> = fixture.records().collect();
    assert_eq!(records[0].parse::<Even>("n"), Ok(Even(4)));
    let error = records[1].parse::<Even>("n").expect_err("odd");
    assert_eq!(
        error.kind(),
        &Kind::InvalidField {
            column: "n".to_owned(),
            text: "3".to_owned(),
            reason: "3 is odd".to_owned(),
        }
    );
    assert_eq!(error.line(), Some(COLUMN_LINE + 2));
}

/// Rule 7 at the limits of `f64`: the extreme finite values and subnormals
/// read exactly; decimal text that overflows is rejected, because infinities
/// must be written as `inf` or `-inf`; text below the smallest subnormal
/// rounds to zero, as correct rounding requires.
#[test]
fn real_numbers_at_the_limits_of_f64() {
    let table = "id\tvalue\n\
        a\t1.7976931348623157e308\nb\t-1.7976931348623157e308\nc\t5e-324\n\
        d\t2.2250738585072014e-308\ne\t1e309\nf\t-1e309\ng\t1e-400\n\
        h\t0.30000000000000004\ni\t-0\nj\t00.50\n";
    let fixture = Fixture::parse(&with_table(table)).expect("valid");
    let read: Vec<Option<u64>> = fixture
        .records()
        .map(|record| record.real("value").ok().map(f64::to_bits))
        .collect();
    assert_eq!(
        read,
        [
            Some(f64::MAX.to_bits()),
            Some(f64::MIN.to_bits()),
            Some(1),
            Some(f64::MIN_POSITIVE.to_bits()),
            None,
            None,
            Some(0),
            Some((0.1_f64 + 0.2).to_bits()),
            Some((-0.0_f64).to_bits()),
            Some(0.5_f64.to_bits()),
        ]
    );
}

/// Rule 7: bit patterns keep every bit, including the sign of zero and the
/// payload of a NaN.
#[test]
fn bit_patterns_keep_every_bit() {
    for bits in [
        0,
        1,
        0x8000_0000_0000_0000,
        0x7FF0_0000_0000_0000,
        0xFFF0_0000_0000_0000,
        0x7FF8_0000_0000_0001,
        0xFFF4_0000_0000_00AB,
        u64::MAX,
    ] {
        let fixture =
            Fixture::parse(&with_table(&format!("id\tbits\nA\t{bits:#018x}\n"))).expect("valid");
        let record = fixture.records().next().expect("one record");
        assert_eq!(record.real_bits("bits").expect("bits").to_bits(), bits);
    }
}

/// Characters for generated header values and fields: letters, digits,
/// punctuation, spaces, and non-ASCII text.
const VALUE_CHARS: [char; 16] = [
    'a',
    'Z',
    '0',
    '9',
    ' ',
    ':',
    '-',
    '.',
    '/',
    '(',
    ')',
    '#',
    '\u{e9}',
    '\u{e01}',
    '\u{20ac}',
    '\u{1f600}',
];
const NAME_START: [char; 4] = ['a', 'k', 'q', 'z'];
const KEY_CHARS: [char; 6] = ['a', 'k', 'z', '0', '9', '-'];
const COLUMN_CHARS: [char; 6] = ['a', 'k', 'z', '0', '9', '_'];

fn random_text(rng: &mut SplitMix64, alphabet: &[char], min: u64, max: u64) -> String {
    let length = min + rng.below(max - min + 1);
    (0..length).map(|_| *rng.choose(alphabet)).collect()
}

/// A valid header value: non-empty, without surrounding whitespace.
fn header_value(rng: &mut SplitMix64) -> String {
    loop {
        let value = random_text(rng, &VALUE_CHARS, 1, 12);
        if value.trim() == value {
            return value;
        }
    }
}

fn random_name(rng: &mut SplitMix64, rest: &[char]) -> String {
    let mut name = String::from(*rng.choose(&NAME_START));
    name.extend((0..rng.below(6)).map(|_| *rng.choose(rest)));
    name
}

fn shuffle<T>(rng: &mut SplitMix64, items: &mut [T]) {
    for i in (1..items.len()).rev() {
        let bound = u64::try_from(i + 1).expect("a short list");
        let j = usize::try_from(rng.below(bound)).expect("a short list");
        items.swap(i, j);
    }
}

/// A random valid fixture, with the content it encodes.
struct Generated {
    text: String,
    header: Vec<(String, String)>,
    columns: Vec<String>,
    records: Vec<Vec<String>>,
}

/// Generates a valid fixture: the required keys in random order after
/// `schema: 1`, extra keys, random columns and records, LF or CRLF chosen
/// line by line, and an optional final line ending.
fn generate(rng: &mut SplitMix64) -> Generated {
    let mut header: Vec<(String, String)> = REQUIRED_KEYS[1..]
        .iter()
        .map(|key| ((*key).to_owned(), header_value(rng)))
        .collect();
    for _ in 0..rng.below(4) {
        let key = random_name(rng, &KEY_CHARS);
        if key != "schema" && !header.iter().any(|(existing, _)| *existing == key) {
            let value = header_value(rng);
            header.push((key, value));
        }
    }
    shuffle(rng, &mut header);
    header.insert(0, ("schema".to_owned(), "1".to_owned()));

    let mut columns = vec!["id".to_owned()];
    for _ in 0..rng.below(6) {
        let column = random_name(rng, &COLUMN_CHARS);
        if !columns.contains(&column) {
            columns.push(column);
        }
    }
    let count = 1 + rng.below(20);
    let mut records = Vec::new();
    for index in 0..count {
        let mut fields = vec![format!("R{index}|{}", random_text(rng, &VALUE_CHARS, 0, 6))];
        for _ in 1..columns.len() {
            fields.push(random_text(rng, &VALUE_CHARS, 0, 8));
        }
        records.push(fields);
    }

    let mut lines: Vec<String> = header
        .iter()
        .map(|(key, value)| format!("{key}: {value}"))
        .collect();
    lines.push(String::new());
    lines.push(columns.join("\t"));
    lines.extend(records.iter().map(|fields| fields.join("\t")));
    let mut text = String::new();
    for (index, line) in lines.iter().enumerate() {
        text.push_str(line);
        if index + 1 < lines.len() || rng.below(2) == 0 {
            text.push_str(if rng.below(2) == 0 { "\n" } else { "\r\n" });
        }
    }
    Generated {
        text,
        header,
        columns,
        records,
    }
}

/// F-CORE-06: every generated valid fixture reads back exactly: each header
/// value, the columns, and each record's id, line number and fields.
#[test]
fn generated_fixtures_read_back_exactly() {
    prng::check("fixture round trip", 0xF1C7_0002, 2_000, |rng| {
        let generated = generate(rng);
        let fixture = Fixture::parse(&generated.text)
            .map_err(|error| format!("{error} in {:?}", generated.text))?;
        for (key, value) in &generated.header {
            if fixture.header(key) != Some(value.as_str()) {
                return Err(format!("header {key:?} read as {:?}", fixture.header(key)));
            }
        }
        if fixture.columns() != generated.columns.as_slice() {
            return Err(format!("columns {:?}", fixture.columns()));
        }
        if fixture.record_count() != generated.records.len()
            || fixture.records().len() != generated.records.len()
        {
            return Err(format!("{} records", fixture.record_count()));
        }
        let first_line = generated.header.len() + 3;
        for (index, (record, fields)) in fixture.records().zip(&generated.records).enumerate() {
            if record.line() != first_line + index || record.id() != fields[0] {
                return Err(format!("record {index} at line {}", record.line()));
            }
            for (column, field) in generated.columns.iter().zip(fields) {
                let text = record.text(column).map_err(|error| error.to_string())?;
                if text != field {
                    return Err(format!("record {index}, {column}: {text:?} != {field:?}"));
                }
            }
        }
        Ok(())
    });
}

/// Characters that corrupt a fixture: separators, line breaks, control
/// characters, a byte-order mark, and others.
const TRICKY: [char; 12] = [
    '\t', '\n', '\r', '\0', ':', ' ', '-', 'A', '\u{feff}', '\u{e9}', '#', '\u{85}',
];

/// Checks the rules on a fixture that the reader accepted.
fn check_rules(fixture: &Fixture) -> Result<(), String> {
    for key in REQUIRED_KEYS {
        if fixture.header(key).is_none() {
            return Err(format!("accepted without {key:?}"));
        }
    }
    if fixture.header("schema") != Some("1") {
        return Err("accepted another schema".to_owned());
    }
    let columns = fixture.columns();
    if columns.first().map(String::as_str) != Some("id") {
        return Err(format!("accepted columns {columns:?}"));
    }
    let mut names = std::collections::BTreeSet::new();
    for name in columns {
        let valid = name.starts_with(|c: char| c.is_ascii_lowercase())
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !valid || !names.insert(name) {
            return Err(format!("accepted column {name:?}"));
        }
    }
    let mut ids = std::collections::BTreeSet::new();
    for record in fixture.records() {
        if record.id().is_empty() || !ids.insert(record.id()) {
            return Err(format!("accepted id {:?}", record.id()));
        }
        for column in columns {
            let text = record.text(column).map_err(|error| error.to_string())?;
            if text.chars().any(char::is_control) {
                return Err(format!("accepted field {text:?}"));
            }
        }
    }
    Ok(())
}

/// Fuzzing of the reader (NFR-02 for the test tooling): for corruptions of
/// valid fixtures, the reader never panics; what it accepts still satisfies
/// every rule; and an error points to a line that exists.
#[test]
fn corrupted_fixtures_never_panic_and_never_break_the_rules() {
    // How often each outcome occurs, so the test can show that it exercises
    // acceptance, errors on a line, and whole-file errors.
    let (mut accepted, mut on_a_line, mut whole_file) = (0_u32, 0_u32, 0_u32);
    prng::check("fixture fuzzing", 0xF1C7_0001, 3_000, |rng| {
        let mut chars: Vec<char> = generate(rng).text.chars().collect();
        for _ in 0..1 + rng.below(3) {
            let length = u64::try_from(chars.len()).expect("a short text");
            let position = usize::try_from(rng.below(length + 1)).expect("a short text");
            match rng.below(5) {
                0 => chars.insert(position, *rng.choose(&TRICKY)),
                1 if position < chars.len() => {
                    chars.remove(position);
                }
                2 if position < chars.len() => chars[position] = *rng.choose(&TRICKY),
                3 => chars.truncate(position),
                _ => {
                    let other = usize::try_from(rng.below(length + 1)).expect("a short text");
                    if position < chars.len() && other < chars.len() {
                        chars.swap(position, other);
                    }
                }
            }
        }
        let text: String = chars.into_iter().collect();
        let lines = text.lines().count().max(1);
        match std::panic::catch_unwind(|| Fixture::parse(&text)) {
            Err(_) => Err(format!("the reader panicked on {text:?}")),
            Ok(Ok(fixture)) => {
                accepted += 1;
                check_rules(&fixture).map_err(|problem| format!("{problem} in {text:?}"))
            }
            Ok(Err(error)) => match error.line() {
                Some(line) if line == 0 || line > lines => {
                    Err(format!("line {line} of {lines}: {error} in {text:?}"))
                }
                Some(_) => {
                    on_a_line += 1;
                    Ok(())
                }
                None => {
                    whole_file += 1;
                    Ok(())
                }
            },
        }
    });
    for (outcome, count) in [
        ("accepted", accepted),
        ("errors on a line", on_a_line),
        ("whole-file errors", whole_file),
    ] {
        assert!(count >= 30, "only {count} cases ended in {outcome}");
    }
}
