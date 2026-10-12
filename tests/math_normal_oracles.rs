//! Exact-input two-precision evidence for normal probabilities and quantiles.

mod support;

use sabiquant::error::{NonFinite, ValueError};
use sabiquant::math::{
    NormalDistribution, NormalError, standard_normal_cdf, standard_normal_inverse_cdf,
    standard_normal_log_cdf, standard_normal_log_survival, standard_normal_pdf,
    standard_normal_survival,
};
use support::fixture::{Fixture, Record};
use support::float::{Tolerance, ulp_distance};

const UNIT: f64 = f64::EPSILON / 2.0;
const MIN_SUB: f64 = f64::from_bits(1);

#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "strict decoding of committed oracle rows"
)]
fn execute(row: &Record<'_>) -> Result<f64, NormalError> {
    let argument = row.real_bits("argument_bits").unwrap();
    let mean = row.real_bits("mean_bits").unwrap();
    let scale = row.real_bits("scale_bits").unwrap();
    let operation = row.text("operation").unwrap();
    if mean == 0.0 && scale.to_bits() == 1.0_f64.to_bits() {
        match operation {
            "pdf" => standard_normal_pdf(argument),
            "cdf" => standard_normal_cdf(argument),
            "survival" => standard_normal_survival(argument),
            "log-cdf" => standard_normal_log_cdf(argument),
            "log-survival" => standard_normal_log_survival(argument),
            "inverse" => standard_normal_inverse_cdf(argument),
            _ => panic!("unknown normal operation"),
        }
    } else {
        let distribution = NormalDistribution::new(mean, scale)?;
        match operation {
            "pdf" => distribution.pdf(argument),
            "cdf" => distribution.cdf(argument),
            "survival" => distribution.survival(argument),
            "log-cdf" => distribution.log_cdf(argument),
            "log-survival" => distribution.log_survival(argument),
            "inverse" => distribution.inverse_cdf(argument),
            _ => panic!("unknown normal operation"),
        }
    }
}

