//! Independently specified roots, representability, callback and budget tests.

#![expect(
    clippy::disallowed_types,
    reason = "test-only shared callback counters and traces; no library interior mutability"
)]
#![expect(
    clippy::float_cmp,
    reason = "test callbacks select exact cached coordinates and endpoint scenarios"
)]

use sabiquant::math::{
    EvaluationStage, RootBracket, SolveReport, SolverError, SolverOptions, TerminationReason,
    bisect, brent, safeguarded_newton,
};
use std::cell::{Cell, RefCell};
use std::error::Error;

#[derive(Clone, Copy, Debug)]
enum Method {
    Bisection,
    Brent,
    Newton,
}
const METHODS: [Method; 3] = [Method::Bisection, Method::Brent, Method::Newton];

fn run(
    method: Method,
    function: &mut dyn FnMut(f64) -> f64,
    derivative: &mut dyn FnMut(f64) -> f64,
    initial: f64,
    bracket: RootBracket,
    options: SolverOptions,
) -> Result<SolveReport, SolverError> {
    match method {
        Method::Bisection => bisect(function, bracket, options),
        Method::Brent => brent(function, bracket, options),
        Method::Newton => safeguarded_newton(function, derivative, initial, bracket, options),
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "test helper fails on invalid scenario coordinates"
)]
fn bracket(lower: f64, upper: f64) -> RootBracket {
    RootBracket::new(lower, upper).unwrap()
}
#[expect(
    clippy::unwrap_used,
    reason = "test helper fails on invalid scenario options"
)]
fn options(absolute: f64, relative: f64, budget: u32) -> SolverOptions {
    SolverOptions::new(absolute, relative, budget).unwrap()
}

#[expect(
    clippy::panic,
    reason = "test helper rejects an unrecognized successful termination reason"
)]
fn check_report(report: SolveReport, original: RootBracket) {
    assert!(report.point().is_finite());
    assert!(report.residual().is_finite());
    let final_bracket = report.bracket();
    assert!(original.lower() <= final_bracket.lower());
    assert!(original.upper() >= final_bracket.upper());
    assert!(final_bracket.lower() < final_bracket.upper());
    assert!(report.point() >= final_bracket.lower() && report.point() <= final_bracket.upper());
    match report.reason() {
        TerminationReason::FunctionZero => assert_eq!(report.residual().to_bits(), 0),
        TerminationReason::BracketTolerance => assert!(
            report.point().to_bits() == final_bracket.lower().to_bits()
                || report.point().to_bits() == final_bracket.upper().to_bits()
        ),
        other => panic!("unexpected reason {other:?}"),
    }
}

#[test]
fn checked_values_reject_all_bad_inputs_and_canonicalize_zero() {
    for nonfinite in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            RootBracket::new(nonfinite, 1.0).unwrap_err().kind(),
            "solver.value"
        );
        assert_eq!(
            RootBracket::new(0.0, nonfinite).unwrap_err().kind(),
            "solver.value"
        );
        assert_eq!(
            SolverOptions::new(nonfinite, 0.0, 1).unwrap_err().kind(),
            "solver.value"
        );
        assert_eq!(
            SolverOptions::new(1.0, nonfinite, 1).unwrap_err().kind(),
            "solver.value"
        );
    }
    for (a, b) in [(0.0, -0.0), (1.0, 1.0), (1.0, -1.0), (f64::MAX, f64::MIN)] {
        assert_eq!(
            RootBracket::new(a, b).unwrap_err().kind(),
            "solver.invalid_bracket"
        );
    }
    for absolute in [0.0, -0.0, -1.0] {
        assert!(SolverOptions::new(absolute, 0.0, 1).is_err());
    }
    for relative in [-f64::from_bits(1), -1.0, 1.0, 2.0] {
        assert!(SolverOptions::new(1.0, relative, 1).is_err());
    }
    assert_eq!(
        SolverOptions::new(1.0, 0.0, 0).unwrap_err().kind(),
        "solver.invalid_budget"
    );
    assert_eq!(bracket(-0.0, 1.0).lower().to_bits(), 0);
    assert_eq!(bracket(-1.0, -0.0).upper().to_bits(), 0);
    let configured = options(f64::from_bits(1), -0.0, u32::MAX);
    assert_eq!(configured.absolute().to_bits(), 1);
    assert_eq!(configured.relative().to_bits(), 0);
    assert_eq!(configured.max_evaluations(), u32::MAX);
}

