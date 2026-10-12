//! Normal-domain, tail, inverse-conditioning and location/scale regressions.

mod support;

use sabiquant::error::{NonFinite, ValueError};
use sabiquant::math::{
    NormalDistribution, NormalError, standard_normal_cdf, standard_normal_inverse_cdf,
    standard_normal_log_cdf, standard_normal_log_survival, standard_normal_pdf,
    standard_normal_survival,
};
use std::error::Error;
use support::float::{Tolerance, assert_close};
use support::prng::SplitMix64;

const UNIT: f64 = f64::EPSILON / 2.0;

#[test]
fn every_public_real_input_rejects_nonfinite_values_before_domain_checks() {
    let distribution = NormalDistribution::new(2.0, 3.0).unwrap();
    for (input, class) in [
        (f64::NAN, NonFinite::NaN),
        (f64::INFINITY, NonFinite::PositiveInfinity),
        (f64::NEG_INFINITY, NonFinite::NegativeInfinity),
    ] {
        for function in [
            standard_normal_pdf,
            standard_normal_cdf,
            standard_normal_survival,
            standard_normal_log_cdf,
            standard_normal_log_survival,
        ] {
            assert!(matches!(function(input),
                Err(NormalError::Value(ValueError::NotFinite {
                    parameter: "normal variate", value, ..
                })) if value == class));
        }
        for result in [
            distribution.pdf(input),
            distribution.cdf(input),
            distribution.survival(input),
            distribution.log_cdf(input),
            distribution.log_survival(input),
        ] {
            assert!(matches!(result,
                Err(NormalError::Value(ValueError::NotFinite {
                    parameter: "normal variate", value, ..
                })) if value == class));
        }
        for result in [
            standard_normal_inverse_cdf(input),
            distribution.inverse_cdf(input),
        ] {
            assert!(matches!(result,
                Err(NormalError::Value(ValueError::NotFinite {
                    parameter: "normal probability", value, ..
                })) if value == class));
        }
        assert!(matches!(
            NormalDistribution::new(input, 1.0),
            Err(NormalError::Value(ValueError::NotFinite {
                parameter: "normal mean",
                ..
            }))
        ));
        assert!(matches!(
            NormalDistribution::new(0.0, input),
            Err(NormalError::Value(ValueError::NotFinite {
                parameter: "normal standard deviation",
                ..
            }))
        ));
    }
}

#[test]
fn location_scale_and_probability_domains_are_explicit() {
    for scale in [-f64::MAX, -1.0, -f64::from_bits(1), -0.0, 0.0] {
        assert!(matches!(
            NormalDistribution::new(0.0, scale),
            Err(NormalError::Value(ValueError::OutOfDomain { .. }))
        ));
    }
    for probability in [-1.0, -0.0, 0.0, 1.0, 1.0_f64.next_up(), f64::MAX] {
        assert!(matches!(
            standard_normal_inverse_cdf(probability),
            Err(NormalError::Value(ValueError::OutOfDomain { .. }))
        ));
    }
    for mean in [-f64::MAX, -1.0, -0.0, 0.0, f64::MAX] {
        for scale in [f64::from_bits(1), f64::MIN_POSITIVE, 1.0, f64::MAX] {
            let distribution = NormalDistribution::new(mean, scale).unwrap();
            assert_eq!(
                distribution.mean().to_bits(),
                if mean == 0.0 { 0 } else { mean.to_bits() }
            );
            assert_eq!(distribution.standard_deviation().to_bits(), scale.to_bits());
            assert_eq!(distribution.cdf(mean).unwrap().to_bits(), 0.5_f64.to_bits());
            assert_eq!(
                distribution.survival(mean).unwrap().to_bits(),
                0.5_f64.to_bits()
            );
            assert_eq!(
                distribution.inverse_cdf(0.5).unwrap().to_bits(),
                distribution.mean().to_bits()
            );
        }
    }
    for zero in [-0.0, 0.0] {
        assert_eq!(
            standard_normal_cdf(zero).unwrap().to_bits(),
            0.5_f64.to_bits()
        );
        assert_eq!(
            standard_normal_survival(zero).unwrap().to_bits(),
            0.5_f64.to_bits()
        );
    }
    assert_eq!(standard_normal_inverse_cdf(0.5).unwrap().to_bits(), 0);
}

