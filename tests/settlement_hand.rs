//! Independent public hand/adversarial controls for FR-SETTLE-01..16.
//!
//! Authored from the frozen M8 settlement contract before reading settlement
//! production. Expectations use integer identities, signed rounding tables and
//! exact IEEE bit values, not production output or the generated oracle corpus.
//! TOL-E0 applies. Proof-excluded InternalCapacity is deliberately not forged.

#![expect(
    clippy::expect_used,
    reason = "hand controls explicitly assert successful construction/evaluation"
)]

use sabiquant::amounts::{AmountError, AmountQuantum, AmountRounding as R, DecimalAmount};
use sabiquant::error::{Error as RootError, IdentifierError, NonFinite};
use sabiquant::settlement::{
    CashAggregation as Aggregation, SettlementDecision as Decision, SettlementError,
    SettlementEvidence as Evidence, SettlementGuarantee as Guarantee,
    SettlementInstruction as Instruction, SettlementReceipt as Receipt,
    SettlementRounding as Rounding,
};
use std::collections::{BTreeSet, HashSet};
use std::error::Error;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};

const SIGN: u64 = 1 << 63;
const TWO130: u64 = (1023 + 130) << 52;

fn a(coefficient: i128, scale: u8) -> DecimalAmount {
    DecimalAmount::new(coefficient, scale).expect("valid hand amount")
}

fn p(text: &str) -> DecimalAmount {
    text.parse().expect("valid exact hand decimal")
}

fn d(text: &str) -> Evidence {
    Evidence::exact_decimal(p(text))
}

fn b(bits: u64) -> Evidence {
    Evidence::supplied_binary64(f64::from_bits(bits)).expect("finite hand bits")
}

fn di(lower: &str, upper: &str) -> Evidence {
    Evidence::declared_decimal_interval(p(lower), p(upper)).expect("ordered decimal bounds")
}

fn bi(lower: u64, upper: u64) -> Evidence {
    Evidence::declared_binary64_interval(f64::from_bits(lower), f64::from_bits(upper))
        .expect("ordered finite binary bounds")
}

fn round(coefficient: i128, scale: u8, mode: R) -> Rounding {
    Rounding::new(
        AmountQuantum::new(coefficient, scale).expect("positive hand quantum"),
        mode,
    )
}

fn dirty(nominal: DecimalAmount, evidence: Evidence, rounding: Rounding) -> Instruction {
    Instruction::dirty_per_100(nominal, evidence, rounding).expect("positive par nominal")
}

fn clean(
    nominal: DecimalAmount,
    primary: Evidence,
    secondary: Evidence,
    aggregation: Aggregation,
    rounding: Rounding,
) -> Instruction {
    Instruction::clean_accrued_per_100(nominal, primary, secondary, aggregation, rounding)
        .expect("positive par nominal")
}

fn staged(
    nominal: DecimalAmount,
    primary: Evidence,
    secondary: Evidence,
    source_modes: [R; 2],
    aggregation: Aggregation,
    rounding: Rounding,
) -> Instruction {
    Instruction::staged_price100_accrued1000(
        nominal,
        primary,
        secondary,
        source_modes[0],
        source_modes[1],
        aggregation,
        rounding,
    )
    .expect("positive par nominal")
}

#[expect(
    clippy::panic,
    reason = "a mismatched public decision fails its hand control"
)]
fn receipt(instruction: Instruction, expected: DecimalAmount, guarantee: Guarantee) -> Receipt {
    match instruction.evaluate().expect("determined hand evaluation") {
        Decision::Determined(value) => {
            assert_eq!(value.amount(), expected);
            assert_eq!(value.instruction(), &instruction);
            assert_eq!(value.guarantee(), guarantee);
            value
        }
        other => panic!("expected determined {expected}, got {other:?}"),
    }
}

fn ambiguous(instruction: Instruction, lower: DecimalAmount, upper: DecimalAmount) {
    assert!(lower < upper);
    assert!(matches!(
        instruction.evaluate(),
        Ok(Decision::QuantumAmbiguous { lower_amount, upper_amount, .. })
            if lower_amount == lower && upper_amount == upper
    ));
}

fn unresolved(instruction: Instruction, stage: &'static str) {
    assert!(matches!(
        instruction.evaluate(),
        Ok(Decision::ExactStageUnresolved { stage: actual, .. }) if actual == stage
    ));
}

fn inexact(instruction: Instruction, stage: &'static str, endpoint: &'static str) {
    let error = instruction
        .evaluate()
        .expect_err("hand point is not a quantum multiple");
    assert!(matches!(
        error,
        SettlementError::Arithmetic {
            stage: actual_stage,
            endpoint: actual_endpoint,
            cause: AmountError::Inexact { operation: "quantize", .. },
            ..
        } if actual_stage == stage && actual_endpoint == endpoint
    ));
}

fn out_of_range(
    instruction: Instruction,
    stage: &'static str,
    endpoint: &'static str,
    operation: &'static str,
    negative: bool,
) {
    let error = instruction
        .evaluate()
        .expect_err("hand final coefficient exceeds i128");
    assert!(matches!(
        error,
        SettlementError::Arithmetic {
            stage: actual_stage,
            endpoint: actual_endpoint,
            cause: AmountError::OutOfRange {
                operation: actual_operation,
                negative: actual_negative,
                ..
            },
            ..
        } if actual_stage == stage && actual_endpoint == endpoint
            && actual_operation == operation && actual_negative == negative
    ));
}

fn hash(value: &impl Hash) -> u64 {
    let mut state = DefaultHasher::new();
    value.hash(&mut state);
    state.finish()
}

