//! Bracketed scalar root finding with an explicit shared callback budget.
//!
//! Algorithms operate on the computed callback, which need not faithfully
//! represent a continuous mathematical function. Callback zeros and small
//! brackets are separate termination reasons; a small residual is never a
//! stopping rule. Caller callback panics propagate.

use crate::error::{Interval, ValueError};
use crate::numeric;
use std::error::Error as StdError;
use std::fmt;
use std::ops::Bound;

/// A finite, ordered, nonempty input or final coordinate bracket.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RootBracket {
    lower: f64,
    upper: f64,
}

impl RootBracket {
    /// Creates a finite bracket with `lower < upper`.
    ///
    /// Signed zero is canonicalized. This validates coordinates, not callback
    /// signs; the solver checks those after evaluating the endpoints.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::Value`] for a nonfinite endpoint and
    /// [`SolverError::InvalidBracket`] for equal or reversed coordinates.
    pub fn new(lower: f64, upper: f64) -> Result<Self, SolverError> {
        let lower = zero(numeric::finite("bracket lower", lower)?);
        let upper = zero(numeric::finite("bracket upper", upper)?);
        if lower >= upper {
            return Err(SolverError::InvalidBracket { lower, upper });
        }
        Ok(Self { lower, upper })
    }

    /// Returns the finite lower endpoint.
    #[must_use]
    pub const fn lower(self) -> f64 {
        self.lower
    }

    /// Returns the finite upper endpoint.
    #[must_use]
    pub const fn upper(self) -> f64 {
        self.upper
    }
}

/// Explicit positive absolute tolerance, relative tolerance and callback cap.
///
/// No default tolerance or implicit machine-epsilon floor is supplied.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolverOptions {
    absolute: f64,
    relative: f64,
    max_evaluations: u32,
}

impl SolverOptions {
    /// Validates finite `absolute > 0`, `0 <= relative < 1` and a nonzero cap.
    ///
    /// The cap counts both function and derivative invocations, including
    /// endpoints and any nonfinite callback result. It is checked before each
    /// invocation.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError::Value`] for an invalid tolerance or
    /// [`SolverError::InvalidBudget`] for a zero cap.
    pub fn new(absolute: f64, relative: f64, max_evaluations: u32) -> Result<Self, SolverError> {
        let absolute =
            numeric::within("absolute tolerance", absolute, Interval::greater_than(0.0))?;
        let relative = zero(numeric::within(
            "relative tolerance",
            relative,
            Interval::new(Bound::Included(0.0), Bound::Excluded(1.0)),
        )?);
        if max_evaluations == 0 {
            return Err(SolverError::InvalidBudget);
        }
        Ok(Self {
            absolute,
            relative,
            max_evaluations,
        })
    }

    /// Returns the strictly positive absolute coordinate tolerance.
    #[must_use]
    pub const fn absolute(self) -> f64 {
        self.absolute
    }

    /// Returns the relative coordinate tolerance, in `[0, 1)`.
    #[must_use]
    pub const fn relative(self) -> f64 {
        self.relative
    }

    /// Returns the maximum combined callback count.
    #[must_use]
    pub const fn max_evaluations(self) -> u32 {
        self.max_evaluations
    }
}

/// Separate function and derivative invocation counts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvaluationCounts {
    function: u32,
    derivative: u32,
}

impl EvaluationCounts {
    /// Returns the number of function invocations.
    #[must_use]
    pub const fn function(self) -> u32 {
        self.function
    }

    /// Returns the number of derivative invocations.
    #[must_use]
    pub const fn derivative(self) -> u32 {
        self.derivative
    }

    /// Returns the combined invocation count, without integer overflow.
    #[must_use]
    pub const fn total(self) -> u64 {
        self.function as u64 + self.derivative as u64
    }
}

/// The observed reason a solver stopped successfully.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TerminationReason {
    /// A callback returned exactly zero, including negative zero.
    FunctionZero,
    /// The rounded full-bracket tolerance comparison passed.
    BracketTolerance,
}