#[test]
fn invalid_newton_initial_is_checked_before_any_callback() {
    for initial in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1, 1.1] {
        let calls = Cell::new(0);
        let mut function = |_| {
            calls.set(calls.get() + 1);
            0.0
        };
        let mut derivative = |_| {
            calls.set(calls.get() + 1);
            1.0
        };
        let error = safeguarded_newton(
            &mut function,
            &mut derivative,
            initial,
            bracket(0.0, 1.0),
            options(1e-12, 0.0, 100),
        )
        .unwrap_err();
        assert_eq!(error.kind(), "solver.value");
        assert_eq!(calls.get(), 0);
    }
}

#[test]
fn endpoint_protocol_and_exact_zero_short_circuit_are_explicit() {
    for method in METHODS {
        for (target, budget, expected_points) in
            [(0.0, 1, vec![0.0_f64]), (1.0, 2, vec![0.0_f64, 1.0])]
        {
            let points = RefCell::new(Vec::new());
            let mut function = |x| {
                points.borrow_mut().push(x);
                if x == target { -0.0 } else { x - target }
            };
            let mut derivative = |_| panic!("endpoint zero must not request derivative");
            let report = run(
                method,
                &mut function,
                &mut derivative,
                0.5,
                bracket(0.0, 1.0),
                options(1e-12, 0.0, budget),
            )
            .unwrap();
            assert_eq!(report.point().to_bits(), target.to_bits());
            assert_eq!(report.residual().to_bits(), 0);
            assert_eq!(*points.borrow(), expected_points);
            assert_eq!(report.evaluations().total(), u64::from(budget));
            assert_eq!(report.evaluations().derivative(), 0);
            assert_eq!(report.iterations(), 0);
            assert_eq!(report.bracket(), bracket(0.0, 1.0));
        }
    }
}

