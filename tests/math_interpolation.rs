//! Public interpolation contracts, independent small-grid properties and extremes.

mod support;

use sabiquant::error::{Error as CrateError, NonFinite, ValueError};
use sabiquant::math::{Extrapolation, InterpolationError, InterpolationMethod, Interpolator};
use std::error::Error;
use std::sync::Arc;
use support::float::{Tolerance, assert_close};

const U: f64 = f64::EPSILON / 2.0;
const MIN_SUB: f64 = f64::from_bits(1);

fn make(
    nodes: &[(f64, f64)],
    method: InterpolationMethod,
) -> Result<Interpolator, InterpolationError> {
    Interpolator::new(nodes.to_vec(), method, Extrapolation::Reject)
}

#[test]
fn method_and_policy_identifiers_and_public_value_traits_are_explicit() {
    fn choice<T: Copy + Eq + std::hash::Hash + std::fmt::Debug + Send + Sync + 'static>() {}
    fn value<T: Clone + PartialEq + std::fmt::Debug + Send + Sync + 'static>() {}
    fn error<T: Error + Clone + PartialEq + Send + Sync + 'static>() {}
    choice::<InterpolationMethod>();
    choice::<Extrapolation>();
    value::<Interpolator>();
    error::<InterpolationError>();
    macro_rules! identifiers {
        ($ty:ty) => {
            for value in <$ty>::ALL {
                let text = value.to_string();
                assert_eq!(text, value.canonical_identifier());
                assert_eq!(text.parse::<$ty>().unwrap(), value);
                for bad in [
                    format!(" {text}"),
                    format!("{text} "),
                    text.to_lowercase(),
                    format!("{text}\0"),
                    format!("{text}🙂"),
                ] {
                    assert_eq!(bad.parse::<$ty>().unwrap_err().kind(), "identifier.unknown");
                }
            }
        };
    }
    identifiers!(InterpolationMethod);
    identifiers!(Extrapolation);
    assert!(
        matches!("Cubic".parse::<InterpolationMethod>(), Err(sabiquant::error::IdentifierError::Ambiguous { candidates, .. }) if candidates == ["NaturalCubic", "MonotoneCubicHermite"])
    );
    for bad in [
        "",
        "Cubic",
        "MonotonicCubicNaturalSpline",
        "true",
        "false",
        "线性",
    ] {
        assert!(bad.parse::<InterpolationMethod>().is_err());
        assert!(bad.parse::<Extrapolation>().is_err());
    }
}

#[test]
fn constructor_rejects_invalid_pairs_with_indexed_typed_causes() -> Result<(), Box<dyn Error>> {
    for method in InterpolationMethod::ALL {
        for nodes in [vec![], vec![(0.0, 1.0)]] {
            assert!(
                matches!(make(&nodes, method), Err(InterpolationError::NotEnoughNodes { actual, minimum: 2, .. }) if actual == nodes.len())
            );
        }
        for index in 0..3 {
            for component in 0..2 {
                for (bad, expected) in [
                    (f64::NAN, NonFinite::NaN),
                    (f64::INFINITY, NonFinite::PositiveInfinity),
                    (f64::NEG_INFINITY, NonFinite::NegativeInfinity),
                ] {
                    let mut nodes = vec![(0.0, 1.0), (1.0, 2.0), (2.0, 3.0)];
                    if component == 0 {
                        nodes[index].0 = bad;
                    } else {
                        nodes[index].1 = bad;
                    }
                    let error = make(&nodes, method).unwrap_err();
                    assert!(
                        matches!(&error, InterpolationError::InvalidNode { index: actual, cause: ValueError::NotFinite { value, .. }, .. } if *actual == index && *value == expected)
                    );
                    assert_eq!(error.clone(), error);
                    assert!(error.source().is_some_and(|cause| cause.is::<ValueError>()));
                    assert!(!error.to_string().is_empty());
                }
            }
        }
        for nodes in [
            vec![(0.0, 1.0), (0.0, 2.0)],
            vec![(1.0, 1.0), (-1.0, 2.0)],
            vec![(-0.0, 1.0), (0.0, 2.0)],
        ] {
            assert!(matches!(
                make(&nodes, method),
                Err(InterpolationError::NotStrictlyIncreasing { index: 1, .. })
            ));
        }
    }
    for value in [0.0, -0.0, -MIN_SUB, -1.0, -f64::MAX] {
        let error = make(&[(0.0, 1.0), (1.0, value)], InterpolationMethod::LogLinear).unwrap_err();
        assert!(matches!(
            error,
            InterpolationError::InvalidNode {
                index: 1,
                cause: ValueError::OutOfDomain { .. },
                ..
            }
        ));
    }
    let cause = make(&[], InterpolationMethod::Linear).unwrap_err();
    let root = CrateError::from(cause.clone());
    assert_eq!(root.kind(), cause.kind());
    assert_eq!(
        root.source()
            .and_then(|e| e.downcast_ref::<InterpolationError>()),
        Some(&cause)
    );
    Ok(())
}