/// The stage of a rejected nonfinite callback evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EvaluationStage {
    /// Evaluation at the input lower endpoint.
    LowerEndpoint,
    /// Evaluation at the input upper endpoint.
    UpperEndpoint,
    /// Newton's explicit initial point, when not a cached endpoint.
    InitialPoint,
    /// A bisection, interpolation or Newton function trial.
    TrialPoint,
    /// A derivative at a cached finite function point.
    Derivative,
}

/// A finite evaluated point, cached residual and maintained final bracket.
///
/// Neither reason proves continuity, callback accuracy or uniqueness. A
/// callback-zero result preserves the current non-collapsed bracket. For
/// bracket termination the point is the cached endpoint with the smaller
/// absolute residual, with the lower endpoint winning a tie.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolveReport {
    point: f64,
    residual: f64,
    bracket: RootBracket,
    evaluations: EvaluationCounts,
    iterations: u32,
    reason: TerminationReason,
}

impl SolveReport {
    /// Returns the finite, evaluated point inside the final bracket.
    #[must_use]
    pub const fn point(self) -> f64 {
        self.point
    }

    /// Returns its cached finite callback value; no extra callback is made.
    #[must_use]
    pub const fn residual(self) -> f64 {
        self.residual
    }

    /// Returns the final ordered, non-collapsed coordinate bracket.
    #[must_use]
    pub const fn bracket(self) -> RootBracket {
        self.bracket
    }

    /// Returns callback costs, including initial endpoint evaluations.
    #[must_use]
    pub const fn evaluations(self) -> EvaluationCounts {
        self.evaluations
    }

    /// Returns function trials after endpoints, including an interior Newton
    /// initial point. Derivative invocations do not increment this count.
    #[must_use]
    pub const fn iterations(self) -> u32 {
        self.iterations
    }

    /// Returns the observed termination reason.
    #[must_use]
    pub const fn reason(self) -> TerminationReason {
        self.reason
    }
}

/// A rejected solver input, callback output or unsuccessful bounded solve.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum SolverError {
    /// A nonfinite input or invalid tolerance/initial point; `solver.value`.
    Value(ValueError),
    /// Coordinates are finite but not increasing; `solver.invalid_bracket`.
    #[non_exhaustive]
    InvalidBracket {
        /// Rejected lower endpoint.
        lower: f64,
        /// Rejected upper endpoint.
        upper: f64,
    },
    /// The callback cap is zero; `solver.invalid_budget`.
    InvalidBudget,
    /// Nonzero endpoint values have the same sign; `solver.not_bracketed`.
    #[non_exhaustive]
    NotBracketed {
        /// Validated input coordinate bracket.
        bracket: RootBracket,
        /// Cached lower endpoint callback value.
        lower_value: f64,
        /// Cached upper endpoint callback value.
        upper_value: f64,
        /// Actual invocation counts.
        evaluations: EvaluationCounts,
    },
    /// A callback returned NaN or infinity; `solver.non_finite_evaluation`.
    #[non_exhaustive]
    NonFiniteEvaluation {
        /// Callback evaluation stage.
        stage: EvaluationStage,
        /// Finite callback argument.
        point: f64,
        /// Underlying numeric result error.
        cause: ValueError,
        /// Actual invocations, including the rejected result.
        evaluations: EvaluationCounts,
    },
    /// The next callback would exceed the cap; `solver.evaluation_limit`.
    #[non_exhaustive]
    EvaluationLimit {
        /// Current coordinate bracket; signs may be unverified during initialization.
        bracket: RootBracket,
        /// Actual invocations; the rejected next invocation did not occur.
        evaluations: EvaluationCounts,
        /// Known finite point and cached value, if any. This is not a root claim.
        best: Option<(f64, f64)>,
    },
    /// No representable interior midpoint remains and tolerance fails;
    /// `solver.precision_exhausted`.
    #[non_exhaustive]
    PrecisionExhausted {
        /// Maintained sign bracket whose endpoints are adjacent floats.
        bracket: RootBracket,
        /// Actual invocations.
        evaluations: EvaluationCounts,
        /// Better endpoint and its finite cached value.
        best: (f64, f64),
    },
}

