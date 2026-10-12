//! # SabiQuant
//!
//! An independent quantitative finance library written natively in Rust,
//! using only the standard library.
//!
//! **Status: foundations and M6 numerics, under local validation.** This
//! unreleased development version contains shared errors, checked dates and
//! tenors, immutable calendars, Western Easter, guarded TARGET coverage and
//! eight day-count conventions, explicit interest rates and compounding,
//! checked numerical solvers, interpolation, normal functions and summation.
//! US 30/360, AFB and further pricing remain later work ([roadmap]).
//!
//! # Principles
//!
//! - **Correctness first.** Every financial computation follows a written
//!   specification with cited sources ([requirements]).
//! - **Standard library only.** The crate has no dependencies of any kind.
//! - **Explicit state.** There is no global configuration, no implicit
//!   "today", and no I/O: results depend only on the arguments.
//! - **No panics on input.** Every operation that can fail returns a typed
//!   error; see [`error`].
//! - **Values, not objects.** Public types are immutable and can be sent and
//!   shared between threads.
//!
//! # Modules
//!
//! | Module | Contents |
//! |---|---|
//! | [`error`] | The crate-level [`Error`](error::Error) and the error types that every domain shares |
//! | [`math`] | Bracketed solvers, explicit interpolation, normal probabilities and checked reductions |
//! | [`time`] | Gregorian dates, months, weekdays, tenors and periodic frequencies |
//! | [`calendar`] | Immutable calendars, guarded TARGET coverage and Western Easter |
//! | [`daycount`] | Source-verified scalar and contextual day counts and accrual fractions |
//! | [`rates`] | Explicit quotes, five compounding methods, factors, conversions and present values |
//!
//! # Relationship to QuantLib
//!
//! SabiQuant uses [QuantLib](https://www.quantlib.org/) 1.43 as an external
//! behavioral and numerical reference. It is not a port of QuantLib, not a
//! binding to it, and contains no QuantLib code. SabiQuant is not affiliated
//! with or endorsed by the QuantLib project.
//!
//! [roadmap]: https://github.com/iceman951/sabiquant/blob/main/docs/ROADMAP.md
//! [requirements]: https://github.com/iceman951/sabiquant/blob/main/docs/REQUIREMENTS.md

pub mod calendar;
pub mod daycount;
pub mod error;
pub mod math;
pub mod rates;
pub mod time;

mod identifier;
mod numeric;

// The shared test support lives in `tests/support` so that the integration
// tests and the unit tests use the same code (TESTING_STRATEGY §3).
#[cfg(test)]
#[path = "../tests/support/mod.rs"]
mod support;