/// FR-SETTLE-04/12: exact registries describe policies, not a hidden default.
#[test]
fn registries_round_trip_and_reject_case_trimming_and_aliases() {
    assert_eq!(
        Aggregation::ALL,
        [Aggregation::RoundTotal, Aggregation::RoundComponentsThenSum]
    );
    assert_eq!(
        Guarantee::ALL,
        [
            Guarantee::ForSuppliedDecimals,
            Guarantee::ForSuppliedBinary64,
            Guarantee::UniqueForDeclaredEnclosure,
        ]
    );
    for (value, identifier) in Aggregation::ALL
        .into_iter()
        .zip(["RoundTotal", "RoundComponentsThenSum"])
    {
        assert_eq!(value.canonical_identifier(), identifier);
        assert_eq!(value.to_string(), identifier);
        assert_eq!(identifier.parse::<Aggregation>(), Ok(value));
    }
    for (value, identifier) in Guarantee::ALL.into_iter().zip([
        "ForSuppliedDecimals",
        "ForSuppliedBinary64",
        "UniqueForDeclaredEnclosure",
    ]) {
        assert_eq!(value.canonical_identifier(), identifier);
        assert_eq!(value.to_string(), identifier);
        assert_eq!(identifier.parse::<Guarantee>(), Ok(value));
    }
    for text in [
        "",
        "roundtotal",
        " RoundTotal",
        "RoundTotal ",
        "Total",
        "\0",
        "รวม",
    ] {
        assert!(matches!(
            text.parse::<Aggregation>(),
            Err(IdentifierError::Unknown { concept: "cash aggregation", text: actual, .. })
                if actual == text
        ));
    }
    for text in [
        "",
        "forsupplieddecimals",
        "ForSuppliedDecimals\n",
        " UniqueForDeclaredEnclosure",
        "Certified",
        "\0",
    ] {
        assert!(matches!(
            text.parse::<Guarantee>(),
            Err(IdentifierError::Unknown { concept: "settlement guarantee", text: actual, .. })
                if actual == text
        ));
    }
}

/// FR-SETTLE-02/03/12: kind and declared provenance survive numerical equality.
#[test]
fn evidence_bounds_identity_and_signed_zero_are_canonical() {
    let decimal = d("1.25");
    let binary = b(1.25_f64.to_bits());
    let decimal_interval = di("1.2500", "1.25");
    let binary_interval = bi(1.25_f64.to_bits(), 1.25_f64.to_bits());
    assert_eq!(decimal.decimal_bounds(), Some((p("1.25"), p("1.25"))));
    assert_eq!(decimal.binary64_bounds(), None);
    assert_eq!(decimal_interval.decimal_bounds(), decimal.decimal_bounds());
    assert_eq!(binary.decimal_bounds(), None);
    for evidence in [binary, binary_interval] {
        let (lower, upper) = evidence.binary64_bounds().expect("binary kind bounds");
        assert_eq!([lower.to_bits(), upper.to_bits()], [1.25_f64.to_bits(); 2]);
    }
    assert!(!decimal.is_declared_interval());
    assert!(!binary.is_declared_interval());
    assert!(decimal_interval.is_declared_interval());
    assert!(binary_interval.is_declared_interval());
    assert_eq!(
        HashSet::from([decimal, binary, decimal_interval, binary_interval]).len(),
        4
    );
    assert_eq!(decimal, Evidence::exact_decimal(a(12500, 4)));
    assert_eq!(hash(&decimal), hash(&Evidence::exact_decimal(a(12500, 4))));

    assert_eq!(b(0), b(SIGN));
    assert_eq!(hash(&b(0)), hash(&b(SIGN)));
    for evidence in [b(0), b(SIGN), bi(SIGN, 0), bi(0, SIGN)] {
        let (lower, upper) = evidence.binary64_bounds().expect("zero binary bounds");
        assert_eq!([lower.to_bits(), upper.to_bits()], [0; 2]);
    }
    for bits in [1, (1 << 52) - 1, 1 << 52, f64::MAX.to_bits(), SIGN | 1] {
        let (lower, upper) = b(bits).binary64_bounds().expect("finite IEEE boundary");
        assert_eq!([lower.to_bits(), upper.to_bits()], [bits; 2]);
    }
}

/// FR-SETTLE-03/14: all finite checks precede order and zero shortcuts.
#[test]
fn nonfinite_endpoint_order_is_lower_then_upper() {
    let cases = [
        (f64::INFINITY.to_bits(), NonFinite::PositiveInfinity),
        (f64::NEG_INFINITY.to_bits(), NonFinite::NegativeInfinity),
        (0x7FF8_0000_0000_0000, NonFinite::NaN),
        (0xFFF8_0000_0000_0001, NonFinite::NaN),
        (0x7FF0_0000_0000_0001, NonFinite::NaN),
        (0xFFF7_FFFF_FFFF_FFFF, NonFinite::NaN),
    ];
    for (bits, class) in cases {
        assert!(matches!(
            Evidence::supplied_binary64(f64::from_bits(bits)),
            Err(SettlementError::NonFinite { endpoint: "value", value, .. }) if value == class
        ));
        for other in [f64::NEG_INFINITY, f64::MAX, 0.0, f64::NAN] {
            assert!(matches!(
                Evidence::declared_binary64_interval(f64::from_bits(bits), other),
                Err(SettlementError::NonFinite { endpoint: "lower", value, .. }) if value == class
            ));
        }
        for lower in [-f64::MAX, -0.0, 0.0, f64::MAX] {
            assert!(matches!(
                Evidence::declared_binary64_interval(lower, f64::from_bits(bits)),
                Err(SettlementError::NonFinite { endpoint: "upper", value, .. }) if value == class
            ));
        }
    }
    assert!(matches!(
        Evidence::declared_binary64_interval(1.0, -1.0),
        Err(SettlementError::BinaryIntervalOrder { lower_bits, upper_bits, .. })
            if lower_bits == 1.0_f64.to_bits() && upper_bits == (-1.0_f64).to_bits()
    ));
    assert!(matches!(
        Evidence::declared_binary64_interval(f64::from_bits(1), -0.0),
        Err(SettlementError::BinaryIntervalOrder {
            lower_bits: 1,
            upper_bits: 0,
            ..
        })
    ));
    assert!(matches!(
        Evidence::declared_decimal_interval(p("0.101"), p("0.1")),
        Err(SettlementError::DecimalIntervalOrder { lower, upper, .. })
            if lower == p("0.101") && upper == p("0.1")
    ));
}