#[test]
fn one_call_cap_stops_before_upper_and_preserves_known_state() {
    for method in METHODS {
        let points = RefCell::new(Vec::new());
        let mut function = |x| {
            points.borrow_mut().push(x);
            x - 0.25
        };
        let mut derivative = |_| panic!("no derivative budget");
        let error = run(
            method,
            &mut function,
            &mut derivative,
            0.5,
            bracket(0.0, 1.0),
            options(1e-12, 0.0, 1),
        )
        .unwrap_err();
        match error {
            SolverError::EvaluationLimit {
                evaluations,
                best,
                bracket: current,
                ..
            } => {
                assert_eq!(evaluations.function(), 1);
                assert_eq!(evaluations.derivative(), 0);
                assert_eq!(best, Some((0.0, -0.25)));
                assert_eq!(current, bracket(0.0, 1.0));
            }
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(*points.borrow(), [0.0]);
    }
}

#[test]
fn callback_counts_never_exceed_any_shared_budget() {
    for method in METHODS {
        for budget in 1..=80 {
            let function_count = Cell::new(0_u32);
            let derivative_count = Cell::new(0_u32);
            let total = Cell::new(0_u32);
            let mut function = |x: f64| {
                assert!(total.get() < budget);
                function_count.set(function_count.get() + 1);
                total.set(total.get() + 1);
                assert!(x.is_finite() && (0.0..=2.0).contains(&x));
                x * x - 2.0
            };
            let mut derivative = |x: f64| {
                assert!(total.get() < budget);
                derivative_count.set(derivative_count.get() + 1);
                total.set(total.get() + 1);
                assert!(x.is_finite() && (0.0..=2.0).contains(&x));
                2.0 * x
            };
            let result = run(
                method,
                &mut function,
                &mut derivative,
                1.0,
                bracket(0.0, 2.0),
                options(1e-12, 0.0, budget),
            );
            let counts = match result {
                Ok(report) => {
                    check_report(report, bracket(0.0, 2.0));
                    report.evaluations()
                }
                Err(SolverError::EvaluationLimit { evaluations, .. }) => {
                    assert_eq!(evaluations.total(), u64::from(budget));
                    evaluations
                }
                other => panic!("{method:?}, budget {budget}: {other:?}"),
            };
            assert_eq!(counts.function(), function_count.get());
            assert_eq!(counts.derivative(), derivative_count.get());
            assert_eq!(counts.total(), u64::from(total.get()));
            assert!(counts.total() <= u64::from(budget));
        }
    }
}

#[test]
fn newton_shared_budget_distinguishes_initial_derivative_and_trial_cost() {
    for (budget, function_calls, derivative_calls, current_lower) in
        [(2, 2, 0, 0.0_f64), (3, 3, 0, 1.0), (4, 3, 1, 1.0)]
    {
        let error = safeguarded_newton(
            |x| x * x - 2.0,
            |x| 2.0 * x,
            1.0,
            bracket(0.0, 2.0),
            options(1e-12, 0.0, budget),
        )
        .unwrap_err();
        match error {
            SolverError::EvaluationLimit {
                evaluations,
                bracket: current,
                best,
                ..
            } => {
                assert_eq!(evaluations.function(), function_calls);
                assert_eq!(evaluations.derivative(), derivative_calls);
                assert_eq!(evaluations.total(), u64::from(budget));
                assert_eq!(current.lower().to_bits(), current_lower.to_bits());
                assert_eq!(best.unwrap().0.to_bits(), current_lower.to_bits());
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}

#[test]
fn even_multiple_root_needs_a_sign_change_and_odd_multiple_roots_are_allowed() {
    for method in METHODS {
        let error = run(
            method,
            &mut |x| x * x,
            &mut |x| 2.0 * x,
            0.0,
            bracket(-1.0, 1.0),
            options(1e-12, 0.0, 100),
        )
        .unwrap_err();
        assert_eq!(error.kind(), "solver.not_bracketed");
        let report = run(
            method,
            &mut |x| (x + 1.0) * x * (x - 1.0),
            &mut |x| 3.0 * x * x - 1.0,
            0.0,
            bracket(-2.0, 2.0),
            options(1e-12, 0.0, 100),
        )
        .unwrap();
        assert_eq!(report.point().to_bits(), 0);
        assert_eq!(report.reason(), TerminationReason::FunctionZero);
    }
}

#[test]
fn sign_reversed_functions_preserve_the_same_known_roots() {
    for method in METHODS {
        for scale in [-1e200_f64, -1.0, -1e-200, 1e-200, 1.0, 1e200] {
            let report = run(
                method,
                &mut |x| scale * (x * x - 2.0),
                &mut |x| scale * 2.0 * x,
                1.0,
                bracket(0.0, 2.0),
                options(1e-12, 0.0, 500),
            )
            .unwrap();
            assert!((report.point() - std::f64::consts::SQRT_2).abs() <= 1e-12);
            check_report(report, bracket(0.0, 2.0));
        }
    }
}

#[test]
fn same_sign_values_are_rejected_without_multiplication() {
    for method in METHODS {
        for scale in [f64::from_bits(1), 1e-300, 1.0, 1e300, f64::MAX] {
            for sign in [-1.0, 1.0] {
                let calls = Cell::new(0);
                let mut function = |_| {
                    calls.set(calls.get() + 1);
                    sign * scale
                };
                let mut derivative = |_| panic!("not bracketed");
                let error = run(
                    method,
                    &mut function,
                    &mut derivative,
                    0.5,
                    bracket(0.0, 1.0),
                    options(1e-12, 0.0, 20),
                )
                .unwrap_err();
                assert_eq!(error.kind(), "solver.not_bracketed");
                assert_eq!(calls.get(), 2);
            }
        }
    }
}

#[test]
fn tiny_nonzero_residual_is_not_an_endpoint_root() {
    for method in METHODS {
        let report = run(
            method,
            &mut |x| 1e-300 * (x - 1.0),
            &mut |_| 1e-300,
            0.5,
            bracket(0.0, 2.0),
            options(1e-12, 0.0, 100),
        )
        .unwrap();
        assert!((report.point() - 1.0).abs() <= 1e-12);
        assert!(report.evaluations().function() >= 3);
        check_report(report, bracket(0.0, 2.0));
    }
}

#[test]
fn all_nonfinite_endpoint_trial_initial_and_derivative_stages_are_contextual() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for method in METHODS {
            for stage in [
                EvaluationStage::LowerEndpoint,
                EvaluationStage::UpperEndpoint,
                EvaluationStage::TrialPoint,
            ] {
                let count = Cell::new(0);
                let mut function = |x: f64| {
                    count.set(count.get() + 1);
                    if matches!(stage, EvaluationStage::LowerEndpoint) && x == 0.0
                        || matches!(stage, EvaluationStage::UpperEndpoint) && x == 2.0
                        || matches!(stage, EvaluationStage::TrialPoint) && x > 0.0 && x < 2.0
                    {
                        bad
                    } else {
                        x * x - 2.0
                    }
                };
                // Endpoint initial avoids classifying the first interior function
                // as InitialPoint in Newton.
                let error = run(
                    method,
                    &mut function,
                    &mut |x| 2.0 * x,
                    0.0,
                    bracket(0.0, 2.0),
                    options(1e-12, 0.0, 100),
                )
                .unwrap_err();
                match error {
                    SolverError::NonFiniteEvaluation {
                        stage: observed,
                        point,
                        evaluations,
                        ref cause,
                        ..
                    } => {
                        assert_eq!(observed, stage);
                        assert!(point.is_finite());
                        assert_eq!(evaluations.function(), count.get());
                        assert!(cause.to_string().contains("solver callback"));
                    }
                    other => panic!("unexpected {other:?}"),
                }
            }
        }
        let error = safeguarded_newton(
            |x| if x > 0.0 && x < 2.0 { bad } else { x * x - 2.0 },
            |_| 1.0,
            1.0,
            bracket(0.0, 2.0),
            options(1e-12, 0.0, 100),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            SolverError::NonFiniteEvaluation {
                stage: EvaluationStage::InitialPoint,
                ..
            }
        ));
        let error = safeguarded_newton(
            |x| x * x - 2.0,
            |_| bad,
            1.0,
            bracket(0.0, 2.0),
            options(1e-12, 0.0, 100),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            SolverError::NonFiniteEvaluation {
                stage: EvaluationStage::Derivative,
                ..
            }
        ));
        assert_eq!(
            error.source().unwrap().to_string(),
            match &error {
                SolverError::NonFiniteEvaluation { cause, .. } => cause.to_string(),
                _ => panic!("wrong error"),
            }
        );
    }
}