impl SolverError {
    /// Returns the stable domain error kind.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Value(_) => "solver.value",
            Self::InvalidBracket { .. } => "solver.invalid_bracket",
            Self::InvalidBudget => "solver.invalid_budget",
            Self::NotBracketed { .. } => "solver.not_bracketed",
            Self::NonFiniteEvaluation { .. } => "solver.non_finite_evaluation",
            Self::EvaluationLimit { .. } => "solver.evaluation_limit",
            Self::PrecisionExhausted { .. } => "solver.precision_exhausted",
        }
    }
}

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(cause) => write!(f, "solver input: {cause}"),
            Self::InvalidBracket { lower, upper } => {
                write!(f, "solver bracket must increase: [{lower}, {upper}]")
            }
            Self::InvalidBudget => f.write_str("solver callback budget must be nonzero"),
            Self::NotBracketed {
                bracket,
                lower_value,
                upper_value,
                ..
            } => write!(
                f,
                "solver has no sign change on [{}, {}]: values {lower_value}, {upper_value}",
                bracket.lower, bracket.upper
            ),
            Self::NonFiniteEvaluation {
                stage,
                point,
                cause,
                ..
            } => write!(f, "solver {stage:?} callback at {point}: {cause}"),
            Self::EvaluationLimit {
                evaluations,
                bracket,
                ..
            } => write!(
                f,
                "solver callback budget exhausted after {} calls on [{}, {}]",
                evaluations.total(),
                bracket.lower,
                bracket.upper
            ),
            Self::PrecisionExhausted { bracket, .. } => write!(
                f,
                "solver has no representable interior point on [{}, {}]",
                bracket.lower, bracket.upper
            ),
        }
    }
}

impl StdError for SolverError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Value(cause) | Self::NonFiniteEvaluation { cause, .. } => Some(cause),
            _ => None,
        }
    }
}

impl From<ValueError> for SolverError {
    fn from(value: ValueError) -> Self {
        Self::Value(value)
    }
}

/// Finds a computed callback zero or a sufficiently small sign bracket by
/// bisection. Every argument is finite and inside the original bracket.
///
/// The lower endpoint is evaluated first; its exact zero needs one callback.
/// Otherwise both endpoints must have finite, nonzero opposite signs. There
/// is no residual-only success or automatic bracket expansion.
///
/// Full-width termination evaluates `upper-lower <= absolute +
/// relative*max(abs(lower),abs(upper))` in that operation order. Only when
/// subtraction overflows, it compares `upper/2-lower/2` with `absolute/2 +
/// relative*(max_abs/2)`. These are rounded binary64 comparisons; they do not
/// silently change user tolerances. Finite subnormal widths are never halved.
///
/// ```
/// use sabiquant::math::{RootBracket, SolverOptions, TerminationReason, bisect};
/// let bracket = RootBracket::new(0.0, 1.0)?;
/// let options = SolverOptions::new(1e-12, 0.0, 100)?;
/// let report = bisect(|x| x - 0.25, bracket, options)?;
/// assert_eq!(report.reason(), TerminationReason::FunctionZero);
/// assert_eq!(report.point(), 0.25);
/// # Ok::<(), sabiquant::math::SolverError>(())
/// ```
///
/// # Errors
///
/// Returns an error for nonfinite endpoint/trial callback values, missing
/// endpoint sign change, callback exhaustion or unattainable positional
/// precision. The callback cap is checked before each invocation.
///
/// # Panics
///
/// A panic from the caller's callback propagates.
pub fn bisect<F: FnMut(f64) -> f64>(
    mut function: F,
    bracket: RootBracket,
    options: SolverOptions,
) -> Result<SolveReport, SolverError> {
    let mut counts = EvaluationCounts {
        function: 0,
        derivative: 0,
    };
    let mut state = match initialize(&mut function, bracket, options, &mut counts)? {
        Initialization::Done(report) => return Ok(report),
        Initialization::Bracket(state) => state,
    };
    let mut iterations = 0;
    loop {
        if tolerance_met(state.bracket, options) {
            return Ok(state.report(counts, iterations));
        }
        let point = interior_midpoint(state, counts)?;
        let value = evaluate(
            &mut function,
            point,
            EvaluationStage::TrialPoint,
            &mut counts,
            options,
            state.bracket,
            Some(state.best()),
        )?;
        iterations += 1;
        if is_zero(value) {
            return Ok(report(
                point,
                value,
                state.bracket,
                counts,
                iterations,
                TerminationReason::FunctionZero,
            ));
        }
        state.retain(point, value);
    }
}

