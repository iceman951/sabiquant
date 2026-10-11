//! # SabiQuant
//!
//! An independent quantitative finance library written natively in Rust,
//! using only the standard library.
//!
//! **Status: milestone M2 (dates and periods), under validation.** This
//! unreleased development version contains shared errors and checked date,
//! tenor and frequency APIs. Calendars, day counts and rates arrive in later
//! milestones ([roadmap]).
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
//! | [`time`] | Gregorian dates, months, weekdays, tenors and periodic frequencies |
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
pub mod time;

mod identifier;
mod numeric;

// The shared test support lives in `tests/support` so that the integration
// tests and the unit tests use the same code (TESTING_STRATEGY §3).
#[cfg(test)]
#[path = "../tests/support/mod.rs"]
mod support;
