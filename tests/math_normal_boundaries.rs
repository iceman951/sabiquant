//! Public overflow context and canonical saturation at normal range boundaries.

use sabiquant::error::{NonFinite, ValueError};
use sabiquant::math::{
    NormalDistribution, NormalError, standard_normal_cdf, standard_normal_log_cdf,
    standard_normal_log_survival, standard_normal_survival,
};
use std::error::Error;

#[test]
fn unrepresentable_lower_tail_logs_keep_quantity_and_negative_infinity_context() {
    let tiny = NormalDistribution::new(10.0, f64::from_bits(1)).unwrap();
    for result in [
        standard_normal_log_cdf(-f64::MAX),
        standard_normal_log_survival(f64::MAX),
        tiny.log_cdf(-f64::MAX),
        tiny.log_survival(f64::MAX),
    ] {
        let error = result.unwrap_err();
        assert!(matches!(
            &error,
            NormalError::Value(ValueError::ResultNotFinite {
                quantity: "normal logarithmic tail",
                value: NonFinite::NegativeInfinity,
                ..
            })
        ));
        assert_eq!(error.kind(), "normal.value");
        assert!(error.source().unwrap().is::<ValueError>());
        assert!(error.to_string().contains("normal logarithmic tail"));
        let wrapper = sabiquant::error::Error::from(error);
        assert!(wrapper.source().unwrap().is::<NormalError>());
    }
    for result in [
        standard_normal_cdf(-f64::MAX),
        standard_normal_survival(f64::MAX),
        standard_normal_log_cdf(f64::MAX),
        standard_normal_log_survival(-f64::MAX),
        tiny.cdf(-f64::MAX),
        tiny.survival(f64::MAX),
        tiny.log_cdf(f64::MAX),
        tiny.log_survival(-f64::MAX),
    ] {
        assert_eq!(result.unwrap().to_bits(), 0);
    }
}

#[test]
fn a_maximum_mean_retains_the_half_spacing_overflow_midpoint() {
    // This independently selected central quantile is also checked against
    // the two-precision oracle. A MAX mean leaves a real half-spacing before
    // overflow; it must not be mistaken for a result already outside range.
    let half_spacing = f64::from_bits((1023 + 970) << 52);
    let scale = half_spacing / 0.674_489_750_196_081_7;
    for (mean, p) in [(-f64::MAX, 0.25), (f64::MAX, 0.75)] {
        let distribution = NormalDistribution::new(mean, scale).unwrap();
        assert_eq!(
            distribution.inverse_cdf(0.5).unwrap().to_bits(),
            mean.to_bits()
        );
        let error = distribution.inverse_cdf(p).unwrap_err();
        assert!(matches!(
            &error,
            NormalError::NumericalBoundary {
                quantity: "normal quantile",
                logarithm,
                ..
            } if logarithm.is_finite() && *logarithm > 700.0
        ));
        assert_eq!(error.kind(), "normal.numerical_boundary");
        assert!(error.source().is_none());
        assert!(error.to_string().contains("normal quantile"));
        let wrapper = sabiquant::error::Error::from(error);
        assert!(wrapper.source().unwrap().is::<NormalError>());
    }
}

#[test]
fn density_overflow_context_is_independent_of_the_supplied_mean() {
    let critical_scale = 0.398_942_280_401_432_7 / f64::MAX;
    for mean in [-f64::MAX, -10.0, 10.0, f64::MAX] {
        let overflow = NormalDistribution::new(mean, f64::from_bits(1)).unwrap();
        assert!(matches!(
            overflow.pdf(mean),
            Err(NormalError::Value(ValueError::ResultNotFinite {
                quantity: "normal density",
                value: NonFinite::PositiveInfinity,
                ..
            }))
        ));
        let guarded = NormalDistribution::new(mean, critical_scale).unwrap();
        assert!(matches!(
            guarded.pdf(mean),
            Err(NormalError::NumericalBoundary {
                quantity: "normal density",
                logarithm,
                ..
            }) if logarithm.is_finite()
        ));
    }
}
