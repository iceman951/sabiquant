//! Conditioned PCHIP approximation studies on pinned analytic functions/grids.

use sabiquant::math::{Extrapolation, InterpolationMethod, Interpolator};
use std::error::Error;

const U: f64 = f64::EPSILON / 2.0;
const COUNTS: [i32; 5] = [16, 32, 64, 128, 256];

#[derive(Clone, Copy, Debug)]
enum Mesh {
    Uniform,
    Alternating,
    Graded,
}

#[derive(Clone, Copy, Debug)]
enum Function {
    IncreasingCubic,
    QuadraticPeak(f64),
}

impl Function {
    fn value(self, x: f64) -> f64 {
        match self {
            // f'=1+3x²>=1, f'''=6; no true derivative vanishes on [0,1].
            Self::IncreasingCubic => x * x * x + x,
            // f''=-2; the peak is smooth even when it falls between nodes.
            Self::QuadraticPeak(center) => 1.0 - (x - center) * (x - center),
        }
    }
}

#[derive(Debug)]
struct Study {
    count: i32,
    maximum_error: f64,
    maximum_width: f64,
    rounding_floor: f64,
    approximation_queries: usize,
    shape_queries: usize,
}

fn study(mesh: Mesh, function: Function, count: i32) -> Result<Study, Box<dyn Error>> {
    let n = f64::from(count);
    let nodes: Vec<_> = (0..=count)
        .map(|i| {
            let x = match mesh {
                Mesh::Uniform => f64::from(i) / n,
                // Repeating widths [1,3]/(2n); the ratio stays 3 as n grows.
                Mesh::Alternating => {
                    if i % 2 == 0 {
                        f64::from(i) / n
                    } else {
                        f64::from(i - 1) / n + 0.5 / n
                    }
                }
                // x_i=(i/n)²; H~2/n and the first interval shrinks as n^-2.
                Mesh::Graded => {
                    let t = f64::from(i) / n;
                    t * t
                }
            };
            (x, function.value(x))
        })
        .collect();
    let f = Interpolator::new(
        nodes.clone(),
        InterpolationMethod::MonotoneCubicHermite,
        Extrapolation::Reject,
    )?;
    let gross_scale = nodes.iter().map(|&(_, y)| y.abs()).fold(1.0, f64::max);
    let rounding_floor = 256.0 * U * gross_scale;
    let mut maximum_error = 0.0_f64;
    let mut maximum_width = 0.0_f64;
    let mut approximation_queries = 0;
    let mut shape_queries = 0;
    for &(x, y) in &nodes {
        assert_eq!(f.value(x)?.to_bits(), y.to_bits());
    }
    for pair in nodes.windows(2) {
        let [(x0, y0), (x1, y1)] = [pair[0], pair[1]];
        if matches!(function, Function::IncreasingCubic) {
            assert!(y1 > y0);
        }
        let h = x1 - x0;
        maximum_width = maximum_width.max(h);
        let low = y0.min(y1);
        let high = y0.max(y1);
        let mut previous = y0;
        for sample in 0..=16 {
            let query = x0 + h * f64::from(sample) / 16.0;
            let actual = f.value(query)?;
            assert!(actual >= low - rounding_floor && actual <= high + rounding_floor);
            if y1 >= y0 {
                assert!(actual >= previous - rounding_floor);
            } else {
                assert!(actual <= previous + rounding_floor);
            }
            previous = actual;
            shape_queries += 1;
        }
        // Quarter and three-quarter samples avoid midpoint-only cancellation
        // of the leading Hermite derivative error. Truth is the analytic
        // function, never a polynomial reconstructed from the interpolator.
        for fraction in [0.25, 0.5, 0.75] {
            let query = x0 + h * fraction;
            maximum_error = maximum_error.max((f.value(query)? - function.value(query)).abs());
            approximation_queries += 1;
        }
    }
    if let Function::QuadraticPeak(center) = function {
        let actual = f.value(center)?;
        maximum_error = maximum_error.max((actual - 1.0).abs());
        approximation_queries += 1;
        assert!(actual <= 1.0 + rounding_floor);
        let aligned = nodes.iter().any(|&(x, _)| x.to_bits() == center.to_bits());
        let expected_alignment =
            !matches!(mesh, Mesh::Graded) && center.to_bits() == 0.5_f64.to_bits();
        assert_eq!(aligned, expected_alignment);
        if aligned {
            assert_eq!(actual.to_bits(), 1.0_f64.to_bits());
        } else {
            let distance = nodes
                .iter()
                .map(|&(x, _)| (x - center).abs())
                .fold(f64::INFINITY, f64::min);
            let highest_node = nodes
                .iter()
                .map(|&(_, y)| y)
                .fold(f64::NEG_INFINITY, f64::max);
            assert!(actual <= highest_node + rounding_floor);
            // Shape preservation cannot reproduce a between-node quadratic
            // maximum higher than every node. This is approximation error.
            assert!(1.0 - actual >= distance * distance - rounding_floor);
        }
    }
    // This gross-scale rounding proxy is not a full algorithm-error proof.
    // All pinned studies stay far above it: the measured rate is approximation
    // behavior rather than a plateau at the basic-operation rounding scale.
    assert!(maximum_error > 1024.0 * rounding_floor);
    Ok(Study {
        count,
        maximum_error,
        maximum_width,
        rounding_floor,
        approximation_queries,
        shape_queries,
    })
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "reports separately conditioned empirical approximation refinement under --nocapture"
)]
fn smooth_strictly_monotone_cubic_refinement_is_pinned_per_mesh() -> Result<(), Box<dyn Error>> {
    for mesh in [Mesh::Uniform, Mesh::Alternating, Mesh::Graded] {
        let studies: Vec<_> = COUNTS
            .into_iter()
            .map(|n| study(mesh, Function::IncreasingCubic, n))
            .collect::<Result<_, _>>()?;
        println!("PCHIP smooth monotone {mesh:?}: {studies:?}");
        let ratios: Vec<_> = studies
            .windows(2)
            .map(|pair| pair[0].maximum_error / pair[1].maximum_error)
            .collect();
        let (power, lower, upper, ratio_lower, ratio_upper) = match mesh {
            Mesh::Uniform => (3, 0.30, 0.45, 7.5, 8.3),
            Mesh::Alternating => (2, 0.10, 0.20, 3.4, 4.3),
            Mesh::Graded => (3, 0.20, 0.30, 7.2, 8.4),
        };
        // The slope's Taylor error includes f''(h_next-h_previous)/6.
        // Abrupt 3:1 alternation leaves an O(H) derivative error, hence O(H²)
        // values; uniform/smoothly graded adjacent widths give O(H²) slope
        // error and these observed O(H³) values. These envelopes are pinned
        // to this function, mesh family and quarter-sample set only.
        for (i, measured) in studies.iter().enumerate() {
            assert_eq!(measured.count, COUNTS[i]);
            assert_eq!(
                measured.approximation_queries,
                usize::try_from(measured.count)? * 3
            );
            assert_eq!(
                measured.shape_queries,
                usize::try_from(measured.count)? * 17
            );
            let scale = measured.maximum_width.powi(power);
            assert!(measured.maximum_error > lower * scale - measured.rounding_floor);
            assert!(measured.maximum_error < upper * scale + measured.rounding_floor);
        }
        assert!(
            ratios
                .iter()
                .all(|&ratio| ratio > ratio_lower && ratio < ratio_upper)
        );
        println!("PCHIP smooth monotone {mesh:?} halving ratios: {ratios:?}");
    }
    Ok(())
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "reports aligned and unaligned extrema separately without a universal convergence order"
)]
fn smooth_extrema_refinement_is_conditioned_by_mesh_alignment() -> Result<(), Box<dyn Error>> {
    for mesh in [Mesh::Uniform, Mesh::Alternating, Mesh::Graded] {
        for center in [0.5, 0.3] {
            let studies: Vec<_> = COUNTS
                .into_iter()
                .map(|n| study(mesh, Function::QuadraticPeak(center), n))
                .collect::<Result<_, _>>()?;
            println!("PCHIP smooth extremum {mesh:?} center={center}: {studies:?}");
            let ratios: Vec<_> = studies
                .windows(2)
                .map(|pair| pair[0].maximum_error / pair[1].maximum_error)
                .collect();
            for (i, measured) in studies.iter().enumerate() {
                assert_eq!(measured.count, COUNTS[i]);
                assert_eq!(
                    measured.approximation_queries,
                    usize::try_from(measured.count)? * 3 + 1
                );
                assert_eq!(
                    measured.shape_queries,
                    usize::try_from(measured.count)? * 17
                );
                // For f''=-2, a same-sign harmonic node slope lies between
                // adjacent secants, each within H of f' at that node. A
                // zero/sign/3-secant endpoint limitation also differs by at
                // most H here. Exact quadratic Hermite interpolation then
                // bounds value error by h*H*(|h10|+|h11|)<=H²/4.
                assert!(
                    measured.maximum_error
                        <= measured.maximum_width * measured.maximum_width / 4.0
                            + measured.rounding_floor
                );
            }
            assert!(
                studies
                    .windows(2)
                    .all(|pair| pair[1].maximum_error < pair[0].maximum_error)
            );
            if !matches!(mesh, Mesh::Graded) && center.to_bits() == 0.5_f64.to_bits() {
                assert!(ratios.iter().all(|&ratio| ratio > 3.99 && ratio < 4.01));
            }
            println!("PCHIP smooth extremum {mesh:?} center={center} halving ratios: {ratios:?}");
        }
    }
    Ok(())
}
