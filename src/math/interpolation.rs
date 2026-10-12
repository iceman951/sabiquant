//! Immutable one-dimensional interpolation with explicitly selected policies.
//!
//! Mathematics and numerical support are recorded in the
//! [local contract](https://github.com/iceman951/sabiquant/blob/main/docs/M6_INTERPOLATION_RESEARCH.md).
//! Linear follows [NIST DLMF 3.3.14](https://dlmf.nist.gov/3.3.E14).
//! Monotone Hermite uses the weighted PCHIP rules documented by
//! [SciPy](https://docs.scipy.org/doc/scipy/reference/generated/scipy.interpolate.PchipInterpolator.html)
//! and [Moler's interpolation chapter](https://www.mathworks.com/content/dam/mathworks/mathworks-dot-com/moler/interp.pdf).
//! No reference-library implementation is copied or translated.

use crate::error::{IdentifierError, Interval, ValueError};
use crate::identifier::{Ambiguous, Entry, Identifiers};
use crate::numeric;
use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;

/// The explicitly chosen mathematical interpolation rule.
///
/// Parsing accepts exact variant identifiers without aliases. The bare label
/// `Cubic` is ambiguous and reports its natural/monotone candidates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InterpolationMethod {
    /// Piecewise affine combinations of signed node values.
    #[doc(alias = "LinearInterpolation")]
    Linear,
    /// Piecewise affine logarithms of strictly positive node values.
    #[doc(alias = "LogLinearInterpolation")]
    LogLinear,
    /// C2 cubic spline with zero endpoint second derivatives; overshoot is allowed.
    #[doc(alias = "CubicNaturalSpline")]
    NaturalCubic,
    /// C1 weighted-harmonic PCHIP with one-sided, shape-limited endpoint slopes.
    ///
    /// This is neither a monotonic natural spline nor a promise to reproduce
    /// QuantLib's FritschButlandCubic on nonuniform intervals.
    #[doc(alias = "PCHIP")]
    MonotoneCubicHermite,
}

impl InterpolationMethod {
    /// Every currently supported method.
    pub const ALL: [Self; 4] = [
        Self::Linear,
        Self::LogLinear,
        Self::NaturalCubic,
        Self::MonotoneCubicHermite,
    ];

    /// Returns the exact, case-sensitive method identifier.
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::LogLinear => "LogLinear",
            Self::NaturalCubic => "NaturalCubic",
            Self::MonotoneCubicHermite => "MonotoneCubicHermite",
        }
    }
}

impl fmt::Display for InterpolationMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_identifier())
    }
}
impl FromStr for InterpolationMethod {
    type Err = IdentifierError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        METHOD_IDENTIFIERS.parse(text)
    }
}

/// The explicit behavior for finite queries outside the closed node domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Extrapolation {
    /// Reject an out-of-domain query.
    Reject,
    /// Return the corresponding stored boundary value.
    Flat,
    /// Extend the first or last segment's own formula.
    ///
    /// Cubic shape/positivity guarantees do not extend beyond its interval.
    ExtendBoundarySegment,
}

impl Extrapolation {
    /// Every currently supported policy.
    pub const ALL: [Self; 3] = [Self::Reject, Self::Flat, Self::ExtendBoundarySegment];
    /// Returns the exact, case-sensitive policy identifier.
    #[must_use]
    pub const fn canonical_identifier(self) -> &'static str {
        match self {
            Self::Reject => "Reject",
            Self::Flat => "Flat",
            Self::ExtendBoundarySegment => "ExtendBoundarySegment",
        }
    }
}
impl fmt::Display for Extrapolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_identifier())
    }
}
impl FromStr for Extrapolation {
    type Err = IdentifierError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        EXTRAPOLATION_IDENTIFIERS.parse(text)
    }
}