#[test]
fn independent_dyadic_roots_cover_translation_scale_and_negative_intervals() {
    let mut case_count = 0;
    for method in METHODS {
        for exponent in [-1000, -500, -20, 0, 20, 500, 1000] {
            let scale = 2.0_f64.powi(exponent);
            for numerator in -63..=63 {
                let root = f64::from(numerator) * scale / 64.0;
                let input = bracket(-2.0 * scale, 2.0 * scale);
                let opts = options(scale * 1e-13, 0.0, 500);
                let mut function = |x: f64| x - root;
                let report = run(method, &mut function, &mut |_| 1.0, 0.0, input, opts).unwrap();
                assert_eq!(
                    report.point().to_bits(),
                    if root == 0.0 { 0 } else { root.to_bits() }
                );
                assert_eq!(report.residual().to_bits(), 0);
                check_report(report, input);
                case_count += 1;
            }
        }
    }
    assert_eq!(case_count, 2667);
}

#[test]
fn irrational_polynomial_roots_are_inside_small_final_brackets() {
    for method in METHODS {
        for (power, target, expected, initial) in [
            (2, 2.0_f64, std::f64::consts::SQRT_2, 1.0),
            (3, 2.0, 1.259_921_049_894_873_2, 0.1),
            (5, 7.0, 1.475_773_161_594_552, 1.9),
            (9, 3.0, 1.129_830_963_909_753, 0.0),
        ] {
            let report = run(
                method,
                &mut |x: f64| x.powi(power) - target,
                &mut |x: f64| f64::from(power) * x.powi(power - 1),
                initial,
                bracket(0.0, 2.0),
                options(1e-12, 0.0, 1000),
            )
            .unwrap();
            assert!(
                (report.point() - expected).abs() <= 1.01e-12,
                "{method:?}, x^{power}={target}: {report:?}"
            );
            assert!((report.point().powi(power) - target - report.residual()).abs() <= 1e-14);
            check_report(report, bracket(0.0, 2.0));
        }
    }
}

#[test]
fn zero_wrong_extreme_and_repeated_newton_trials_use_safe_fallback() {
    for slope in [0.0, -0.0, -1.0, f64::from_bits(1), f64::MAX] {
        let points = RefCell::new(Vec::new());
        let report = safeguarded_newton(
            |x: f64| {
                assert!(x.is_finite() && (0.0..=2.0).contains(&x));
                points.borrow_mut().push(x);
                x * x - 2.0
            },
            |_| slope,
            0.0,
            bracket(0.0, 2.0),
            options(1e-12, 0.0, 1000),
        )
        .unwrap();
        assert!((report.point() - std::f64::consts::SQRT_2).abs() <= 1.01e-12);
        assert!(report.evaluations().derivative() > 0);
        assert_eq!(report.evaluations().function(), report.iterations() + 2);
        check_report(report, bracket(0.0, 2.0));
    }
}

#[test]
fn cached_initial_endpoints_are_not_evaluated_twice() {
    for initial in [0.0, 2.0] {
        let points = RefCell::new(Vec::new());
        let report = safeguarded_newton(
            |x: f64| {
                points.borrow_mut().push(x);
                x * x - 2.0
            },
            |x| 2.0 * x,
            initial,
            bracket(0.0, 2.0),
            options(1e-12, 0.0, 1000),
        )
        .unwrap();
        assert_eq!(points.borrow().iter().filter(|&&x| x == initial).count(), 1);
        assert_eq!(report.evaluations().function(), report.iterations() + 2);
    }
}