/// Finds a callback zero or a small sign bracket using Brent's method.
///
/// The mathematical method follows [Brent's 1973 book, Chapter 4 §2,
/// pp. 48–52](https://maths-people.anu.edu.au/~brent/pd/rpb011i.pdf).
/// The best endpoint, opposing endpoint and previous best point supply an
/// inverse-linear or inverse-quadratic trial. Interpolation requires a smaller
/// absolute function value than the previous best point, an interior point
/// before three quarters across the bracket, and a step strictly smaller
/// than half the step before the last. Bisection resets both historical step
/// magnitudes. Degenerate, nonfinite or repeating trials also bisect.
///
/// This implementation independently evaluates inverse Lagrange interpolation
/// using scaled ordinates and coordinate differences. It adapts termination
/// to [`bisect`]'s explicit full-width test. It uses no hidden epsilon floor
/// or forced minimum interpolation step. All callbacks remain inside the
/// original finite bracket; residual magnitude alone is never success.
///
/// ```
/// use sabiquant::math::{RootBracket, SolverOptions, brent};
/// let report = brent(|x| x*x - 2.0, RootBracket::new(0.0, 2.0)?,
///     SolverOptions::new(1e-12, 0.0, 100)?)?;
/// assert!((report.point() - std::f64::consts::SQRT_2).abs() <= 1e-12);
/// assert_eq!(report.evaluations().derivative(), 0);
/// # Ok::<(), sabiquant::math::SolverError>(())
/// ```
///
/// # Errors
///
/// Returns the same endpoint, callback, budget and precision errors as
/// [`bisect`]. Arithmetic failure in an optional interpolation falls back to
/// bisection rather than invoking a callback at a nonfinite point.
///
/// # Panics
///
/// A panic from the caller's callback propagates.
pub fn brent<F: FnMut(f64) -> f64>(
    mut function: F,
    bracket: RootBracket,
    options: SolverOptions,
) -> Result<SolveReport, SolverError> {
    let mut counts = EvaluationCounts {
        function: 0,
        derivative: 0,
    };
    let mut state = match initialize(&mut function, bracket, options, &mut counts)? {
        Initialization::Done(report) => return Ok(report),
        Initialization::Bracket(state) => state,
    };
    let mut previous_best = state.opposite();
    let mut previous_step = bracket.upper - bracket.lower;
    let mut historical_step = previous_step;
    let mut iterations = 0;
    loop {
        if tolerance_met(state.bracket, options) {
            return Ok(state.report(counts, iterations));
        }
        let midpoint = interior_midpoint(state, counts)?;
        let best = state.best();
        let opposite = state.opposite();
        let trial = if best.1.abs() < previous_best.1.abs() {
            inverse_trial(previous_best, best, opposite).filter(|point| {
                acceptable_trial(*point, best.0, opposite.0, state.bracket, historical_step)
            })
        } else {
            None
        };
        let point = match trial {
            Some(point) => {
                historical_step = previous_step;
                previous_step = (point - best.0).abs();
                point
            }
            None => {
                previous_step = (midpoint - best.0).abs();
                historical_step = previous_step;
                midpoint
            }
        };
        let value = evaluate(
            &mut function,
            point,
            EvaluationStage::TrialPoint,
            &mut counts,
            options,
            state.bracket,
            Some(best),
        )?;
        iterations += 1;
        if is_zero(value) {
            return Ok(report(
                point,
                value,
                state.bracket,
                counts,
                iterations,
                TerminationReason::FunctionZero,
            ));
        }
        previous_best = best;
        state.retain(point, value);
    }
}

