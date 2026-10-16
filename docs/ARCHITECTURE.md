# Architecture

| Field | Value |
|---|---|
| Milestone | M0; in force from M1 |
| Document status | Foundational architecture accepted through the M0 ADRs on 2026-10-10; date/calendar decisions subsequently adopted for local M2/M3 under delegated authority. M5 corrections and later extensions remain local work, subject to their recorded review gates |
| Related | [DOMAIN_MODEL](DOMAIN_MODEL.md), [API_DESIGN_PRINCIPLES](API_DESIGN_PRINCIPLES.md), [ADR index](adr/README.md), [QUANTLIB_REFERENCE §6](QUANTLIB_REFERENCE.md#6-architecture-analysis-what-transfers-to-rust) |

SabiQuant is meant to be a long-lived financial domain library. The
architecture aims for the **simplest structure that meets the known
requirements** and stays extensible as the domains grow. M1–M4 foundations
are implemented and locally tested; M5 reviewed the public API and package.
M6 numerics and scoped M7 schedules/cash-flow analytics have their own local
contracts and evidence: [numerical validation](M6_NUMERICAL_VALIDATION.md),
[ICMA references](M7_ICMA_VALIDATION.md),
[cash-flow validation](M7_CASHFLOW_VALIDATION.md), and the
[modern CDS date contract](M7_CDS_CONTRACT.md). Future modules below remain
conceptual until their own contracts and evidence are recorded; formal
acceptance/platform gates remain open.

---

## 1. Architectural drivers

| Driver | Source | Architectural consequence |
|---|---|---|
| Financial correctness | C-05, NFR-01 | Specification-first; pure functions; convention parameters held in types |
| Standard library only | C-02, ADR-0002 | No dependency graph to manage; in-house numerics; std-parsable fixtures |
| Determinism | NFR-03 | No global state; no clock; no hidden caches; deterministic ordering |
| Thread safety | NFR-05 | Immutable values; all public types can be sent and shared between threads |
| No panics on invalid input | NFR-02 | `Result` everywhere; checked integer arithmetic |
| API stability | NFR-10, ADR-0009 | Small public surface; private fields; non-exhaustive enums |
| Maintainability by a solo maintainer | A-01 | One crate; clear module boundaries; few abstractions |
| Long-term growth to QuantLib-scale domains | VISION | Acyclic domain layering that leaves room for new modules |

## 2. Owner constraints that shape the architecture

Native Rust ([ADR-0001](adr/ADR-0001-native-rust.md)), standard library only
([ADR-0002](adr/ADR-0002-standard-library-only.md)), independent
implementation with QuantLib as reference only
([ADR-0003](adr/ADR-0003-quantlib-compatibility.md)), and correctness before
performance.

## 3. Alternatives evaluated

### 3.1 Single crate or multi-crate workspace ([ADR-0004](adr/ADR-0004-project-structure.md))

| Criterion | Single crate with modules | Workspace with several crates |
|---|---|---|
| Fits the initial scope (four foundational modules) | Yes | More structure than needed |
| Boundary enforcement | Module privacy (`pub(crate)`); reviewed dependency rules | Enforced by the compiler between crates |
| Versioning and releases | One version, one changelog | One version per crate, or lockstep; coordination cost |
| Std-only constraint | No effect | No effect (internal crates are not third-party) |
| Refactoring across domains | Cheap | Costly (API between crates) |
| Compile time | Fine at foundation size | Better parallelism at large size |
| Optional data-heavy parts (calendars) | Cargo features | Separate crates |

**Decision: one library crate, `sabiquant`** (OQ-01, accepted 2026-10-10; the
name was unclaimed on crates.io on 2026-10-10, and has not been reserved). Reconsideration
triggers: one component needs an independent release cadence (for example
holiday-data updates); compile times become a measured problem; or a
component must be optional for size reasons and Cargo features are not
enough.

### 3.2 Domain modules or technical layers

Modules are organized **by financial domain** (time, calendar, day count,
rates, schedules, cash flows, and later curves). They are not organized by
technical layer (models, services, utilities). The one exception is a `math`
module for numerical algorithms that several domains share (from M6). Domain
modules match the ubiquitous language and how QuantLib users look for things,
and they let each milestone add a module rather than editing every layer.

### 3.3 Public and internal API

- The public surface is the smallest that meets the requirements. Helpers are
  internal to the crate.
- Fields of invariant-carrying types are private. Construction goes through
  validating constructors.
- Each domain module exposes its own types. A crate-level convenience
  re-export or "prelude" is **deferred** until after the M5 API review, to
  avoid freezing namespace decisions early.
- Test-only support code (pseudo-random generators, ULP comparison, fixture
  reading) is not public API.

### 3.4 Shared or domain-specific types

Shared code is limited to the error conventions, the numeric validation
policy, and the machinery for canonical identifiers (FR-CORE-05/06).
Everything else lives in its domain module, including each type's identifier
table. This avoids a "common" module that grows without limit and pulls
everything into one coupling point.

### 3.5 Traits or enums

| Situation | Choice | Reason |
|---|---|---|
| Closed, market-defined sets (conventions, frequencies, compounding, units) | Enum | Exhaustive, `Copy`, no allocation, stable identifiers |
| Parameterized conventions (30E/360 ISDA, Business/252, ICMA) | Separate immutable types carrying required context | Required inputs are enforced; stateless `DayCount` remains `Copy` |
| User-extensible behavior (custom calendars) | Data-driven values first; a trait only when a concrete need is shown | A trait is a permanent API commitment |
| Numerical algorithms (local M6) | Pure functions with explicit callbacks/options; immutable interpolation/distribution values and method enums | No persistent callback object or speculative implementation trait; future extension points are decided from concrete needs |

### 3.6 Static or dynamic dispatch

Static dispatch (enum `match`, generics) is the default. Dynamic dispatch
(trait objects) is used only at explicit extension points where heterogeneous
collections are needed, and always with thread-safety bounds. Foundational
code has no performance reason to prefer either. The choice is about API
clarity.

### 3.7 Error handling

Summary of [ADR-0007](adr/ADR-0007-error-handling.md): `Result` for every
fallible operation; one non-exhaustive error enum per domain plus a
crate-level error that wraps them; structured context; no panics on invalid
input; no panicking arithmetic operators in the first release.

### 3.8 Data ownership

- Small values (dates, periods, enumerations, quotes without calendars) are
  `Copy`.
- Values holding variable-size data (calendars with holiday sets, conventions
  holding calendars) are `Clone`. Current calendars and ICMA contexts own
  their collections; cloning copies that data. Cheap clones through immutable
  `Arc` sharing are a possible measured optimization, not a current guarantee.
- Public types never use reference counting or interior mutability that is
  only safe on a single thread.
- Functions borrow inputs and return owned values. Public types have no
  lifetime parameters in H1, which keeps the API simple.

### 3.9 Immutability

All public values are immutable. "Modifying" a value (for example adding a
holiday) returns a new value. This rules out QuantLib-style shared mutation
(B-07) and makes values safe to cache and to share.

### 3.10 Thread safety and concurrency

All public types can be sent and shared between threads (NFR-05). There is no
interior mutability, no global cache, and no thread-local state. The library
starts no threads in H1. When parallel algorithms arrive (Monte Carlo, M13),
they will use std's scoped threads with deterministic work partitioning and
reduction order, so results do not depend on scheduling (NFR-03).

### 3.11 Financial state management ([ADR-0011](adr/ADR-0011-explicit-state.md))

There is no global evaluation date, settings singleton, fixing store, or
shared mutable calendar registry. The valuation date and market data are
explicit inputs. From M9, an immutable **valuation context** (valuation date,
curves, fixings, quotes) is passed explicitly to pricing functions. Scenarios
are new contexts derived from old ones. This trades QuantLib's automatic
recalculation for reproducibility, testability, and freedom from data races.
The cost is recorded as risk [R-07](IMPLEMENTATION_READINESS.md#6-risk-register).

### 3.12 Dependency direction

Dependencies point **down** the domain stack (§5). Lower modules never know
about higher ones, and there are no cycles. New domains are added on top.

### 3.13 Backward compatibility

See [ADR-0009](adr/ADR-0009-api-stability.md): semantic versioning; 0.x until
the foundations are validated; non-exhaustive enums; private fields;
deprecation before removal; and changes to numerical behavior are flagged in
release notes even when the API does not change.

### 3.14 Feature growth

- Each milestone adds a module, or extends one, without changing the modules
  below it except to add capability.
- Cargo features are used only for optional, size-heavy components such as
  large sets of market calendars, and only if size becomes a measured
  problem. There are no default-on optional features in H1.
- No build scripts and no procedural macros (std-only; simplicity).

### 3.15 Cross-platform behavior

The crate is pure Rust on std, with no code specific to an operating system or
architecture. Integer and date logic is identical on every platform.
Floating-point basic operations are IEEE 754 correctly rounded on all
supported targets. Transcendental functions are not guaranteed identical
across platforms ([NUMERICAL_CORRECTNESS §6](NUMERICAL_CORRECTNESS.md#6-determinism-and-cross-platform-reproducibility)).
Supported-platform tiers were decided in [OQ-08](IMPLEMENTATION_READINESS.md#5-decision-register)
on 2026-10-10. Primary tier: x86_64 Linux, aarch64 macOS, x86_64 Windows.
Secondary tier, best effort: aarch64 Linux, x86_64 macOS.

## 4. Proposed crate and module structure

| Module (conceptual) | Responsibility | Milestone | Public? |
|---|---|---|---|
| `error` | Crate-level error and shared error conventions | M1 | Yes |
| `numeric` | Finiteness and domain validation, stable elementary formulations, internal constants | M1 | Internal (crate-private) |
| `identifier` | Identifier tables: canonical identifiers, aliases, and ambiguous labels that are always rejected (FR-CORE-05/06). Each domain module defines the table of its own types | M1 | Internal (crate-private) |
| `time` | Date, Weekday, Month, Period, TimeUnit, Frequency, ISO text | M2 | Yes |
| `calendar` | Calendar values (null, weekends-only, custom, joint, market calendars), business-day conventions, adjustment, advancement, counting, Easter, provenance metadata | M3 | Yes |
| `daycount` | Day-count conventions, day counts, year fractions | M3 | Yes |
| `rates` | Compounding, interest-rate quote, compound and discount factors, implied and equivalent rates, rate-based discounting | M4 | Yes |
| `math` | Checked root solvers, owned interpolation/distributions and compensated/exact summation; local validation in progress | M6 | Yes |
| `schedule` | Anchored Forward/Backward, explicit coupon roles, Zero, four civil month rolls and automatic local OldCDS; requested/resolved provenance and explicit-frequency ICMA references; separate modern CDS quoted-maturity resolution and Following payment/accrual plans with guarded calendar context | M7 | Yes |
| `cashflows` | Immutable signed analytical flows/plain fixed coupons, shared complete accrual context, raw-schedule legs and flat NPV/outstanding accrual with explicit cutoff/work budget; settlement remains NI-01 | M7 | Yes |
| *Future*: `termstructures`, `indexes`, `instruments`, `pricing`, `processes`, `models`, `risk` | See [ROADMAP §4](ROADMAP.md#4-future-milestones-provisional) | M7+ | Decided per milestone |

The modern CDS surface adds `CdsMaturityConvention`, `CdsMaturityOutcome`,
`CdsMaturityResolution`, `CdsDatePlan` and `CdsCouponDates`, bringing the
then public-type catalog to 71. The pure maturity resolver uses explicit
trade/quoted-tenor/regime inputs and returns Date or Matured without calendar
adjustment. The date plan separately accepts an already resolved maturity,
retaining payment-selection cutoff, raw quarters, adjusted payments and
ordinary/final accrual roles. It has no implicit Schedule, ICMA or plain
fixed-rate-leg conversion. Coupon caps bound its storage; TARGET guards cover
the original and required extended spans. The contract is in
[M7_CDS_CONTRACT](M7_CDS_CONTRACT.md); integrated validation is tracked in
[M7_CDS_VALIDATION](M7_CDS_VALIDATION.md).

The automatic OldCDS rule adds `OldCdsProvenance`, bringing the then-current catalog to 72.
`ScheduleBuilder::build_old_cds` returns an ordinary `Schedule` whose
provenance retains the generation start, requested end, initial quarter,
recurrence anchor, resolved terminal and actually omitted quarter. Generated
ICMA contexts read the anchor from that provenance; the ordinary
`lattice_anchor` stays None. All ten rule labels are constructible
([M7_OLD_CDS_CONTRACT](M7_OLD_CDS_CONTRACT.md)). CDS pricing/protection and
settlement remain separate obligations.

`core` is deliberately **not** used as a module name. Paths that start with
`core` would become ambiguous between the local module and Rust's built-in
`core` crate.

## 5. Module dependency graph

```mermaid
flowchart BT
  error["error"]
  numeric["numeric (internal)"]
  identifier["identifier (internal)"]
  time["time"]
  calendar["calendar"]
  daycount["daycount"]
  rates["rates"]
  math["math (M6)"]
  schedule["schedule (M7)"]
  cashflows["cashflows (M7)"]
  ts["termstructures (M9)"]
  indexes["indexes (M10)"]
  instruments["instruments (M8+)"]
  pricing["pricing (M10+)"]
  processes["processes / models (M13+)"]
  risk["risk (M16)"]
  numeric --> error
  identifier --> error
  time --> error
  time --> identifier
  calendar --> time
  daycount --> calendar
  daycount --> time
  rates --> daycount
  rates --> numeric
  math --> numeric
  math --> identifier
  schedule --> calendar
  schedule --> daycount
  schedule --> time
  schedule --> identifier
  cashflows --> schedule
  cashflows --> rates
  cashflows --> math
  cashflows --> numeric
  cashflows --> time
  ts -.-> rates
  ts -.-> math
  indexes -.-> ts
  instruments -.-> cashflows
  pricing -.-> instruments
  pricing -.-> ts
  pricing -.-> indexes
  processes -.-> ts
  processes -.-> math
  risk -.-> pricing
  classDef future stroke-dasharray: 5 5;
  class ts,indexes,instruments,pricing,processes,risk future;
```

Solid edges are locally implemented foundations and analytics (M1–M7 local). Dashed edges are future and
provisional. For readability the graph draws `identifier` only below `time`;
`calendar`, `daycount` and `rates` use it in the same way for their own
identifier tables.

## 6. Dependency rules

1. A module may depend only on modules below it in §5. A dependency upwards
   or sideways that is not in the graph needs an ADR update.
2. `error` depends on nothing in the crate. Every module may use it. The
   one exception is structural: from M2 the crate-level error enumeration
   names each domain's error type in order to wrap it. It carries no
   behavior of those domains.
3. `numeric` and `identifier` are internal and contain no financial
   conventions. The identifier tables of conventions live with the
   conventions.
4. Calendar data and rules live only in `calendar`. Day-count rules live only
   in `daycount`. Date primitives live only in `time`.
5. No module reads global state, the clock, the environment, or files
   (NFR-07, NFR-13).
6. Code that exists only for tests never becomes public API.

## 7. Numerical architecture

- `f64` throughout H1 ([ADR-0006](adr/ADR-0006-numeric-policy.md)).
- Numerically sensitive formulas (compounding, implied and equivalent rates)
  are implemented once, in internal helpers, with documented stable
  formulations ([NUMERICAL_CORRECTNESS §9](NUMERICAL_CORRECTNESS.md#9-analyses-of-foundational-computations)).
- Integer date and day-count arithmetic is checked. Library correctness never
  depends on build-profile overflow checks, because a library's profile
  settings do not apply to the projects that depend on it.
- No fast-math, no reliance on floating-point contraction, and fused
  multiply-add only where a specification calls for it explicitly.

## 8. State, context, and concurrency architecture

| Concern | H1 design | Future design |
|---|---|---|
| Valuation date | Not needed by H1 operations, which are pure functions of dates | An explicit field of the valuation context (M9) |
| Market data | None in H1 | An immutable valuation context. Scenarios are derived contexts |
| Historical fixings | None in H1 | Explicit fixing history inside the context (M10) |
| Calendar customisation | New calendar values | Same |
| Caching | None | Allowed only inside explicit, owned context objects. Never process-wide. Must not change results |
| Parallelism | None | Scoped threads, deterministic partitioning and reduction |

## 9. Extension points

| Extension need | H1 mechanism | Later |
|---|---|---|
| A market calendar SabiQuant does not provide | Custom calendar (weekends plus holiday dates); joint calendars | A public calendar-rule trait, if justified (ADR) |
| A day-count convention SabiQuant does not provide | Not supported | A trait-based extension, if justified (ADR) |
| New compounding methods | Not supported (closed set) | Add variants (non-exhaustive enum) |
| Numerical callbacks | M6 solvers accept explicit `FnMut` callbacks and do not store them | Additional contracts need per-milestone justification |
| Interpolation methods | M6 immutable node values with four non-exhaustive named methods and explicit extrapolation | Add supported variants with specified boundary/shape rules |
| Pricing methods | — | Traits, designed per milestone |

## 10. Repository layout

| Path | Content | Introduced |
|---|---|---|
| `README.md`, `docs/` | Project documentation, ADRs | M0 |
| `Cargo.toml`, `Cargo.lock`, `src/` | The `sabiquant` crate, with modules as in §4. The lockfile is committed: it lists SabiQuant alone, which makes any dependency visible | M1 |
| `clippy.toml` | Lint configuration that enforces the explicit-state and no-I/O rules (§11) | M1 |
| `tests/` | Integration tests (public API, repository policies) | M1 |
| `tests/support/` | Shared test-only support: deterministic PRNG, ULP and tolerance comparison, fixture reader. Unit tests include it too | M1 |
| `tests/data/` | Small test inputs, such as the sample fixture of the reader tests | M1 |
| `.github/workflows/ci.yml` | CI: the test suite on the primary platforms, the MSRV, formatting, lints, documentation, the dependency policy, and DCO sign-off | M1 |
| `validation/fixtures/` | Committed reference fixtures (data only) | M2 |
| `validation/generator/` | External reference-generation tooling, **excluded from the published crate** ([ADR-0008](adr/ADR-0008-validation-strategy.md), OQ-13) | M2 |
| `validation/DISCREPANCIES.md` | Discrepancy log | M2 |
| Benchmark directory | Custom-harness benchmarks | Later (M6+) |
| `LICENSE-MIT`, `LICENSE-APACHE`, `CHANGELOG.md`, root `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md` | Governance files (OQ-02, OQ-03, OQ-26) | M1 |

The package allowlist includes the library, licence/readme/changelog material,
Clippy configuration and standard-library unit-test support. Integration
suites, research documents, fixtures and external validation tooling are
excluded. No release has been published.

## 11. Build configuration policy (to apply from M1)

- Rust edition 2024. The MSRV, **Rust 1.99**, is declared in the manifest
  ([OQ-04](IMPLEMENTATION_READINESS.md#5-decision-register), decided
  2026-10-10). It was the latest stable Rust when M1 started.
- The manifest declares **no dependencies** of any kind (NFR-09). A policy
  test and a CI check enforce it.
- Crate-level lint policy, in the manifest: unsafe code is forbidden (NFR-06),
  and missing documentation on public items is an error (NFR-08). Clippy
  also flags `unwrap`, `expect` and `panic!` in library code (NFR-02),
  output macros (no logging), lossy numeric casts and float equality, missing
  "Errors" and "Panics" sections, and public enums and structs that are not
  non-exhaustive (ADR-0009). Each deliberate exception carries an `#[expect]`
  attribute that states its reason.
- `clippy.toml` forbids, in library code, the types and functions that would
  bring hidden state or I/O: interior-mutability and synchronization types,
  the clock, environment access, file access and thread spawning (ADR-0011,
  NFR-07, NFR-13). Test code that reads committed fixtures says so with an
  `#[expect]` attribute. CI runs Clippy with warnings as errors.
- Toolchain components (rustfmt, Clippy, rustdoc, built-in coverage
  instrumentation) may be used as **developer tools**. They are not crate
  dependencies and do not conflict with ADR-0002.
- No build scripts, no procedural macros, no nightly features.

## 12. Architecture risks and open questions

| ID | Item | Mitigation or owner action |
|---|---|---|
| R-06 | Std-only means large in-house numerical surface (random numbers, linear algebra, special functions) | Phased roadmap; rigorous testing; ADR-0002 reconsideration triggers |
| R-07 | Explicit state may be heavy for complex, interdependent market data (curves depending on quotes) | Design the valuation context in M9 with an ADR; consider explicit dependency graphs there |
| R-08 | Concrete `f64` blocks AAD later | ADR-0006 reconsideration trigger before pricing engines |
| OQ-01 | Single crate approval | Decided 2026-10-10: one crate |
| OQ-08 | Platform tiers | Decided 2026-10-10: see §3.15 |
| OQ-18 | Calendar behavior outside verified coverage | Owner decision before M3 |

The US/AFB tranche adds `Thirty360Us`, bringing the public-type catalog to 73
while retaining 103 error variants. US keeps an explicit Boolean security EOM
qualification in its Eq/Hash context and `AccrualConvention`; no parser or
adapter supplies that qualification. AFB is a sixth stateless `DayCount`
variant. Its maximal backward-year recurrence is reduced to constant-time
integer calendar arithmetic, then one residual division and one addition.
Both contexts consume one accrual-work unit per query across the full civil
range. Independent US endpoint tables and AFB annual rollback oracles are
separate from this production derivation ([contract](M3_US_AFB_CONTRACT.md)).
