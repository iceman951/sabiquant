# Public API Design Principles

| Field | Value |
|---|---|
| Milestone | M0 |
| Document status | **Adopted**; applied from M1 (the governing ADR-0007 and ADR-0009 were accepted on 2026-10-10) |
| Related | [ARCHITECTURE](ARCHITECTURE.md), [DOMAIN_MODEL](DOMAIN_MODEL.md), [ADR-0007](adr/ADR-0007-error-handling.md), [ADR-0009](adr/ADR-0009-api-stability.md) |

These principles govern every public item SabiQuant adds. They build on the
[Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) and the
[Cargo SemVer compatibility guide](https://doc.rust-lang.org/cargo/reference/semver.html),
adapted to financial software. Concrete APIs are designed in each milestone.
This document deliberately contains no signatures.

---

## 1. Principles at a glance

| # | Principle |
|---|---|
| P1 | **Correct by construction.** Invalid financial states cannot be built. |
| P2 | **Explicit over implicit.** No global defaults, no implicit "today", no default conventions. |
| P3 | **Fallibility is visible.** Any operation that can fail returns `Result`. |
| P4 | **Values, not objects.** Immutable, comparable, thread-safe values. |
| P5 | **The finance reads clearly at the call site.** Descriptive enums instead of booleans; full names. |
| P6 | **One concept, one name.** The same term means the same thing everywhere ([DOMAIN_MODEL §2](DOMAIN_MODEL.md#2-ubiquitous-language)). |
| P7 | **Small public surface.** Public only what is needed; helpers stay private. |
| P8 | **Documented semantics.** Every item states its definition, formula, units, errors and sources. |
| P9 | **Stable by design.** Non-exhaustive enums, private fields, deprecation before removal. |
| P10 | **No surprises in numbers.** No silent rounding, no percent inputs, no sentinel values. |

## 2. Naming conventions

1. Follow Rust's casing conventions: types and enum variants in UpperCamelCase,
   functions and modules in snake_case, constants in SCREAMING_SNAKE_CASE.
2. Follow the API Guidelines' conversion vocabulary: `as_` for cheap borrowed
   views, `to_` for computed conversions, `into_` for consuming conversions,
   and `from_` / `try_from_` for constructors. Getters are not prefixed with
   `get_`.
3. Use **full financial words** in code identifiers: year fraction, discount
   factor, business day, compound factor. Market abbreviations (A/360, 30E/360)
   appear only in canonical identifier strings and documentation.
4. **Convention names must not be ambiguous.** There is no bare "Thirty360"
   or "ActualActual". Each variant names its exact convention, for example
   Actual/Actual (ISDA) as opposed to Actual/Actual (ICMA).
5. Date-interval parameters are named **start** and **end**, in chronological
   meaning, everywhere. Never `d1`/`d2`.
6. Verbs are consistent across modules: *adjust*, *advance*, *year fraction*,
   *day count*, *compound factor*, *discount factor*, *implied rate*,
   *equivalent rate*, *is business day*.
7. Where a QuantLib user would search for a QuantLib name, the documentation
   includes that name as a rustdoc search alias. This does not create API
   compatibility; it only helps people find things.

## 3. Units and quantities

- Rates are annual decimal fractions. 0.05 means 5%.
- Times are year fractions computed by a named day-count convention.
- Amounts are plain `f64` in H1, with no currency ([SCOPE](SCOPE.md)).
- Percent and basis-point conversions are explicit, named operations. No API
  accepts percentages.
- Scalar wrapper types are an open question
  ([OQ-24](IMPLEMENTATION_READINESS.md#5-decision-register),
  [DOMAIN_MODEL §6.4](DOMAIN_MODEL.md#64-scalar-wrapper-types-oq-24)).

## 4. Type and signature principles

1. **Make invalid states unrepresentable where it is cheap.** Required
   convention parameters are part of the convention value. Periodic
   compounding carries its frequency.
2. **Validate at the boundary.** A validating constructor guards every value
   with invariants. Once built, a value is valid for its whole lifetime.
3. **Enumerations instead of booleans.** End-of-month handling, inclusion of
   start and end dates when counting, and the joint-calendar rule are
   two-valued enumerations with descriptive variant names.
4. **Borrow inputs, return owned values.** No lifetimes in public H1 types.
5. **Minimise bare `f64` parameters.** Prefer a typed value (a quote, a
   convention) plus at most one bare scalar, to limit argument mix-ups.
6. **Keep argument order consistent.** Calendar operations take the date
   first and then the options. Interval operations take start before end.
7. **No defaults for conventions.** There is no default day-count convention,
   calendar, compounding, or business-day convention. QuantLib's defaults
   (for example Business/252 defaulting to the Brazil calendar) are not
   copied.
8. **No "null" values.** Use an optional value when something may be absent.

## 5. Error semantics

Summarizing [ADR-0007](adr/ADR-0007-error-handling.md):

- Every fallible operation returns `Result` with a **domain error type**.
- Error types are non-exhaustive enumerations. Each variant has structured
  context, such as the offending date components or the parameter name and
  value.
- A crate-level error wraps the domain errors, so cross-domain code can
  propagate any failure with `?`.
- All error types implement `std::error::Error`, `Display`, and `Debug`, and
  are `Clone` and `PartialEq` so tests can match them precisely.
- **Message wording is not part of the stable API.** Variants and their data
  are.
- **Kind names.** Every error type has a `kind` method that returns the
  stable kind name of the variant, of the form `<domain>.<variant>` in
  lowercase ASCII letters and underscores, for example `value.not_finite` or
  `identifier.ambiguous`. Fixtures state expected errors with these names. A
  kind name never changes meaning and is never reused.
- The crate-level error is **transparent**: its message and its
  `Error::source` are those of the domain error it wraps. A domain error that
  was caused by another error returns that error from `source`.
- Optional values express absence (for example, a period that maps to no
  frequency). Errors express invalid input or domain violations.
- **No panics on any input.** If a public function can panic because of an
  internal defect, its documentation says so in a "Panics" section. The goal
  is that no such section is needed.
- The first release has no panicking arithmetic operators. Date arithmetic is
  provided as checked operations
  ([OQ-11](IMPLEMENTATION_READINESS.md#5-decision-register)).

## 6. Method discoverability

- An operation lives on the type that owns the concept. For example,
  adjustment is a calendar operation, and year fraction is a day-count
  convention operation.
- Symmetric operations involving two equal-ranking values may be free
  functions in the owning module.
- Each module's documentation starts with a short overview: the concepts it
  provides, links to the specification sections, and a table mapping QuantLib
  names to SabiQuant names.
- Error types sit next to the operations that produce them.

## 7. Standard trait policy

| Trait | Policy |
|---|---|
| `Debug` | All public types |
| `Clone` | All public types |
| `Copy` | Small values with no heap data (dates, periods, enumerations, quotes without calendars) |
| `PartialEq` / `Eq` / `Hash` | Discrete types (dates, periods with normalized semantics, enumerations, calendars by definition). Types holding `f64` implement `PartialEq` only, and the docs explain the semantics |
| `PartialOrd` / `Ord` | Dates are totally ordered. Periods are partially ordered (FR-PER-02). No ordering for types holding `f64` unless it has a defined financial meaning |
| `Display` / `FromStr` | Canonical identifiers, ISO dates, period text. Locale-independent |
| `Default` | Only where a financially neutral default exists. **Never** for conventions, calendars, or quotes |
| `Send` / `Sync` | Required for all public types (NFR-05) |
| `std::error::Error` | All error types |

## 8. Extension points

Extension mechanisms are listed in [ARCHITECTURE §9](ARCHITECTURE.md#9-extension-points).
A public trait that users are not meant to implement is **sealed**, so new
methods can be added without breaking anyone. Opening a sealed trait needs an
ADR.

## 9. API stability

- Semantic versioning per the Cargo SemVer guide
  ([ADR-0009](adr/ADR-0009-api-stability.md)).
- Enumerations and structs that may grow are non-exhaustive.
- Fields are private. Construction and inspection go through methods.
- Deprecation comes before removal. Deprecated items name their replacement.
- **Changes to numerical behavior are part of the API contract.** Any change
  that alters numerical output, even a bug fix, is listed under a "Behavior
  changes" heading in release notes, with affected conventions and typical
  magnitudes.

## 10. Documentation requirements

Every public item's documentation has these sections:

| Section | Content |
|---|---|
| Summary | One line |
| Definition | Financial meaning and formula in mathematical notation |
| Conventions and units | Day count, compounding, units, interval conventions |
| Preconditions | Valid input domain |
| Errors | Each error variant and when it occurs |
| Panics | "None", unless justified |
| Examples | From M2 on, compiled and run as documentation tests. **Not in M0** |
| References | Primary sources, plus the specification section ID |
| QuantLib correspondence | The equivalent in QuantLib 1.43 and **known behavioral differences** |
| Numerical notes | Accuracy, tolerance class, stability considerations, where relevant |

## 11. Financial precision and presentation

- No API rounds silently. Market rounding conventions are a separate,
  explicit feature (F-CORE-07, Deferred).
- Floating-point output is formatted with Rust's shortest round-trip decimal
  representation, so printed values parse back to the same value.
- Parsing and formatting are locale-independent.

## 12. Anti-patterns to avoid

| Anti-pattern | Why it is avoided | SabiQuant alternative |
|---|---|---|
| Global settings or singletons | Hidden coupling; non-reproducible; races | Explicit context ([ADR-0011](adr/ADR-0011-explicit-state.md)) |
| Conventions given as strings in core APIs | Typos and ambiguity found only at runtime | Enumerations, with parsing only at the edges |
| Boolean parameters | Call sites unreadable | Two-valued enumerations |
| Sentinel or null values | Silent misuse | Optional values |
| Default conventions | Silent wrong results | Explicit choice required |
| Mutating shared values | Action at a distance | New values |
| Panicking on invalid input | Crashes in production | `Result` |
| Implicit "today" | Non-reproducible | Explicit valuation date |
| Percent inputs | Factor-of-100 errors | Decimals, with explicit conversions |
| One global epsilon | Unjustified tolerances | Per-quantity tolerance policy |