#[test]
fn interior_initial_zero_stops_without_derivative() {
    let report = safeguarded_newton(
        |x| x - 0.25,
        |_| panic!("initial zero"),
        0.25,
        bracket(0.0, 1.0),
        options(1e-12, 0.0, 3),
    )
    .unwrap();
    assert_eq!(report.point().to_bits(), 0.25_f64.to_bits());
    assert_eq!(report.evaluations().function(), 3);
    assert_eq!(report.evaluations().derivative(), 0);
    assert_eq!(report.iterations(), 1);
}

#[test]
fn sign_jump_only_reports_positional_convergence() {
    for method in METHODS {
        let report = run(
            method,
            &mut |x| if x < 0.0 { -1.0 } else { 1.0 },
            &mut |_| 0.0,
            0.5,
            bracket(-1.0, 1.0),
            options(1e-12, 0.0, 200),
        )
        .unwrap();
        assert_eq!(report.reason(), TerminationReason::BracketTolerance);
        assert_eq!(report.residual().abs().to_bits(), 1.0_f64.to_bits());
        assert!(report.bracket().lower() < 0.0);
        assert!(report.bracket().upper() >= 0.0);
        assert!(report.bracket().upper() - report.bracket().lower() <= 1e-12);
    }
}

#[test]
fn full_finite_range_uses_only_finite_arguments_and_finds_zero() {
    for method in METHODS {
        let input = bracket(f64::MIN, f64::MAX);
        let report = run(
            method,
            &mut |x: f64| {
                assert!(x.is_finite());
                x
            },
            &mut |_| 1.0,
            0.0,
            input,
            options(f64::from_bits(1), 0.0, 100),
        )
        .unwrap();
        assert_eq!(report.point().to_bits(), 0);
        assert_eq!(report.residual().to_bits(), 0);
        assert_eq!(report.evaluations().function(), 3);
        assert_eq!(report.evaluations().derivative(), 0);
        check_report(report, input);
    }
}

#[test]
fn overflow_half_comparison_does_not_accept_infinity_against_infinity() {
    for method in METHODS {
        let input = bracket(-f64::MAX, f64::MAX);
        let mut calls = 0;
        let report = run(
            method,
            &mut |x| {
                calls += 1;
                if x < 0.0 { -1.0 } else { 1.0 }
            },
            &mut |_| 0.0,
            0.0,
            input,
            options(f64::MAX, 1.0_f64.next_down(), 100),
        )
        .unwrap();
        assert!(
            calls >= 3,
            "the original exact width exceeds configured allowance"
        );
        assert_eq!(report.reason(), TerminationReason::BracketTolerance);
        assert!(
            report.bracket().lower() > input.lower() || report.bracket().upper() < input.upper()
        );
    }
}

#[test]
fn finite_width_with_infinite_allowance_succeeds_without_trials() {
    for method in METHODS {
        let input = bracket(f64::MAX / 2.0, f64::MAX);
        let report = run(
            method,
            &mut |x| if x < f64::MAX { -1.0 } else { 1.0 },
            &mut |_| panic!("tolerance already met"),
            input.lower(),
            input,
            options(f64::MAX, 1.0_f64.next_down(), 2),
        )
        .unwrap();
        assert_eq!(report.evaluations().function(), 2);
        assert_eq!(report.iterations(), 0);
        assert_eq!(report.reason(), TerminationReason::BracketTolerance);
        assert_eq!(report.point().to_bits(), input.lower().to_bits());
    }
}

#[test]
fn finite_subnormal_width_is_never_halved_for_stopping() {
    let min = f64::from_bits(1);
    for method in METHODS {
        let input = bracket(0.0, 5.0 * min);
        let report = run(
            method,
            &mut |x| if x < 3.0 * min { -1.0 } else { 1.0 },
            &mut |_| 0.0,
            0.0,
            input,
            options(4.0 * min, 0.0, 100),
        )
        .unwrap();
        assert!(report.evaluations().function() >= 3);
        assert!(report.bracket().upper() - report.bracket().lower() <= 4.0 * min);
        assert_ne!(report.bracket(), input);
    }
}