#[test]
fn exact_knots_closed_domains_and_extrapolation_never_infer_a_policy() -> Result<(), Box<dyn Error>>
{
    let nodes = vec![(0.0, 1.0), (1.0, 4.0), (3.0, 2.0), (6.0, 8.0)];
    for method in InterpolationMethod::ALL {
        for policy in Extrapolation::ALL {
            let f = Interpolator::new(nodes.clone(), method, policy)?;
            assert_eq!(f.nodes(), nodes);
            assert_eq!(f.domain(), (0.0, 6.0));
            assert_eq!(f.method(), method);
            assert_eq!(f.extrapolation(), policy);
            for &(x, y) in &nodes {
                assert_eq!(f.value(x)?.to_bits(), y.to_bits());
            }
            for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                assert!(matches!(
                    f.value(bad),
                    Err(InterpolationError::Value(ValueError::NotFinite {
                        parameter: "interpolation query",
                        ..
                    }))
                ));
            }
            for query in [-MIN_SUB, 6.0_f64.next_up()] {
                match policy {
                    Extrapolation::Reject => assert!(matches!(
                        f.value(query),
                        Err(InterpolationError::OutsideDomain { .. })
                    )),
                    Extrapolation::Flat => assert_eq!(
                        f.value(query)?.to_bits(),
                        (if query < 0.0 { 1.0_f64 } else { 8.0_f64 }).to_bits()
                    ),
                    Extrapolation::ExtendBoundarySegment => assert!(f.value(query)?.is_finite()),
                    other => return Err(format!("unexpected policy {other:?}").into()),
                }
            }
        }
    }
    let nodes = [(0.0, 2.0), (1.0, 4.0), (2.0, 8.0)];
    for (method, left, right) in [
        (InterpolationMethod::Linear, 0.0, 12.0),
        (InterpolationMethod::LogLinear, 1.0, 16.0),
    ] {
        let f = Interpolator::new(nodes.to_vec(), method, Extrapolation::ExtendBoundarySegment)?;
        assert_close(f.value(-1.0)?, left, Tolerance::Ulps(8));
        assert_close(f.value(3.0)?, right, Tolerance::Ulps(8));
    }
    Ok(())
}

#[test]
fn constants_and_two_node_identities_survive_extreme_unused_derivatives()
-> Result<(), Box<dyn Error>> {
    for method in InterpolationMethod::ALL {
        for value in [-f64::MAX, -1.0, -MIN_SUB, -0.0, 0.0, MIN_SUB, 1.0, f64::MAX] {
            if method == InterpolationMethod::LogLinear && value <= 0.0 {
                continue;
            }
            let f = Interpolator::new(
                vec![(-f64::MAX, value), (0.0, value), (f64::MAX, value)],
                method,
                Extrapolation::ExtendBoundarySegment,
            )?;
            let expected = if value == 0.0 { 0 } else { value.to_bits() };
            for query in [-f64::MAX, -MIN_SUB, -0.0, 0.0, MIN_SUB, f64::MAX] {
                assert_eq!(f.value(query)?.to_bits(), expected);
            }
        }
        let (first, last) = if method == InterpolationMethod::LogLinear {
            (MIN_SUB, 1.0)
        } else {
            (0.0, 1.0)
        };
        let f = make(&[(0.0, first), (MIN_SUB, last)], method)?;
        assert_eq!(f.value(0.0)?.to_bits(), first.to_bits());
        assert_eq!(f.value(MIN_SUB)?.to_bits(), last.to_bits());
    }
    let f = make(&[(-0.0, -0.0), (1.0, 1.0)], InterpolationMethod::Linear)?;
    assert_eq!(f.nodes()[0].0.to_bits(), 0);
    assert_eq!(f.value(-0.0)?.to_bits(), 0);
    Ok(())
}