/// An immutable one-dimensional interpolant over explicitly ordered finite nodes.
///
/// No units, node order or extrapolation policy are inferred. Signed zeros
/// are canonicalized; duplicate abscissas are rejected. Exact knots return
/// their stored values, including for logarithmic interpolation. Equality
/// compares input nodes, method and policy, not prepared floating coefficients.
/// There is no floating ordering, hashing, default or mutable lookup cache.
///
/// Cubic preparation requires representable positive widths and ordinate
/// differences, and finite required dimensionless coefficients/solve pivots.
/// Required nonzero normalized quantities must retain normal relative precision;
/// a physical secant or curvature need not itself be representable.
/// Unsupported numerical grids are typed failures even when some mathematical
/// answers would be finite. Two-node and constant cubic data use the linear
/// rule without requiring those unused intermediates. Linear/log-linear need
/// no cached slope and support overflowing raw interval widths by scaling.
///
/// ```
/// use sabiquant::math::{Extrapolation, InterpolationMethod, Interpolator};
/// let f = Interpolator::new(vec![(0.0, 1.0), (2.0, 3.0)],
///                          InterpolationMethod::Linear, Extrapolation::Reject)?;
/// assert_eq!(f.value(1.0)?.to_bits(), 2.0_f64.to_bits());
/// assert_eq!(f.value(2.0)?.to_bits(), 3.0_f64.to_bits());
/// assert!(f.value(3.0).is_err());
/// # Ok::<(), sabiquant::math::InterpolationError>(())
/// ```
#[derive(Clone, Debug)]
pub struct Interpolator {
    nodes: Vec<(f64, f64)>,
    method: InterpolationMethod,
    extrapolation: Extrapolation,
    prepared: Prepared,
}

#[derive(Clone, Debug)]
enum Prepared {
    Linear,
    Logarithms(Vec<f64>),
    Cubic {
        coefficients: Vec<[f64; 4]>,
        ordinate_scale: f64,
    },
}

impl PartialEq for Interpolator {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
            && self.method == other.method
            && self.extrapolation == other.extrapolation
    }
}

