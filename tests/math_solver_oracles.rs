//! Exact Fraction stopping decisions, including rounding at the real boundary.

mod support;

use sabiquant::math::{
    RootBracket, SolverError, SolverOptions, TerminationReason, bisect, brent, safeguarded_newton,
};
use support::fixture::Fixture;

#[test]
fn exact_rational_operational_tolerance_matches_all_three_solvers() {
    let data = Fixture::load(Fixture::repository_path(
        "validation/fixtures/math/solver-tolerance-oracle.fixture",
    ))
    .unwrap();
    assert_eq!(data.record_count(), 97_851);
    let mut statuses = [0_usize; 3];
    let mut branches = [0_usize; 2];
    let mut rounded_only = [0_usize; 2];
    for row in data.records() {
        let lower = row.real_bits("lower").unwrap();
        let upper = row.real_bits("upper").unwrap();
        let absolute = row.real_bits("absolute").unwrap();
        let relative = row.real_bits("relative").unwrap();
        let bracket = RootBracket::new(lower, upper).unwrap();
        let options = SolverOptions::new(absolute, relative, 2).unwrap();
        let full_width = upper - lower;
        let magnitude = lower.abs().max(upper.abs());
        let (width, tolerance) = if full_width.is_finite() {
            assert_eq!(row.text("branch").unwrap(), "full", "{}", row.id());
            branches[0] += 1;
            (full_width, absolute + relative * magnitude)
        } else {
            assert_eq!(row.text("branch").unwrap(), "overflow-half", "{}", row.id());
            branches[1] += 1;
            (
                upper / 2.0 - lower / 2.0,
                absolute / 2.0 + relative * (magnitude / 2.0),
            )
        };
        assert_eq!(
            width.to_bits(),
            row.real_bits("operational_width").unwrap().to_bits(),
            "{} rounded width",
            row.id()
        );
        assert_eq!(
            tolerance.to_bits(),
            row.real_bits("operational_tolerance").unwrap().to_bits(),
            "{} rounded allowance",
            row.id()
        );
        let accepted = row.text("status").unwrap() == "accepted";
        let exact_accept = row.text("exact_accept").unwrap() == "yes";
        if accepted && !exact_accept {
            rounded_only[0] += 1;
        }
        if !accepted && exact_accept {
            rounded_only[1] += 1;
        }
        assert_eq!(
            width <= tolerance,
            accepted,
            "{} exact rational rounding",
            row.id()
        );
        let index = match row.text("status").unwrap() {
            "accepted" => 0,
            "budget" => 1,
            "precision" => 2,
            other => panic!("unknown fixture status {other}"),
        };
        statuses[index] += 1;
        for method in 0..3 {
            let mut calls = 0;
            let mut function = |x: f64| {
                // Any third callback is a defect, regardless of its result.
                assert!(calls < 2, "{} shared cap", row.id());
                let expected = if calls == 0 {
                    bracket.lower()
                } else {
                    bracket.upper()
                };
                assert_eq!(
                    x.to_bits(),
                    expected.to_bits(),
                    "{} endpoint order",
                    row.id()
                );
                calls += 1;
                if calls == 1 { -1.0 } else { 1.0 }
            };
            let result = match method {
                0 => bisect(&mut function, bracket, options),
                1 => brent(&mut function, bracket, options),
                _ => safeguarded_newton(
                    &mut function,
                    |_| panic!("{} no derivative budget", row.id()),
                    bracket.lower(),
                    bracket,
                    options,
                ),
            };
            assert_eq!(calls, 2, "{} endpoint calls", row.id());
            match (index, result) {
                (0, Ok(report)) => {
                    assert_eq!(
                        report.reason(),
                        TerminationReason::BracketTolerance,
                        "{}",
                        row.id()
                    );
                    assert_eq!(report.bracket(), bracket, "{}", row.id());
                    assert_eq!(
                        report.point().to_bits(),
                        bracket.lower().to_bits(),
                        "{} lower tie",
                        row.id()
                    );
                    assert_eq!(report.residual().to_bits(), (-1.0_f64).to_bits());
                    assert_eq!(report.iterations(), 0);
                    assert_eq!(report.evaluations().function(), 2);
                    assert_eq!(report.evaluations().derivative(), 0);
                }
                (
                    1,
                    Err(SolverError::EvaluationLimit {
                        bracket: observed,
                        evaluations,
                        ..
                    }),
                )
                | (
                    2,
                    Err(SolverError::PrecisionExhausted {
                        bracket: observed,
                        evaluations,
                        ..
                    }),
                ) => {
                    assert_eq!(observed, bracket, "{}", row.id());
                    assert_eq!(evaluations.function(), 2);
                    assert_eq!(evaluations.derivative(), 0);
                }
                (_, other) => panic!("{} method {method}, status {index}: {other:?}", row.id()),
            }
        }
    }
    assert_eq!(statuses.iter().sum::<usize>(), 97_851);
    assert_eq!(statuses[0], 59_429);
    assert_eq!(branches, [97_108, 743]);
    assert_eq!(rounded_only, [6_734, 3]);
}