#[test]
fn independent_selected_decimal_values_cover_center_and_tails() {
    // Rounded values from independent 200-digit Decimal Mills convergents.
    // Complete committed two-precision evidence lives in the oracle suite.
    for (x, expected) in [
        (1.5, 0.066_807_201_268_858_07),
        (2.0, 0.022_750_131_948_179_21),
        (3.0, 0.001_349_898_031_630_094_6),
        (5.0, 2.866_515_718_791_939e-7),
        (8.0, 6.220_960_574_271_784e-16),
        (20.0, 2.753_624_118_606_233_7e-89),
        (37.0, 5.725_571_222_524_577e-300),
        (38.0, 2.885_428_36e-316),
    ] {
        let tolerance = Tolerance::Mixed {
            absolute: f64::from_bits(1),
            relative: 8.0 * UNIT * (1.0 + x * x),
        };
        assert_close(standard_normal_survival(x).unwrap(), expected, tolerance);
        assert_close(standard_normal_cdf(-x).unwrap(), expected, tolerance);
    }
    for (p, expected) in [
        (0.025, -1.959_963_984_540_054_3),
        (0.1, -1.281_551_565_544_600_4),
        (0.25, -0.674_489_750_196_081_7),
        (0.5, 0.0),
        (0.75, 0.674_489_750_196_081_7),
        (0.9, 1.281_551_565_544_600_6),
        (0.975, 1.959_963_984_540_053_8),
    ] {
        assert_close(
            standard_normal_inverse_cdf(p).unwrap(),
            expected,
            Tolerance::Mixed {
                absolute: 0.0,
                relative: 16.0 * UNIT,
            },
        );
    }
}

#[test]
fn logarithmic_tails_outlive_probability_underflow_and_report_range_errors() {
    for x in [40.0, 50.0, 1000.0, 1.0e100] {
        assert_eq!(standard_normal_survival(x).unwrap().to_bits(), 0);
        assert_eq!(standard_normal_cdf(-x).unwrap().to_bits(), 0);
        assert_eq!(standard_normal_pdf(x).unwrap().to_bits(), 0);
        let logarithm = standard_normal_log_survival(x).unwrap();
        assert!(logarithm.is_finite() && logarithm < -745.0);
        assert_close(
            standard_normal_log_cdf(-x).unwrap(),
            logarithm,
            Tolerance::Mixed {
                absolute: 0.0,
                relative: 8.0 * UNIT,
            },
        );
        assert_eq!(standard_normal_log_cdf(x).unwrap().to_bits(), 0);
        assert_eq!(standard_normal_log_survival(-x).unwrap().to_bits(), 0);
    }
    for x in [f64::MAX, -f64::MAX] {
        assert!(standard_normal_cdf(x).unwrap().is_finite());
        assert!(standard_normal_survival(x).unwrap().is_finite());
        assert_eq!(standard_normal_pdf(x).unwrap().to_bits(), 0);
    }
    assert!(matches!(
        standard_normal_log_cdf(-f64::MAX),
        Err(NormalError::Value(ValueError::ResultNotFinite { .. }))
    ));
    assert!(matches!(
        standard_normal_log_survival(f64::MAX),
        Err(NormalError::Value(ValueError::ResultNotFinite { .. }))
    ));
}

#[test]
fn median_neighbor_quantiles_preserve_centered_mass() {
    let mut lower = 0.5_f64;
    let mut upper = 0.5_f64;
    for _ in 0..128 {
        lower = lower.next_down();
        upper = upper.next_up();
        for p in [lower, upper] {
            let expected_linear = (p - 0.5) * 2.506_628_274_631_000_7;
            let z = standard_normal_inverse_cdf(p).unwrap();
            assert_close(
                z,
                expected_linear,
                Tolerance::Mixed {
                    absolute: 0.0,
                    relative: 8.0 * UNIT,
                },
            );
            assert_close(
                standard_normal_cdf(z).unwrap(),
                p,
                Tolerance::Mixed {
                    absolute: 2.0 * UNIT,
                    relative: 0.0,
                },
            );
        }
    }
}