impl Interpolator {
    /// Constructs a value without sorting or deduplicating its nodes.
    ///
    /// Preparation takes O(n) time and storage. All methods require at least
    /// two nodes. LogLinear also requires strictly positive ordinates.
    ///
    /// # Errors
    /// [`InterpolationError::NotEnoughNodes`], [`InterpolationError::InvalidNode`],
    /// [`InterpolationError::NotStrictlyIncreasing`] or a cubic/logarithm
    /// [`InterpolationError::NumericalGrid`] support failure.
    #[expect(
        clippy::float_cmp,
        reason = "constant input values are an exact identity path, not an approximate numerical comparison"
    )]
    pub fn new(
        mut nodes: Vec<(f64, f64)>,
        method: InterpolationMethod,
        extrapolation: Extrapolation,
    ) -> Result<Self, InterpolationError> {
        if nodes.len() < 2 {
            return Err(InterpolationError::NotEnoughNodes {
                actual: nodes.len(),
                minimum: 2,
            });
        }
        let mut previous = None;
        for (index, (x, y)) in nodes.iter_mut().enumerate() {
            *x = zero(
                numeric::finite("node abscissa", *x)
                    .map_err(|cause| InterpolationError::InvalidNode { index, cause })?,
            );
            *y = zero(
                if method == InterpolationMethod::LogLinear {
                    numeric::within("node ordinate", *y, Interval::greater_than(0.0))
                } else {
                    numeric::finite("node ordinate", *y)
                }
                .map_err(|cause| InterpolationError::InvalidNode { index, cause })?,
            );
            if let Some(before) = previous
                && before >= *x
            {
                return Err(InterpolationError::NotStrictlyIncreasing {
                    index,
                    previous: before,
                    current: *x,
                });
            }
            previous = Some(*x);
        }
        let constant = nodes.iter().all(|&(_, y)| y == nodes[0].1);
        let prepared = match method {
            InterpolationMethod::Linear => Prepared::Linear,
            InterpolationMethod::LogLinear => Prepared::Logarithms(
                nodes
                    .iter()
                    .enumerate()
                    .map(|(index, &(_, y))| grid(index, "node logarithm", y.ln()))
                    .collect::<Result<_, _>>()?,
            ),
            _ if nodes.len() == 2 || constant => Prepared::Linear,
            InterpolationMethod::NaturalCubic => prepare_natural(&nodes)?,
            InterpolationMethod::MonotoneCubicHermite => prepare_monotone(&nodes)?,
        };
        Ok(Self {
            nodes,
            method,
            extrapolation,
            prepared,
        })
    }

    /// Returns the canonical ordered node pairs without permitting mutation.
    #[must_use]
    pub fn nodes(&self) -> &[(f64, f64)] {
        &self.nodes
    }
    /// Returns the closed abscissa domain.
    #[must_use]
    pub fn domain(&self) -> (f64, f64) {
        (self.nodes[0].0, self.nodes[self.nodes.len() - 1].0)
    }
    /// Returns the explicitly chosen method.
    #[must_use]
    pub const fn method(&self) -> InterpolationMethod {
        self.method
    }
    /// Returns the explicitly chosen extrapolation policy.
    #[must_use]
    pub const fn extrapolation(&self) -> Extrapolation {
        self.extrapolation
    }

    /// Evaluates one finite query with O(log n) lookup and O(1) arithmetic.
    ///
    /// Exact knots, constant linear/log-linear segments, all-constant data
    /// and Flat boundaries are exact identity paths. Equal adjacent ordinates
    /// do not make a natural cubic segment constant: overshoot is allowed.
    /// Other values use conditioned composite/elementary/algorithm
    /// accuracy; zero is valid for signed methods. LogLinear rejects a result
    /// rounded to zero. Far extrapolation can exceed numerical support.
    ///
    /// # Errors
    /// [`InterpolationError::Value`] for invalid query or unrepresentable
    /// output, [`InterpolationError::OutsideDomain`] under Reject, or
    /// [`InterpolationError::NumericalGrid`] for an unrepresentable required
    /// coordinate or affine intermediate; that does not prove mathematical
    /// output overflow.
    #[expect(
        clippy::float_cmp,
        reason = "stored-knot and constant-segment identities use exact canonical input equality"
    )]
    pub fn value(&self, query: f64) -> Result<f64, InterpolationError> {
        let query = zero(numeric::finite("interpolation query", query)?);
        let (lower, upper) = self.domain();
        if query < lower || query > upper {
            match self.extrapolation {
                Extrapolation::Reject => {
                    return Err(InterpolationError::OutsideDomain {
                        query,
                        lower,
                        upper,
                    });
                }
                Extrapolation::Flat => {
                    return Ok(if query < lower {
                        self.nodes[0].1
                    } else {
                        self.nodes[self.nodes.len() - 1].1
                    });
                }
                Extrapolation::ExtendBoundarySegment => {}
            }
        }
        let index = match self.nodes.binary_search_by(|node| node.0.total_cmp(&query)) {
            Ok(index) => return Ok(self.nodes[index].1),
            Err(index) => index.saturating_sub(1).min(self.nodes.len() - 2),
        };
        let (x0, y0) = self.nodes[index];
        let (x1, y1) = self.nodes[index + 1];
        if y0 == y1 && !matches!(self.prepared, Prepared::Cubic { .. }) {
            return Ok(y0);
        }
        let value = match &self.prepared {
            Prepared::Linear => linear_value(index, query, x0, x1, y0, y1)?,
            Prepared::Logarithms(logs) => {
                let t = coordinate(index, query, x0, x1)?;
                blend(t, logs[index], logs[index + 1]).exp()
            }
            Prepared::Cubic {
                coefficients,
                ordinate_scale,
            } => {
                let [a, b, c, d] = coefficients[index];
                if a == 0.0 && b == 0.0 {
                    linear_value(index, query, x0, x1, y0, y1)?
                } else {
                    let t = coordinate(index, query, x0, x1)?;
                    let polynomial = a.mul_add(t, b).mul_add(t, c);
                    if *ordinate_scale == 1.0 {
                        polynomial.mul_add(t, d)
                    } else {
                        ordinate_scale.mul_add(polynomial * t, d)
                    }
                }
            }
        };
        let value = if self.method == InterpolationMethod::LogLinear {
            numeric::result_within("interpolated value", value, Interval::greater_than(0.0))?
        } else {
            numeric::finite_result("interpolated value", value)?
        };
        Ok(zero(value))
    }
}

