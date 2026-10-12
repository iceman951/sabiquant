//! Two-precision independent Decimal evidence for M4 numerical calibration.

mod support;

use sabiquant::daycount::DayCount;
use sabiquant::rates::{Compounding, CompoundingMethod, InterestRate, RateError};
use support::fixture::{Fixture, Record};
use support::float::{Tolerance, ulp_distance};

#[expect(
    clippy::unwrap_used,
    reason = "fixture decoding fails immediately for malformed, committed test inputs"
)]
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

#[expect(
    clippy::unwrap_used,
    reason = "fixture helper decodes validated scalar inputs and operation names"
)]
fn execute(row: &Record<'_>) -> Result<f64, RateError> {
    let source = compounding(row, "method", "frequency");
    let time = row.real("time").unwrap();
    let factor = row.real("factor").unwrap();
    match row.text("operation").unwrap() {
        "implied" => {
            Ok(InterestRate::implied_rate(factor, time, DayCount::Actual365Fixed, source)?.rate())
        }
        "implied-discount" => Ok(InterestRate::implied_rate_from_discount(
            factor,
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

#[expect(
    clippy::expect_used,
    reason = "the helper receives public values whose hybrid frequency was validated at construction"
)]
fn simple_phase(compounding: Compounding, time: f64) -> bool {
    match compounding.method() {
        CompoundingMethod::Simple => true,
        CompoundingMethod::SimpleThenCompounded | CompoundingMethod::CompoundedThenSimple => {
            let count = f64::from(
                compounding
                    .frequency()
                    .expect("hybrid frequency")
                    .events_per_year()
                    .expect("periodic frequency"),
            );
            if compounding.method() == CompoundingMethod::SimpleThenCompounded {
                time <= 1.0 / count
            } else {
                time > 1.0 / count
            }
        }
        _ => false,
    }
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "records measured numerical distributions with --nocapture for the M4 report"
)]
fn all_rate_operations_match_two_precision_decimal_values_and_exact_domain_statuses() {
    let unit = f64::EPSILON / 2.0;
    let mut factors = Vec::new();
    let mut rates = Vec::new();
    let mut subnormal_factors = 0;
    let mut saturated_rate_bounds = 0;
    let mut maximum_ulp = [0_u64; 5];
    let mut values = [0; 5];
    let mut errors = [0; 5];
    for (index, (operation, count)) in [
        ("compound", 7404),
        ("discount", 7404),
        ("implied", 2816),
        ("implied-discount", 2816),
        ("equivalent", 42624),
    ]
    .into_iter()
    .enumerate()
    {
        let data = Fixture::load(Fixture::repository_path(&format!(
            "validation/fixtures/rates/{operation}-oracle.fixture"
        )))
        .unwrap();
        assert_eq!(data.record_count(), count);
        for row in data.records() {
            assert_eq!(row.text("operation").unwrap(), operation);
            let actual = execute(&row);
            if row.text("behavior").unwrap() == "error" {
                errors[index] += 1;
                let error = actual.expect_err(row.id());
                assert_eq!(
                    error.kind(),
                    row.text("error_kind").unwrap(),
                    "{}: {error}",
                    row.id()
                );
                continue;
            }
            values[index] += 1;
            let actual =
                actual.unwrap_or_else(|error| panic!("{}: unexpected error {error}", row.id()));
            let expected = row.real("expected").unwrap();
            assert!(actual.is_finite());
            maximum_ulp[index] = maximum_ulp[index].max(ulp_distance(actual, expected).unwrap());
            if index < 2 {
                assert_eq!(row.text("tolerance").unwrap(), "TOL-E3");
                if simple_phase(
                    compounding(&row, "method", "frequency"),
                    row.real("time").unwrap(),
                ) {
                    // Stronger E1 check for the fused Simple factor and its
                    // single reciprocal, independently rounded from Fraction.
                    assert_eq!(actual.to_bits(), expected.to_bits(), "{}: E1", row.id());
                }
                let scale = unit * (1.0 + expected.ln().abs());
                let absolute = if expected.is_subnormal() {
                    subnormal_factors += 1;
                    f64::from_bits(1)
                } else {
                    0.0
                };
                let tolerance = Tolerance::Mixed {
                    absolute,
                    relative: 8.0 * scale,
                };
                tolerance.check(actual, expected).unwrap_or_else(|error| {
                    panic!(
                        "{} {operation} {:?}/{:?},r={},t={}: {error}",
                        row.id(),
                        row.text("method"),
                        row.text("frequency"),
                        row.text("rate").unwrap(),
                        row.text("time").unwrap()
                    )
                });
                if absolute == 0.0 {
                    factors.push((actual - expected).abs() / expected / scale);
                }
            } else {
                assert_eq!(row.text("tolerance").unwrap(), "TOL-E4");
                if expected.to_bits() == 0 {
                    assert_eq!(actual.to_bits(), 0, "{}: canonical zero", row.id());
                    continue;
                }
                let time = row.real("time").unwrap();
                let target = if operation == "equivalent" {
                    compounding(&row, "target_method", "target_frequency")
                } else {
                    compounding(&row, "method", "frequency")
                };
                let periodic = match target.method() {
                    CompoundingMethod::Compounded => true,
                    CompoundingMethod::SimpleThenCompounded => {
                        time > 1.0
                            / f64::from(target.frequency().unwrap().events_per_year().unwrap())
                    }
                    CompoundingMethod::CompoundedThenSimple => {
                        time <= 1.0
                            / f64::from(target.frequency().unwrap().events_per_year().unwrap())
                    }
                    _ => false,
                };
                // E4's original 1/time term models rounded-factor conditioning.
                // Exponential inference additionally amplifies argument error:
                // dr/dx = m*exp(x) = m+r, x=ln(1+r/m).
                let extra = if periodic {
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
                } else if operation == "equivalent"
                    && target.method() != CompoundingMethod::Continuous
                {
                    // dr/dx=exp(x)/time for a simple rate obtained from growth.
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
                let scale = unit * expected.abs() + unit / time + extra;
                let bound = 8.0 * scale;
                let absolute = if bound.is_finite() {
                    bound
                } else {
                    saturated_rate_bounds += 1;
                    f64::MAX
                };
                Tolerance::Mixed {
                    absolute,
                    relative: 0.0,
                }
                .check(actual, expected)
                .unwrap_or_else(|error| {
                    panic!(
                        "{} {operation} {:?}/{:?},target={:?}/{:?},r={},t={},C={}: {error}",
                        row.id(),
                        row.text("method"),
                        row.text("frequency"),
                        row.text("target_method"),
                        row.text("target_frequency"),
                        row.text("rate").unwrap(),
                        row.text("time").unwrap(),
                        row.text("factor").unwrap()
                    )
                });
                if scale.is_finite() && scale > 0.0 {
                    rates.push((actual - expected).abs() / scale);
                }
            }
        }
    }
    assert_eq!(values, [5694, 5692, 1532, 1533, 38724]);
    assert_eq!(errors, [1710, 1712, 1284, 1283, 3900]);
    factors.sort_by(f64::total_cmp);
    rates.sort_by(f64::total_cmp);
    for (name, samples) in [("E3", factors), ("E4", rates)] {
        println!(
            "{name}: samples={}, p50={}, p95={}, p99={}, maximum_normalized={}",
            samples.len(),
            samples[samples.len() / 2],
            samples[samples.len() * 95 / 100],
            samples[samples.len() * 99 / 100],
            samples.last().unwrap()
        );
    }
    println!(
        "values={values:?}; errors={errors:?}; maximum_ulp={maximum_ulp:?}; subnormal_factors={subnormal_factors}; saturated_rate_bounds={saturated_rate_bounds}"
    );
}