/// Finds a callback zero or a small sign bracket using safeguarded Newton.
///
/// The explicit finite `initial` must lie in the closed input bracket; a
/// cached endpoint is reused. Newton's trial is `x-f(x)/f'(x)`. Accept a finite
/// interior, non-repeating point before three quarters across the current
/// bracket and a step smaller than half the step before the last; otherwise
/// bisect. Bisection resets both historical step magnitudes. These safeguards
/// maintain the sign bracket and force progress; they do not promise global
/// quadratic convergence. A zero derivative falls back to bisection.
///
/// Function and derivative invocations share one check-before-call budget.
/// Endpoint zeros, initial zeros and bracket tolerance stop before requesting
/// a derivative. A nonfinite derivative is an error, not a fallback.
/// Termination uses [`bisect`]'s full-width positional comparison.
///
/// ```
/// use sabiquant::math::{RootBracket, SolverOptions, safeguarded_newton};
/// let report = safeguarded_newton(|x| x*x - 2.0, |x| 2.0*x, 1.0,
///     RootBracket::new(0.0, 2.0)?, SolverOptions::new(1e-12, 0.0, 100)?)?;
/// assert!((report.point() - std::f64::consts::SQRT_2).abs() <= 1e-12);
/// assert!(report.evaluations().total() <= 100);
/// # Ok::<(), sabiquant::math::SolverError>(())
/// ```
///
/// # Errors
///
/// Returns [`SolverError::Value`] for an invalid initial point, checked before
/// any callback. Other failures are the endpoint, callback, budget and
/// precision errors described by [`bisect`], including nonfinite derivatives.
///
/// # Panics
///
/// A panic from either caller callback propagates.
pub fn safeguarded_newton<F: FnMut(f64) -> f64, D: FnMut(f64) -> f64>(
    mut function: F,
    mut derivative: D,
    initial: f64,
    bracket: RootBracket,
    options: SolverOptions,
) -> Result<SolveReport, SolverError> {
    let initial = zero(numeric::within(
        "initial point",
        initial,
        Interval::new(
            Bound::Included(bracket.lower),
            Bound::Included(bracket.upper),
        ),
    )?);
    let mut counts = EvaluationCounts {
        function: 0,
        derivative: 0,
    };
    let mut state = match initialize(&mut function, bracket, options, &mut counts)? {
        Initialization::Done(report) => return Ok(report),
        Initialization::Bracket(state) => state,
    };
    if tolerance_met(state.bracket, options) {
        return Ok(state.report(counts, 0));
    }
    let mut iterations = 0;
    let value = if same_point(initial, bracket.lower) {
        state.lower_value
    } else if same_point(initial, bracket.upper) {
        state.upper_value
    } else {
        let value = evaluate(
            &mut function,
            initial,
            EvaluationStage::InitialPoint,
            &mut counts,
            options,
            state.bracket,
            Some(state.best()),
        )?;
        iterations += 1;
        if is_zero(value) {
            return Ok(report(
                initial,
                value,
                state.bracket,
                counts,
                iterations,
                TerminationReason::FunctionZero,
            ));
        }
        state.retain(initial, value);
        value
    };
    let mut current = (initial, value);
    let mut previous_step = bracket.upper - bracket.lower;
    let mut historical_step = previous_step;
    loop {
        if tolerance_met(state.bracket, options) {
            return Ok(state.report(counts, iterations));
        }
        let midpoint = interior_midpoint(state, counts)?;
        let slope = evaluate(
            &mut derivative,
            current.0,
            EvaluationStage::Derivative,
            &mut counts,
            options,
            state.bracket,
            Some(state.best()),
        )?;
        let candidate = if is_zero(slope) {
            f64::NAN
        } else {
            current.0 - current.1 / slope
        };
        let opposite = if same_sign(current.1, state.lower_value) {
            state.bracket.upper
        } else {
            state.bracket.lower
        };
        let point = if acceptable_trial(
            candidate,
            current.0,
            opposite,
            state.bracket,
            historical_step,
        ) {
            historical_step = previous_step;
            previous_step = (candidate - current.0).abs();
            candidate
        } else {
            previous_step = (midpoint - current.0).abs();
            historical_step = previous_step;
            midpoint
        };
        let value = evaluate(
            &mut function,
            point,
            EvaluationStage::TrialPoint,
            &mut counts,
            options,
            state.bracket,
            Some(state.best()),
        )?;
        iterations += 1;
        if is_zero(value) {
            return Ok(report(
                point,
                value,
                state.bracket,
                counts,
                iterations,
                TerminationReason::FunctionZero,
            ));
        }
        state.retain(point, value);
        current = (point, value);
    }
}