fn zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}
fn grid(index: usize, quantity: &'static str, value: f64) -> Result<f64, InterpolationError> {
    numeric::finite_result(quantity, value).map_err(|cause| InterpolationError::NumericalGrid {
        interval: index,
        quantity,
        cause,
    })
}
fn positive_grid(
    index: usize,
    quantity: &'static str,
    value: f64,
) -> Result<f64, InterpolationError> {
    numeric::result_within(quantity, value, Interval::greater_than(0.0)).map_err(|cause| {
        InterpolationError::NumericalGrid {
            interval: index,
            quantity,
            cause,
        }
    })
}

// Dimensionless required quantities must retain normal relative precision.
// Final signed outputs may still be subnormal; scale is restored at evaluation.
fn normal_grid(
    index: usize,
    quantity: &'static str,
    value: f64,
) -> Result<f64, InterpolationError> {
    let value = grid(index, quantity, value)?;
    if value.is_subnormal() {
        numeric::result_within(quantity, value.abs(), Interval::at_least(f64::MIN_POSITIVE))
            .map_err(|cause| InterpolationError::NumericalGrid {
                interval: index,
                quantity,
                cause,
            })?;
    }
    Ok(value)
}

fn coordinate(index: usize, query: f64, x0: f64, x1: f64) -> Result<f64, InterpolationError> {
    let width = x1 - x0;
    let distance = query - x0;
    let value = if width.is_finite() && distance.is_finite() {
        distance / width
    } else {
        (query * 0.5 - x0 * 0.5) / (x1 * 0.5 - x0 * 0.5)
    };
    grid(index, "relative interpolation coordinate", value)
}

fn blend(t: f64, left: f64, right: f64) -> f64 {
    if left.is_sign_positive() != right.is_sign_positive() {
        (1.0 - t).mul_add(left, t * right)
    } else if t <= 0.5 {
        t.mul_add(right - left, left)
    } else {
        (t - 1.0).mul_add(right - left, right)
    }
}

fn linear_value(
    index: usize,
    query: f64,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
) -> Result<f64, InterpolationError> {
    let value = linear_unscaled(index, query, x0, x1, y0, y1);
    if value.as_ref().is_ok_and(|value| value.is_finite()) {
        return value;
    }
    // A contribution can exceed MAX before cancellation with a large base
    // ordinate, even though the final affine answer is finite. Rescale the
    // ordinates once; the largest normalized magnitude is one. No clipping
    // or inference about a mathematically overflowing answer is performed.
    let scale = y0.abs().max(y1.abs());
    if scale > 1.0 {
        return linear_unscaled(index, query, x0, x1, y0 / scale, y1 / scale)
            .map(|value| value * scale);
    }
    value
}

fn linear_unscaled(
    index: usize,
    query: f64,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
) -> Result<f64, InterpolationError> {
    let width = x1 - x0;
    let change = y1 - y0;
    if !width.is_finite()
        || !change.is_finite()
        || !(query - x0).is_finite()
        || !(x1 - query).is_finite()
    {
        let middle_y = y0 * 0.5 + y1 * 0.5;
        let half_width = x1 * 0.5 - x0 * 0.5;
        let half_change = y1 * 0.5 - y0 * 0.5;
        if half_width <= 0.0 {
            return positive_grid(index, "scaled interval width", half_width);
        }
        let distance = query - (x0 * 0.5 + x1 * 0.5);
        let contribution = if distance.is_finite() {
            multiply_divide(distance, half_change, half_width)
        } else {
            let half_distance = query * 0.5 - (x0 * 0.25 + x1 * 0.25);
            multiply_divide(half_distance, half_change, half_width) * 2.0
        };
        return grid(index, "centered affine contribution", contribution)
            .map(|value| middle_y + value);
    }
    let t = coordinate(index, query, x0, x1)?;
    Ok(if t <= 0.5 {
        y0 + multiply_divide(query - x0, change, width)
    } else {
        y1 - multiply_divide(x1 - query, change, width)
    })
}

