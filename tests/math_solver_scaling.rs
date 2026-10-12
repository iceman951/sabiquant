//! Bracket preservation when finite residuals span the whole binary64 range.

use sabiquant::math::{RootBracket, SolverOptions, TerminationReason, brent};

#[test]
fn collapsed_scaled_ordinates_fall_back_without_losing_a_continuous_sign_bracket() {
    let tiny = f64::from_bits(1);
    let mut points = Vec::new();
    // Independently defined continuous piecewise-linear function. The central
    // root is exactly 5/8; endpoint residuals round to ±MAX, while the central
    // residuals are only a few MIN_SUB. Normalizing those distinct small
    // residuals by MAX erases their difference, so inverse interpolation must
    // remain optional. No spline/solver coefficient constructs this function.
    let report = brent(
        |x: f64| {
            points.push(x);
            if x < 0.0 {
                x * f64::MAX - 5.0 * tiny
            } else if x <= 0.875 {
                (x - 0.625) * (8.0 * tiny)
            } else {
                ((x - 0.875) / 0.125) * f64::MAX + 2.0 * tiny
            }
        },
        RootBracket::new(-1.0, 1.0).unwrap(),
        SolverOptions::new(1e-12, 0.0, 20).unwrap(),
    )
    .unwrap();
    assert_eq!(report.reason(), TerminationReason::FunctionZero);
    assert_eq!(report.point().to_bits(), 0.625_f64.to_bits());
    assert_eq!(report.residual().to_bits(), 0);
    assert_eq!(report.evaluations().derivative(), 0);
    assert_eq!(
        usize::try_from(report.evaluations().function()).unwrap(),
        points.len()
    );
    assert!(
        points
            .iter()
            .all(|x| x.is_finite() && *x >= -1.0 && *x <= 1.0)
    );
    // Endpoint protocol, first midpoint, and the two midpoint fallbacks are
    // observed through the public callback rather than private solver state.
    let observed: Vec<u64> = points.iter().take(5).map(|x| x.to_bits()).collect();
    let expected = [-1.0_f64, 1.0, 0.0, 0.5, 0.75].map(f64::to_bits);
    assert_eq!(observed, expected);
}
