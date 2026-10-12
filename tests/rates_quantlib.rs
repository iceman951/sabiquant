//! Public QuantLib differential evidence, with narrow numerical/domain classifications.

#![expect(
    clippy::unwrap_used,
    reason = "malformed committed fixtures fail immediately in reference validation"
)]
#![expect(
    clippy::float_cmp,
    reason = "exact zero/domain boundaries and recorded binary rounding stages are the contract"
)]

mod support;

use std::collections::{BTreeMap, BTreeSet};

use sabiquant::daycount::DayCount;
use sabiquant::rates::{Compounding, CompoundingMethod, InterestRate, RateError};
use sabiquant::time::Frequency;
use support::fixture::{Fixture, Record};
use support::float::Tolerance;

fn compounding(row: &Record<'_>, method: &str, frequency: &str) -> Compounding {
    match row.parse::<CompoundingMethod>(method).unwrap() {
        CompoundingMethod::Simple => Compounding::SIMPLE,
        CompoundingMethod::Continuous => Compounding::CONTINUOUS,
        CompoundingMethod::Compounded => {
            Compounding::compounded(row.parse(frequency).unwrap()).unwrap()
        }
        CompoundingMethod::SimpleThenCompounded => {
            Compounding::simple_then_compounded(row.parse(frequency).unwrap()).unwrap()
        }
        CompoundingMethod::CompoundedThenSimple => {
            Compounding::compounded_then_simple(row.parse(frequency).unwrap()).unwrap()
        }
    }
}

fn execute(row: &Record<'_>) -> Result<f64, RateError> {
    let source = compounding(row, "method", "frequency");
    let time = row.real("time").unwrap();
    match row.text("operation").unwrap() {
        "implied" => Ok(InterestRate::implied_rate(
            row.real("factor").unwrap(),
            time,
            DayCount::Actual365Fixed,
            source,
        )?
        .rate()),
        operation => {
            let quote =
                InterestRate::new(row.real("rate").unwrap(), DayCount::Actual365Fixed, source)?;
            match operation {
                "compound" => quote.compound_factor(time),
                "discount" => quote.discount_factor(time),
                _ => Ok(quote
                    .equivalent_rate(compounding(row, "target_method", "target_frequency"), time)?
                    .rate()),
            }
        }
    }
}

fn periodic(compounding: Compounding, time: f64) -> bool {
    match compounding.method() {
        CompoundingMethod::Compounded => true,
        CompoundingMethod::SimpleThenCompounded => {
            time > 1.0 / f64::from(compounding.frequency().unwrap().events_per_year().unwrap())
        }
        CompoundingMethod::CompoundedThenSimple => {
            time <= 1.0 / f64::from(compounding.frequency().unwrap().events_per_year().unwrap())
        }
        _ => false,
    }
}

fn simple(compounding: Compounding, time: f64) -> bool {
    compounding.method() != CompoundingMethod::Continuous && !periodic(compounding, time)
}

fn factor_tolerance(expected: f64) -> Tolerance {
    Tolerance::Mixed {
        absolute: if expected.is_subnormal() {
            f64::from_bits(1)
        } else {
            0.0
        },
        relative: 8.0 * (f64::EPSILON / 2.0) * (1.0 + expected.ln().abs()),
    }
}

fn tolerance(operation: &str, expected: f64, time: f64, target: Compounding) -> Tolerance {
    if matches!(operation, "compound" | "discount") {
        return factor_tolerance(expected);
    }
    if expected == 0.0 {
        return Tolerance::Mixed {
            absolute: 0.0,
            relative: 0.0,
        };
    }
    let unit = f64::EPSILON / 2.0;
    // Same derived E4 bound as the independent-oracle test. These derivatives
    // account for elementary argument error; source pow/base rounding has a
    // separate D5 record and never changes this implementation budget.
    let extra = if periodic(target, time) {
        let count = f64::from(target.frequency().unwrap().events_per_year().unwrap());
        let scaled = expected / count;
        if scaled.abs() < f64::EPSILON {
            unit * expected.abs()
        } else {
            let argument = if scaled <= -0.5 {
                ((count + expected) / count).ln()
            } else {
                scaled.ln_1p()
            };
            (unit * argument.abs()) * (count + expected)
        }
    } else if operation == "equivalent" && simple(target, time) {
        let product = expected * time;
        if product <= -1.0 {
            0.0
        } else if product.abs() < f64::EPSILON {
            unit * expected.abs()
        } else {
            let argument = if product.is_infinite() {
                expected.ln() + time.ln()
            } else {
                product.ln_1p()
            };
            (unit * argument.abs()) * expected.abs() + (unit * argument.abs()) / time
        }
    } else {
        0.0
    };
    let bound = 8.0 * (unit * expected.abs() + unit / time + extra);
    Tolerance::Mixed {
        absolute: if bound.is_finite() { bound } else { f64::MAX },
        relative: 0.0,
    }
}