// Independent binary exponent scaling keeps a*b/c from overflowing or losing
// a useful contribution before the final multiplication. Each input is finite;
// c is a validated positive interval width.
#[expect(
    clippy::float_cmp,
    reason = "identical finite factors cancel exactly before exponent scaling, preserving subnormal affine identities"
)]
fn multiply_divide(a: f64, b: f64, c: f64) -> f64 {
    if a == 0.0 || b == 0.0 {
        return 0.0;
    }
    if b.abs() == c {
        return if b.is_sign_positive() { a } else { -a };
    }
    if a.abs() == c {
        return if a.is_sign_positive() { b } else { -b };
    }
    let (am, ae) = decompose(a.abs());
    let (bm, be) = decompose(b.abs());
    let (cm, ce) = decompose(c);
    let mut mantissa = (am * bm) / cm;
    let mut exponent = ae + be - ce;
    if mantissa >= 2.0 {
        mantissa *= 0.5;
        exponent += 1;
    } else if mantissa < 1.0 {
        mantissa *= 2.0;
        exponent -= 1;
    }
    let value = power_scaled(mantissa, exponent);
    if a.is_sign_positive() == b.is_sign_positive() {
        value
    } else {
        -value
    }
}

fn decompose(value: f64) -> (f64, i32) {
    let (bits, adjustment) = if value.is_subnormal() {
        ((value * 4_503_599_627_370_496.0).to_bits(), -52)
    } else {
        (value.to_bits(), 0)
    };
    let exponent = ((bits >> 52) & 0x7ff) as i32 - 1023 + adjustment;
    (
        f64::from_bits((bits & 0x000f_ffff_ffff_ffff) | (1023_u64 << 52)),
        exponent,
    )
}

#[expect(
    clippy::cast_sign_loss,
    reason = "the guarded biased exponent or subnormal bit position is nonnegative"
)]
fn power_scaled(mantissa: f64, exponent: i32) -> f64 {
    if exponent > 1023 {
        f64::INFINITY
    } else if exponent >= -1022 {
        mantissa * f64::from_bits(((exponent + 1023) as u64) << 52)
    } else if exponent >= -1074 {
        mantissa * f64::from_bits(1_u64 << (exponent + 1074) as u32)
    } else if exponent == -1075 {
        (mantissa * 0.5) * f64::from_bits(1)
    } else {
        0.0
    }
}

struct NormalizedIntervals {
    widths: Vec<f64>,
    slopes: Vec<f64>,
    changes: Vec<f64>,
    ordinate_scale: f64,
}

fn normalized_intervals(nodes: &[(f64, f64)]) -> Result<NormalizedIntervals, InterpolationError> {
    let mut raw_widths = Vec::with_capacity(nodes.len() - 1);
    let mut raw_changes = Vec::with_capacity(nodes.len() - 1);
    for (index, pair) in nodes.windows(2).enumerate() {
        let width = positive_grid(index, "interval width", pair[1].0 - pair[0].0)?;
        let difference = grid(index, "ordinate difference", pair[1].1 - pair[0].1)?;
        raw_widths.push(width);
        raw_changes.push(difference);
    }
    let width_scale = raw_widths.iter().copied().fold(0.0, f64::max);
    let ordinate_scale = raw_changes
        .iter()
        .map(|change| change.abs())
        .fold(0.0, f64::max);
    let mut h = Vec::with_capacity(raw_widths.len());
    let mut changes = Vec::with_capacity(raw_widths.len());
    let mut delta = Vec::with_capacity(raw_widths.len());
    for (i, (&raw_width, &raw_change)) in raw_widths.iter().zip(&raw_changes).enumerate() {
        let width = positive_grid(
            i,
            "normalized interval width",
            normal_grid(i, "normalized interval width", raw_width / width_scale)?,
        )?;
        let change = normal_grid(
            i,
            "normalized ordinate difference",
            raw_change / ordinate_scale,
        )?;
        if raw_change != 0.0 && change == 0.0 {
            positive_grid(i, "normalized ordinate-difference magnitude", change.abs())?;
        }
        let slope = normal_grid(i, "normalized secant", change / width)?;
        if change != 0.0 && slope == 0.0 {
            positive_grid(i, "normalized secant magnitude", slope.abs())?;
        }
        h.push(width);
        changes.push(change);
        delta.push(slope);
    }
    Ok(NormalizedIntervals {
        widths: h,
        slopes: delta,
        changes,
        ordinate_scale,
    })
}