#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "strict decoding of committed numerical inputs"
)]
fn allowance(row: &Record<'_>, expected: f64) -> f64 {
    if expected == 0.0 {
        return MIN_SUB;
    }
    let operation = row.text("operation").unwrap();
    match operation {
        "pdf" => {
            let condition = row.real("uncertainty_scale").unwrap();
            MIN_SUB.max(expected.abs() * (8.0 * UNIT * condition))
        }
        "cdf" | "survival" => {
            if expected >= 0.5 {
                8.0 * UNIT
            } else {
                let z = row.real("standardized").unwrap();
                MIN_SUB.max(expected * (8.0 * UNIT * (1.0 + z * z)))
            }
        }
        "log-cdf" | "log-survival" => {
            if expected.abs() >= 0.5 {
                // Form the small coefficient first, avoiding overflow of
                // 1+|log| for logarithms close to the binary64 range limit.
                8.0 * UNIT + expected.abs() * (8.0 * UNIT)
            } else {
                let z = row.real("standardized").unwrap();
                MIN_SUB.max(expected.abs() * (8.0 * UNIT * (1.0 + z * z)))
            }
        }
        "inverse" => {
            let z = row.real("standardized").unwrap().abs();
            let p = row.real_bits("argument_bits").unwrap();
            let scale = row.real_bits("scale_bits").unwrap();
            let standard_allowance = if (0.25..=0.75).contains(&p) {
                16.0 * UNIT * z
            } else {
                let log_tail = p.min(1.0 - p).ln().abs();
                // Mills R(z)<1/z yields a conservative bound on propagation
                // of the rounded input logarithm in the tail inverse.
                8.0 * UNIT * (1.0 + z + log_tail / z)
            };
            MIN_SUB.max(scale * standard_allowance + UNIT * expected.abs())
        }
        _ => panic!("unknown normal operation"),
    }
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "--nocapture reports measured conditional numerical distributions"
)]
fn six_normal_operations_match_precision_checked_independent_oracles() {
    let mut total = 0;
    for (operation, count) in [
        ("pdf", 3571),
        ("cdf", 1522),
        ("survival", 1522),
        ("log-cdf", 1522),
        ("log-survival", 1522),
        ("inverse", 10287),
    ] {
        let fixture = Fixture::load(Fixture::repository_path(&format!(
            "validation/fixtures/math/normal/{operation}-oracle.fixture"
        )))
        .unwrap();
        assert_eq!(fixture.header("domain"), Some("math.normal"));
        assert_eq!(fixture.record_count(), count);
        assert_eq!(
            fixture.header("fixture-set"),
            Some(format!("NORMAL-ORACLE-{}", operation.to_ascii_uppercase()).as_str())
        );
        assert_eq!(fixture.header("reference"), Some("oracle:Python Decimal"));
        assert!(
            fixture
                .header("reference-version")
                .unwrap()
                .ends_with("Decimal 120/200 digits")
        );
        assert!(
            fixture
                .header("reproduction")
                .unwrap()
                .contains("generator-sha256=")
        );
        assert!(
            fixture
                .header("oracle-method")
                .unwrap()
                .contains("inverse 180 bisections")
        );
        let mut values = 0;
        let mut errors = 0;
        let mut edge_values = 0;
        let mut edge_errors = 0;
        let mut edge_reference_differences = 0;
        let mut edge_true_signs = [[0; 2]; 2];
        let mut subnormals = 0;
        let mut zeros = 0;
        let mut maximum_ulp = 0;
        let mut normalized = Vec::new();
        for row in fixture.records() {
            total += 1;
            assert_eq!(row.text("operation").unwrap(), operation);
            assert_eq!(
                row.text("tolerance").unwrap(),
                if operation == "inverse" {
                    "TOL-E4"
                } else {
                    "TOL-E3"
                }
            );
            assert!(["ordinary", "guard-edge"].contains(&row.text("tier").unwrap()));
            let actual = execute(&row);
            if let Err(NormalError::NumericalBoundary {
                quantity,
                logarithm,
                ..
            }) = &actual
            {
                assert_eq!(
                    *quantity,
                    if operation == "inverse" {
                        "normal quantile"
                    } else {
                        "normal density"
                    }
                );
                assert!(logarithm.is_finite());
            }
            let edge = row.text("tier").unwrap() == "guard-edge";
            if edge {
                assert!(["pdf", "inverse"].contains(&operation));
                if operation == "inverse" {
                    let distance = row.real("affine_guard_distance").unwrap();
                    let allowance = row.real("affine_guard_allowance").unwrap();
                    assert!(distance.is_finite() && allowance.is_finite() && allowance > 0.0);
                    assert!((0.75..=1.25).contains(&(distance / allowance)));
                    let sign = usize::from(row.real("standardized").unwrap().is_sign_positive());
                    let overflow = usize::from(row.text("true_behavior").unwrap() == "error");
                    edge_true_signs[sign][overflow] += 1;
                }
                let declared = row.text("behavior").unwrap();
                let observed = if actual.is_ok() { "value" } else { "error" };
                if declared != observed
                    || actual
                        .as_ref()
                        .err()
                        .is_some_and(|error| error.kind() != row.text("error_kind").unwrap())
                {
                    edge_reference_differences += 1;
                }
                match (row.text("true_behavior").unwrap(), &actual) {
                    ("value", Ok(_)) => edge_values += 1,
                    ("value", Err(NormalError::NumericalBoundary { .. })) => edge_errors += 1,
                    ("error", Err(NormalError::NumericalBoundary { .. })) => {
                        edge_errors += 1;
                    }
                    ("error", Err(NormalError::Value(cause))) => {
                        let class = if operation == "pdf"
                            || row.real("standardized").unwrap().is_sign_positive()
                        {
                            NonFinite::PositiveInfinity
                        } else {
                            NonFinite::NegativeInfinity
                        };
                        let quantity = if operation == "pdf" {
                            "normal density"
                        } else {
                            "normal quantile"
                        };
                        assert!(
                            matches!(cause, ValueError::ResultNotFinite { quantity: actual_quantity, value, .. }
                            if *actual_quantity == quantity && *value == class),
                            "{} wrong overflow context: {cause}",
                            row.id()
                        );
                        edge_errors += 1;
                    }
                    _ => panic!(
                        "{} invalid guard-edge result {actual:?}",
                        row.text("id").unwrap()
                    ),
                }
                if actual.is_err() {
                    errors += 1;
                    continue;
                }
            } else if row.text("behavior").unwrap() == "error" {
                let error = actual.unwrap_err();
                assert_eq!(
                    error.kind(),
                    row.text("error_kind").unwrap(),
                    "{}",
                    row.text("id").unwrap()
                );
                errors += 1;
                continue;
            }
            let actual = actual.unwrap();
            assert!(actual.is_finite());
            match operation {
                "pdf" => assert!(actual >= 0.0),
                "cdf" | "survival" => assert!((0.0..=1.0).contains(&actual)),
                "log-cdf" | "log-survival" => assert!(actual <= 0.0),
                "inverse" => {}
                _ => panic!("unknown normal operation"),
            }
            let expected = row.real_bits("expected_bits").unwrap();
            let bound = allowance(&row, expected);
            assert!(bound.is_finite() && bound >= 0.0);
            if expected == 0.0 {
                zeros += 1;
            }
            let tolerance = Tolerance::Mixed {
                absolute: bound,
                relative: 0.0,
            };
            if let Err(report) = tolerance.check(actual, expected) {
                panic!("{} {report}; bound={bound:e}", row.text("id").unwrap());
            }
            if actual == 0.0 {
                assert_eq!(
                    actual.to_bits(),
                    0,
                    "{} observed zero must be canonical",
                    row.text("id").unwrap()
                );
            }
            if expected != 0.0 {
                normalized.push((actual - expected).abs() / bound);
                if expected.is_subnormal() {
                    subnormals += 1;
                }
            }
            maximum_ulp = maximum_ulp.max(ulp_distance(actual, expected).unwrap());
            values += 1;
        }
        normalized.sort_by(f64::total_cmp);
        let n = normalized.len();
        if operation == "inverse" {
            assert!(edge_true_signs.iter().flatten().all(|&count| count > 0));
        }
        println!(
            "normal {operation}: rows={} values={values} errors={errors} zeros={zeros} subnormals={subnormals} guard_edge_values={edge_values} guard_edge_errors={edge_errors} guard_edge_reference_differences={edge_reference_differences} edge_true_signs={edge_true_signs:?}; normalized p50={} p95={} p99={} max={} max_ulp={maximum_ulp}",
            fixture.record_count(),
            normalized[n / 2],
            normalized[n * 95 / 100],
            normalized[n * 99 / 100],
            normalized[n - 1]
        );
    }
    assert_eq!(total, 19_946);
}