fn same_binary(left: f64, right: f64) -> bool {
    left.to_bits() == right.to_bits() || left.is_nan() && right.is_nan()
}

fn model_matches(actual: f64, expected: f64) -> bool {
    if expected.is_finite() && expected > 0.0 {
        factor_tolerance(expected).check(actual, expected).is_ok()
    } else {
        same_binary(actual, expected)
    }
}

fn count(compounding: Compounding) -> Option<f64> {
    compounding
        .frequency()
        .map(|frequency| f64::from(frequency.events_per_year().unwrap()))
}

#[expect(
    clippy::panic,
    reason = "a boundary error must name a declared guarded quantity"
)]
fn check_numerical_boundary(error: &RateError) {
    if let RateError::NumericalBoundary {
        quantity,
        logarithm,
        ..
    } = error
    {
        assert!(logarithm.is_finite());
        let near = |boundary: f64| {
            (logarithm - boundary).abs() <= 8.0 * (f64::EPSILON / 2.0) * (1.0 + logarithm.abs())
        };
        match *quantity {
            "compound factor" => assert!(
                near(f64::MAX.ln()) || near(f64::from_bits(1).ln() - std::f64::consts::LN_2)
            ),
            "discount factor" => assert!(near(f64::MAX.ln())),
            _ => panic!("unknown guarded quantity {quantity}"),
        }
    }
}