#[test]
fn adjacent_floats_report_precision_exhaustion_without_hidden_epsilon_floor() {
    for method in METHODS {
        for lower in [-1.0_f64, 1.0, 1e100, f64::MAX.next_down()] {
            let upper = lower.next_up();
            let input = bracket(lower, upper);
            let points = RefCell::new(Vec::new());
            let mut function = |x| {
                points.borrow_mut().push(x);
                if x == lower { -1.0 } else { 1.0 }
            };
            let mut derivative = |_| panic!("precision exhausted before derivative");
            let error = run(
                method,
                &mut function,
                &mut derivative,
                lower,
                input,
                options(f64::from_bits(1), 0.0, 100),
            )
            .unwrap_err();
            assert_eq!(error.kind(), "solver.precision_exhausted");
            assert_eq!(*points.borrow(), [lower, upper]);
            match error {
                SolverError::PrecisionExhausted {
                    bracket,
                    evaluations,
                    best,
                    ..
                } => {
                    assert_eq!(bracket, input);
                    assert_eq!(evaluations.function(), 2);
                    assert_eq!(evaluations.derivative(), 0);
                    assert_eq!(best, (lower, -1.0));
                }
                _ => panic!("wrong error"),
            }
        }
    }
}

#[test]
fn continuous_linear_root_between_adjacent_floats_is_unrepresentable() {
    for method in METHODS {
        for lower in [-1.0_f64, 1.0, 1e100, f64::MAX.next_down()] {
            let upper = lower.next_up();
            let half_gap = (upper - lower) / 2.0;
            // The exact real zero is lower + half_gap, strictly between
            // adjacent floats. Subtraction at both endpoints is exact.
            let error = run(
                method,
                &mut |x| (x - lower) - half_gap,
                &mut |_| 1.0,
                lower,
                bracket(lower, upper),
                options(f64::from_bits(1), 0.0, 10),
            )
            .unwrap_err();
            assert_eq!(error.kind(), "solver.precision_exhausted");
            if let SolverError::PrecisionExhausted {
                evaluations, best, ..
            } = error
            {
                assert_eq!(evaluations.function(), 2);
                assert_eq!(best.0.to_bits(), lower.to_bits());
                assert_eq!(best.1.to_bits(), (-half_gap).to_bits());
            }
        }
    }
}

#[test]
fn full_finite_range_can_contract_to_the_subnormal_resolution() {
    for method in METHODS {
        for threshold in [
            -f64::MIN_POSITIVE,
            -f64::from_bits(1),
            f64::from_bits(1),
            f64::MIN_POSITIVE,
        ] {
            let input = bracket(f64::MIN, f64::MAX);
            let report = run(
                method,
                &mut |x: f64| {
                    assert!(x.is_finite() && x >= input.lower() && x <= input.upper());
                    if x < threshold { -1.0 } else { 1.0 }
                },
                &mut |_| 0.0,
                0.0,
                input,
                options(f64::from_bits(1), 0.0, 5000),
            )
            .unwrap();
            assert_eq!(report.reason(), TerminationReason::BracketTolerance);
            assert_eq!(
                (report.bracket().upper() - report.bracket().lower()).to_bits(),
                1
            );
            assert!(report.bracket().lower() < threshold && report.bracket().upper() >= threshold);
            assert!(report.evaluations().total() <= 5000);
            check_report(report, input);
        }
    }
}

#[test]
fn adjacent_subnormals_can_satisfy_an_exact_one_unit_tolerance() {
    let min = f64::from_bits(1);
    for method in METHODS {
        for lower in [-2.0 * min, -min, 0.0, min] {
            let upper = lower + min;
            let input = bracket(lower, upper);
            let report = run(
                method,
                &mut |x| if x == lower { -1.0 } else { 1.0 },
                &mut |_| panic!("already small"),
                lower,
                input,
                options(min, 0.0, 2),
            )
            .unwrap();
            assert_eq!(report.reason(), TerminationReason::BracketTolerance);
            assert_eq!(report.evaluations().function(), 2);
            assert_eq!(report.bracket(), input);
        }
    }
}

#[test]
fn bisection_contracts_the_actual_observed_sign_bracket_at_every_trial() {
    let lower = Cell::new(-2.0_f64);
    let upper = Cell::new(3.0_f64);
    let calls = Cell::new(0);
    let input = bracket(lower.get(), upper.get());
    let report = bisect(
        |x: f64| {
            calls.set(calls.get() + 1);
            let value = x * x * x - 2.0;
            if calls.get() <= 2 {
                return value;
            }
            assert_eq!(x.to_bits(), lower.get().midpoint(upper.get()).to_bits());
            let before = upper.get() - lower.get();
            if value < 0.0 {
                lower.set(x);
            } else {
                upper.set(x);
            }
            assert!(upper.get() - lower.get() <= before / 2.0 + f64::EPSILON * 3.0);
            value
        },
        input,
        options(1e-12, 0.0, 100),
    )
    .unwrap();
    assert_eq!(report.bracket().lower().to_bits(), lower.get().to_bits());
    assert_eq!(report.bracket().upper().to_bits(), upper.get().to_bits());
    assert_eq!(report.evaluations().function(), calls.get());
    assert_eq!(report.iterations() + 2, calls.get());
}