fn prepare_natural(nodes: &[(f64, f64)]) -> Result<Prepared, InterpolationError> {
    let NormalizedIntervals {
        widths: h,
        slopes: delta,
        changes,
        ordinate_scale,
    } = normalized_intervals(nodes)?;
    // Solve dimensionless K=M*H²/S, not physical second derivatives M.
    // A subnormal physical M may round severely before a large h² restores
    // an otherwise ordinary ordinate contribution. H=max h and S=max |dy|.
    let n = nodes.len();
    let mut upper = vec![0.0; n];
    let mut rhs = vec![0.0; n];
    for i in 1..n - 1 {
        let scale = h[i - 1].max(h[i]);
        let left = positive_grid(i, "scaled previous interval", h[i - 1] / scale)?;
        let right = positive_grid(i, "scaled next interval", h[i] / scale)?;
        let pivot = positive_grid(
            i,
            "scaled spline pivot",
            2.0 * (left + right) - left * upper[i - 1],
        )?;
        let difference = grid(i, "secant difference", delta[i] - delta[i - 1])?;
        let value = normal_grid(i, "spline right-hand side", 6.0 * (difference / scale))?;
        if difference != 0.0 && value == 0.0 {
            positive_grid(i, "spline right-hand-side magnitude", value.abs())?;
        }
        upper[i] = grid(i, "scaled spline upper coefficient", right / pivot)?;
        rhs[i] = normal_grid(
            i,
            "spline forward coefficient",
            (value - left * rhs[i - 1]) / pivot,
        )?;
    }
    let mut curvature = vec![0.0; n];
    for i in (1..n - 1).rev() {
        curvature[i] = normal_grid(
            i,
            "spline curvature",
            (-upper[i]).mul_add(curvature[i + 1], rhs[i]),
        )?;
    }
    let mut coefficients = Vec::with_capacity(n - 1);
    for i in 0..n - 1 {
        let first = normal_grid(
            i,
            "scaled left curvature",
            multiply_divide(curvature[i], h[i], 6.0) * h[i],
        )?;
        let last = normal_grid(
            i,
            "scaled right curvature",
            multiply_divide(curvature[i + 1], h[i], 6.0) * h[i],
        )?;
        if curvature[i] != 0.0 && first == 0.0 {
            positive_grid(i, "scaled left-curvature magnitude", first.abs())?;
        }
        if curvature[i + 1] != 0.0 && last == 0.0 {
            positive_grid(i, "scaled right-curvature magnitude", last.abs())?;
        }
        let change = changes[i];
        let a = normal_grid(i, "normalized cubic coefficient", last - first)?;
        let b = normal_grid(i, "normalized cubic coefficient", 3.0 * first)?;
        let c = normal_grid(
            i,
            "normalized cubic coefficient",
            change - 2.0 * first - last,
        )?;
        coefficients.push([a, b, c, nodes[i].1]);
    }
    Ok(Prepared::Cubic {
        coefficients,
        ordinate_scale,
    })
}

