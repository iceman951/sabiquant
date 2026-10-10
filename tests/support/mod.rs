//! Shared test support for SabiQuant. Test-only: never public API
//! (ARCHITECTURE §6, rule 6).
//!
//! - [`prng`]: the deterministic SplitMix64 generator and the property-test
//!   runner (F-CORE-05);
//! - [`float`]: ULP distance and tolerance comparisons (F-CORE-05);
//! - [`fixture`]: the reader for reference fixtures (F-CORE-06).
//!
//! Integration tests include it with `mod support;`. The library's unit tests
//! include the same files through a `#[path]` module in `src/lib.rs`, so both
//! use one implementation (TESTING_STRATEGY §3). The self-tests of this code
//! are in `tests/test_support.rs`.

// Every test crate compiles all of the support code but uses only part of it.
#![allow(dead_code, unreachable_pub)]
// Assertion helpers report failure by panicking, as test code does.
#![allow(clippy::panic, clippy::expect_used)]

pub mod fixture;
pub mod float;
pub mod prng;
