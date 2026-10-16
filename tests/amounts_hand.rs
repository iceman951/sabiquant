//! Independent public hand/adversarial cases for FR-AMT-01 through FR-AMT-14.
//!
//! Expectations come from the frozen M8 amount contract and elementary integer
//! identities. This suite was authored without reading amounts production or
//! its private wide-integer kernel. TOL-E0 applies throughout.

use sabiquant::amounts::{
    AmountError, AmountQuantum, AmountRounding as Rounding, DecimalAmount, exact_amount_sum,
};
use sabiquant::error::{Error as CrateError, IdentifierError};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::error::Error;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};

#[expect(
    clippy::expect_used,
    reason = "hand inputs explicitly require construction success"
)]
fn a(coefficient: i128, scale: u8) -> DecimalAmount {
    DecimalAmount::new(coefficient, scale).expect("valid hand amount")
}

#[expect(
    clippy::expect_used,
    reason = "hand inputs explicitly require construction success"
)]
fn q(coefficient: i128, scale: u8) -> AmountQuantum {
    AmountQuantum::new(coefficient, scale).expect("positive hand quantum")
}

#[expect(
    clippy::expect_used,
    reason = "the hand spelling is an exact valid decimal"
)]
fn p(text: &str) -> DecimalAmount {
    text.parse().expect("valid hand decimal")
}

#[expect(
    clippy::expect_used,
    reason = "the stated arithmetic result is representable"
)]
fn good(result: Result<DecimalAmount, AmountError>) -> DecimalAmount {
    result.expect("representable hand result")
}

#[expect(
    clippy::expect_used,
    reason = "the case explicitly requires this public rejection"
)]
fn rejected<T: Debug>(result: Result<T, AmountError>) -> AmountError {
    result.expect_err("the hand precondition requires an error")
}

fn assert_parts(actual: DecimalAmount, mut coefficient: i128, mut scale: u8) {
    if coefficient == 0 {
        scale = 0;
    } else {
        while scale > 0 && coefficient % 10 == 0 {
            coefficient /= 10;
            scale -= 1;
        }
    }
    assert_eq!((actual.coefficient(), actual.scale()), (coefficient, scale));
}

fn assert_syntax(text: &str, index: usize, reason: &'static str) {
    let error = rejected(text.parse::<DecimalAmount>());
    assert_eq!(error.kind(), "amount.invalid_syntax");
    assert!(
        matches!(error, AmountError::InvalidSyntax { index:i, reason:r, .. }
        if (i,r)==(index,reason)),
        "{text:?}: {error:?}"
    );
}

fn assert_range(error: AmountError, operation: &'static str, negative: bool) {
    assert_eq!(error.kind(), "amount.out_of_range");
    assert!(
        matches!(error, AmountError::OutOfRange { operation:o, negative:n, .. }
        if (o,n)==(operation,negative)),
        "{error:?}"
    );
}

fn assert_inexact(error: AmountError, operation: &'static str) {
    assert_eq!(error.kind(), "amount.inexact");
    assert!(
        matches!(error, AmountError::Inexact { operation:o, .. } if o==operation),
        "{error:?}"
    );
}

fn hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn all_39_scales_accept_every_signed_boundary_and_canonicalize_only_after_admission() {
    // FR-AMT-01/03/06/08: constructor scale is a resource/domain decision.
    assert_eq!(DecimalAmount::MAX_SCALE, 38);
    assert_eq!(DecimalAmount::MAX_TEXT_BYTES, 128);
    for scale in 0..=38 {
        for coefficient in [
            i128::MIN,
            i128::MIN + 1,
            -100,
            -10,
            -1,
            0,
            1,
            10,
            100,
            i128::MAX,
        ] {
            let value = a(coefficient, scale);
            assert_parts(value, coefficient, scale);
            assert_eq!(p(&value.to_string()), value);
            assert_eq!(format!("{value:.0}"), value.to_string());
            assert_eq!(format!("{value:.2}"), value.to_string());
            assert_eq!(format!("{value:.99}"), value.to_string());
        }
        assert_eq!(q(1, scale).amount(), a(1, scale));
        assert_eq!(q(10, scale).amount(), a(10, scale));
        for coefficient in [i128::MIN, -1, 0] {
            let error = rejected(AmountQuantum::new(coefficient, scale));
            assert!(
                matches!(error, AmountError::InvalidQuantum { coefficient:c, scale:s, .. }
                if (c,s)==(coefficient,scale)),
                "{error:?}"
            );
        }
    }
    for scale in [39, 40, 127, 255] {
        for coefficient in [i128::MIN, -10, 0, 10, i128::MAX] {
            for error in [
                rejected(DecimalAmount::new(coefficient, scale)),
                rejected(AmountQuantum::new(coefficient, scale)),
            ] {
                assert_eq!(error.kind(), "amount.invalid_scale");
                assert!(
                    matches!(error, AmountError::InvalidScale { scale:s, maximum:38, .. }
                    if s==usize::from(scale)),
                    "{error:?}"
                );
            }
        }
    }
}

