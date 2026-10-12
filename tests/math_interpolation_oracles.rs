//! Exact polynomial constraints and two-precision logarithmic interpolation.

mod support;

use sabiquant::math::{Extrapolation, InterpolationMethod, Interpolator};
use support::fixture::Fixture;
use support::float::ulp_distance;

#[test]
#[expect(
    clippy::print_stdout,
    reason = "reports independent oracle calibration distributions and maxima under --nocapture"
)]
fn all_small_grid_polynomials_match_independent_constraint_oracles() {
    let data = Fixture::load(Fixture::repository_path(
        "validation/fixtures/math/interpolation-oracle.fixture",
    ))
    .unwrap();
    assert_eq!(data.record_count(), 99_072);
    let mut maximum_ulp = [0; 4];
    let mut maximum_budget_ratio = [0.0_f64; 4];
    let mut normalized_errors: [Vec<f64>; 4] = std::array::from_fn(|_| Vec::new());
    let mut exact_knots = [0; 4];
    let mut counts = [0; 4];
    let mut last_definition = String::new();
    let mut last_method = InterpolationMethod::Linear;
    let mut interpolator = None;
    for row in data.records() {
        let method: InterpolationMethod = row.parse("method").unwrap();
        let definition = row.text("nodes").unwrap();
        if definition != last_definition || method != last_method {
            let nodes = definition
                .split(';')
                .map(|pair| {
                    let (x, y) = pair.split_once(',').unwrap();
                    (x.parse::<f64>().unwrap(), y.parse::<f64>().unwrap())
                })
                .collect();
            interpolator = Some(
                Interpolator::new(nodes, method, Extrapolation::ExtendBoundarySegment).unwrap(),
            );
            last_definition = definition.to_owned();
            last_method = method;
        }
        let index = InterpolationMethod::ALL
            .iter()
            .position(|&candidate| candidate == method)
            .unwrap();
        counts[index] += 1;
        let expected = row.real_bits("expected").unwrap();
        let actual = interpolator
            .as_ref()
            .unwrap()
            .value(row.real("query").unwrap())
            .unwrap();
        assert!(actual.is_finite(), "{}", row.id());
        maximum_ulp[index] = maximum_ulp[index].max(ulp_distance(actual, expected).unwrap());
        if row.text("exact_knot").unwrap() == "yes" {
            exact_knots[index] += 1;
            assert_eq!(
                actual.to_bits(),
                expected.to_bits(),
                "{} exact knot",
                row.id()
            );
        } else {
            let budget = row.real("absolute_budget").unwrap();
            assert!(
                budget.is_finite() && budget > 0.0,
                "{} informative budget",
                row.id()
            );
            let error = (actual - expected).abs();
            maximum_budget_ratio[index] = maximum_budget_ratio[index].max(error / budget);
            normalized_errors[index].push(error / budget);
            assert!(
                error <= budget,
                "{} {method}: {actual:?} vs {expected:?}; {error:?} > {budget:?}",
                row.id()
            );
        }
    }
    assert_eq!(counts, [24_768; 4]);
    assert_eq!(exact_knots, [6_552; 4]);
    println!(
        "interpolation oracle: counts {counts:?}, exact knots {exact_knots:?}, max normalized errors {maximum_budget_ratio:?}, raw ULP {maximum_ulp:?}"
    );
    for (method, normalized) in InterpolationMethod::ALL
        .into_iter()
        .zip(&mut normalized_errors)
    {
        normalized.sort_by(f64::total_cmp);
        let n = normalized.len();
        assert_eq!(n, 24_768 - 6_552);
        // Match the normal-oracle report's zero-based floor(n*p/100) rule.
        // Exact knots have a zero acceptance budget and remain separate E0 tests.
        println!(
            "interpolation {method}: non-knot normalized absolute error ratios n={n}; sorted floor(n*p/100) p50={:?} p95={:?} p99={:?}",
            normalized[n / 2],
            normalized[n * 95 / 100],
            normalized[n * 99 / 100]
        );
    }
}