#[test]
fn brent_and_newton_are_fast_on_a_well_conditioned_polynomial() {
    let input = bracket(0.0, 2.0);
    let opts = options(1e-12, 0.0, 200);
    let baseline = bisect(|x| x * x - 2.0, input, opts).unwrap();
    for method in [Method::Brent, Method::Newton] {
        let report = run(
            method,
            &mut |x| x * x - 2.0,
            &mut |x| 2.0 * x,
            1.0,
            input,
            opts,
        )
        .unwrap();
        assert!(
            report.evaluations().function() < baseline.evaluations().function() / 2,
            "{method:?}: {report:?} baseline {baseline:?}"
        );
        assert!((report.point() - std::f64::consts::SQRT_2).abs() <= 1e-12);
    }
}

#[test]
fn brent_public_trace_contains_secant_iqi_and_two_step_history_rejection() {
    // Independent direct inverse-Lagrange ordinates, on a moderate-scale
    // polynomial: this does not use the implementation's scaled difference
    // algebra. Reconstruct the sign bracket from observed callback arguments.
    let mut lower = (0.0_f64, -2.0_f64);
    let mut upper = (2.0_f64, 2.0_f64);
    let mut previous = upper;
    let mut last_step = 2.0_f64;
    let mut historical_step = 2.0_f64;
    let mut calls = 0;
    let mut linear_trials = 0;
    let mut quadratic_trials = 0;
    let mut history_rejections = 0;
    let report = brent(
        |point: f64| {
            calls += 1;
            let value = point * point - 2.0;
            if calls <= 2 {
                return value;
            }
            let (best, opposite) = if lower.1.abs() <= upper.1.abs() {
                (lower, upper)
            } else {
                (upper, lower)
            };
            let midpoint = lower.0.midpoint(upper.0);
            let quadratic = previous.0 != opposite.0 && previous.1 != opposite.1;
            let candidate = if best.1.abs() < previous.1.abs() {
                if quadratic {
                    previous.0 * best.1 * opposite.1
                        / ((previous.1 - best.1) * (previous.1 - opposite.1))
                        + best.0 * previous.1 * opposite.1
                            / ((best.1 - previous.1) * (best.1 - opposite.1))
                        + opposite.0 * previous.1 * best.1
                            / ((opposite.1 - previous.1) * (opposite.1 - best.1))
                } else {
                    (previous.0 * best.1 - best.0 * previous.1) / (best.1 - previous.1)
                }
            } else {
                f64::NAN
            };
            let limit = best.0.midpoint(opposite.0).midpoint(opposite.0);
            let across = if opposite.0 > best.0 {
                candidate > best.0 && candidate < limit
            } else {
                candidate < best.0 && candidate > limit
            };
            if point.to_bits() == midpoint.to_bits() {
                if candidate.is_finite()
                    && candidate > lower.0
                    && candidate < upper.0
                    && across
                    && (candidate - best.0).abs() >= historical_step / 2.0
                {
                    history_rejections += 1;
                }
                last_step = (point - best.0).abs();
                historical_step = last_step;
            } else {
                assert!(best.1.abs() < previous.1.abs());
                assert!(
                    (point - candidate).abs() < 2e-15,
                    "observed {point}, independent interpolation {candidate}"
                );
                assert!((point - best.0).abs() < historical_step / 2.0);
                assert!(if opposite.0 > best.0 {
                    point > best.0 && point < limit
                } else {
                    point < best.0 && point > limit
                });
                historical_step = last_step;
                last_step = (point - best.0).abs();
                if quadratic {
                    quadratic_trials += 1;
                } else {
                    linear_trials += 1;
                }
            }
            previous = best;
            if value < 0.0 {
                lower = (point, value);
            } else {
                upper = (point, value);
            }
            value
        },
        bracket(0.0, 2.0),
        options(1e-12, 0.0, 200),
    )
    .unwrap();
    assert!(linear_trials > 0, "inverse-linear trials exercised");
    assert!(quadratic_trials > 0, "inverse-quadratic trials exercised");
    assert!(
        history_rejections > 0,
        "a valid interior IQI was rejected specifically by two-step history"
    );
    assert_eq!(report.evaluations().function(), calls);
    assert!((report.point() - std::f64::consts::SQRT_2).abs() <= 1e-12);
}