/// FR-SETTLE-01/04/05/12: inspectors preserve units, absence and source modes.
#[test]
fn all_four_recipe_inspectors_retain_every_explicit_policy() {
    let nominal = p("123.45");
    let primary = di("99.1234564", "99.1234566");
    let secondary = b(2.0_f64.to_bits());
    let policy = round(25, 3, R::TowardNegativeInfinity);
    let instructions = [
        dirty(nominal, primary, policy),
        clean(
            nominal,
            primary,
            secondary,
            Aggregation::RoundComponentsThenSum,
            policy,
        ),
        staged(
            nominal,
            primary,
            secondary,
            [R::NearestTiesAwayFromZero, R::TowardPositiveInfinity],
            Aggregation::RoundTotal,
            policy,
        ),
        Instruction::coupon_per_1000(nominal, primary, policy).expect("positive nominal"),
    ];
    let ids = [
        "DirtyPer100",
        "CleanAccruedPer100",
        "StagedPrice100Accrued1000",
        "CouponPer1000",
    ];
    for (index, instruction) in instructions.into_iter().enumerate() {
        assert_eq!(instruction.nominal(), nominal);
        assert_eq!(instruction.primary(), primary);
        assert_eq!(instruction.cash_rounding(), policy);
        assert_eq!(instruction.recipe_identifier(), ids[index]);
        assert_eq!(
            instruction.secondary(),
            if index == 1 || index == 2 {
                Some(secondary)
            } else {
                None
            }
        );
        assert_eq!(
            instruction.aggregation(),
            match index {
                1 => Some(Aggregation::RoundComponentsThenSum),
                2 => Some(Aggregation::RoundTotal),
                _ => None,
            }
        );
        if index == 2 {
            assert_eq!(
                instruction.source_price_rounding(),
                Some(round(1, 6, R::NearestTiesAwayFromZero))
            );
            assert_eq!(
                instruction.source_accrued_rounding(),
                Some(round(1, 5, R::TowardPositiveInfinity))
            );
        } else {
            assert_eq!(instruction.source_price_rounding(), None);
            assert_eq!(instruction.source_accrued_rounding(), None);
        }
    }
    assert_eq!(policy.quantum().amount(), p("0.025"));
    assert_eq!(policy.mode(), R::TowardNegativeInfinity);
}

/// FR-SETTLE-01/14: zero or huge price never waives positive exact par.
#[test]
fn every_factory_rejects_zero_and_negative_nominal_without_evaluating_prices() {
    let huge = b(f64::MAX.to_bits());
    let policy = round(1, 38, R::Exact);
    for nominal in [a(0, 0), a(-1, 38), a(-1, 0), a(i128::MIN, 0)] {
        let results = [
            Instruction::dirty_per_100(nominal, huge, policy),
            Instruction::clean_accrued_per_100(
                nominal,
                d("0"),
                huge,
                Aggregation::RoundTotal,
                policy,
            ),
            Instruction::staged_price100_accrued1000(
                nominal,
                huge,
                d("0.000001"),
                R::Exact,
                R::Exact,
                Aggregation::RoundComponentsThenSum,
                policy,
            ),
            Instruction::coupon_per_1000(nominal, d("0"), policy),
        ];
        for result in results {
            assert!(matches!(
                result,
                Err(SettlementError::NonPositiveNominal { nominal: actual, .. }) if actual == nominal
            ));
        }
    }
    for nominal in [a(1, 38), a(i128::MAX, 0)] {
        receipt(
            dirty(nominal, d("0"), policy),
            a(0, 0),
            Guarantee::ForSuppliedDecimals,
        );
    }
}

/// FR-SETTLE-01/05: par is a monetary nominal; coupon units differ by ten.
#[test]
fn units_and_signed_party_directions_follow_the_four_formulas() {
    let policy = round(1, 2, R::Exact);
    for (dirty_text, clean_text, accrued_text, coupon_text, trade, coupon) in [
        ("101.25", "100", "1.25", "12.5", "1012.5", "12.5"),
        ("-101.25", "-100", "-1.25", "-12.5", "-1012.5", "-12.5"),
        ("0", "0", "0", "0", "0", "0"),
    ] {
        let nominal = p("1000");
        let instructions = [
            dirty(nominal, d(dirty_text), policy),
            clean(
                nominal,
                d(clean_text),
                d(accrued_text),
                Aggregation::RoundTotal,
                policy,
            ),
            staged(
                nominal,
                d(clean_text),
                d(coupon_text),
                [R::Exact; 2],
                Aggregation::RoundComponentsThenSum,
                policy,
            ),
        ];
        for instruction in instructions {
            receipt(instruction, p(trade), Guarantee::ForSuppliedDecimals);
        }
        receipt(
            Instruction::coupon_per_1000(nominal, d(coupon_text), policy)
                .expect("positive nominal"),
            p(coupon),
            Guarantee::ForSuppliedDecimals,
        );
    }
}

/// FR-SETTLE-04/05: ties use quantum-count parity and the signed expression.
#[test]
fn all_seven_modes_apply_to_signed_arbitrary_quantum_counts() {
    for (mode, positive, negative) in [
        (R::TowardZero, "0.1", "-0.1"),
        (R::AwayFromZero, "0.15", "-0.15"),
        (R::TowardPositiveInfinity, "0.15", "-0.1"),
        (R::TowardNegativeInfinity, "0.1", "-0.15"),
        (R::NearestTiesEven, "0.1", "-0.1"),
        (R::NearestTiesAwayFromZero, "0.15", "-0.15"),
    ] {
        for (source, expected) in [("0.125", positive), ("-0.125", negative)] {
            receipt(
                dirty(p("100"), d(source), round(5, 2, mode)),
                p(expected),
                Guarantee::ForSuppliedDecimals,
            );
        }
    }
    for source in ["0.125", "-0.125"] {
        inexact(
            dirty(p("100"), d(source), round(5, 2, R::Exact)),
            "cash total",
            "point",
        );
    }
    for (source, quantum, expected) in [
        ("0.175", (5, 2), "0.2"),
        ("0.3125", (125, 3), "0.25"),
        ("4.5", (3, 0), "6"),
        ("-4.5", (3, 0), "-6"),
        ("0.05", (2, 2), "0.04"),
    ] {
        receipt(
            dirty(
                p("100"),
                d(source),
                round(quantum.0, quantum.1, R::NearestTiesEven),
            ),
            p(expected),
            Guarantee::ForSuppliedDecimals,
        );
    }
}

/// FR-SETTLE-04/13: every legal scale and mode keeps an exact multiple exact.
#[test]
fn all_thirty_nine_scales_have_exact_quantum_controls() {
    for scale in 0..=38 {
        for mode in R::ALL {
            for sign in [-1, 1] {
                let amount = a(21 * sign, scale);
                receipt(
                    dirty(
                        p("100"),
                        Evidence::exact_decimal(amount),
                        round(7, scale, mode),
                    ),
                    amount,
                    Guarantee::ForSuppliedDecimals,
                );
            }
        }
    }
}