#[test]
fn decimal_grammar_retains_exact_first_byte_and_missing_digit_reason() {
    // FR-AMT-02/11: errors are state-based byte positions, including Unicode.
    for (text, index, reason) in [
        ("", 0, "missing integer digits"),
        ("+", 1, "missing integer digits"),
        ("-", 1, "missing integer digits"),
        (".1", 0, "missing integer digits"),
        ("abc", 0, "missing integer digits"),
        ("é", 0, "missing integer digits"),
        ("١", 0, "missing integer digits"),
        ("１", 0, "missing integer digits"),
        (" 1", 0, "missing integer digits"),
        ("\t1", 0, "missing integer digits"),
        ("++1", 1, "missing integer digits"),
        ("--1", 1, "missing integer digits"),
        ("+-1", 1, "missing integer digits"),
        ("-+1", 1, "missing integer digits"),
        ("+é", 1, "missing integer digits"),
        ("1.", 2, "missing fractional digits"),
        ("1..2", 2, "missing fractional digits"),
        ("1.a", 2, "missing fractional digits"),
        ("1.+2", 2, "missing fractional digits"),
        ("1.-2", 2, "missing fractional digits"),
        ("1.é", 2, "missing fractional digits"),
        ("1a", 1, "unexpected byte"),
        ("1.2.3", 3, "unexpected byte"),
        ("1.2a", 3, "unexpected byte"),
        ("1é", 1, "unexpected byte"),
        ("1.2é", 3, "unexpected byte"),
        ("1e2", 1, "unexpected byte"),
        ("1E-2", 1, "unexpected byte"),
        ("1_000", 1, "unexpected byte"),
        ("1,2", 1, "unexpected byte"),
        ("1 ", 1, "unexpected byte"),
        ("1\n", 1, "unexpected byte"),
        ("1\0", 1, "unexpected byte"),
        ("-1.2x", 4, "unexpected byte"),
    ] {
        assert_syntax(text, index, reason);
    }
}