// Inverse Lagrange interpolation is evaluated relative to the best point.
// Scaling all ordinates by one common finite factor leaves the mathematical
// interpolant unchanged, while avoiding products of enormous function values.
fn inverse_trial(previous: (f64, f64), best: (f64, f64), opposite: (f64, f64)) -> Option<f64> {
    let scale = previous.1.abs().max(best.1.abs()).max(opposite.1.abs());
    let a = previous.1 / scale;
    let b = best.1 / scale;
    let c = opposite.1 / scale;
    let candidate = if same_point(previous.0, opposite.0) || same_point(a, c) {
        let ratio = best.1 / previous.1;
        best.0 + (best.0 - previous.0) * ratio / (1.0 - ratio)
    } else if same_point(a, b) || same_point(b, c) {
        return None;
    } else {
        let wa = (b / (a - b)) * (c / (a - c));
        let wc = (a / (c - a)) * (b / (c - b));
        best.0 + ((previous.0 - best.0) * wa + (opposite.0 - best.0) * wc)
    };
    candidate.is_finite().then_some(zero(candidate))
}

fn acceptable_trial(
    point: f64,
    from: f64,
    to: f64,
    bracket: RootBracket,
    historical_step: f64,
) -> bool {
    if !point.is_finite() || point <= bracket.lower || point >= bracket.upper {
        return false;
    }
    let limit = from.midpoint(to).midpoint(to);
    let before_limit = if to > from {
        point > from && point < limit
    } else {
        point < from && point > limit
    };
    before_limit && (point - from).abs() < historical_step / 2.0
}

#[expect(
    clippy::float_cmp,
    reason = "detecting exactly cached coordinates and degenerate interpolation ordinates"
)]
fn same_point(a: f64, b: f64) -> bool {
    a == b
}

#[derive(Clone, Copy)]
struct BracketState {
    bracket: RootBracket,
    lower_value: f64,
    upper_value: f64,
}

impl BracketState {
    fn best(self) -> (f64, f64) {
        if self.lower_value.abs() <= self.upper_value.abs() {
            (self.bracket.lower, self.lower_value)
        } else {
            (self.bracket.upper, self.upper_value)
        }
    }

    fn opposite(self) -> (f64, f64) {
        if self.lower_value.abs() <= self.upper_value.abs() {
            (self.bracket.upper, self.upper_value)
        } else {
            (self.bracket.lower, self.lower_value)
        }
    }

    fn retain(&mut self, point: f64, value: f64) {
        if same_sign(value, self.lower_value) {
            self.bracket.lower = zero(point);
            self.lower_value = value;
        } else {
            self.bracket.upper = zero(point);
            self.upper_value = value;
        }
    }

