//! Numerical algorithms with explicit inputs and checked results.
//!
//! These algorithms are independently implemented using Rust's standard
//! library. No algorithm reads global settings or performs I/O. See the
//! [local M6 contract](https://github.com/iceman951/sabiquant/blob/main/docs/M6_LOCAL_READINESS.md)
//! for the scope and validation status of this unreleased module.
//!
//! [`compensated_sum`] preserves the supplied reduction order and checks
//! intermediate arithmetic. [`exact_sum`] retains the exact sum of finite
//! binary64 inputs and rounds only once, including after severe cancellation.
//!
//! [`bisect`], [`brent`] and [`safeguarded_newton`] require a finite
//! [`RootBracket`] and explicit [`SolverOptions`]. The callback cap counts
//! both function and derivative calls, checked before each invocation.
//! Callback zeros and positional tolerance have different meanings;
//! caller continuity, function accuracy and callback panics remain explicit.
//!
//! [`Interpolator`] owns ordered nodes, a named [`InterpolationMethod`] and
//! an [`Extrapolation`] policy. Natural cubic and monotone Hermite are distinct
//! definitions; numerical construction support and approximation error are
//! documented separately. Exact stored knots retain their input values.
//!
//! [`standard_normal_survival`] evaluates small tails directly;
//! [`standard_normal_log_survival`] remains useful after probability underflow.
//! [`standard_normal_inverse_cdf`] supports strictly interior binary64
//! probabilities, including subnormals. [`NormalDistribution`] owns explicit
//! location and positive scale. Elementary precision is platform dependent;
//! these algorithms do not promise correctly rounded transcendental results.

mod interpolation;
mod normal;
mod solver;
mod sum;

pub use interpolation::{Extrapolation, InterpolationError, InterpolationMethod, Interpolator};
pub use normal::{
    NormalDistribution, NormalError, standard_normal_cdf, standard_normal_inverse_cdf,
    standard_normal_log_cdf, standard_normal_log_survival, standard_normal_pdf,
    standard_normal_survival,
};
pub use solver::{
    EvaluationCounts, EvaluationStage, RootBracket, SolveReport, SolverError, SolverOptions,
    TerminationReason, bisect, brent, safeguarded_newton,
};
pub(crate) use solver::{FallibleSolverError, brent_with_payload};
pub use sum::{SumError, compensated_sum, exact_sum};