#[test]
fn brent_degenerate_ordinates_flat_roots_and_steep_scales_preserve_bracket() {
    for power in [1, 3, 7, 15] {
        for scale in [1e-250_f64, 1.0, 1e250] {
            let report = brent(
                |x: f64| scale * (x - 0.375).powi(power),
                bracket(-0.5, 1.0),
                options(1e-12, 0.0, 1000),
            )
            .unwrap();
            let distance = (report.point() - 0.375).abs();
            if distance > 1e-12 {
                // An extremely flat scaled callback can round to zero before
                // its real mathematical zero. FunctionZero states only the
                // observed callback result; do not assert a fictitious root
                // accuracy guarantee for this underflowing callback.
                assert_eq!(report.reason(), TerminationReason::FunctionZero);
                assert_eq!((scale * distance.powi(power)).to_bits(), 0);
            }
            assert!(report.bracket().lower() <= 0.375 && report.bracket().upper() >= 0.375);
            check_report(report, bracket(-0.5, 1.0));
        }
    }
    let report = brent(
        |x| if x < 0.375 { -f64::MAX } else { f64::MAX },
        bracket(-f64::MAX, f64::MAX),
        options(1e-12, 0.0, 2000),
    )
    .unwrap();
    assert_eq!(report.reason(), TerminationReason::BracketTolerance);
    assert!(report.bracket().lower() < 0.375 && report.bracket().upper() >= 0.375);
}

#[test]
fn repeated_runs_have_identical_outputs_and_callback_sequences() {
    for method in METHODS {
        let mut previous = None;
        for _ in 0..3 {
            let sequence = RefCell::new(Vec::new());
            let report = run(
                method,
                &mut |x| {
                    sequence.borrow_mut().push((false, x.to_bits()));
                    x * x * x - 2.0
                },
                &mut |x| {
                    sequence.borrow_mut().push((true, x.to_bits()));
                    3.0 * x * x
                },
                0.1,
                bracket(-2.0, 2.0),
                options(1e-12, 1e-12, 500),
            )
            .unwrap();
            let result = (report, sequence.into_inner());
            if let Some(ref expected) = previous {
                assert_eq!(&result, expected);
            }
            previous = Some(result);
        }
    }
}

#[test]
fn callback_panics_propagate_instead_of_becoming_solver_errors() {
    for method in METHODS {
        let result = std::panic::catch_unwind(|| {
            run(
                method,
                &mut |_| panic!("caller function panic"),
                &mut |_| 1.0,
                0.0,
                bracket(-1.0, 1.0),
                options(1e-12, 0.0, 100),
            )
        });
        assert!(result.is_err());
    }
    let result = std::panic::catch_unwind(|| {
        safeguarded_newton(
            |x| x * x - 2.0,
            |_| panic!("caller derivative panic"),
            1.0,
            bracket(0.0, 2.0),
            options(1e-12, 0.0, 100),
        )
    });
    assert!(result.is_err());
}

#[test]
fn diagnostic_errors_have_stable_kinds_display_and_numeric_source_chains() {
    let errors = [
        RootBracket::new(f64::NAN, 1.0).unwrap_err(),
        RootBracket::new(1.0, 0.0).unwrap_err(),
        SolverOptions::new(1.0, 0.0, 0).unwrap_err(),
        bisect(|_| 1.0, bracket(0.0, 1.0), options(1e-12, 0.0, 10)).unwrap_err(),
        bisect(|_| f64::NAN, bracket(0.0, 1.0), options(1e-12, 0.0, 10)).unwrap_err(),
        bisect(|x| x - 0.5, bracket(0.0, 1.0), options(1e-12, 0.0, 1)).unwrap_err(),
        bisect(
            |x| if x == 1.0 { -1.0 } else { 1.0 },
            bracket(1.0, 1.0_f64.next_up()),
            options(f64::from_bits(1), 0.0, 10),
        )
        .unwrap_err(),
    ];
    let kinds = [
        "solver.value",
        "solver.invalid_bracket",
        "solver.invalid_budget",
        "solver.not_bracketed",
        "solver.non_finite_evaluation",
        "solver.evaluation_limit",
        "solver.precision_exhausted",
    ];
    for (index, (error, kind)) in errors.iter().zip(kinds).enumerate() {
        assert_eq!(error.kind(), kind);
        assert_eq!(&error.clone(), error);
        assert!(error.to_string().contains("solver"));
        assert_eq!(error.source().is_some(), index == 0 || index == 4);
    }
}
