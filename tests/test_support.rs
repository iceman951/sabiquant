//! Self-tests of the shared test support (F-CORE-05, F-CORE-06;
//! TESTING_STRATEGY §14, M1 exit criteria): the PRNG and the ULP support are
//! tested, and the fixture reader rejects malformed input with typed errors.

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