fn whole_grammar_accepts(text: &str) -> bool {
    // Whole-string structural recognition, independent of streaming error states.
    let body = text
        .strip_prefix('+')
        .or_else(|| text.strip_prefix('-'))
        .unwrap_or(text);
    let pieces: Vec<_> = body.split('.').collect();
    matches!(pieces.len(), 1 | 2)
        && pieces
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

#[test]
fn complete_short_ascii_language_agrees_with_the_whole_string_grammar() {
    // FR-AMT-02/03/12: all 19,608 words of length 0..5 over a seven-byte alphabet.
    const ALPHABET: &[u8; 7] = b"07.+-x ";
    let mut visited = 0;
    let mut accepted = 0;
    for length in 0..=5_u32 {
        for mut code in 0..7_usize.pow(length) {
            let mut text = String::new();
            for _ in 0..length {
                text.push(char::from(ALPHABET[code % 7]));
                code /= 7;
            }
            let parsed = text.parse::<DecimalAmount>();
            assert_eq!(parsed.is_ok(), whole_grammar_accepts(&text), "{text:?}");
            if let Ok(value) = parsed {
                let negative = text.starts_with('-');
                let coefficient = text
                    .bytes()
                    .filter(u8::is_ascii_digit)
                    .fold(0_i128, |n, byte| n * 10 + i128::from(byte - b'0'));
                let scale = text
                    .split_once('.')
                    .map_or(0, |(_, fraction)| fraction.len());
                assert_parts(
                    value,
                    if negative { -coefficient } else { coefficient },
                    u8::try_from(scale).unwrap_or(255),
                );
                assert_eq!(p(&value.to_string()), value);
                accepted += 1;
            }
            visited += 1;
        }
    }
    assert_eq!(visited, 19_608);
    assert_eq!(accepted, 230);
}

#[test]
fn parser_resource_syntax_precision_and_normalized_range_precedence_are_distinct() {
    // FR-AMT-02/11: admission order must survive rescuable zeros and bad suffixes.
    let max = i128::MAX.to_string();
    let min = i128::MIN.to_string();
    let exact_128 = format!("{}{}", "0".repeat(128 - max.len()), max);
    assert_eq!(exact_128.len(), 128);
    assert_eq!(p(&exact_128), a(i128::MAX, 0));
    assert_eq!(p(&format!("{max}.{}", "0".repeat(38))), a(i128::MAX, 0));
    assert_eq!(p(&format!("{min}.{}", "0".repeat(38))), a(i128::MIN, 0));
    assert_eq!(p(&"0".repeat(128)), a(0, 0));
    for text in [
        "0".repeat(129),
        format!("{}x", "0".repeat(128)),
        "é".repeat(65),
    ] {
        let error = rejected(text.parse::<DecimalAmount>());
        assert!(
            matches!(error, AmountError::TextTooLong { length, maximum:128, .. }
            if length==text.len()),
            "{error:?}"
        );
    }
    let bad_suffix = format!("0.{}x", "0".repeat(39));
    assert_syntax(&bad_suffix, 41, "unexpected byte");
    for integer in ["0", "9999999999999999999999999999999999999999999"] {
        let text = format!("{integer}.{}", "0".repeat(39));
        let error = rejected(text.parse::<DecimalAmount>());
        assert!(
            matches!(
                error,
                AmountError::InvalidScale {
                    scale: 39,
                    maximum: 38,
                    ..
                }
            ),
            "{error:?}"
        );
    }
    assert_range(
        rejected("170141183460469231731687303715884105728".parse::<DecimalAmount>()),
        "parse",
        false,
    );
    assert_range(
        rejected("-170141183460469231731687303715884105729".parse::<DecimalAmount>()),
        "parse",
        true,
    );
    for text in ["+0000.0000", "-0", "-000.000", "00000"] {
        let value = p(text);
        assert_eq!((value.coefficient(), value.scale()), (0, 0));
        assert_eq!(value.to_string(), "0");
    }
}

#[test]
fn canonical_fixed_display_preserves_every_decimal_point_and_precision() {
    // FR-AMT-03: literal expected strings do not call a production formatter.
    for (text, canonical) in [
        ("+00123.45000", "123.45"),
        ("-00123.45000", "-123.45"),
        ("000100.000", "100"),
        ("0.0000100", "0.00001"),
        ("-0.1000", "-0.1"),
        (
            "1.00000000000000000000000000000000000001",
            "1.00000000000000000000000000000000000001",
        ),
        (
            "170141183460469231731687303715884105727",
            "170141183460469231731687303715884105727",
        ),
        (
            "-170141183460469231731687303715884105728",
            "-170141183460469231731687303715884105728",
        ),
        (
            "1.70141183460469231731687303715884105727",
            "1.70141183460469231731687303715884105727",
        ),
        (
            "-1.70141183460469231731687303715884105728",
            "-1.70141183460469231731687303715884105728",
        ),
    ] {
        let value = p(text);
        assert_eq!(value.to_string(), canonical);
        assert_eq!(format!("{value:.0}"), canonical);
        assert_eq!(format!("{value:.1}"), canonical);
        assert_eq!(format!("{value:.50}"), canonical);
    }
    for scale in 1..=38_u8 {
        let expected = format!("0.{}1", "0".repeat(usize::from(scale - 1)));
        assert_eq!(a(1, scale).to_string(), expected);
        assert_eq!(a(-1, scale).to_string(), format!("-{expected}"));
    }
}

#[test]
fn rounding_registry_is_exact_and_unknown_mode_text_retains_its_full_input() {
    // FR-AMT-10/11/13: decimal input cap is not an identifier parser cap.
    let expected = [
        (Rounding::TowardZero, "TowardZero"),
        (Rounding::AwayFromZero, "AwayFromZero"),
        (Rounding::TowardPositiveInfinity, "TowardPositiveInfinity"),
        (Rounding::TowardNegativeInfinity, "TowardNegativeInfinity"),
        (Rounding::NearestTiesEven, "NearestTiesEven"),
        (Rounding::NearestTiesAwayFromZero, "NearestTiesAwayFromZero"),
        (Rounding::Exact, "Exact"),
    ];
    assert_eq!(Rounding::ALL, expected.map(|(mode, _)| mode));
    assert_eq!(HashSet::from(Rounding::ALL).len(), 7);
    for (mode, text) in expected {
        assert_eq!(mode.canonical_identifier(), text);
        assert_eq!(mode.to_string(), text);
        assert_eq!(text.parse::<Rounding>(), Ok(mode));
        for bad in [text.to_lowercase(), format!(" {text}"), format!("{text} ")] {
            assert!(
                matches!(bad.parse::<Rounding>(),Err(IdentifierError::Unknown { concept:"amount rounding",text:input,.. }) if input==bad)
            );
        }
    }
    for bad in [
        String::new(),
        "HALF_EVEN".into(),
        "Nearest".into(),
        "x".repeat(513),
    ] {
        assert!(
            matches!(bad.parse::<Rounding>(),Err(IdentifierError::Unknown { concept:"amount rounding",text:input,.. }) if input==bad)
        );
    }
}

#[test]
fn small_exact_arithmetic_grid_has_independent_aligned_integer_answers() {
    // FR-AMT-04/12: 23,409 pairs at three scales, no arbitrary precision needed.
    for left_scale in 0..=2_u8 {
        for right_scale in 0..=2_u8 {
            for left in -25..=25_i128 {
                for right in -25..=25_i128 {
                    let x = a(left, left_scale);
                    let y = a(right, right_scale);
                    let basis = left_scale.max(right_scale);
                    let l = left * 10_i128.pow(u32::from(basis - left_scale));
                    let r = right * 10_i128.pow(u32::from(basis - right_scale));
                    assert_parts(good(x.checked_add(y)), l + r, basis);
                    assert_parts(good(x.checked_sub(y)), l - r, basis);
                    assert_parts(
                        good(x.checked_mul(y)),
                        left * right,
                        left_scale + right_scale,
                    );
                    assert_eq!(good(x.checked_add(y)), good(y.checked_add(x)));
                    assert_eq!(good(x.checked_mul(y)), good(y.checked_mul(x)));
                }
            }
        }
    }
}

#[test]
fn signed_limits_and_normalization_rescue_are_checked_on_the_final_result() {
    // FR-AMT-01/04/08/09: unsigned magnitudes are necessary for valid MIN.
    let min = a(i128::MIN, 0);
    let max = a(i128::MAX, 0);
    assert_eq!(good(min.checked_add(max)), a(-1, 0));
    assert_eq!(good(min.checked_sub(min)), a(0, 0));
    assert_eq!(good(max.checked_sub(max)), a(0, 0));
    assert_eq!(good(min.checked_add(a(1, 0))), a(i128::MIN + 1, 0));
    assert_range(rejected(min.checked_neg()), "negate", false);
    assert_range(rejected(a(0, 0).checked_sub(min)), "subtract", false);
    assert_range(rejected(min.checked_add(a(-1, 0))), "add", true);
    assert_range(rejected(max.checked_add(a(1, 0))), "add", false);
    assert_range(rejected(min.checked_mul(a(-1, 0))), "multiply", false);
    let integer = p("17014118346046923173168730371588410573");
    assert_eq!(good(integer.checked_add(p("-0.3"))), a(i128::MAX, 1));
    assert_eq!(good(integer.checked_sub(p("0.3"))), a(i128::MAX, 1));
    assert_eq!(
        good(p("-17014118346046923173168730371588410573").checked_add(p("0.2"))),
        a(i128::MIN, 1)
    );
    // 2^126 * 5^54 = 2^72 * 10^54: scale76 reduces to22 only after full product.
    let two = 2_i128.pow(126);
    let five = 5_i128.pow(54);
    assert_eq!(
        good(a(two, 38).checked_mul(a(five, 38))),
        a(2_i128.pow(72), 22)
    );
    for scale in 0..=38 {
        assert_range(rejected(a(i128::MIN, scale).checked_neg()), "negate", false);
        assert_eq!(good(a(i128::MIN, scale).checked_mul(a(0, 38))), a(0, 0));
        assert_eq!(
            good(a(i128::MIN, scale).checked_add(a(0, 38))),
            a(i128::MIN, scale)
        );
    }
    for value in [a(1, 38), a(i128::MAX, 38)] {
        let error = rejected(value.checked_mul(value));
        assert!(
            matches!(
                error,
                AmountError::ResultScale {
                    operation: "multiply",
                    scale: 76,
                    maximum: 38,
                    ..
                }
            ),
            "{error:?}"
        );
    }
}

#[test]
fn all_scales_and_modes_resolve_even_and_odd_quantum_count_ties() {
    // FR-AMT-06/07/08: all39 scales, both signs, even/odd count, no binary floats.
    for scale in 0..=38_u8 {
        let quantum = q(3, scale);
        let denominator = 2_u128 * 10_u128.pow(u32::from(scale));
        for (numerator, counts) in [
            (15_i128, [2_i128, 3, 3, 2, 2, 3]),
            (21_i128, [3_i128, 4, 4, 3, 4, 4]),
            (-15_i128, [-2_i128, -3, -2, -3, -2, -3]),
            (-21_i128, [-3_i128, -4, -3, -4, -4, -4]),
        ] {
            for (mode, count) in Rounding::ALL.into_iter().take(6).zip(counts) {
                let expected = a(count * 3, scale);
                assert_eq!(
                    good(DecimalAmount::from_ratio(
                        numerator,
                        denominator,
                        quantum,
                        mode
                    )),
                    expected
                );
                if scale < 38 {
                    let value = a(numerator * 5, scale + 1);
                    assert_eq!(good(value.quantize(quantum, mode)), expected);
                }
            }
            assert_inexact(
                rejected(DecimalAmount::from_ratio(
                    numerator,
                    denominator,
                    quantum,
                    Rounding::Exact,
                )),
                "ratio",
            );
        }
        for mode in Rounding::ALL {
            for count in [-11_i128, -1, 0, 1, 11] {
                let value = a(count * 3, scale);
                assert!(value.is_multiple_of(quantum));
                assert_eq!(good(value.quantize(quantum, mode)), value);
            }
        }
    }
}

#[test]
fn arbitrary_quantum_below_at_above_ties_and_directed_zero_have_literal_answers() {
    // FR-AMT-06/08: parity belongs to quantum count, not decimal cents.
    for (input, quantum, answers) in [
        (
            "0.124",
            "0.05",
            ["0.10", "0.15", "0.15", "0.10", "0.10", "0.10"],
        ),
        (
            "0.125",
            "0.05",
            ["0.10", "0.15", "0.15", "0.10", "0.10", "0.15"],
        ),
        (
            "0.126",
            "0.05",
            ["0.10", "0.15", "0.15", "0.10", "0.15", "0.15"],
        ),
        (
            "0.174",
            "0.05",
            ["0.15", "0.20", "0.20", "0.15", "0.15", "0.15"],
        ),
        (
            "0.175",
            "0.05",
            ["0.15", "0.20", "0.20", "0.15", "0.20", "0.20"],
        ),
        (
            "0.176",
            "0.05",
            ["0.15", "0.20", "0.20", "0.15", "0.20", "0.20"],
        ),
        (
            "-0.125",
            "0.05",
            ["-0.10", "-0.15", "-0.10", "-0.15", "-0.10", "-0.15"],
        ),
        (
            "-0.175",
            "0.05",
            ["-0.15", "-0.20", "-0.15", "-0.20", "-0.20", "-0.20"],
        ),
        (
            "0.0625",
            "0.025",
            ["0.05", "0.075", "0.075", "0.05", "0.05", "0.075"],
        ),
        (
            "0.0925",
            "0.037",
            ["0.074", "0.111", "0.111", "0.074", "0.074", "0.111"],
        ),
        ("-0.00001", "0.05", ["0", "-0.05", "0", "-0.05", "0", "0"]),
        ("0.00001", "0.05", ["0", "0.05", "0.05", "0", "0", "0"]),
    ] {
        let value = p(input);
        let unit = p(quantum);
        let quantum = q(unit.coefficient(), unit.scale());
        assert!(!value.is_multiple_of(quantum));
        for (mode, expected) in Rounding::ALL.into_iter().take(6).zip(answers) {
            let result = good(value.quantize(quantum, mode));
            assert_eq!(result, p(expected), "{input} / {unit} / {mode}");
            assert!(result.is_multiple_of(quantum));
        }
        assert_inexact(
            rejected(value.quantize(quantum, Rounding::Exact)),
            "quantize",
        );
    }
}

#[test]
fn quantization_checks_remainder_before_range_and_allows_huge_quantum_counts() {
    // FR-AMT-06/08/09/11: k need not fit an i128 if final amount does.
    let max = a(i128::MAX, 0);
    let min = a(i128::MIN, 0);
    assert_inexact(rejected(max.quantize(q(2, 0), Rounding::Exact)), "quantize");
    for mode in [
        Rounding::AwayFromZero,
        Rounding::TowardPositiveInfinity,
        Rounding::NearestTiesEven,
        Rounding::NearestTiesAwayFromZero,
    ] {
        assert_range(rejected(max.quantize(q(2, 0), mode)), "quantize", false);
    }
    assert_eq!(
        good(max.quantize(q(2, 0), Rounding::TowardZero)),
        a(i128::MAX - 1, 0)
    );
    assert_eq!(good(min.quantize(q(2, 0), Rounding::Exact)), min);
    assert_eq!(
        good(min.quantize(q(3, 0), Rounding::TowardZero)),
        a(i128::MIN + 2, 0)
    );
    assert_range(
        rejected(min.quantize(q(3, 0), Rounding::NearestTiesEven)),
        "quantize",
        true,
    );
    assert_eq!(
        good(min.quantize(q(i128::MAX, 0), Rounding::NearestTiesEven)),
        a(-i128::MAX, 0)
    );
    for mode in Rounding::ALL {
        assert_eq!(good(max.quantize(q(1, 38), mode)), max);
        assert_eq!(good(min.quantize(q(1, 38), mode)), min);
        assert_eq!(
            good(DecimalAmount::from_ratio(i128::MAX, 1, q(1, 38), mode)),
            max
        );
        assert_eq!(
            good(DecimalAmount::from_ratio(i128::MIN, 1, q(1, 38), mode)),
            min
        );
    }
}

#[test]
fn zero_denominators_precede_every_zero_operand_shortcut_and_rounding_choice() {
    // FR-AMT-07/11: no mathematical zero expression validates a zero divisor.
    for mode in Rounding::ALL {
        for value in [a(0, 0), a(1, 38), a(i128::MIN, 0)] {
            for (error, operation) in [
                (
                    rejected(DecimalAmount::from_ratio(
                        value.coefficient(),
                        0,
                        q(1, 38),
                        mode,
                    )),
                    "ratio",
                ),
                (
                    rejected(value.multiply_ratio(0, 0, q(1, 38), mode)),
                    "multiply_ratio",
                ),
                (
                    rejected(a(0, 0).multiply_ratio(value.coefficient(), 0, q(1, 38), mode)),
                    "multiply_ratio",
                ),
                (
                    rejected(value.multiply_divide(a(0, 0), 0, q(1, 38), mode)),
                    "multiply_divide",
                ),
                (
                    rejected(a(0, 0).multiply_divide(value, 0, q(1, 38), mode)),
                    "multiply_divide",
                ),
            ] {
                assert!(
                    matches!(error,AmountError::ZeroDenominator {operation:o,..} if o==operation),
                    "{error:?}"
                );
                assert_eq!(error.kind(), "amount.zero_denominator");
            }
        }
    }
}

#[test]
fn coprime_unsigned_denominators_and_widest_fused_paths_keep_exact_remainders() {
    // FR-AMT-07/09/12: signed MIN and u128::MAX cannot use signed abs/division.
    let tiny = q(1, 38);
    let half = a(5, 1);
    assert_eq!(
        good(DecimalAmount::from_ratio(
            i128::MIN,
            u128::MAX,
            tiny,
            Rounding::NearestTiesEven
        )),
        a(-5, 1)
    );
    assert_eq!(
        good(DecimalAmount::from_ratio(
            i128::MAX,
            u128::MAX,
            tiny,
            Rounding::NearestTiesEven
        )),
        half
    );
    assert_eq!(
        good(DecimalAmount::from_ratio(
            i128::MIN,
            u128::MAX,
            tiny,
            Rounding::TowardZero
        )),
        a(-5, 1)
    );
    assert_eq!(
        good(DecimalAmount::from_ratio(
            i128::MIN,
            u128::MAX,
            tiny,
            Rounding::AwayFromZero
        )),
        a(-(5 * 10_i128.pow(37) + 1), 38)
    );
    assert_eq!(
        good(DecimalAmount::from_ratio(
            i128::MAX,
            u128::MAX,
            tiny,
            Rounding::TowardZero
        )),
        a(5 * 10_i128.pow(37) - 1, 38)
    );
    assert_eq!(
        good(DecimalAmount::from_ratio(
            1,
            u128::MAX,
            tiny,
            Rounding::TowardZero
        )),
        a(0, 0)
    );
    assert_eq!(
        good(DecimalAmount::from_ratio(
            1,
            u128::MAX,
            tiny,
            Rounding::AwayFromZero
        )),
        a(1, 38)
    );
    assert_inexact(
        rejected(DecimalAmount::from_ratio(
            1,
            u128::MAX,
            tiny,
            Rounding::Exact,
        )),
        "ratio",
    );
    let min = a(i128::MIN, 0);
    let max = a(i128::MAX, 0);
    assert_eq!(
        good(min.multiply_divide(min, u128::MAX, q(1, 0), Rounding::NearestTiesEven)),
        a(2_i128.pow(126), 0)
    );
    assert_eq!(
        good(max.multiply_divide(max, u128::MAX, q(1, 0), Rounding::NearestTiesEven)),
        a(2_i128.pow(126) - 1, 0)
    );
    assert_eq!(
        good(min.multiply_divide(max, u128::MAX, q(1, 0), Rounding::NearestTiesEven)),
        a(-2_i128.pow(126), 0)
    );
    let extreme = q(i128::MAX, 38);
    let small_min = a(i128::MIN, 38);
    assert_eq!(
        good(small_min.multiply_divide(small_min, u128::MAX, extreme, Rounding::TowardZero)),
        a(0, 0)
    );
    assert_eq!(
        good(small_min.multiply_divide(small_min, u128::MAX, extreme, Rounding::AwayFromZero)),
        extreme.amount()
    );
    assert_inexact(
        rejected(small_min.multiply_divide(small_min, u128::MAX, extreme, Rounding::Exact)),
        "multiply_divide",
    );
}

#[test]
fn fused_two_decimal_product_rounds_once_and_rescues_unrepresentable_intermediates() {
    // FR-AMT-07/08: the frozen discriminator cannot use checked_mul or price/100 first.
    let max = a(i128::MAX, 0);
    let price = a(10_i128.pow(37) + 1, 37);
    assert_range(rejected(max.checked_mul(price)), "multiply", false);
    assert_eq!(
        good(max.multiply_divide(price, 100, q(1, 0), Rounding::NearestTiesEven)),
        p("1701411834604692317316873037158841057")
    );
    assert_eq!(
        good(price.multiply_divide(max, 100, q(1, 0), Rounding::NearestTiesEven)),
        p("1701411834604692317316873037158841057")
    );
    // Staged rounding loses .005: (1.005*2)/1 rounds to2.01, not2.00.
    let fine = p("1.005");
    let direct = good(fine.multiply_divide(a(2, 0), 1, q(1, 2), Rounding::NearestTiesEven));
    let staged = good(good(fine.quantize(q(1, 2), Rounding::NearestTiesEven)).checked_mul(a(2, 0)));
    assert_eq!(direct, p("2.01"));
    assert_eq!(staged, p("2"));
    assert_ne!(direct, staged);
    for scale in 0..=38 {
        let value = a(i128::MIN, scale);
        assert_eq!(
            good(value.multiply_ratio(
                i128::MAX,
                i128::MAX.unsigned_abs(),
                q(1, 38),
                Rounding::Exact
            )),
            value
        );
        assert_eq!(
            good(value.multiply_divide(a(1, 0), 1, q(1, 38), Rounding::Exact)),
            value
        );
    }
    assert_inexact(
        rejected(p("1.01").multiply_ratio(1, 2, q(1, 2), Rounding::Exact)),
        "multiply_ratio",
    );
    assert_range(
        rejected(max.multiply_ratio(2, 1, q(1, 0), Rounding::Exact)),
        "multiply_ratio",
        false,
    );
    assert_range(
        rejected(max.multiply_divide(a(2, 0), 1, q(1, 0), Rounding::Exact)),
        "multiply_divide",
        false,
    );
}

fn for_each_permutation(
    values: &mut [DecimalAmount],
    index: usize,
    check: &mut impl FnMut(&[DecimalAmount]),
) {
    if index == values.len() {
        check(values);
    } else {
        for other in index..values.len() {
            values.swap(index, other);
            for_each_permutation(values, index + 1, check);
            values.swap(index, other);
        }
    }
}

#[test]
fn exact_sum_defers_admission_until_every_cancelled_prefix_is_removed() {
    // FR-AMT-05/09: all permutations preserve value or complete error class.
    assert_eq!(good(exact_amount_sum(&[])), a(0, 0));
    let max = a(i128::MAX, 0);
    let min = a(i128::MIN, 0);
    for (mut values, expected) in [
        (vec![max, max, min, min], a(-2, 0)),
        (vec![max, max, a(-i128::MAX, 0), a(-i128::MAX, 0)], a(0, 0)),
        (vec![max, a(1, 38), a(-i128::MAX, 0)], a(1, 38)),
        (
            vec![
                p("17014118346046923173168730371588410573"),
                p("-0.1"),
                p("-0.2"),
            ],
            a(i128::MAX, 1),
        ),
    ] {
        for_each_permutation(&mut values, 0, &mut |order| {
            assert_eq!(good(exact_amount_sum(order)), expected)
        });
    }
    for (mut values, negative) in [
        (vec![max, max, min, a(2, 0)], false),
        (vec![min, min, max], true),
    ] {
        for_each_permutation(&mut values, 0, &mut |order| {
            assert_range(rejected(exact_amount_sum(order)), "sum", negative)
        });
    }
    let mut balanced = Vec::new();
    for scale in 0..=38 {
        balanced.extend([a(i128::MAX, scale), a(-i128::MAX, scale)]);
    }
    balanced.extend([max, max, min, min, a(2, 0)]);
    assert_eq!(good(exact_amount_sum(&balanced)), a(0, 0));
    balanced.reverse();
    assert_eq!(good(exact_amount_sum(&balanced)), a(0, 0));
}

#[test]
fn numeric_order_and_hash_ignore_scale_spelling_but_preserve_sign() {
    // FR-AMT-01: Ord compares exact numeric values across huge scale differences.
    let expected = [
        a(i128::MIN, 0),
        a(-i128::MAX, 0),
        a(-2, 0),
        a(i128::MIN, 38),
        p("-1.5"),
        a(-1, 0),
        a(-1, 38),
        a(0, 0),
        a(1, 38),
        a(9, 38),
        a(1, 37),
        p("0.99"),
        a(1, 0),
        p("1.5"),
        a(i128::MAX, 38),
        a(2, 0),
        a(i128::MAX, 0),
    ];
    for (left_index, left) in expected.iter().enumerate() {
        for (right_index, right) in expected.iter().enumerate() {
            assert_eq!(left.cmp(right), left_index.cmp(&right_index));
            assert_eq!(left.partial_cmp(right), Some(left_index.cmp(&right_index)));
        }
    }
    let tree: BTreeSet<_> = expected.into_iter().rev().collect();
    assert_eq!(tree.into_iter().collect::<Vec<_>>(), expected);
    let mut cache = HashMap::new();
    let mut units = HashSet::new();
    for scale in 0..=38 {
        let coefficient = 10_i128.pow(u32::from(scale));
        for sign in [-1, 0, 1] {
            let value = a(coefficient * sign, scale);
            assert_eq!(value, a(sign, 0));
            assert_eq!(hash(&value), hash(&a(sign, 0)));
            cache.insert(value, sign);
        }
        let quantum = q(coefficient, scale);
        assert_eq!(quantum, q(1, 0));
        assert_eq!(hash(&quantum), hash(&q(1, 0)));
        units.insert(quantum);
    }
    assert_eq!(cache.len(), 3);
    assert_eq!(units.len(), 1);
    assert!(q(1, 38) < q(1, 37));
    assert!(q(i128::MAX, 38) < q(2, 0));
}

#[test]
fn every_reachable_domain_error_preserves_context_and_one_root_source_layer() {
    // FR-AMT-11: construct errors through public inputs; Display text is not stable.
    let errors = [
        rejected("0".repeat(129).parse::<DecimalAmount>()),
        rejected("1.a".parse::<DecimalAmount>()),
        rejected(DecimalAmount::new(0, 39)),
        rejected(AmountQuantum::new(-10, 1)),
        rejected(DecimalAmount::from_ratio(0, 0, q(1, 0), Rounding::Exact)),
        rejected(a(1, 1).quantize(q(1, 0), Rounding::Exact)),
        rejected(a(1, 38).checked_mul(a(1, 38))),
        rejected(a(i128::MIN, 0).checked_neg()),
    ];
    let kinds = [
        "amount.text_too_long",
        "amount.invalid_syntax",
        "amount.invalid_scale",
        "amount.invalid_quantum",
        "amount.zero_denominator",
        "amount.inexact",
        "amount.result_scale",
        "amount.out_of_range",
    ];
    for (error, kind) in errors.into_iter().zip(kinds) {
        assert_eq!(error.kind(), kind);
        assert!(error.source().is_none());
        assert_eq!(error, error.clone());
        assert!(!format!("{error:?}").is_empty());
        assert!(!error.to_string().is_empty());
        let root: CrateError = error.clone().into();
        assert_eq!(root.kind(), kind);
        assert_eq!(root.to_string(), error.to_string());
        assert!(matches!(&root,CrateError::Amount(cause) if cause==&error));
        assert!(
            root.source()
                .is_some_and(
                    |source| source.downcast_ref::<AmountError>() == Some(&error)
                        && source.source().is_none()
                )
        );
    }
}

#[test]
fn immutable_amounts_quanta_modes_and_errors_are_shareable_across_threads() {
    // FR-AMT-13: shared immutable contexts do not acquire a global rounding mode.
    fn ordered<T: Copy + Clone + Debug + Eq + Hash + Ord + PartialOrd + Send + Sync + 'static>() {}
    fn mode<T: Copy + Clone + Debug + Eq + Hash + Send + Sync + 'static>() {}
    fn domain_error<T: Clone + Debug + Eq + Error + Send + Sync + 'static>() {}
    ordered::<DecimalAmount>();
    ordered::<AmountQuantum>();
    mode::<Rounding>();
    domain_error::<AmountError>();
    let values = [p("0.125"), p("-0.125"), a(i128::MIN, 38), a(i128::MAX, 0)];
    let quantum = q(5, 2);
    std::thread::scope(|scope| {
        let mut workers = Vec::new();
        for index in 0..8 {
            let shared = &values;
            workers.push(scope.spawn(move || {
                for _ in 0..256 {
                    let mode = if index % 2 == 0 {
                        Rounding::NearestTiesEven
                    } else {
                        Rounding::NearestTiesAwayFromZero
                    };
                    let magnitude = if index % 2 == 0 { p("0.1") } else { p("0.15") };
                    assert_eq!(good(shared[0].quantize(quantum, mode)), magnitude);
                    assert_eq!(
                        good(shared[1].quantize(quantum, mode)),
                        good(magnitude.checked_neg())
                    );
                    assert_eq!(
                        good(exact_amount_sum(&[shared[3], a(-i128::MAX, 0), shared[2]])),
                        shared[2]
                    );
                    assert_eq!(p(&shared[2].to_string()), shared[2]);
                }
            }));
        }
        for worker in workers {
            assert!(worker.join().is_ok());
        }
    });
}