    fn report(self, counts: EvaluationCounts, iterations: u32) -> SolveReport {
        let (point, residual) = self.best();
        report(
            point,
            residual,
            self.bracket,
            counts,
            iterations,
            TerminationReason::BracketTolerance,
        )
    }
}

enum Initialization {
    Done(SolveReport),
    Bracket(BracketState),
}

fn initialize<F: FnMut(f64) -> f64>(
    function: &mut F,
    bracket: RootBracket,
    options: SolverOptions,
    counts: &mut EvaluationCounts,
) -> Result<Initialization, SolverError> {
    let lower_value = evaluate(
        function,
        bracket.lower,
        EvaluationStage::LowerEndpoint,
        counts,
        options,
        bracket,
        None,
    )?;
    if is_zero(lower_value) {
        return Ok(Initialization::Done(report(
            bracket.lower,
            lower_value,
            bracket,
            *counts,
            0,
            TerminationReason::FunctionZero,
        )));
    }
    let upper_value = evaluate(
        function,
        bracket.upper,
        EvaluationStage::UpperEndpoint,
        counts,
        options,
        bracket,
        Some((bracket.lower, lower_value)),
    )?;
    if is_zero(upper_value) {
        return Ok(Initialization::Done(report(
            bracket.upper,
            upper_value,
            bracket,
            *counts,
            0,
            TerminationReason::FunctionZero,
        )));
    }
    if same_sign(lower_value, upper_value) {
        return Err(SolverError::NotBracketed {
            bracket,
            lower_value,
            upper_value,
            evaluations: *counts,
        });
    }
    Ok(Initialization::Bracket(BracketState {
        bracket,
        lower_value,
        upper_value,
    }))
}

fn evaluate<F: FnMut(f64) -> f64>(
    function: &mut F,
    point: f64,
    stage: EvaluationStage,
    counts: &mut EvaluationCounts,
    options: SolverOptions,
    bracket: RootBracket,
    best: Option<(f64, f64)>,
) -> Result<f64, SolverError> {
    if counts.total() >= u64::from(options.max_evaluations) {
        return Err(SolverError::EvaluationLimit {
            bracket,
            evaluations: *counts,
            best,
        });
    }
    if matches!(stage, EvaluationStage::Derivative) {
        counts.derivative += 1;
    } else {
        counts.function += 1;
    }
    let value = function(zero(point));
    numeric::finite_result("solver callback", value)
        .map(zero)
        .map_err(|cause| SolverError::NonFiniteEvaluation {
            stage,
            point: zero(point),
            cause,
            evaluations: *counts,
        })
}

fn tolerance_met(bracket: RootBracket, options: SolverOptions) -> bool {
    let width = bracket.upper - bracket.lower;
    let magnitude = bracket.lower.abs().max(bracket.upper.abs());
    if width.is_finite() {
        width <= options.absolute + options.relative * magnitude
    } else {
        bracket.upper / 2.0 - bracket.lower / 2.0
            <= options.absolute / 2.0 + options.relative * (magnitude / 2.0)
    }
}

fn interior_midpoint(state: BracketState, counts: EvaluationCounts) -> Result<f64, SolverError> {
    let point = zero(state.bracket.lower.midpoint(state.bracket.upper));
    if point > state.bracket.lower && point < state.bracket.upper {
        Ok(point)
    } else {
        Err(SolverError::PrecisionExhausted {
            bracket: state.bracket,
            evaluations: counts,
            best: state.best(),
        })
    }
}

fn report(
    point: f64,
    residual: f64,
    bracket: RootBracket,
    evaluations: EvaluationCounts,
    iterations: u32,
    reason: TerminationReason,
) -> SolveReport {
    SolveReport {
        point: zero(point),
        residual: zero(residual),
        bracket,
        evaluations,
        iterations,
        reason,
    }
}

fn same_sign(a: f64, b: f64) -> bool {
    (a < 0.0) == (b < 0.0)
}

fn is_zero(value: f64) -> bool {
    value == 0.0
}

fn zero(value: f64) -> f64 {
    if is_zero(value) { 0.0 } else { value }
}