#[test]
fn every_binary_probability_exponent_including_subnormal_tails_round_trips() {
    let mut probability = 0.5;
    for _ in 1..=1074 {
        let z = standard_normal_inverse_cdf(probability).unwrap();
        assert!(z.is_finite() && z <= 0.0);
        let log_probability = probability.ln();
        assert_close(
            standard_normal_log_cdf(z).unwrap(),
            log_probability,
            Tolerance::Mixed {
                absolute: 16.0 * UNIT * (1.0 + log_probability.abs()),
                relative: 0.0,
            },
        );
        if probability < 0.5 {
            assert!(z < 0.0);
        }
        probability *= 0.5;
    }
    for bits in 1..=512 {
        let p = f64::from_bits(bits);
        let z = standard_normal_inverse_cdf(p).unwrap();
        assert!(z.is_finite() && z < -37.0);
        assert_close(standard_normal_cdf(z).unwrap(), p, Tolerance::Ulps(1));
    }
}

#[test]
fn affine_extremes_recover_densities_and_finite_cancellation() {
    let maximum_scale = NormalDistribution::new(-f64::MAX, f64::MAX).unwrap();
    assert_close(
        maximum_scale.cdf(f64::MAX).unwrap(),
        standard_normal_cdf(2.0).unwrap(),
        Tolerance::Mixed {
            absolute: 4.0 * UNIT,
            relative: 0.0,
        },
    );
    let cancellation = maximum_scale.inverse_cdf(0.975).unwrap();
    assert!(cancellation.is_finite() && cancellation > 0.0);
    let z = standard_normal_inverse_cdf(0.975).unwrap();
    assert_eq!(
        cancellation.to_bits(),
        f64::MAX.mul_add(z, -f64::MAX).to_bits()
    );
    assert!(matches!(
        maximum_scale.inverse_cdf(1.0_f64.next_down()),
        Err(NormalError::Value(ValueError::ResultNotFinite { .. }))
    ));

    let min_sub = f64::from_bits(1);
    let tiny_scale = NormalDistribution::new(0.0, min_sub).unwrap();
    for z in [39.0, 40.0] {
        assert_eq!(standard_normal_pdf(z).unwrap().to_bits(), 0);
        let density = tiny_scale.pdf(z * min_sub).unwrap();
        assert!(density.is_finite() && density > 0.0);
    }
    assert!(matches!(
        tiny_scale.pdf(0.0),
        Err(NormalError::Value(ValueError::ResultNotFinite { .. }))
    ));
    assert_eq!(tiny_scale.pdf(f64::MAX).unwrap().to_bits(), 0);
    assert_eq!(
        tiny_scale.cdf(f64::MAX).unwrap().to_bits(),
        1.0_f64.to_bits()
    );
    assert_eq!(tiny_scale.log_cdf(f64::MAX).unwrap().to_bits(), 0);
    assert!(tiny_scale.log_cdf(-f64::MAX).is_err());
}

#[test]
fn dense_probability_shapes_and_seeded_inverse_reuse_have_no_hidden_state() {
    let mut previous = 0.0;
    for index in -20_000..=20_000 {
        let x = f64::from(index) / 2000.0;
        let cdf = standard_normal_cdf(x).unwrap();
        let survival = standard_normal_survival(x).unwrap();
        let density = standard_normal_pdf(x).unwrap();
        assert!((0.0..=1.0).contains(&cdf) && cdf >= previous);
        assert!((0.0..=1.0).contains(&survival) && density >= 0.0);
        assert_close(
            cdf + survival,
            1.0,
            Tolerance::Mixed {
                absolute: 4.0 * UNIT,
                relative: 0.0,
            },
        );
        previous = cdf;
    }
    let mut random = SplitMix64::new(0x21c0_1024_5481_0027);
    for _ in 0..10_000 {
        let p = random.next_f64();
        if p == 0.0 {
            continue;
        }
        let z = standard_normal_inverse_cdf(p).unwrap();
        assert_close(
            standard_normal_cdf(z).unwrap(),
            p,
            Tolerance::Mixed {
                absolute: 8.0 * UNIT,
                relative: 0.0,
            },
        );
        let distribution = NormalDistribution::new(10.0, 2.0).unwrap();
        let affine = distribution.inverse_cdf(p).unwrap();
        assert_close(
            distribution.cdf(affine).unwrap(),
            p,
            Tolerance::Mixed {
                absolute: 16.0 * UNIT,
                relative: 0.0,
            },
        );
    }
}

