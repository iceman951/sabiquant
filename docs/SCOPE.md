# SabiQuant — Scope

| Field | Value |
|---|---|
| Milestone | M0 |
| Document status | Draft for project-owner review |
| Related | [VISION](VISION.md), [REQUIREMENTS](REQUIREMENTS.md), [FEATURE_MATRIX](FEATURE_MATRIX.md), [ROADMAP](ROADMAP.md) |

This document sets the boundary of what SabiQuant will and will not do, at
each planning horizon. The per-feature breakdown is in
[FEATURE_MATRIX](FEATURE_MATRIX.md). Precise behavior is in
[REQUIREMENTS](REQUIREMENTS.md).

---

## 1. Scope horizons

| Horizon | Milestones | Content | Commitment level |
|---|---|---|---|
| H0 — Planning | M0 | Research, specifications, architecture, ADRs, roadmap. Documentation only. | Committed (this milestone) |
| H1 — Foundations | M1–M5 | Core types, dates, periods and frequencies, calendars, business-day conventions, day-count conventions, interest rates, compounding, rate-based discounting, validation and release readiness | Planned. Implementation needs the owner's authorization (Gate F) |
| H2 — Expansion | M6–M16 (provisional) | Numerical foundations, schedules, cash flows, bonds, term structures, indexes, curve bootstrapping, option analytics, stochastic processes and Monte Carlo, lattice/FD methods, models, risk analytics | Provisional. Re-planned after M5 |
| H3 — Long-term ambition | Unnumbered | Remaining QuantLib domains (credit, inflation, exotics, market models, and others) | Aspirational. No dates, no promise |

## 2. Foundations scope (H1)

