//! # SabiQuant
//!
//! An independent quantitative finance library written natively in Rust,
//! using only the standard library.
//!
//! **Status: foundations, numerics, schedules and cash-flow analytics under local validation.** This
//! unreleased development version contains shared errors, checked dates and
//! tenors, immutable calendars, Western Easter, guarded TARGET coverage and
//! ten day-count conventions, explicit interest rates and compounding,
//! checked numerical solvers, interpolation, normal functions and summation,
//! and anchored Forward/Backward/Zero schedules with explicit coupon dates,
//! adjustment and checked ICMA reference contexts, four explicit civil month
//! rolls with requested/resolved provenance, plus simple cash flows,
//! plain fixed coupons, bounded legs, flat NPV and outstanding accrued amounts.
//! Modern CDS date operations provide explicit quoted-maturity regimes and
//! Following payment/accrual plans with separate raw, payment and accrual roles;
//! the automatic local OldCDS rule builds twentieth schedules on quarterly
//! anchors with explicit provenance. US 30/360 requires explicit security EOM
//! qualification; AFB follows ISDA's published backward-year clarification.
//! Exact decimal amounts supply explicit arbitrary quanta, seven rounding
//! modes and checked cancellation-safe/fused arithmetic, with four explicitly
//! interpreted binary64 conversions. Zero-coupon and fixed-rate bond core
//! definitions provide explicit settlement/ex-coupon entitlement,
//! earned-minus-detached accrued interest, exact binary64 per-100 quote
//! conversion, and two-stage direct-horizon pricing plans. Explicit settlement
//! recipes retain supplied decimal/binary or declared interval evidence through
//! source and cash rounding, with complete instruction/receipt provenance.
//! Generic bonds provide bounded yield inversion and all six analytical risk
//! views. Conventional UK gilts add explicit complete quasi dates, source
//! first-dividend publication and accrued interest, source-model price/yield/risk
//! and four exact cash recipes. Fixed-principal US Treasuries retain separate
//! original dated/issue dates and explicit daily/publication policies, with
//! deterministic rational source-model price/yield/risk and four exact cash
//! recipes. Canadian models and analytical enclosures remain later work.
//! CDS premium-leg pricing, legal
//! protection and settlement remain later work ([roadmap]).
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
//! | [`amounts`] | Exact decimals, explicit arbitrary quanta, seven rounding modes and explicit binary64 conversions |
//! | [`bonds`] | Generic bullet bonds, conventional UK gilts and fixed-principal US Treasuries; explicit dates/policies, source accrual, price/yield/risk and exact named-model cash recipes |
//! | [`error`] | The crate-level [`Error`](error::Error) and the error types that every domain shares |
//! | [`math`] | Bracketed solvers, explicit interpolation, normal probabilities and checked reductions |
//! | [`time`] | Gregorian dates, months, weekdays, tenors and periodic frequencies |
//! | [`cashflows`] | Plain signed analytical coupons/legs, explicit inclusion, bounded flat NPV and outstanding accrual |
//! | [`calendar`] | Immutable calendars, guarded TARGET coverage and Western Easter |
//! | [`daycount`] | Source-verified scalar and contextual day counts and accrual fractions |
//! | [`rates`] | Explicit quotes, five compounding methods, factors, conversions and present values |
//! | [`schedule`] | Anchored schedules, explicit coupon roles, civil month rolls, automatic OldCDS, checked ICMA references, explicit modern CDS maturities and Following coupon-date plans |
//! | [`settlement`] | Four explicit exact settlement recipes, declared enclosure decisions, source/cash rounding and retained evidence/provenance |
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

pub mod amounts;
pub mod bonds;
pub mod calendar;
pub mod cashflows;
pub mod daycount;
pub mod error;
pub mod math;
pub mod rates;
pub mod schedule;
pub mod settlement;
pub mod time;

mod identifier;
mod numeric;

// The shared test support lives in `tests/support` so that the integration
// tests and the unit tests use the same code (TESTING_STRATEGY §3).
#[cfg(test)]
#[path = "../tests/support/mod.rs"]
mod support;