/// FR-SETTLE-06/07: component rounding is observable and has no final re-round.
#[test]
fn aggregation_and_directed_rounding_have_distinct_exact_results() {
    for (mode, total, components) in [
        (R::NearestTiesEven, "1", "0"),
        (R::NearestTiesAwayFromZero, "1", "2"),
    ] {
        for (aggregation, expected) in [
            (Aggregation::RoundTotal, total),
            (Aggregation::RoundComponentsThenSum, components),
        ] {
            receipt(
                clean(p("100"), d("0.5"), d("0.5"), aggregation, round(1, 0, mode)),
                p(expected),
                Guarantee::ForSuppliedDecimals,
            );
        }
    }
    for (aggregation, expected) in [
        (Aggregation::RoundTotal, "0"),
        (Aggregation::RoundComponentsThenSum, "1"),
    ] {
        receipt(
            clean(
                p("100"),
                d("0.2"),
                d("-0.2"),
                aggregation,
                round(1, 0, R::TowardPositiveInfinity),
            ),
            p(expected),
            Guarantee::ForSuppliedDecimals,
        );
    }
    for (aggregation, expected) in [
        (Aggregation::RoundTotal, "0.09"),
        (Aggregation::RoundComponentsThenSum, "0.12"),
    ] {
        receipt(
            clean(
                p("100"),
                d("0.045"),
                d("0.045"),
                aggregation,
                round(3, 2, R::NearestTiesEven),
            ),
            p(expected),
            Guarantee::ForSuppliedDecimals,
        );
    }
}

/// FR-SETTLE-05/06: source rounding and unrounded coupon scaling are different.
#[test]
fn fixed_source_stages_are_observable_before_cash_rounding() {
    let nominal = p("1000000");
    let policy = round(1, 2, R::NearestTiesEven);
    for (source_mode, expected) in [
        (R::NearestTiesEven, "1000000"),
        (R::NearestTiesAwayFromZero, "1000000.02"),
    ] {
        receipt(
            staged(
                nominal,
                d("100.0000005"),
                d("0.000005"),
                [source_mode; 2],
                Aggregation::RoundTotal,
                policy,
            ),
            p(expected),
            Guarantee::ForSuppliedDecimals,
        );
    }
    receipt(
        clean(
            nominal,
            d("100.0000005"),
            d("0.0000005"),
            Aggregation::RoundTotal,
            policy,
        ),
        p("1000000.01"),
        Guarantee::ForSuppliedDecimals,
    );
    receipt(
        Instruction::coupon_per_1000(nominal, d("0.000005"), round(1, 3, R::Exact))
            .expect("positive nominal"),
        p("0.005"),
        Guarantee::ForSuppliedDecimals,
    );
    for (mode, expected) in [
        (R::TowardPositiveInfinity, "0"),
        (R::TowardNegativeInfinity, "-0.01"),
    ] {
        receipt(
            staged(
                nominal,
                d("-0.0000005"),
                d("0"),
                [mode, R::Exact],
                Aggregation::RoundTotal,
                round(1, 2, R::Exact),
            ),
            p(expected),
            Guarantee::ForSuppliedDecimals,
        );
    }
}

/// FR-SETTLE-02/05/12: binary .1 is above decimal .1, not its shortest text.
#[test]
fn supplied_binary64_is_the_exact_represented_real() {
    let decimal = dirty(p("100"), d("0.1"), round(1, 18, R::NearestTiesEven));
    let binary = dirty(
        p("100"),
        b(0x3FB9_9999_9999_999A),
        round(1, 18, R::NearestTiesEven),
    );
    receipt(decimal, p("0.1"), Guarantee::ForSuppliedDecimals);
    // 3602879701896397 / 2^55 = .100000000000000005551115... .
    receipt(
        binary,
        p("0.100000000000000006"),
        Guarantee::ForSuppliedBinary64,
    );
    receipt(
        dirty(p("100"), d("0.1"), round(1, 1, R::Exact)),
        p("0.1"),
        Guarantee::ForSuppliedDecimals,
    );
    inexact(
        dirty(p("100"), b(0x3FB9_9999_9999_999A), round(1, 1, R::Exact)),
        "cash total",
        "point",
    );
    for (bits, sign) in [(1, 1), (SIGN | 1, -1)] {
        receipt(
            dirty(p("100"), b(bits), round(1, 38, R::TowardZero)),
            a(0, 0),
            Guarantee::ForSuppliedBinary64,
        );
        receipt(
            dirty(p("100"), b(bits), round(1, 38, R::AwayFromZero)),
            a(sign, 38),
            Guarantee::ForSuppliedBinary64,
        );
        inexact(
            dirty(p("100"), b(bits), round(1, 38, R::Exact)),
            "cash total",
            "point",
        );
    }
}

/// FR-SETTLE-06/13: source values need not fit any public decimal intermediate.
#[test]
fn huge_source_and_cash_components_can_be_rescued_only_by_the_final_net() {
    // 2^130 / 10^40 = .1361129467683753853853498429727072845824.
    for instruction in [
        dirty(a(1, 38), b(TWO130), round(1, 2, R::NearestTiesEven)),
        staged(
            a(1, 38),
            b(TWO130),
            d("0"),
            [R::Exact; 2],
            Aggregation::RoundTotal,
            round(1, 2, R::NearestTiesEven),
        ),
    ] {
        receipt(instruction, p("0.14"), Guarantee::ForSuppliedBinary64);
    }

    let positive = Evidence::exact_decimal(a(i128::MAX, 0));
    let negative = Evidence::exact_decimal(a(-i128::MAX, 0));
    for (aggregation, expected) in [
        (Aggregation::RoundTotal, 0),
        (Aggregation::RoundComponentsThenSum, 1),
    ] {
        // X=MAX^2/100 has remainder29/100. ceil(X)+ceil(-X)=1;
        // both individual rounded magnitudes are much larger than i128.
        receipt(
            clean(
                a(i128::MAX, 0),
                positive,
                negative,
                aggregation,
                round(1, 0, R::TowardPositiveInfinity),
            ),
            a(expected, 0),
            Guarantee::ForSuppliedDecimals,
        );
    }

    // Price=2^1020 and accrued/1000=-10*2^1020 cancel with the actual 10+1
    // common-basis weights. Both finite source values exceed public range.
    let price_bits = (1023_u64 + 1020) << 52;
    let accrued_bits = SIGN | ((1023_u64 + 1023) << 52) | (1 << 50);
    for aggregation in Aggregation::ALL {
        for mode in R::ALL {
            receipt(
                staged(
                    p("1000"),
                    b(price_bits),
                    b(accrued_bits),
                    [R::Exact; 2],
                    aggregation,
                    round(1, 0, mode),
                ),
                a(0, 0),
                Guarantee::ForSuppliedBinary64,
            );
        }
    }
}