#[expect(
    clippy::panic,
    reason = "false finite outputs and unjustified guard errors are defects"
)]
fn check_guard_edge(row: &Record<'_>, actual: &Result<f64, RateError>) -> Option<String> {
    let classification = row.text("guard_classification").unwrap();
    if classification.is_empty() {
        return None;
    }
    assert_eq!(classification, "D5-RANGE-GUARD-EDGE");
    let score = row.real("guard_edge_score").unwrap();
    assert!((6.0..=10.0).contains(&score));
    let scores = row
        .text("guard_scores")
        .unwrap()
        .split(',')
        .map(|part| part.parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    // A guard's outer edge cannot excuse the central region of another
    // applicable guard. Check compound guards plus rounded-C reciprocal.
    assert!(scores.len() >= 2 && scores.iter().all(|score| *score >= 6.0));
    let true_finite = row.text("mathematical_behavior").unwrap() == "value";
    match (true_finite, actual) {
        (true, Ok(value)) => {
            let expected = row.real("mathematical_expected").unwrap();
            assert!(value.is_finite() && *value > 0.0);
            assert!(expected.is_finite() && expected > 0.0);
            factor_tolerance(expected)
                .check(*value, expected)
                .unwrap_or_else(|error| panic!("{}: guard-edge value {error}", row.id()));
        }
        (_, Err(error @ RateError::NumericalBoundary { .. })) => check_numerical_boundary(error),
        (false, Err(RateError::Value(_))) => {}
        (true, Err(error)) => panic!("{}: finite truth cannot justify {error}", row.id()),
        (false, Ok(value)) => panic!("{}: false finite guard-edge value {value}", row.id()),
        (false, Err(error)) => panic!("{}: unexpected guard-edge error {error}", row.id()),
    }
    Some(match actual {
        Ok(_) => "value".to_owned(),
        Err(error) => error.kind().to_owned(),
    })
}

fn check_reference_pipeline(row: &Record<'_>, source: Compounding, target: Compounding) {
    let operation = row.text("operation").unwrap();
    let time = row.real("time").unwrap();
    let rate = row.real("rate").unwrap();
    let reference_behavior = row.text("reference_behavior").unwrap();
    if operation != "implied"
        && rate.is_finite()
        && time.is_finite()
        && time >= 0.0
        && count(source).is_none_or(|count| rate > -count)
    {
        let factor = row.real("source_factor").unwrap();
        assert!(
            model_matches(factor, row.real("source_model").unwrap()),
            "{}: independent source-factor model",
            row.id()
        );
        if periodic(source, time) {
            let count = count(source).unwrap();
            assert!(same_binary(
                row.real("rounded_base").unwrap(),
                1.0 + rate / count
            ));
            assert!(same_binary(
                row.real("rounded_exponent").unwrap(),
                count * time
            ));
        }
        if operation == "discount" && reference_behavior != "raises" {
            assert!(same_binary(
                row.real("reference_value").unwrap(),
                1.0 / factor
            ));
        }
    }
    let factor = if operation == "implied" {
        row.real("factor").unwrap()
    } else {
        row.real("source_factor").unwrap()
    };
    if matches!(operation, "implied" | "equivalent")
        && time.is_finite()
        && time > 0.0
        && factor > 0.0
        && (operation != "implied" || factor.is_finite())
        && (operation != "equivalent" || rate.is_finite())
        && reference_behavior != "raises"
    {
        let reference = row.real("reference_value").unwrap();
        assert!(same_binary(reference, row.real("inverse_model").unwrap()));
        if operation == "equivalent" {
            assert!(same_binary(
                reference,
                row.real("reference_implied").unwrap()
            ));
        }
        if factor != 1.0 && periodic(target, time) {
            let count = count(target).unwrap();
            let power = row.real("inverse_power").unwrap();
            assert!(same_binary(
                row.real("inverse_exponent").unwrap(),
                1.0 / (count * time)
            ));
            assert!(model_matches(
                power,
                row.real("inverse_power_model").unwrap()
            ));
            assert!(same_binary(reference, (power - 1.0) * count));
        }
    }
}

#[expect(
    clippy::panic,
    reason = "unknown discrepancy labels are fixture defects"
)]
fn check_classification(row: &Record<'_>, source: Compounding, target: Compounding) {
    let code = row.text("discrepancy").unwrap();
    let operation = row.text("operation").unwrap();
    let time = row.real("time").unwrap();
    let rate = row.real("rate").unwrap();
    let factor = row.real("source_factor").unwrap();
    let error = row.text("behavior").unwrap() == "error";
    let rejected = row.text("reference_behavior").unwrap() == "raises";
    let reference = if rejected {
        f64::NAN
    } else {
        row.real("reference_value").unwrap()
    };
    let differs = !error
        && (reference != 0.0 || row.real("expected").unwrap() == 0.0)
        && tolerance(operation, row.real("expected").unwrap(), time, target)
            .check(reference, row.real("expected").unwrap())
            .is_err();
    let zero_loss = !error && reference == 0.0 && row.real("expected").unwrap() != 0.0;
    match code {
        "" => {
            if error {
                assert!(rejected, "{}: unclassified accepted domain", row.id());
            } else {
                assert!(!rejected && !differs && !zero_loss, "{}", row.id());
            }
        }
        "D9-QUOTE-BASE" => {
            assert!(error && !rejected && operation != "implied");
            assert!(rate <= -count(source).unwrap());
        }
        "D9-QUOTE-NONFINITE" => {
            assert!(error && !rejected && operation != "implied" && !rate.is_finite());
        }
        "D9-TIME-NONFINITE" => {
            assert!(error && !rejected && !time.is_finite());
        }
        "D9-FACTOR-INPUT" => {
            assert!(error && !rejected && operation == "implied");
            assert!(!row.real("factor").unwrap().is_finite());
        }
        "D9-NUMERICAL-BOUNDARY" => {
            assert!(error && !rejected && matches!(operation, "compound" | "discount"));
            assert_eq!(row.text("error_kind").unwrap(), "rate.numerical_boundary");
            assert!(!simple(source, time));
            assert!(
                row.text("guard_scores")
                    .unwrap()
                    .split(',')
                    .any(|score| score.parse::<f64>().unwrap() <= 8.0)
            );
        }
        "D9-FACTOR-RESULT" => {
            assert!(error && !rejected && matches!(operation, "compound" | "discount"));
            assert!(!reference.is_finite() || reference <= 0.0);
        }
        "D9-NONFINITE-RATE" => {
            assert!(error && !rejected && !reference.is_finite());
            assert!(matches!(operation, "implied" | "equivalent"));
        }
        "D9-INFERRED-BASE" => {
            assert!(error && !rejected && reference <= -count(target).unwrap());
        }
        "D9-INFERRED-SIMPLE" => {
            assert!(error && !rejected && simple(target, time));
            assert!(reference.mul_add(time, 1.0) <= 0.0);
        }
        "D9-RATE-UNDERFLOW" => {
            assert!(error && !rejected && operation == "implied" && time > 0.0);
            assert!(reference == 0.0 && row.real("factor").unwrap() != 1.0);
        }
        "D5-BASE-ERASURE" => {
            assert!(error && !rejected && matches!(operation, "compound" | "discount"));
            assert!(periodic(source, time) && rate != 0.0);
            assert!(row.real("rounded_base").unwrap() == 1.0);
        }
        "D5-ROUNDED-BASE" => {
            assert!(!error && (differs || zero_loss) && periodic(source, time));
            assert!(matches!(operation, "compound" | "discount"));
        }
        "D5-ROUNDED-BASE-RANGE" => {
            assert!(error && !rejected && periodic(source, time));
            assert!(matches!(operation, "compound" | "discount"));
            assert_eq!(row.text("error_kind").unwrap(), "rate.value");
            assert!(reference.is_finite() && reference > 0.0);
            assert!(rate > -count(source).unwrap());
        }
        "D5-INTERMEDIATE-FACTOR" => {
            assert_eq!(operation, "equivalent");
            assert!(!factor.is_finite() || factor <= 0.0);
            assert!(error || rejected || differs || zero_loss);
        }
        "D5-FACTOR-ERASURE" => {
            assert_eq!(operation, "equivalent");
            assert!(factor == 1.0 && rate != 0.0 && time > 0.0);
            assert!(reference == 0.0);
            assert!(error || differs || zero_loss);
        }
        "D5-POW-SUBTRACTION" => {
            assert!(matches!(operation, "implied" | "equivalent"));
            assert!(!error && !rejected && periodic(target, time));
            assert!(differs || zero_loss);
            if zero_loss {
                assert!(row.real("inverse_power").unwrap() == 1.0);
            }
        }
        "D5-ROUNDED-BASE-ROUNDTRIP" => {
            assert_eq!(operation, "equivalent");
            assert!(!error && !rejected && periodic(source, time));
            assert!(!periodic(target, time) && (differs || zero_loss));
        }
        "D5-FACTOR-ROUNDTRIP" => {
            assert_eq!(operation, "equivalent");
            assert!(!error && !rejected && !periodic(source, time));
            assert!(!periodic(target, time) && (differs || zero_loss));
        }
        _ => panic!("{}: unknown discrepancy {code}", row.id()),
    }
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "reports reproducible discrepancy counts with --nocapture"
)]
fn actual_quantlib_results_have_independent_oracles_and_precise_discrepancy_predicates() {
    let mut values = [0; 4];
    let mut errors = [0; 4];
    let mut guard_mismatches = [0; 4];
    for (index, (operation, count)) in [
        ("compound", 8848),
        ("discount", 8848),
        ("implied", 3810),
        ("equivalent", 55940),
    ]
    .into_iter()
    .enumerate()
    {
        let data = Fixture::load(Fixture::repository_path(&format!(
            "validation/fixtures/rates/{operation}-quantlib.fixture"
        )))
        .unwrap();
        assert_eq!(data.record_count(), count);
        assert_eq!(data.header("reference"), Some("QuantLib"));
        assert_eq!(data.header("reference-version"), Some("1.43"));
        assert_eq!(
            data.header("quantlib-wheel-sha256"),
            Some("85d70b28778afca84b9b6ae74ef05a1069b2dca00e80fc74aa8ddf438259dc0c")
        );
        let oracle = Fixture::load(Fixture::repository_path(&format!(
            "validation/fixtures/rates/{operation}-oracle.fixture"
        )))
        .unwrap();
        let oracle_hash = oracle
            .header("reproduction")
            .unwrap()
            .strip_prefix("generator-sha256=")
            .unwrap()
            .split(';')
            .next()
            .unwrap();
        assert_eq!(data.header("oracle-generator-sha256"), Some(oracle_hash));
        let mut classifications = BTreeMap::new();
        let mut guard_truth_counts = BTreeMap::new();
        let mut guard_variants = BTreeMap::new();
        let mut configurations = BTreeSet::new();
        let mut pairs = BTreeSet::new();
        let mut horizons = BTreeSet::new();
        for row in data.records() {
            assert_eq!(row.text("operation").unwrap(), operation);
            let source = compounding(&row, "method", "frequency");
            let target = if operation == "equivalent" {
                compounding(&row, "target_method", "target_frequency")
            } else {
                source
            };
            let source_name = (
                row.text("method").unwrap().to_owned(),
                row.text("frequency").unwrap().to_owned(),
            );
            configurations.insert(source_name.clone());
            pairs.insert((
                source_name.clone(),
                (
                    row.text("target_method").unwrap().to_owned(),
                    row.text("target_frequency").unwrap().to_owned(),
                ),
            ));
            horizons.insert((source_name, row.real("time").unwrap().to_bits()));
            let actual = execute(&row);
            let ideal_error = row.text("behavior").unwrap() == "error";
            if ideal_error {
                errors[index] += 1;
            } else {
                values[index] += 1;
            }
            if let Err(error) = &actual {
                check_numerical_boundary(error);
            }
            if let Some(variant) = check_guard_edge(&row, &actual) {
                let ideal = if ideal_error {
                    row.text("error_kind").unwrap()
                } else {
                    "value"
                };
                if variant != ideal {
                    guard_mismatches[index] += 1;
                }
                *guard_truth_counts
                    .entry(row.text("mathematical_behavior").unwrap().to_owned())
                    .or_insert(0_usize) += 1;
                *guard_variants
                    .entry((
                        row.text("mathematical_behavior").unwrap().to_owned(),
                        ideal.to_owned(),
                        variant,
                    ))
                    .or_insert(0_usize) += 1;
            } else if ideal_error {
                assert_eq!(
                    actual.expect_err(row.id()).kind(),
                    row.text("error_kind").unwrap(),
                    "{}: r={},t={},C={}",
                    row.id(),
                    row.text("rate").unwrap(),
                    row.text("time").unwrap(),
                    row.text("factor").unwrap()
                );
            } else {
                let expected = row.real("expected").unwrap();
                let actual = actual.unwrap_or_else(|error| panic!("{}: {error}", row.id()));
                assert!(actual.is_finite(), "{}", row.id());
                if expected == 0.0 {
                    assert_eq!(actual.to_bits(), 0, "{}: canonical zero", row.id());
                } else {
                    tolerance(operation, expected, row.real("time").unwrap(), target)
                        .check(actual, expected)
                        .unwrap_or_else(|error| panic!("{}: {error}", row.id()));
                }
            }
            check_reference_pipeline(&row, source, target);
            check_classification(&row, source, target);
            *classifications
                .entry(row.text("discrepancy").unwrap().to_owned())
                .or_insert(0_usize) += 1;
        }
        assert_eq!(configurations.len(), 32);
        if operation == "equivalent" {
            assert_eq!(pairs.len(), 1024);
        }
        for frequency in Frequency::ALL {
            let Some(events) = frequency.events_per_year() else {
                continue;
            };
            let threshold = 1.0 / f64::from(events);
            for method in ["SimpleThenCompounded", "CompoundedThenSimple"] {
                for bits in [
                    threshold.to_bits() - 1,
                    threshold.to_bits(),
                    threshold.to_bits() + 1,
                ] {
                    assert!(horizons.contains(&((method.to_owned(), frequency.to_string()), bits)));
                }
            }
        }
        let mut recorded_counts = classifications
            .iter()
            .map(|(key, count)| {
                (
                    if key.is_empty() {
                        "agreement"
                    } else {
                        key.as_str()
                    },
                    count,
                )
            })
            .collect::<Vec<_>>();
        // Normalize the empty classification before sorting keys. Sorting
        // rendered "key=count" strings would incorrectly reorder prefixes.
        recorded_counts.sort_by_key(|(key, _)| *key);
        let recorded_counts = recorded_counts
            .into_iter()
            .map(|(key, count)| format!("{key}={count}"))
            .collect::<Vec<_>>();
        assert_eq!(
            data.header("classification-counts"),
            Some(recorded_counts.join("; ").as_str())
        );
        let guard_counts = guard_truth_counts
            .iter()
            .map(|(key, count)| format!("{key}={count}"))
            .collect::<Vec<_>>()
            .join("; ");
        assert_eq!(
            data.header("guard-edge-counts"),
            Some(if guard_counts.is_empty() {
                "none"
            } else {
                &guard_counts
            })
        );
        println!("{operation}: {classifications:?}");
        println!("{operation} guard variants (true,ideal,observed): {guard_variants:?}");
    }
    assert_eq!(values, [6215, 5942, 1982, 49704]);
    assert_eq!(errors, [2633, 2906, 1828, 6236]);
    assert_eq!(guard_mismatches, [19, 19, 0, 0]);
}