fn prepare_monotone(nodes: &[(f64, f64)]) -> Result<Prepared, InterpolationError> {
    let NormalizedIntervals {
        widths: h,
        slopes: delta,
        changes,
        ordinate_scale,
    } = normalized_intervals(nodes)?;
    let n = nodes.len();
    let mut slopes = vec![0.0; n];
    for i in 1..n - 1 {
        let previous = delta[i - 1];
        let next = delta[i];
        if previous == 0.0 || next == 0.0 || previous.is_sign_positive() != next.is_sign_positive()
        {
            continue;
        }
        let scale = h[i - 1].max(h[i]);
        let hp = positive_grid(i, "scaled previous interval", h[i - 1] / scale)?;
        let hn = positive_grid(i, "scaled next interval", h[i] / scale)?;
        let w1 = 2.0 * hn + hp;
        let w2 = hn + 2.0 * hp;
        let small = previous.abs().min(next.abs());
        let magnitude = normal_grid(
            i,
            "harmonic slope magnitude",
            small * ((w1 + w2) / (w1 * (small / previous.abs()) + w2 * (small / next.abs()))),
        )?;
        slopes[i] = if previous.is_sign_positive() {
            magnitude
        } else {
            -magnitude
        };
    }
    slopes[0] = endpoint_slope(0, h[0], h[1], delta[0], delta[1])?;
    slopes[n - 1] = endpoint_slope(n - 2, h[n - 2], h[n - 3], delta[n - 2], delta[n - 3])?;
    let mut coefficients = Vec::with_capacity(n - 1);
    for i in 0..n - 1 {
        let left = normal_grid(i, "scaled left slope", h[i] * slopes[i])?;
        let right = normal_grid(i, "scaled right slope", h[i] * slopes[i + 1])?;
        if slopes[i] != 0.0 && left == 0.0 {
            positive_grid(i, "scaled left-slope magnitude", left.abs())?;
        }
        if slopes[i + 1] != 0.0 && right == 0.0 {
            positive_grid(i, "scaled right-slope magnitude", right.abs())?;
        }
        let change = changes[i];
        coefficients.push(checked_coefficients(
            i,
            [
                left + right - 2.0 * change,
                3.0 * change - 2.0 * left - right,
                left,
                nodes[i].1,
            ],
        )?);
    }
    Ok(Prepared::Cubic {
        coefficients,
        ordinate_scale,
    })
}

#[expect(
    clippy::float_cmp,
    reason = "equal secants and zero/sign shape limits are exact branches of the pinned endpoint rule"
)]
fn endpoint_slope(
    index: usize,
    first_width: f64,
    next_width: f64,
    first: f64,
    next: f64,
) -> Result<f64, InterpolationError> {
    if first == 0.0 {
        return Ok(0.0);
    }
    if first == next {
        return Ok(first);
    }
    let scale = first_width.max(next_width);
    let weight = (first_width / scale) / (first_width / scale + next_width / scale);
    let candidate = normal_grid(
        index,
        "endpoint slope",
        first.mul_add(1.0 + weight, -weight * next),
    )?;
    if candidate == 0.0 || candidate.is_sign_positive() != first.is_sign_positive() {
        return Ok(0.0);
    }
    if first.is_sign_positive() != next.is_sign_positive() && candidate.abs() / first.abs() > 3.0 {
        return grid(index, "limited endpoint slope", 3.0 * first);
    }
    Ok(candidate)
}

fn checked_coefficients(index: usize, values: [f64; 4]) -> Result<[f64; 4], InterpolationError> {
    for value in &values[..3] {
        normal_grid(index, "normalized cubic coefficient", *value)?;
    }
    Ok(values)
}