/// FR-SETTLE-07/10: mandatory Exact stages have a complete stable order.
#[test]
fn exact_source_price_then_source_accrued_then_cash_components() {
    let policy = round(1, 0, R::Exact);
    inexact(
        staged(
            p("100"),
            d("0.0000001"),
            d("0.000001"),
            [R::Exact; 2],
            Aggregation::RoundComponentsThenSum,
            policy,
        ),
        "source price",
        "point",
    );
    inexact(
        staged(
            p("100"),
            d("0"),
            d("0.000001"),
            [R::Exact; 2],
            Aggregation::RoundComponentsThenSum,
            policy,
        ),
        "source accrued",
        "point",
    );
    unresolved(
        staged(
            p("100"),
            di("0.0000001", "0.0000002"),
            d("0.000001"),
            [R::Exact; 2],
            Aggregation::RoundComponentsThenSum,
            policy,
        ),
        "source price",
    );
    for (capital, accrued, stage) in [
        ("0.5", "0.5", "cash capital"),
        ("1", "0.5", "cash accrued"),
        ("0.5", "-0.5", "cash capital"),
    ] {
        inexact(
            clean(
                p("100"),
                d(capital),
                d(accrued),
                Aggregation::RoundComponentsThenSum,
                policy,
            ),
            stage,
            "point",
        );
    }
    for (capital, accrued, expected) in [("0.5", "0.5", "1"), ("0.5", "-0.5", "0")] {
        receipt(
            clean(
                p("100"),
                d(capital),
                d(accrued),
                Aggregation::RoundTotal,
                policy,
            ),
            p(expected),
            Guarantee::ForSuppliedDecimals,
        );
    }
}

/// FR-SETTLE-07/10/14: wide earlier values do not preempt a later Exact error.
#[test]
fn no_future_range_or_exact_prevalidation_changes_required_stage_order() {
    let policy = round(1, 0, R::Exact);
    inexact(
        clean(
            p("100"),
            d("0.5"),
            b(f64::MAX.to_bits()),
            Aggregation::RoundComponentsThenSum,
            policy,
        ),
        "cash capital",
        "point",
    );
    inexact(
        clean(
            p("100"),
            b(TWO130),
            d("0.5"),
            Aggregation::RoundComponentsThenSum,
            policy,
        ),
        "cash accrued",
        "point",
    );
    inexact(
        staged(
            p("100"),
            b(f64::MAX.to_bits()),
            d("0.000001"),
            [R::Exact; 2],
            Aggregation::RoundTotal,
            policy,
        ),
        "source accrued",
        "point",
    );
    unresolved(
        staged(
            p("100"),
            d("0"),
            di("0", "0.00001"),
            [R::Exact; 2],
            Aggregation::RoundTotal,
            policy,
        ),
        "source accrued",
    );
}

/// FR-SETTLE-09/10: all-real intervals are not finite-float enumerations.
#[test]
fn continuous_exact_images_are_unresolved_but_declared_points_are_point_errors() {
    let policy = round(1, 0, R::Exact);
    unresolved(dirty(p("100"), di("0", "1"), policy), "cash total");
    unresolved(
        dirty(
            p("100"),
            bi(1.0_f64.to_bits(), 1.0_f64.to_bits() + 1),
            policy,
        ),
        "cash total",
    );
    unresolved(
        Instruction::coupon_per_1000(p("1000"), di("0", "1"), policy).expect("positive nominal"),
        "cash coupon",
    );
    for evidence in [di("0.5", "0.5"), bi(0.5_f64.to_bits(), 0.5_f64.to_bits())] {
        inexact(dirty(p("100"), evidence, policy), "cash total", "point");
    }
    receipt(
        dirty(p("100"), di("1", "1"), policy),
        p("1"),
        Guarantee::UniqueForDeclaredEnclosure,
    );
}

/// FR-SETTLE-09: source-count steps, rather than only extrema, govern Exact.
#[test]
fn discrete_exact_images_detect_undefined_interior_counts_and_integral_steps() {
    let evidence = di("0", "0.000002");
    let policy = round(1, 6, R::Exact);
    // Source counts0,1,2 become cash0,.5e-6,1e-6; both endpoints are Exact,
    // but the middle is not. Checking extrema alone incorrectly accepts it.
    unresolved(
        staged(
            p("50"),
            evidence,
            d("0"),
            [R::NearestTiesEven, R::Exact],
            Aggregation::RoundTotal,
            policy,
        ),
        "cash total",
    );
    unresolved(
        staged(
            p("50"),
            evidence,
            d("0"),
            [R::NearestTiesEven, R::Exact],
            Aggregation::RoundComponentsThenSum,
            policy,
        ),
        "cash capital",
    );
    // Scaling by100 gives integral one-quantum steps and a defined Exact image.
    ambiguous(
        staged(
            p("100"),
            evidence,
            d("0"),
            [R::NearestTiesEven, R::Exact],
            Aggregation::RoundTotal,
            policy,
        ),
        p("0"),
        p("0.000002"),
    );
    ambiguous(
        staged(
            p("1000"),
            d("0"),
            di("0", "0.00002"),
            [R::Exact, R::TowardZero],
            Aggregation::RoundComponentsThenSum,
            round(1, 5, R::Exact),
        ),
        p("0"),
        p("0.00002"),
    );
    // A collapsed source image at an inexact base is a point error, even
    // though the original evidence is a nondegenerate declared interval.
    inexact(
        staged(
            p("50"),
            di("0.0000006", "0.0000014"),
            d("0"),
            [R::NearestTiesEven, R::Exact],
            Aggregation::RoundTotal,
            policy,
        ),
        "cash total",
        "point",
    );
}