#[test]
fn affine_identity_recovers_subnormal_and_huge_contributions_before_scaling()
-> Result<(), Box<dyn Error>> {
    for method in [
        InterpolationMethod::Linear,
        InterpolationMethod::NaturalCubic,
        InterpolationMethod::MonotoneCubicHermite,
    ] {
        let f = make(&[(-f64::MAX, -f64::MAX), (f64::MAX, f64::MAX)], method)?;
        for query in [
            MIN_SUB,
            -MIN_SUB,
            f64::MIN_POSITIVE,
            -f64::MIN_POSITIVE,
            1.0,
            -1.0,
            f64::MAX / 2.0,
            -f64::MAX / 2.0,
        ] {
            assert_eq!(
                f.value(query)?.to_bits(),
                query.to_bits(),
                "{method:?},q={query:?}"
            );
        }
    }
    let f = make(
        &[(0.0, 0.0), (f64::MAX, f64::MAX)],
        InterpolationMethod::Linear,
    )?;
    for query in [MIN_SUB, 1e-300, 1e-15, 0.5, 1.0, 1e300] {
        assert_eq!(f.value(query)?.to_bits(), query.to_bits());
    }
    let f = Interpolator::new(
        vec![(0.0, -f64::MAX), (1.0, -f64::MAX / 2.0)],
        InterpolationMethod::Linear,
        Extrapolation::ExtendBoundarySegment,
    )?;
    assert_eq!(f.value(4.0)?.to_bits(), f64::MAX.to_bits());
    let f = make(
        &[(-f64::MAX, 1.0), (f64::MAX, 2.0)],
        InterpolationMethod::LogLinear,
    )?;
    assert_close(f.value(0.0)?, 2.0_f64.sqrt(), Tolerance::Ulps(8));
    let f = make(
        &[(0.0, MIN_SUB), (1.0, f64::MAX)],
        InterpolationMethod::LogLinear,
    )?;
    let budget = 8.0 * U * (1.0 + (MIN_SUB.ln().abs() + f64::MAX.ln().abs()) / 2.0);
    assert_close(
        f.value(0.5)?,
        2.0_f64.powi(-25),
        Tolerance::Mixed {
            absolute: MIN_SUB,
            relative: budget,
        },
    );
    Ok(())
}