#[test]
fn normal_errors_preserve_context_and_source_chains() {
    let error = standard_normal_inverse_cdf(0.0).unwrap_err();
    assert_eq!(error.kind(), "normal.value");
    assert!(error.source().unwrap().is::<ValueError>());
    assert!(error.to_string().contains("normal probability"));
    let wrapper = sabiquant::error::Error::from(error.clone());
    assert!(wrapper.source().unwrap().is::<NormalError>());
    assert_eq!(error.clone(), error);
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<NormalDistribution>();
    assert_send_sync::<NormalError>();
}

#[test]
fn density_overflow_boundary_is_explicit_and_guarded() {
    let critical_scale = 0.398_942_280_401_432_7 / f64::MAX;
    let distribution = NormalDistribution::new(0.0, critical_scale).unwrap();
    let error = distribution.pdf(0.0).unwrap_err();
    assert!(matches!(
        error,
        NormalError::NumericalBoundary {
            quantity: "normal density",
            ..
        }
    ));
    assert_eq!(error.kind(), "normal.numerical_boundary");
    assert!(error.source().is_none());
    assert!(error.to_string().contains("normal density"));
}

#[test]
fn affine_quantile_overflow_midpoint_is_guarded_without_rejecting_safe_maxima() {
    // MAX itself is finite. Its nearest-even overflow midpoint lies one
    // half-spacing above it, so harmless tiny perturbations round to MAX.
    for mean in [-f64::MAX, f64::MAX] {
        for scale in [f64::from_bits(1), f64::MIN_POSITIVE, 1.0] {
            let distribution = NormalDistribution::new(mean, scale).unwrap();
            for p in [f64::from_bits(1), 0.025, 0.5, 0.975, 1.0_f64.next_down()] {
                assert_eq!(
                    distribution.inverse_cdf(p).unwrap().to_bits(),
                    mean.to_bits()
                );
            }
        }
    }

    for p in [0.025, 0.975] {
        let z = standard_normal_inverse_cdf(p).unwrap();
        let critical = f64::MAX / z.abs();
        let distribution = NormalDistribution::new(0.0, critical).unwrap();
        let error = distribution.inverse_cdf(p).unwrap_err();
        assert!(matches!(
            error,
            NormalError::NumericalBoundary {
                quantity: "normal quantile",
                logarithm,
                ..
            } if logarithm.is_finite()
        ));
        assert_eq!(error.kind(), "normal.numerical_boundary");
        assert!(error.source().is_none());
        let finite = NormalDistribution::new(0.0, critical * (1.0 - 1e-12)).unwrap();
        assert!(finite.inverse_cdf(p).unwrap().is_finite());
        let overflow = NormalDistribution::new(0.0, critical * (1.0 + 1e-12)).unwrap();
        assert!(matches!(
            overflow.inverse_cdf(p),
            Err(NormalError::Value(ValueError::ResultNotFinite { .. }))
        ));
    }
}