| Domain | Included | Excluded from H1 (with destination) |
|---|---|---|
| Core types | Error semantics; validation of numeric inputs; closed convention enumerations; canonical identifiers; f64 numeric policy | Money and currency types (Deferred); rounding conventions (Deferred); decimal arithmetic (Requires Decision) |
| Dates | Proleptic Gregorian dates over a documented range; leap years; month lengths; weekday; day-of-year; checked date arithmetic; month and year advancement with end-of-month clamping; ISO 8601 `YYYY-MM-DD` text form | Time of day, time zones, leap seconds, "today" (Out of Scope by design); Excel serial interop ([OQ-12](IMPLEMENTATION_READINESS.md#5-decision-register)) |
| Periods and frequencies | Period (signed length + Days/Weeks/Months/Years); normalization; Period↔Frequency mapping; named frequencies including `Once` | Intraday units; composite period strings such as `1Y6M` (Deferred); QuantLib `OtherFrequency` (Out of Scope) |
| Calendars | Business-day queries; null, weekends-only, custom (user-supplied) and joint calendars; adjustment; advancement; business-day counting; holiday lists; Easter computation; **one** holiday-aware pilot calendar (TARGET proposed, [OQ-17](IMPLEMENTATION_READINESS.md#5-decision-register)) | Other market calendars (Deferred, each needs a data-maintenance plan); trading hours and half days (Out of Scope for H1); calendars based on lunar or other non-Gregorian systems (Under Research) |
| Business-day conventions | Following, Modified Following, Preceding, Modified Preceding, Unadjusted, Half-Month Modified Following, Nearest | FRN/Eurodollar schedule rule (Under Research; belongs to schedules) |
| Day-count conventions | Actual/360, Actual/365 (Fixed), Actual/Actual (ISDA), Actual/Actual (ICMA), Actual/Actual (AFB), the 30/360 family (Bond Basis, 30E/360, 30E/360 (ISDA), 30/360 US), Business/252 | Lower-priority variants (Actual/364, Actual/365.25, Actual/366, Actual/365 No-Leap, Actual/365 Canadian, 30/360 Italian, 30/360 NASD, 30/365, 1/1) are Deferred or Under Research. See [FEATURE_MATRIX §5](FEATURE_MATRIX.md#5-day-count-conventions) |
| Interest rates and compounding | Rate quote (rate, day count, compounding, frequency); Simple, Compounded, Continuous, and the two hybrid compoundings; compound and discount factors; implied rates; equivalent rates; negative rates within valid domains | Term structures, forward curves, spreads on curves (H2) |
| Discounting | Discount factors for one rate over one period; present value of a single known amount | Multi-cash-flow NPV, settlement conventions, credit-adjusted discounting (H2) |
| Validation | Unit, property, exhaustive, boundary, regression, and differential tests; fixtures from QuantLib 1.43 and from independent oracles | CI configuration (created in M1, not M0); performance benchmarks (H2) |

## 3. Explicit non-goals

These are project-level non-goals. Changing any of them needs an ADR and the
owner's approval.

1. **No QuantLib runtime dependency.** QuantLib is never a runtime, build-time,
   or test-time dependency of the crate. Pre-generated reference data may be
   committed as plain data ([ADR-0008](adr/ADR-0008-validation-strategy.md)).
2. **No bindings or FFI.** This covers bindings to QuantLib or any C/C++
   library, and SabiQuant bindings for other languages. Bindings for Python, C,
   or WebAssembly are Out of Scope until an ADR approves them.
3. **No source translation.** QuantLib C++ will not be mechanically or
   manually translated into Rust.
4. **No API compatibility with QuantLib.** The aim is behavioral and numerical
   parity, not matching signatures.
5. **No third-party crates.** This applies to runtime, dev, build, and
   procedural-macro dependencies ([ADR-0002](adr/ADR-0002-standard-library-only.md)).
6. **No implicit global state.** There is no global evaluation date, no
   settings singleton, no process-wide fixing store, and no mutable shared
   calendars ([ADR-0011](adr/ADR-0011-explicit-state.md)).
7. **No system-clock or locale dependence.** The library never reads the
   system clock, time zone, locale, or environment variables.
8. **No I/O in the library.** No file, network, or database access in H1.
   Tests may read committed fixture files.
9. **No market-data acquisition, trading, order management, persistence, or
   user interface.**
10. **No exact-decimal settlement arithmetic in H1.** f64 is used for
    analytics. Exact settlement amounts are a recorded design issue
    ([NUMERICAL_CORRECTNESS §10](NUMERICAL_CORRECTNESS.md#10-future-numeric-representation-issues)).
11. **No automatic differentiation in H1.** This is a recorded trade-off in
    [ADR-0006](adr/ADR-0006-numeric-policy.md).
12. **No promise of bitwise-identical transcendental results across
    platforms.** Agreement is within documented tolerances
    ([NUMERICAL_CORRECTNESS §7](NUMERICAL_CORRECTNESS.md#7-tolerance-policy)).
13. **No claim of global or historical calendar coverage.** Each calendar
    states its verified coverage.
14. **No performance claims without reproducible benchmarks.**
15. **No unsafe code in H1** (proposed; NFR-06 in
    [REQUIREMENTS](REQUIREMENTS.md#2-non-functional-requirements)).

## 4. Long-term parity versus initial requirements

| Topic | Initial requirement (H1) | Long-term ambition (H2/H3) |
|---|---|---|
| Domain coverage | Nine foundational domains | Major QuantLib domains, prioritised by dependency and demand |
| QuantLib parity | Behavioral and numerical parity for the foundational features listed in FEATURE_MATRIX, verified against QuantLib 1.43 | Parity per feature, re-verified against newer QuantLib baselines |
| Calendars | Generic calendars and one pilot market calendar | Additional market calendars, each with provenance and a maintenance owner |
| Numerics | f64 elementary formulas with stable formulations | Solvers, interpolation, optimization, random numbers, linear algebra, all in-house |
| API stability | 0.x releases. Breaking changes allowed with deprecation notes | 1.0 after the API review and a period of real use ([ADR-0009](adr/ADR-0009-api-stability.md)) |

## 5. Scope change control

1. A scope change is proposed in a pull request that updates this document
   and [FEATURE_MATRIX](FEATURE_MATRIX.md) together.
2. If the change affects architecture, dependencies, numerics, validation,
   stability, licensing, or state management, it also needs a new or updated
   ADR.
3. The project owner approves scope changes. A pull request that changes scope
   without approval is not merged.
4. Features move between statuses (Planned, Under Research, Deferred, Out of
   Scope, Requires Decision) only through this process.

## 6. Assumptions

| ID | Assumption | Impact if false |
|---|---|---|
| A-01 | The project starts with a single maintainer (the repository owner). | Governance would need formal review roles ([CONTRIBUTING_GUIDELINES §9](CONTRIBUTING_GUIDELINES.md#9-governance-model)). |
| A-02 | QuantLib-Python wheels for the pinned version stay installable from PyPI, so reference fixtures can be regenerated. | Fixtures would have to come from a recorded C++ build ([ADR-0008](adr/ADR-0008-validation-strategy.md)). |
| A-03 | Users accept f64 analytics and do not need exact decimal settlement in H1. | Decimal design would move earlier ([OQ-06](IMPLEMENTATION_READINESS.md#5-decision-register)). |
| A-04 | Primary sources for conventions can be obtained, including licensed ISDA definitions where needed. | Some conventions stay Under Research ([R-02](IMPLEMENTATION_READINESS.md#6-risk-register)). |
| A-05 | The stable Rust toolchain is enough. No nightly features are needed. | MSRV and feature policy would need revisiting. |
| A-06 | The primary platforms are 64-bit Linux, macOS, and Windows. | The cross-platform test matrix would change ([OQ-08](IMPLEMENTATION_READINESS.md#5-decision-register)). |