#[test]
fn small_linear_grids_match_an_independent_integer_weight_oracle() -> Result<(), Box<dyn Error>> {
    let mut cases = 0;
    for x0 in -4..=2 {
        for width in 1..=6 {
            for y0 in -4..=4 {
                for y1 in -4..=4 {
                    let f = Interpolator::new(
                        vec![
                            (f64::from(x0), f64::from(y0)),
                            (f64::from(x0 + width), f64::from(y1)),
                        ],
                        InterpolationMethod::Linear,
                        Extrapolation::ExtendBoundarySegment,
                    )?;
                    for position in -8..=16 {
                        let query = f64::from(x0) + f64::from(width * position) / 8.0;
                        let expected = f64::from((8 - position) * y0 + position * y1) / 8.0;
                        let absolute = 16.0
                            * U
                            * (f64::from(y0).abs() + f64::from(y1).abs() + 1.0)
                            * (1.0 + f64::from(position).abs() / 8.0);
                        assert_close(
                            f.value(query)?,
                            expected,
                            Tolerance::Mixed {
                                absolute,
                                relative: 0.0,
                            },
                        );
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 7 * 6 * 81 * 25);
    Ok(())
}

fn recovered_coefficients(
    f: &Interpolator,
    first: f64,
    width: f64,
) -> Result<[f64; 4], InterpolationError> {
    let d = f.value(first)?;
    let quarter = f.value(first + width / 4.0)? - d;
    let half = f.value(first + width / 2.0)? - d;
    let last = f.value(first + width)? - d;
    let a = (8.0 / 3.0) * (last - 6.0 * half + 8.0 * quarter);
    let b = 8.0 * half - 16.0 * quarter - 0.75 * a;
    Ok([a, b, last - a - b, d])
}

#[test]
fn monotone_hermite_preserves_small_segment_shapes_and_analytic_derivative_signs()
-> Result<(), Box<dyn Error>> {
    let xs = [0.0, 1.0, 3.0, 6.0];
    for encoded in 0..625 {
        let mut digits = encoded;
        let mut nodes = Vec::new();
        for x in xs {
            nodes.push((x, f64::from(digits % 5 - 2)));
            digits /= 5;
        }
        let f = make(&nodes, InterpolationMethod::MonotoneCubicHermite)?;
        for pair in nodes.windows(2) {
            let width = pair[1].0 - pair[0].0;
            let low = pair[0].1.min(pair[1].1);
            let high = pair[0].1.max(pair[1].1);
            for point in 0..=32 {
                let value = f.value(pair[0].0 + width * f64::from(point) / 32.0)?;
                assert!(
                    value >= low - 256.0 * U && value <= high + 256.0 * U,
                    "shape {nodes:?},q={point},v={value}"
                );
                if pair[0].1.to_bits() == pair[1].1.to_bits() {
                    assert_eq!(value.to_bits(), pair[0].1.to_bits());
                }
            }
            let [a, b, c, _] = recovered_coefficients(&f, pair[0].0, width)?;
            let mut positions = vec![0.0, 1.0];
            if a != 0.0 {
                let vertex = -b / (3.0 * a);
                if vertex > 0.0 && vertex < 1.0 {
                    positions.push(vertex);
                }
            }
            let direction = if pair[1].1 >= pair[0].1 { 1.0 } else { -1.0 };
            for t in positions {
                assert!(
                    direction * ((3.0 * a * t + 2.0 * b) * t + c) >= -1024.0 * U,
                    "derivative {nodes:?},t={t}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn cubic_rules_have_distinct_goldens_and_pinned_nonuniform_endpoint_slopes()
-> Result<(), Box<dyn Error>> {
    let nodes = [(0.0, 0.0), (1.0, 1.0), (2.0, 1.0)];
    let natural = make(&nodes, InterpolationMethod::NaturalCubic)?;
    assert_eq!(natural.value(0.5)?.to_bits(), 0.593_75_f64.to_bits());
    assert_eq!(natural.value(1.5)?.to_bits(), 1.093_75_f64.to_bits());
    let monotone = make(&nodes, InterpolationMethod::MonotoneCubicHermite)?;
    assert_eq!(monotone.value(0.5)?.to_bits(), 0.687_5_f64.to_bits());
    assert_eq!(monotone.value(1.5)?.to_bits(), 1.0_f64.to_bits());
    let f = make(
        &[(0.0, 0.0), (1.0, 1.0), (4.0, 2.0)],
        InterpolationMethod::MonotoneCubicHermite,
    )?;
    // Rational endpoint/interior constraints give305/528 and75/44;
    // these intentionally differ from QL's named FritschButlandCubic.
    assert_close(f.value(0.5)?, 305.0 / 528.0, Tolerance::Ulps(8));
    assert_close(f.value(2.5)?, 75.0 / 44.0, Tolerance::Ulps(8));
    let f = make(
        &[(0.0, 0.0), (1.0, 1.0), (2.0, -10.0)],
        InterpolationMethod::MonotoneCubicHermite,
    )?;
    // Global ordinate normalization includes the neighboring change of -11.
    // This is a composite golden, not an exact-knot identity: its stored
    // normalized coefficients can round before scale is restored.
    assert_close(f.value(0.5)?, 0.875, Tolerance::Ulps(8));
    Ok(())
}

#[test]
fn natural_spline_has_c2_junctions_and_zero_endpoint_curvature() -> Result<(), Box<dyn Error>> {
    let nodes = [(0.0, 0.0), (1.0, 1.0), (3.0, -1.0), (6.0, 2.0)];
    let f = make(&nodes, InterpolationMethod::NaturalCubic)?;
    let coefficients: Vec<_> = nodes
        .windows(2)
        .map(|p| recovered_coefficients(&f, p[0].0, p[1].0 - p[0].0))
        .collect::<Result<_, _>>()?;
    let tolerance = Tolerance::Mixed {
        absolute: 4096.0 * U,
        relative: 0.0,
    };
    assert_close(2.0 * coefficients[0][1], 0.0, tolerance);
    let last = coefficients[2];
    assert_close((6.0 * last[0] + 2.0 * last[1]) / 9.0, 0.0, tolerance);
    for i in 0..2 {
        let [a, b, c, _] = coefficients[i];
        let next = coefficients[i + 1];
        let h = nodes[i + 1].0 - nodes[i].0;
        let hn = nodes[i + 2].0 - nodes[i + 1].0;
        assert_close((3.0 * a + 2.0 * b + c) / h, next[2] / hn, tolerance);
        assert_close(
            (6.0 * a + 2.0 * b) / (h * h),
            2.0 * next[1] / (hn * hn),
            tolerance,
        );
    }
    Ok(())
}

#[test]
fn unsupported_cubic_grids_and_real_extrapolation_failures_remain_typed()
-> Result<(), Box<dyn Error>> {
    for method in [
        InterpolationMethod::NaturalCubic,
        InterpolationMethod::MonotoneCubicHermite,
    ] {
        for nodes in [
            vec![(0.0, 0.0), (MIN_SUB, 1.0), (1.0, 2.0)],
            vec![(-f64::MAX, -1.0), (f64::MAX / 2.0, 0.0), (f64::MAX, 1.0)],
        ] {
            let error = make(&nodes, method).unwrap_err();
            assert!(matches!(error, InterpolationError::NumericalGrid { .. }));
            assert_eq!(error.kind(), "interpolation.numerical_grid");
            assert!(error.source().is_some_and(|e| e.is::<ValueError>()));
            assert_eq!(error.clone(), error);
            assert!(!error.to_string().is_empty());
        }
    }
    let f = Interpolator::new(
        vec![(0.0, 1.0), (1.0, 2.0)],
        InterpolationMethod::LogLinear,
        Extrapolation::ExtendBoundarySegment,
    )?;
    for query in [-2000.0, 2000.0] {
        assert!(matches!(f.value(query), Err(InterpolationError::Value(_))));
    }
    let f = Interpolator::new(
        vec![(0.0, 0.0), (1.0, f64::MAX)],
        InterpolationMethod::Linear,
        Extrapolation::ExtendBoundarySegment,
    )?;
    assert!(matches!(f.value(2.0), Err(InterpolationError::Value(_))));
    Ok(())
}

#[test]
fn refinement_separates_linear_and_boundary_compatible_natural_accuracy()
-> Result<(), Box<dyn Error>> {
    let mut previous_linear = None;
    let mut previous_natural = None;
    for count in [8, 16, 32, 64] {
        let linear_nodes: Vec<_> = (0..=count)
            .map(|i| {
                let x = f64::from(i) / f64::from(count);
                (x, x * x)
            })
            .collect();
        let linear = make(&linear_nodes, InterpolationMethod::Linear)?;
        let natural_nodes: Vec<_> = linear_nodes
            .iter()
            .map(|&(x, _)| (x, (std::f64::consts::PI * x).sin()))
            .collect();
        let natural = make(&natural_nodes, InterpolationMethod::NaturalCubic)?;
        let mut linear_error: f64 = 0.0;
        let mut natural_error: f64 = 0.0;
        for i in 0..count {
            for j in [1, 2, 3] {
                let x = (f64::from(i) + f64::from(j) / 4.0) / f64::from(count);
                linear_error = linear_error.max((linear.value(x)? - x * x).abs());
                natural_error =
                    natural_error.max((natural.value(x)? - (std::f64::consts::PI * x).sin()).abs());
            }
        }
        if let Some(before) = previous_linear {
            assert_close(before / linear_error, 4.0, Tolerance::Ulps(0));
        }
        if let Some(before) = previous_natural {
            let ratio = before / natural_error;
            assert!(
                ratio > 12.0 && ratio < 20.0,
                "natural refinement{count},ratio={ratio}"
            );
        }
        previous_linear = Some(linear_error);
        previous_natural = Some(natural_error);
    }
    Ok(())
}

#[test]
fn immutable_clones_and_reversed_query_orders_preserve_definition_and_values()
-> Result<(), Box<dyn Error>> {
    let nodes = vec![(0.0, 1.0), (1.0, 3.0), (2.0, 2.0), (4.0, 5.0)];
    for method in InterpolationMethod::ALL {
        let f = make(&nodes, method)?;
        let copy = f.clone();
        assert_eq!(copy, f);
        assert_eq!(make(&nodes, method)?, f);
        assert_ne!(
            Interpolator::new(nodes.clone(), method, Extrapolation::Flat)?,
            f
        );
        let queries: Vec<_> = (0..=64).map(|i| f64::from(i) / 16.0).collect();
        let values: Vec<_> = queries
            .iter()
            .map(|&q| f.value(q))
            .collect::<Result<_, _>>()?;
        for (&q, &expected) in queries.iter().zip(&values).rev() {
            assert_close(
                copy.value(q)?,
                expected,
                if method == InterpolationMethod::LogLinear {
                    Tolerance::Mixed {
                        absolute: 0.0,
                        relative: 16.0 * U * (1.0 + expected.ln().abs()),
                    }
                } else {
                    Tolerance::Bitwise
                },
            );
        }
        assert_eq!(f.nodes(), nodes);
    }
    Ok(())
}

#[test]
fn concurrent_interpolator_reuse_has_no_query_order_or_clone_state() -> Result<(), Box<dyn Error>> {
    let nodes = vec![(0.0, 1.0), (1.0, 3.0), (3.0, 2.0), (6.0, 5.0)];
    let values: Vec<_> = InterpolationMethod::ALL
        .into_iter()
        .map(|method| make(&nodes, method))
        .collect::<Result<_, _>>()?;
    let baseline: Vec<Vec<_>> = values
        .iter()
        .map(|f| {
            (0..=128)
                .map(|i| f.value(f64::from(i) * 3.0 / 64.0))
                .collect::<Result<_, _>>()
        })
        .collect::<Result<_, _>>()?;
    let shared = Arc::new(values);
    let mut workers = Vec::new();
    for worker in 0..8 {
        let shared = Arc::clone(&shared);
        #[expect(
            clippy::disallowed_methods,
            reason = "test-only threads verify immutable interpolation query/clone order independence"
        )]
        let task = std::thread::spawn(
            move || -> Result<Vec<(usize, usize, f64)>, InterpolationError> {
                let mut result = Vec::new();
                for round in 0..4 {
                    for (method, original) in shared.iter().enumerate() {
                        let f = original.clone();
                        for offset in 0..=128 {
                            let index = if (worker + round) % 2 == 0 {
                                offset
                            } else {
                                128 - offset
                            };
                            result.push((
                                method,
                                usize::try_from(index).unwrap(),
                                f.value(f64::from(index) * 3.0 / 64.0)?,
                            ));
                        }
                    }
                }
                Ok(result)
            },
        );
        workers.push(task);
    }
    let mut observations = 0;
    for worker in workers {
        for (method, index, actual) in worker
            .join()
            .map_err(|_| "interpolation worker panicked")??
        {
            let expected = baseline[method][index];
            assert_close(
                actual,
                expected,
                if InterpolationMethod::ALL[method] == InterpolationMethod::LogLinear {
                    Tolerance::Mixed {
                        absolute: 0.0,
                        relative: 16.0 * U * (1.0 + 5.0_f64.ln()),
                    }
                } else {
                    Tolerance::Bitwise
                },
            );
            observations += 1;
        }
    }
    assert_eq!(observations, 8 * 4 * 4 * 129);
    Ok(())
}

#[test]
fn subnormal_values_have_explicit_rounding_unit_and_signed_zero_semantics()
-> Result<(), Box<dyn Error>> {
    let f = make(
        &[(0.0, 0.0), (4.0, 8.0 * MIN_SUB)],
        InterpolationMethod::Linear,
    )?;
    for (query, units) in [(0.5, 1.0), (1.0, 2.0), (2.0, 4.0), (3.0, 6.0)] {
        assert_eq!(f.value(query)?.to_bits(), (units * MIN_SUB).to_bits());
    }

    let halfway = make(&[(0.0, 0.0), (2.0, MIN_SUB)], InterpolationMethod::Linear)?;
    assert_eq!(halfway.value(1.0)?.to_bits(), 0);
    assert_eq!(
        halfway.value(1.0_f64.next_up())?.to_bits(),
        MIN_SUB.to_bits()
    );

    let tiny = make(&[(0.0, 0.0), (1.0, MIN_SUB)], InterpolationMethod::Linear)?;
    assert_eq!(tiny.value(MIN_SUB)?.to_bits(), 0);

    // The composite affine evaluation can differ by a subnormal rounding unit.
    // Its signed output permits zero; log-linear output has a stricter domain.
    let decreasing = make(&[(0.0, MIN_SUB), (2.0, 0.0)], InterpolationMethod::Linear)?;
    assert_close(
        decreasing.value(1.0)?,
        0.0,
        Tolerance::Mixed {
            absolute: MIN_SUB,
            relative: 0.0,
        },
    );
    Ok(())
}

#[test]
fn natural_curvature_scaling_preserves_dyadic_targets_after_gradual_underflow()
-> Result<(), Box<dyn Error>> {
    let h = binary_power(500);
    let y = binary_power(-74);
    let f = make(
        &[(0.0, 0.0), (h, 0.0), (2.0 * h, y)],
        InterpolationMethod::NaturalCubic,
    )?;
    // Solving the three-node natural constraints gives M1=3y/(2h²),
    // and the first midpoint is -3y/32. Physical M1 rounds before scaling.
    assert_eq!(f.value(h / 2.0)?.to_bits(), (-3.0 * y / 32.0).to_bits());
    assert_eq!(f.value(1.5 * h)?.to_bits(), (13.0 * y / 32.0).to_bits());
    Ok(())
}

#[test]
fn natural_uniform_and_nonuniform_scales_match_independent_three_node_constraints()
-> Result<(), Box<dyn Error>> {
    // With values [0,0,Y] and widths [aH,bH], the sole interior natural
    // curvature is 3Y/[b(a+b)H²]. The two midpoint values below follow from
    // integrating that linear curvature, independently of a matrix solver.
    for (a, b, first_numerator, second_numerator, denominator) in [
        (1.0, 1.0, -3.0, 13.0, 32.0),
        (1.0, 3.0, -1.0, 23.0, 64.0),
        (3.0, 1.0, -27.0, 29.0, 64.0),
    ] {
        for h_exponent in [-500, -200, 0, 200, 500] {
            let h = binary_power(h_exponent);
            for y_exponent in [-900, -74, 0, 74, 300] {
                let y = binary_power(y_exponent);
                for sign in [-1.0, 1.0] {
                    let nodes = [(0.0, 0.0), (a * h, 0.0), ((a + b) * h, sign * y)];
                    let f = make(&nodes, InterpolationMethod::NaturalCubic)?;
                    for (query, numerator) in [
                        (a * h * 0.5, first_numerator),
                        ((a + b * 0.5) * h, second_numerator),
                    ] {
                        assert_close(
                            f.value(query)?,
                            sign * numerator * y / denominator,
                            Tolerance::Ulps(16),
                        );
                    }
                }
            }
        }
    }

    for units in [16.0, 64.0, 128.0, 4096.0] {
        let y = units * MIN_SUB;
        for h_exponent in [-500, -200, 0] {
            let h = binary_power(h_exponent);
            for (a, b, first, second, denominator) in [
                (1.0, 1.0, -3.0, 13.0, 32.0),
                (1.0, 3.0, -1.0, 23.0, 64.0),
                (3.0, 1.0, -27.0, 29.0, 64.0),
            ] {
                for sign in [-1.0, 1.0] {
                    let f = make(
                        &[(0.0, 0.0), (a * h, 0.0), ((a + b) * h, sign * y)],
                        InterpolationMethod::NaturalCubic,
                    )?;
                    for (query, numerator) in [(a * h * 0.5, first), ((a + b * 0.5) * h, second)] {
                        assert_close(
                            f.value(query)?,
                            sign * numerator * y / denominator,
                            Tolerance::Mixed {
                                absolute: MIN_SUB,
                                relative: 0.0,
                            },
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
fn monotone_uniform_and_nonuniform_scales_preserve_independent_hermite_constraints()
-> Result<(), Box<dyn Error>> {
    // These are direct exact-rational Hermite midpoint values from the
    // documented weighted harmonic and clipped endpoint derivatives.
    // With [0,Y,2Y], the nonuniform [1,3] grid gives 305Y/528 and75Y/44;
    // its mirror gives13Y/44 and751Y/528. Physical derivatives may underflow
    // or overflow while all these requested values remain representable.
    for h_exponent in [-1070, -1000, -500, 0, 500, 1000] {
        let h = binary_power(h_exponent);
        for y_exponent in [-1000, -574, -74, 0, 74, 500, 1000] {
            let y = binary_power(y_exponent);
            for (a, b, first, second) in [
                (1.0, 1.0, 0.5, 1.5),
                (1.0, 3.0, 305.0 / 528.0, 75.0 / 44.0),
                (3.0, 1.0, 13.0 / 44.0, 751.0 / 528.0),
            ] {
                for sign in [-1.0, 1.0] {
                    let f = make(
                        &[(0.0, 0.0), (a * h, sign * y), ((a + b) * h, sign * 2.0 * y)],
                        InterpolationMethod::MonotoneCubicHermite,
                    )?;
                    for (q, expected) in [
                        (a * h * 0.5, sign * first * y),
                        ((a + b * 0.5) * h, sign * second * y),
                    ] {
                        assert_close(f.value(q)?, expected, Tolerance::Ulps(16));
                    }
                    let plateau = make(
                        &[(0.0, 0.0), (a * h, 0.0), ((a + b) * h, sign * y)],
                        InterpolationMethod::MonotoneCubicHermite,
                    )?;
                    assert_eq!(plateau.value(a * h * 0.5)?.to_bits(), 0);
                    let expected = sign * y * ((3.0 * a + 2.0 * b) / (8.0 * (a + b)));
                    assert_close(
                        plateau.value((a + b * 0.5) * h)?,
                        expected,
                        Tolerance::Ulps(16),
                    );
                }
            }
        }
    }
    for units in [16.0, 64.0, 128.0, 4096.0] {
        let y = units * MIN_SUB;
        for h_exponent in [-1070, -500, 0, 500, 1000] {
            let h = binary_power(h_exponent);
            for (a, b, first, second) in [
                (1.0, 1.0, 0.5, 1.5),
                (1.0, 3.0, 305.0 / 528.0, 75.0 / 44.0),
                (3.0, 1.0, 13.0 / 44.0, 751.0 / 528.0),
            ] {
                for sign in [-1.0, 1.0] {
                    let f = make(
                        &[(0.0, 0.0), (a * h, sign * y), ((a + b) * h, sign * 2.0 * y)],
                        InterpolationMethod::MonotoneCubicHermite,
                    )?;
                    for (q, expected) in [
                        (a * h * 0.5, sign * first * y),
                        ((a + b * 0.5) * h, sign * second * y),
                    ] {
                        assert_close(
                            f.value(q)?,
                            expected,
                            Tolerance::Mixed {
                                absolute: MIN_SUB,
                                relative: 0.0,
                            },
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

#[expect(
    clippy::expect_used,
    reason = "test-only exact power construction receives bounded literal exponents in -1074..1023"
)]
fn binary_power(exponent: i32) -> f64 {
    if exponent >= -1022 {
        f64::from_bits(u64::try_from(exponent + 1023).expect("normal biased exponent") << 52)
    } else {
        f64::from_bits(1_u64 << u32::try_from(exponent + 1074).expect("subnormal bit position"))
    }
}
