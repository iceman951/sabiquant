# SabiQuant — Project Vision

| Field | Value |
|---|---|
| Milestone | M0 — Research, Specification, Architecture and Planning |
| Document status | Draft for project-owner review |
| Implementation status | Not started. No usable implementation exists. |
| Related | [SCOPE](SCOPE.md), [ROADMAP](ROADMAP.md), [ARCHITECTURE](ARCHITECTURE.md), [ADR index](adr/README.md) |

Statements labelled **owner constraint** come from the project owner's M0 brief
and are treated as approved. Statements labelled **proposed** are recommendations
that still need the owner's approval.

---

## 1. Mission

SabiQuant is an independent quantitative finance library written natively in
Rust. Over time it aims to reach functional parity with the major domains of
[QuantLib](https://github.com/lballabio/QuantLib). It does this with an idiomatic
Rust API, behavior that is correct in financial terms, and numerical results
that are validated rigorously and reproducibly.

SabiQuant is meant to be a reusable, public, long-lived library with a stable
API. It is not an application, a QuantLib wrapper, or a QuantLib port.

## 2. Why this project exists

1. **Rust's guarantees fit financial software.** Ownership, value semantics,
   explicit `Result`-based error handling, and compile-time thread-safety
   checking stop several classes of defects that cause silent failures in
   numerical finance code. Examples are use-after-free, data races on shared
   market state, and errors that are ignored.
2. **QuantLib is the best available behavioral benchmark.** QuantLib is the
   most widely used open-source quantitative finance library. Its behavior and
   test results are a valuable *external reference* for financial conventions
   and numerical outputs. SabiQuant uses QuantLib to discover features and edge
   cases and to compare results. It does not copy QuantLib's code or structure.
3. **QuantLib's C++ architecture does not translate directly to Rust.**
   QuantLib relies on a process-wide evaluation date, observer/observable
   object graphs, relinkable handles, and calendar implementations that are
   shared and mutable. These patterns fight Rust's ownership model and make
   reproducibility harder (see [QUANTLIB_REFERENCE §6](QUANTLIB_REFERENCE.md#6-architecture-analysis-what-transfers-to-rust)).
   A design native to Rust needs its own architecture.

## 3. Distinguishing commitments

| Commitment | Meaning | Status | Record |
|---|---|---|---|
| Native Rust | Idiomatic Rust abstractions. No FFI, no bindings, no C++ translation. | Owner constraint | [ADR-0001](adr/ADR-0001-native-rust.md) |
| Standard Library only | No third-party crates in the library (runtime, build, or procedural macro). Since 2026-10-27, integration tests may use allowlisted test-only crates. | Owner constraint | [ADR-0002](adr/ADR-0002-standard-library-only.md), [ADR-0012](adr/ADR-0012-test-only-dev-dependencies.md) |
| Independent implementation | Derived from mathematics, literature, market conventions, and public standards. QuantLib is a reference only. | Owner constraint | [ADR-0003](adr/ADR-0003-quantlib-compatibility.md), [ADR-0010](adr/ADR-0010-licensing.md) |
| Specification first | Every financial computation is specified in writing, with sources, before it is implemented. | Proposed | [REQUIREMENTS](REQUIREMENTS.md) |
| Explicit state | No implicit global evaluation date, no process-wide mutable settings, no implicit "today". | Proposed | [ADR-0011](adr/ADR-0011-explicit-state.md) |
| Evidence-based compatibility | Compatibility is claimed per feature and per scenario, only with recorded evidence. | Owner constraint (principle); proposed (mechanics) | [ADR-0003](adr/ADR-0003-quantlib-compatibility.md), [ADR-0008](adr/ADR-0008-validation-strategy.md) |

## 4. Engineering priorities

These priorities are an owner constraint, listed in order. When two conflict,
the earlier one wins.

1. Financial correctness.
2. Numerical accuracy and stability.
3. Memory safety.
4. Idiomatic Rust API design.
5. Deterministic and reproducible behavior.
6. Software reliability.
7. Maintainable modular architecture.
8. Extensibility.
9. Clear technical documentation.
10. Long-term open-source sustainability.
11. Performance, but only after correctness has been demonstrated.

The working shorthand is **correctness → reliability → maintainability →
performance**.

## 5. Target users

| Group | Typical need | What SabiQuant must provide |
|---|---|---|
| Quantitative finance developers (primary) | Trustworthy building blocks for pricing and analytics | Precisely specified conventions and validated numerics |
| Financial engineering researchers (primary) | Reproducible experiments | Deterministic results and no hidden global state |
| Financial software engineers (primary) | Embeddable, maintainable components | Stable API, no dependency tree, thread-safe values |
| Rust developers in finance (primary) | Idiomatic API | `Result`-based errors, enums for closed sets, documented semantics |
| University students (secondary) | Learning | Documentation that explains the finance, not only the API |
| Financial analysts (secondary) | Correct day counts and calendars | Clear names that leave no ambiguity about conventions |
| Independent quantitative researchers (secondary) | Low-friction use | Standard-library-only build and no external runtime |
| Open-source contributors (secondary) | Clear contribution rules | Specifications, ADRs, validation workflow ([CONTRIBUTING_GUIDELINES](CONTRIBUTING_GUIDELINES.md)) |

The library must be useful on its own. It must never require QuantLib or any
other runtime to be installed.

## 6. What success looks like

**Foundations (M1–M5):** A small, rigorously validated core for dates,
periods, frequencies, calendars, business-day conventions, day-count
conventions, interest rates, compounding, and rate-based discounting. Each
public computation has a written specification, sources, exhaustive or
property-based tests, and differential evidence against QuantLib 1.43 where
QuantLib implements the same behavior.

**Long term:** Progressive, evidence-backed coverage of the main QuantLib
domains: schedules, cash flows, fixed income, term structures, derivatives,
pricing engines, stochastic processes, numerical methods, and risk analytics.
The library stays understandable, deterministic, and dependency-free. Each
expansion is planned as its own milestone. See [ROADMAP](ROADMAP.md).

Functional parity is a long-term ambition, not a release requirement or a
dated promise. SabiQuant will not claim QuantLib compatibility for any feature
until that behavior has been verified independently. See
[ADR-0003](adr/ADR-0003-quantlib-compatibility.md).

## 7. Landscape (informational)

A web search on 2026-10-10 found other Rust projects in this space. The
descriptions below are taken from each project's own pages. They were not
reviewed in depth, and no code from them is used.

| Project | Self-description (summarised) | Relationship to SabiQuant |
|---|---|---|
| [RustQuant](https://docs.rs/RustQuant) | General-purpose Rust quant library (autodiff, stochastics, instruments, time); dual MIT/Apache-2.0 | Different design goals; uses third-party crates |
| [libitofin](https://docs.rs/crate/libitofin/latest) | Describes itself as a ground-up Rust port of QuantLib; BSD-3-Clause | SabiQuant is explicitly *not* a port |
| [piquette/quantlib](https://github.com/piquette/quantlib) | Early Rust port listed on QuantLib's extensions page; appears inactive | Not related |

SabiQuant's niche combines four things: a standard-library-only build,
specifications written before code, state that is always explicit, and
validation against both independent oracles and QuantLib with recorded
provenance. No claim is made that SabiQuant is better than these projects.

## 8. Vision-level non-goals

SabiQuant is **not**:

- A binding, wrapper, or FFI layer around QuantLib or any C/C++ library.
- A line-by-line translation of QuantLib.
- An API-compatible clone of QuantLib.
- A trading system, market-data service, database, or user-facing
  application.
- A library that trades correctness for speed.

The complete list of non-goals is in [SCOPE §3](SCOPE.md#3-explicit-non-goals).