/// FR-SETTLE-08/09: the two evidence inputs are independent, not correlated.
#[test]
fn independent_images_cannot_invent_cancellation_or_net_rescue_exact_components() {
    ambiguous(
        clean(
            p("100"),
            di("0", "1"),
            di("-1", "0"),
            Aggregation::RoundTotal,
            round(1, 0, R::NearestTiesEven),
        ),
        p("-1"),
        p("1"),
    );
    unresolved(
        staged(
            p("50"),
            di("0", "0.000002"),
            di("-0.00002", "0"),
            [R::NearestTiesEven; 2],
            Aggregation::RoundTotal,
            round(1, 6, R::Exact),
        ),
        "cash total",
    );
    unresolved(
        staged(
            p("50"),
            d("0"),
            di("0", "0.00002"),
            [R::Exact, R::NearestTiesEven],
            Aggregation::RoundComponentsThenSum,
            round(1, 6, R::Exact),
        ),
        "cash accrued",
    );
}

/// FR-SETTLE-08/11/12: intermediate source differences can yield unique cash.
#[test]
fn intermediate_ambiguity_collapses_only_at_the_required_final_stage() {
    receipt(
        staged(
            p("1"),
            di("1.0000001", "1.0000019"),
            d("0"),
            [R::NearestTiesEven, R::Exact],
            Aggregation::RoundTotal,
            round(1, 2, R::NearestTiesEven),
        ),
        p("0.01"),
        Guarantee::UniqueForDeclaredEnclosure,
    );
    receipt(
        dirty(p("1"), di("0.1", "0.2"), round(1, 2, R::NearestTiesEven)),
        p("0"),
        Guarantee::UniqueForDeclaredEnclosure,
    );
    for aggregation in Aggregation::ALL {
        ambiguous(
            clean(
                p("100"),
                di("0", "1"),
                d("0"),
                aggregation,
                round(1, 0, R::NearestTiesEven),
            ),
            p("0"),
            p("1"),
        );
    }
}

/// Exact small signed count oracle; all callers have |numerator|<100.
#[expect(
    clippy::panic,
    reason = "a future rounding mode needs a new independent hand rule"
)]
fn small_count(numerator: i128, denominator: i128, mode: R) -> Option<i128> {
    let count = numerator / denominator;
    let remainder = (numerator % denominator).abs();
    if remainder == 0 {
        return Some(count);
    }
    let increment = match mode {
        R::TowardZero => false,
        R::AwayFromZero => true,
        R::TowardPositiveInfinity => numerator > 0,
        R::TowardNegativeInfinity => numerator < 0,
        R::NearestTiesEven => {
            2 * remainder > denominator || (2 * remainder == denominator && count % 2 != 0)
        }
        R::NearestTiesAwayFromZero => 2 * remainder >= denominator,
        R::Exact => return None,
        _ => panic!("unreviewed new rounding mode"),
    };
    Some(count + if increment { numerator.signum() } else { 0 })
}