/// A rejected interpolation definition, query or numerical support condition.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum InterpolationError {
    /// Too few node pairs; kind `interpolation.not_enough_nodes`.
    #[non_exhaustive]
    NotEnoughNodes {
        /// Supplied count.
        actual: usize,
        /// Required count.
        minimum: usize,
    },
    /// An input node failed numeric/domain validation; kind `interpolation.invalid_node`.
    #[non_exhaustive]
    InvalidNode {
        /// Rejected node index.
        index: usize,
        /// Numeric cause with component name and value.
        cause: ValueError,
    },
    /// Adjacent abscissas are duplicate or reversed; kind `interpolation.not_strictly_increasing`.
    #[non_exhaustive]
    NotStrictlyIncreasing {
        /// Rejected node index.
        index: usize,
        /// Previous finite abscissa.
        previous: f64,
        /// Current finite abscissa.
        current: f64,
    },
    /// Reject policy excludes this query; kind `interpolation.outside_domain`.
    #[non_exhaustive]
    OutsideDomain {
        /// Finite query.
        query: f64,
        /// Closed lower boundary.
        lower: f64,
        /// Closed upper boundary.
        upper: f64,
    },
    /// Required intermediate arithmetic is unsupported; kind `interpolation.numerical_grid`.
    #[non_exhaustive]
    NumericalGrid {
        /// Interval or node index involved.
        interval: usize,
        /// Required intermediate quantity.
        quantity: &'static str,
        /// Typed numeric cause; not proof of mathematical output overflow.
        cause: ValueError,
    },
    /// Invalid query or computed result; kind `interpolation.value`.
    Value(ValueError),
}

impl InterpolationError {
    /// Returns the stable `interpolation.*` kind.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::NotEnoughNodes { .. } => "interpolation.not_enough_nodes",
            Self::InvalidNode { .. } => "interpolation.invalid_node",
            Self::NotStrictlyIncreasing { .. } => "interpolation.not_strictly_increasing",
            Self::OutsideDomain { .. } => "interpolation.outside_domain",
            Self::NumericalGrid { .. } => "interpolation.numerical_grid",
            Self::Value(_) => "interpolation.value",
        }
    }
}
impl fmt::Display for InterpolationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotEnoughNodes { actual, minimum } => write!(
                f,
                "interpolation needs at least {minimum} nodes, received {actual}"
            ),
            Self::InvalidNode { index, cause } => {
                write!(f, "invalid interpolation node {index}: {cause}")
            }
            Self::NotStrictlyIncreasing {
                index,
                previous,
                current,
            } => write!(
                f,
                "interpolation node {index} is not strictly after {previous}: {current}"
            ),
            Self::OutsideDomain {
                query,
                lower,
                upper,
            } => write!(
                f,
                "interpolation query {query} is outside [{lower}, {upper}]"
            ),
            Self::NumericalGrid {
                interval,
                quantity,
                cause,
            } => write!(
                f,
                "interpolation interval {interval} cannot represent {quantity}: {cause}"
            ),
            Self::Value(cause) => cause.fmt(f),
        }
    }
}
impl StdError for InterpolationError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::InvalidNode { cause, .. }
            | Self::NumericalGrid { cause, .. }
            | Self::Value(cause) => Some(cause),
            _ => None,
        }
    }
}
impl From<ValueError> for InterpolationError {
    fn from(cause: ValueError) -> Self {
        Self::Value(cause)
    }
}

const METHOD_IDENTIFIERS: Identifiers<InterpolationMethod> = Identifiers::new(
    "interpolation method",
    &[
        Entry {
            value: InterpolationMethod::Linear,
            canonical: "Linear",
            aliases: &[],
        },
        Entry {
            value: InterpolationMethod::LogLinear,
            canonical: "LogLinear",
            aliases: &[],
        },
        Entry {
            value: InterpolationMethod::NaturalCubic,
            canonical: "NaturalCubic",
            aliases: &[],
        },
        Entry {
            value: InterpolationMethod::MonotoneCubicHermite,
            canonical: "MonotoneCubicHermite",
            aliases: &[],
        },
    ],
    &[Ambiguous {
        label: "Cubic",
        candidates: &["NaturalCubic", "MonotoneCubicHermite"],
    }],
);
const EXTRAPOLATION_IDENTIFIERS: Identifiers<Extrapolation> = Identifiers::new(
    "extrapolation policy",
    &[
        Entry {
            value: Extrapolation::Reject,
            canonical: "Reject",
            aliases: &[],
        },
        Entry {
            value: Extrapolation::Flat,
            canonical: "Flat",
            aliases: &[],
        },
        Entry {
            value: Extrapolation::ExtendBoundarySegment,
            canonical: "ExtendBoundarySegment",
            aliases: &[],
        },
    ],
    &[],
);
