//! # SabiQuant
//!
//! An independent quantitative finance library written natively in Rust,
//! using only the standard library.
//!
//! **Status: milestone M1 (core foundations).** This version contains the
//! foundations that every financial domain builds on: the error model and the
//! numeric validation policy. It has **no financial functionality yet**.
//! Dates, calendars, day counts and interest rates arrive in milestones M2 to
//! M4 ([roadmap]). Nothing here may be used for financial calculations.
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

pub mod error;

mod identifier;
mod numeric;

// The shared test support lives in `tests/support` so that the integration
// tests and the unit tests use the same code (TESTING_STRATEGY §3).
#[cfg(test)]
#[path = "../tests/support/mod.rs"]
mod support;