/// FR-SETTLE-08/09/16: enumerate complete small source images around all cells.
#[test]
fn complete_small_staged_images_match_all_signed_source_and_cash_modes() {
    let modes = [
        R::TowardZero,
        R::AwayFromZero,
        R::TowardPositiveInfinity,
        R::TowardNegativeInfinity,
        R::NearestTiesEven,
        R::NearestTiesAwayFromZero,
    ];
    let mut cases = 0;
    for price_mode in modes {
        // In source-quantum units these inputs are[-1.5,2.5] and[-2.5,1.5].
        // All seven-mode transitions occur at integer or half-integer cells.
        // Quarter-grid probes include each tie and one point in every adjacent
        // open cell, so these sets exhaust the continuous intervals' images.
        let price_counts: BTreeSet<_> = (-6..=10)
            .map(|quarter| small_count(quarter, 4, price_mode).expect("non-Exact source mode"))
            .collect();
        for accrued_mode in modes {
            let accrued_counts: BTreeSet<_> = (-10..=6)
                .map(|quarter| {
                    small_count(quarter, 4, accrued_mode).expect("non-Exact source mode")
                })
                .collect();
            for cash_mode in modes {
                for aggregation in Aggregation::ALL {
                    let mut final_counts = BTreeSet::new();
                    for price in &price_counts {
                        for accrued in &accrued_counts {
                            // N150 makes either source-count step1.5e-6.
                            // With cash q2e-6, its quantum-count ratio is3/4.
                            let cash_count = match aggregation {
                                Aggregation::RoundTotal => {
                                    small_count(3 * (price + accrued), 4, cash_mode)
                                        .expect("non-Exact cash mode")
                                }
                                Aggregation::RoundComponentsThenSum => {
                                    small_count(3 * price, 4, cash_mode)
                                        .expect("non-Exact capital mode")
                                        + small_count(3 * accrued, 4, cash_mode)
                                            .expect("non-Exact accrued mode")
                                }
                                _ => unreachable_aggregation(),
                            };
                            final_counts.insert(cash_count);
                        }
                    }
                    let lower = a(2 * final_counts.first().expect("nonempty image"), 6);
                    let upper = a(2 * final_counts.last().expect("nonempty image"), 6);
                    let instruction = staged(
                        p("150"),
                        di("-0.0000015", "0.0000025"),
                        di("-0.000025", "0.000015"),
                        [price_mode, accrued_mode],
                        aggregation,
                        round(2, 6, cash_mode),
                    );
                    if lower == upper {
                        receipt(instruction, lower, Guarantee::UniqueForDeclaredEnclosure);
                    } else {
                        ambiguous(instruction, lower, upper);
                    }
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 432);
}

#[expect(
    clippy::panic,
    reason = "a future aggregation requires independent hand expectations"
)]
fn unreachable_aggregation() -> i128 {
    panic!("unreviewed new cash aggregation")
}

/// FR-SETTLE-10/14: final labels follow quantizer input pointness, not output.
#[test]
fn final_admission_preserves_pointness_before_cash_rounding() {
    let policy = round(1, 0, R::NearestTiesEven);
    for evidence in [b(TWO130), bi(TWO130, TWO130)] {
        out_of_range(
            dirty(p("100"), evidence, policy),
            "cash total",
            "point",
            "quantize",
            false,
        );
    }
    out_of_range(
        dirty(p("100"), bi(TWO130, TWO130 + 1), policy),
        "cash total",
        "lower",
        "quantize",
        false,
    );
    // Both cash endpoints round to 2*MAX, which is outside public range.
    // The pre-quantizer image is nonpoint, so the first failure is lower.
    let two128 = (1023_u64 + 128) << 52;
    out_of_range(
        dirty(
            p("100"),
            bi(two128, two128 + 1),
            round(i128::MAX, 0, R::NearestTiesEven),
        ),
        "cash total",
        "lower",
        "quantize",
        false,
    );
    // Source price collapses to101; the later cash input is now a point.
    out_of_range(
        staged(
            a(i128::MAX, 0),
            di("101.0000001", "101.0000002"),
            d("0"),
            [R::NearestTiesEven, R::Exact],
            Aggregation::RoundTotal,
            policy,
        ),
        "cash total",
        "point",
        "quantize",
        false,
    );
    out_of_range(
        Instruction::coupon_per_1000(p("1000"), b(SIGN | TWO130), policy)
            .expect("positive nominal"),
        "cash coupon",
        "point",
        "quantize",
        true,
    );
}

/// FR-SETTLE-10/11/14: net admission is lower then upper, with operation sum.
#[test]
fn cash_net_pointness_and_upper_admission_have_distinct_context() {
    let max = Evidence::exact_decimal(a(i128::MAX, 0));
    let policy = round(1, 0, R::NearestTiesEven);
    for (aggregation, stage, operation) in [
        (Aggregation::RoundTotal, "cash total", "quantize"),
        (Aggregation::RoundComponentsThenSum, "cash net", "sum"),
    ] {
        out_of_range(
            clean(p("100"), di("0", "1"), max, aggregation, policy),
            stage,
            "upper",
            operation,
            false,
        );
        out_of_range(
            clean(
                p("100"),
                di("-170141183460469231731687303715884105727", "0"),
                Evidence::exact_decimal(a(-i128::MAX, 0)),
                aggregation,
                policy,
            ),
            stage,
            "lower",
            operation,
            true,
        );
    }
    // Each variable component cash image collapses toMAX; their exact net
    // image is a point2*MAX, despite both original declared inputs varying.
    out_of_range(
        clean(
            a(i128::MAX, 0),
            di("99", "101"),
            di("99", "101"),
            Aggregation::RoundComponentsThenSum,
            round(i128::MAX, 0, R::NearestTiesEven),
        ),
        "cash net",
        "point",
        "sum",
        false,
    );
}

/// FR-SETTLE-11: representable extrema do not assert every interior fits i128.
#[test]
fn quantum_ambiguity_does_not_certify_interior_decimal_representability() {
    let lower_coefficient = 17014118346046923173168730371588410573_i128;
    let lower = a(lower_coefficient, 37);
    let upper = a(lower_coefficient + 1, 37);
    assert!(lower_coefficient > i128::MAX / 10);
    // At q1e-38 the endpoint counts10*C and10*(C+1) both normalize. Count
    // 10*C+1 lies between them, cannot normalize and exceeds i128::MAX.
    let evidence =
        Evidence::declared_decimal_interval(lower, upper).expect("ordered small interval");
    ambiguous(
        dirty(p("100"), evidence, round(1, 38, R::NearestTiesEven)),
        lower,
        upper,
    );
    unresolved(
        dirty(p("100"), evidence, round(1, 38, R::Exact)),
        "cash total",
    );
}

/// FR-SETTLE-06/13: valid finite extremes test the complete fixed-width route.
#[test]
fn maximum_finite_binary_sources_cancel_without_an_early_public_range_check() {
    let positive = b(f64::MAX.to_bits());
    let negative = b(f64::MAX.to_bits() | SIGN);
    for aggregation in Aggregation::ALL {
        for mode in R::ALL {
            receipt(
                clean(
                    p("1000"),
                    positive,
                    negative,
                    aggregation,
                    round(1, 38, mode),
                ),
                p("0"),
                Guarantee::ForSuppliedBinary64,
            );
        }
    }
    for (evidence, negative) in [(positive, false), (negative, true)] {
        out_of_range(
            dirty(a(1, 38), evidence, round(1, 38, R::NearestTiesEven)),
            "cash total",
            "point",
            "quantize",
            negative,
        );
    }
}

/// FR-SETTLE-04/06/13: signed i128 bounds are asymmetric, including after ties.
#[test]
fn signed_final_limits_and_half_quantum_admission_use_the_exact_result() {
    for scale in 0..=38 {
        for coefficient in [i128::MIN, i128::MAX] {
            let expected = a(coefficient, scale);
            receipt(
                dirty(
                    p("100"),
                    Evidence::exact_decimal(expected),
                    round(1, scale, R::Exact),
                ),
                expected,
                Guarantee::ForSuppliedDecimals,
            );
        }
    }
    let max = Evidence::exact_decimal(a(i128::MAX, 0));
    let minus_max = Evidence::exact_decimal(a(-i128::MAX, 0));
    for mode in [R::NearestTiesEven, R::NearestTiesAwayFromZero] {
        out_of_range(
            clean(
                p("100"),
                max,
                d("0.5"),
                Aggregation::RoundTotal,
                round(1, 0, mode),
            ),
            "cash total",
            "point",
            "quantize",
            false,
        );
        // MAX is odd. The negative tie selects -(MAX+1)=MIN, which fits.
        receipt(
            clean(
                p("100"),
                minus_max,
                d("-0.5"),
                Aggregation::RoundTotal,
                round(1, 0, mode),
            ),
            a(i128::MIN, 0),
            Guarantee::ForSuppliedDecimals,
        );
    }
    inexact(
        clean(
            p("100"),
            max,
            d("0.5"),
            Aggregation::RoundTotal,
            round(1, 0, R::Exact),
        ),
        "cash total",
        "point",
    );
    receipt(
        dirty(
            p("100"),
            Evidence::exact_decimal(a(i128::MIN, 0)),
            round(i128::MAX, 0, R::TowardZero),
        ),
        a(-i128::MAX, 0),
        Guarantee::ForSuppliedDecimals,
    );
    out_of_range(
        dirty(
            p("100"),
            Evidence::exact_decimal(a(i128::MIN, 0)),
            round(i128::MAX, 0, R::AwayFromZero),
        ),
        "cash total",
        "point",
        "quantize",
        true,
    );
}

/// FR-SETTLE-02/12: equality is the complete definition, not just proceeds.
#[test]
fn receipt_identity_retains_evidence_family_and_every_policy() {
    let n = p("100");
    let zero = d("0");
    let total = Aggregation::RoundTotal;
    let component = Aggregation::RoundComponentsThenSum;
    let near = round(1, 2, R::NearestTiesEven);
    let exact = round(1, 2, R::Exact);
    let instructions = [
        dirty(n, zero, near),
        dirty(n, zero, exact),
        dirty(n, zero, round(2, 2, R::NearestTiesEven)),
        dirty(n, b(0), near),
        dirty(n, di("0", "0"), near),
        dirty(n, bi(0, 0), near),
        clean(n, zero, zero, total, near),
        clean(n, zero, zero, component, near),
        staged(n, zero, zero, [R::Exact; 2], total, near),
        staged(n, zero, zero, [R::TowardZero, R::Exact], total, near),
        staged(n, zero, zero, [R::Exact, R::TowardZero], total, near),
        Instruction::coupon_per_1000(n, zero, near).expect("positive nominal"),
    ];
    let mut definitions = HashSet::new();
    let mut receipts = HashSet::new();
    for instruction in instructions {
        assert!(definitions.insert(instruction));
        let guarantee = if instruction.primary().is_declared_interval() {
            Guarantee::UniqueForDeclaredEnclosure
        } else if instruction.primary().binary64_bounds().is_some() {
            Guarantee::ForSuppliedBinary64
        } else {
            Guarantee::ForSuppliedDecimals
        };
        let value = receipt(instruction, p("0"), guarantee);
        assert!(receipts.insert(value));
    }
    assert_eq!(definitions.len(), instructions.len());
    assert_eq!(receipts.len(), instructions.len());
    let canonical = dirty(
        a(10000, 2),
        Evidence::exact_decimal(a(0, 38)),
        round(100, 4, R::NearestTiesEven),
    );
    assert_eq!(canonical, instructions[0]);
    assert_eq!(hash(&canonical), hash(&instructions[0]));
    assert!(receipts.contains(&receipt(canonical, p("0"), Guarantee::ForSuppliedDecimals,)));
    // Guarantee precedence includes secondary evidence and equal declarations.
    for (primary, secondary, guarantee) in [
        (zero, b(0), Guarantee::ForSuppliedBinary64),
        (b(0), di("0", "0"), Guarantee::UniqueForDeclaredEnclosure),
        (di("0", "0"), b(0), Guarantee::UniqueForDeclaredEnclosure),
    ] {
        receipt(clean(n, primary, secondary, total, near), p("0"), guarantee);
    }
}

/// FR-SETTLE-14: root and nested typed source chains retain exact causes.
#[test]
fn reachable_error_catalog_and_typed_root_sources_are_complete() {
    let nominal = Instruction::dirty_per_100(p("0"), d("0"), round(1, 0, R::Exact))
        .expect_err("nonpositive nominal");
    let decimal =
        Evidence::declared_decimal_interval(p("1"), p("0")).expect_err("reversed decimals");
    let binary =
        Evidence::declared_binary64_interval(1.0, 0.0).expect_err("reversed binary bounds");
    let finite = Evidence::supplied_binary64(f64::INFINITY).expect_err("nonfinite source");
    let arithmetic = dirty(p("100"), d("0.5"), round(1, 0, R::Exact))
        .evaluate()
        .expect_err("inexact quantum point");
    let cases = [
        (nominal, "settlement.nonpositive_nominal"),
        (decimal, "settlement.decimal_interval_order"),
        (binary, "settlement.binary_interval_order"),
        (finite, "settlement.non_finite"),
        (arithmetic, "settlement.arithmetic"),
    ];
    for (error, kind) in cases {
        assert_eq!(error.kind(), kind);
        assert_eq!(error, error.clone());
        assert!(!error.to_string().is_empty());
        let root = RootError::from(error.clone());
        assert_eq!(root.kind(), kind);
        assert_eq!(root.to_string(), error.to_string());
        let domain = root
            .source()
            .expect("root exposes settlement domain")
            .downcast_ref::<SettlementError>()
            .expect("typed exact settlement source");
        assert_eq!(domain, &error);
        if kind == "settlement.arithmetic" {
            let amount = domain
                .source()
                .expect("arithmetic exposes AmountError")
                .downcast_ref::<AmountError>()
                .expect("typed amount cause");
            assert!(matches!(
                amount,
                AmountError::Inexact {
                    operation: "quantize",
                    ..
                }
            ));
            assert!(amount.source().is_none());
        } else {
            assert!(domain.source().is_none());
        }
    }
    // InternalCapacity is the sixth domain variant. Its proof excludes valid
    // public inputs; constructing a fake private value would not test the API.
}

/// FR-SETTLE-15: every public value is immutable and reusable across threads.
#[test]
fn all_types_have_required_traits_and_concurrent_reuse_is_deterministic() {
    fn value_traits<T: Copy + Clone + Eq + Hash + Debug + Send + Sync + 'static>() {}
    value_traits::<Evidence>();
    value_traits::<Rounding>();
    value_traits::<Aggregation>();
    value_traits::<Instruction>();
    value_traits::<Decision>();
    value_traits::<Receipt>();
    value_traits::<Guarantee>();
    fn error_traits<T: Clone + Eq + Debug + Error + Send + Sync + 'static>() {}
    error_traits::<SettlementError>();

    let instructions = [
        dirty(
            p("100"),
            b(0x3FB9_9999_9999_999A),
            round(1, 18, R::NearestTiesEven),
        ),
        staged(
            p("100"),
            di("0", "0.000002"),
            d("0"),
            [R::NearestTiesEven, R::Exact],
            Aggregation::RoundTotal,
            round(1, 6, R::Exact),
        ),
        clean(
            p("100"),
            d("0.5"),
            d("0.5"),
            Aggregation::RoundComponentsThenSum,
            round(1, 0, R::Exact),
        ),
        dirty(p("1"), di("0.1", "0.2"), round(1, 2, R::NearestTiesEven)),
    ];
    let expected = instructions.map(|value| value.evaluate());
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    for _ in 0..16 {
                        for (instruction, expected) in instructions.iter().zip(&expected) {
                            assert_eq!(&instruction.evaluate(), expected);
                        }
                    }
                })
            })
            .collect();
        for handle in handles {
            handle.join().expect("immutable settlement thread succeeds");
        }
    });
    assert_eq!(instructions.map(|value| value.evaluate()), expected);
}