#[test]
fn shared_immutable_distributions_retain_conditioned_results_across_threads() {
    let distribution = NormalDistribution::new(-3.0, 7.0).unwrap();
    let probabilities: Vec<_> = (1..=256).map(|index| f64::from(index) / 257.0).collect();
    let expected: Vec<_> = probabilities
        .iter()
        .map(|&p| distribution.inverse_cdf(p).unwrap())
        .collect();
    let mut tasks = Vec::new();
    for worker in 0..8 {
        let probabilities = probabilities.clone();
        let expected = expected.clone();
        #[expect(
            clippy::disallowed_methods,
            reason = "test shares immutable distributions across threads to verify absence of hidden state"
        )]
        let task = std::thread::spawn(move || {
            for round in 0..4 {
                for offset in 0..256 {
                    let index = (offset + worker * 17 + round * 29) % 256;
                    let actual = distribution.inverse_cdf(probabilities[index]).unwrap();
                    assert_close(
                        actual,
                        expected[index],
                        Tolerance::Mixed {
                            absolute: 32.0 * UNIT * (1.0 + expected[index].abs()),
                            relative: 0.0,
                        },
                    );
                    assert_close(
                        distribution.cdf(actual).unwrap(),
                        probabilities[index],
                        Tolerance::Mixed {
                            absolute: 16.0 * UNIT,
                            relative: 0.0,
                        },
                    );
                }
            }
        });
        tasks.push(task);
    }
    for task in tasks {
        task.join().unwrap();
    }
}

#[test]
fn explicit_standard_context_matches_free_functions_with_conditioned_arithmetic() {
    let distribution = NormalDistribution::new(0.0, 1.0).unwrap();
    for index in -1000..=1000 {
        let x = f64::from(index) / 100.0;
        let relative = 8.0 * UNIT * (1.0 + x * x);
        assert_close(
            distribution.pdf(x).unwrap(),
            standard_normal_pdf(x).unwrap(),
            Tolerance::Mixed {
                absolute: f64::from_bits(1),
                relative,
            },
        );
        assert_close(
            distribution.cdf(x).unwrap(),
            standard_normal_cdf(x).unwrap(),
            Tolerance::Mixed {
                absolute: 8.0 * UNIT,
                relative: 0.0,
            },
        );
        assert_close(
            distribution.survival(x).unwrap(),
            standard_normal_survival(x).unwrap(),
            Tolerance::Mixed {
                absolute: 8.0 * UNIT,
                relative: 0.0,
            },
        );
        assert_close(
            distribution.log_cdf(x).unwrap(),
            standard_normal_log_cdf(x).unwrap(),
            Tolerance::Mixed {
                absolute: 8.0 * UNIT,
                relative,
            },
        );
        assert_close(
            distribution.log_survival(x).unwrap(),
            standard_normal_log_survival(x).unwrap(),
            Tolerance::Mixed {
                absolute: 8.0 * UNIT,
                relative,
            },
        );
    }
}

#[test]
fn independent_simpson_quadrature_checks_density_mass_against_cdf_increments() {
    // Simpson's exact-real error <= L*h^4*max|phi''''|/180. On |x|<=8,
    // |phi''''| <= (8^4+6*8^2+3)/sqrt(2pi) < 1800. Here L=2 and
    // h=2/8192, so this truncation bound is <7.2e-14. A 2e-13 absolute
    // allowance also covers compensated reduction and elementary rounding.
    for left in [-8.0, -6.0, -4.0, -2.0, 0.0, 2.0, 4.0, 6.0] {
        let right = left + 2.0;
        let step = (right - left) / 8192.0;
        let mut samples = Vec::with_capacity(8193);
        for index in 0..=8192 {
            let x = f64::from(index).mul_add(step, left);
            let weight = if index == 0 || index == 8192 {
                1.0
            } else if index % 2 == 0 {
                2.0
            } else {
                4.0
            };
            samples.push(weight * standard_normal_pdf(x).unwrap());
        }
        let integral = sabiquant::math::compensated_sum(&samples).unwrap() * step / 3.0;
        let mass = if left >= 0.0 {
            standard_normal_survival(left).unwrap() - standard_normal_survival(right).unwrap()
        } else {
            standard_normal_cdf(right).unwrap() - standard_normal_cdf(left).unwrap()
        };
        assert_close(
            integral,
            mass,
            Tolerance::Mixed {
                absolute: 2e-13,
                relative: 0.0,
            },
        );
    }
}
