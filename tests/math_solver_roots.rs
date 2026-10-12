//! Independent high-precision polynomial roots across extreme coordinate scales.

mod support;

use sabiquant::math::{RootBracket, SolverOptions, bisect, brent, safeguarded_newton};
use support::fixture::Fixture;

#[test]
#[expect(
    clippy::print_stdout,
    reason = "records accuracy and callback cost over independent polynomial roots under --nocapture"
)]
fn all_solvers_match_precision_checked_scaled_polynomial_roots() {
    let data = Fixture::load(Fixture::repository_path(
        "validation/fixtures/math/solver-roots-oracle.fixture",
    ))
    .unwrap();
    assert_eq!(data.record_count(), 21_084);
    let mut maximum_normalized_error = [0.0_f64; 3];
    let mut normalized_errors: [Vec<f64>; 3] = std::array::from_fn(|_| Vec::new());
    let mut maximum_calls = [0_u64; 3];
    for row in data.records() {
        let degree: u32 = row.parse("degree").unwrap();
        let coefficient = row.real("coefficient").unwrap();
        let scale = row.real("scale").unwrap();
        let shift = row.real("shift").unwrap();
        let direction = row.real("direction").unwrap();
        let bracket =
            RootBracket::new(row.real("lower").unwrap(), row.real("upper").unwrap()).unwrap();
        let absolute = row.real("absolute").unwrap();
        let options = SolverOptions::new(absolute, 0.0, 300).unwrap();
        let expected = row.real_bits("expected").unwrap();
        for method in 0..3 {
            let mut function_calls = 0_u32;
            let mut derivative_calls = 0_u32;
            let function = |x: f64| {
                function_calls += 1;
                assert!(
                    x.is_finite() && x >= bracket.lower() && x <= bracket.upper(),
                    "{} callback argument",
                    row.id()
                );
                let z = direction * ((x - shift) / scale);
                let square = z * z;
                (if degree == 2 { square } else { square * z }) - coefficient
            };
            let derivative = |x: f64| {
                derivative_calls += 1;
                assert!(
                    x.is_finite() && x >= bracket.lower() && x <= bracket.upper(),
                    "{} derivative argument",
                    row.id()
                );
                let z = direction * ((x - shift) / scale);
                (direction * f64::from(degree) * (if degree == 2 { z } else { z * z })) / scale
            };
            let result = match method {
                0 => bisect(function, bracket, options),
                1 => brent(function, bracket, options),
                _ => safeguarded_newton(
                    function,
                    derivative,
                    row.real("initial").unwrap(),
                    bracket,
                    options,
                ),
            };
            let report =
                result.unwrap_or_else(|error| panic!("{} method{method}: {error}", row.id()));
            assert_eq!(
                report.evaluations().function(),
                function_calls,
                "{}",
                row.id()
            );
            assert_eq!(
                report.evaluations().derivative(),
                derivative_calls,
                "{}",
                row.id()
            );
            assert!(report.evaluations().total() <= 300);
            assert!(
                report.point() >= report.bracket().lower()
                    && report.point() <= report.bracket().upper()
            );
            let conditioning =
                16.0 * (f64::EPSILON / 2.0) * bracket.lower().abs().max(bracket.upper().abs());
            let budget = absolute + conditioning;
            let error = (report.point() - expected).abs();
            maximum_normalized_error[method] = maximum_normalized_error[method].max(error / budget);
            normalized_errors[method].push(error / budget);
            maximum_calls[method] = maximum_calls[method].max(report.evaluations().total());
            assert!(
                error <= budget,
                "{} method{method}: {:?} vs {expected:?}, error {error:?} > {budget:?}",
                row.id(),
                report.point()
            );
        }
    }
    println!(
        "solver root oracle: {} roots ×3 methods; max normalized errors {maximum_normalized_error:?}, max shared calls {maximum_calls:?}",
        data.record_count()
    );
    for (method, normalized) in ["bisect", "brent", "safeguarded_newton"]
        .into_iter()
        .zip(&mut normalized_errors)
    {
        normalized.sort_by(f64::total_cmp);
        let n = normalized.len();
        assert_eq!(n, data.record_count());
        // Match the normal-oracle report's zero-based floor(n*p/100) rule.
        println!(
            "solver {method}: normalized absolute error ratios n={n}; sorted floor(n*p/100) p50={:?} p95={:?} p99={:?}",
            normalized[n / 2],
            normalized[n * 95 / 100],
            normalized[n * 99 / 100]
        );
    }
}
