# SabiQuant

**SabiQuant** is a planned, independent quantitative finance library written
natively in Rust. Over time it aims to progressively reach functional parity
with the major domains of [QuantLib](https://github.com/lballabio/QuantLib),
with an idiomatic Rust API, financially correct behavior, and rigorously
validated numerics.

> **Project status: M1 is merged; M2–M4 foundations are implemented locally.**
> M3 includes immutable calendars, guarded TARGET coverage, Easter and eight
> day counts; M4 adds explicit quotes, compounding, factors and single-amount
> present values. US 30/360, AFB and further pricing remain later work. This is an
> unreleased development version awaiting milestone review.

## What SabiQuant aims to become

A reusable, long-lived Rust library for quantitative finance. It starts from
rigorously specified foundations:

- dates, periods and frequencies;
- calendars and business-day conventions;
- day-count conventions;
- interest rates, compounding, and discounting.

From there it grows towards schedules, cash flows, bonds, term structures,
derivatives, pricing engines, stochastic processes, numerical methods, and
risk analytics. See the [vision](docs/VISION.md) and the
[roadmap](docs/ROADMAP.md). Functional parity with QuantLib is a long-term
ambition, not a promise. No compatibility is claimed until it is verified
independently ([ADR-0003](docs/adr/ADR-0003-quantlib-compatibility.md)).

## Principles

- **Rust-native.** Designed around ownership, value semantics, enums, traits,
  and explicit `Result`-based errors. Not a binding to QuantLib and not a
  translation of it ([ADR-0001](docs/adr/ADR-0001-native-rust.md)).
- **Standard Library only.** The current dependency policy allows no
  third-party crates of any kind: runtime, dev, build, or procedural macro
  ([ADR-0002](docs/adr/ADR-0002-standard-library-only.md)).
- **QuantLib is a reference, not a dependency.** QuantLib is used to study
  behavior and to compare numerical results. It is never needed to build, run,
  or test SabiQuant ([ADR-0008](docs/adr/ADR-0008-validation-strategy.md)).
- **Correctness before performance.** The order is correctness, then
  reliability, then maintainability, then performance
  ([numerical policy](docs/NUMERICAL_CORRECTNESS.md)).
- **Explicit state.** No global evaluation date, no singletons, no implicit
  "today" ([ADR-0011](docs/adr/ADR-0011-explicit-state.md)).

## Decisions

The native-Rust, Standard-Library-only, and behavioral-compatibility
principles are constraints set by the project owner. On 2026-10-10 the owner
accepted the architecture decisions that M1 needs and authorized M1
([ADR index](docs/adr/README.md),
[decision register](docs/IMPLEMENTATION_READINESS.md#5-decision-register)).
Later decisions are recorded as each locally authorized batch is prepared.

On 2026-10-11 the owner delegated the next implementation choices and asked
for continuous progress with broad tests. M2 decisions and its authorization
are recorded in [M2_READINESS](docs/M2_READINESS.md); validation evidence and
remaining acceptance gates are in [M2_VALIDATION](docs/M2_VALIDATION.md).
The next local batch is recorded in [M3_LOCAL_READINESS](docs/M3_LOCAL_READINESS.md)
and [M3_CORE_VALIDATION](docs/M3_CORE_VALIDATION.md). Local M4 decisions and
numerical evidence are in [M4_LOCAL_READINESS](docs/M4_LOCAL_READINESS.md)
and [M4_NUMERICAL_VALIDATION](docs/M4_NUMERICAL_VALIDATION.md). External writes are not
authorized: the owner selected continued work in the local repository.

## Documentation

| Document | Purpose |
|---|---|
| [VISION](docs/VISION.md) | Mission, users, priorities, landscape |
| [SCOPE](docs/SCOPE.md) | Scope horizons, non-goals, scope control |
| [REQUIREMENTS](docs/REQUIREMENTS.md) | Constraints, non-functional requirements, functional specifications |
| [QUANTLIB_REFERENCE](docs/QUANTLIB_REFERENCE.md) | Pinned QuantLib 1.43 baseline and architecture analysis |
| [FEATURE_MATRIX](docs/FEATURE_MATRIX.md) | Feature inventory with status, priority, milestone, and validation approach |
| [ARCHITECTURE](docs/ARCHITECTURE.md) | Crate and module architecture |
| [DOMAIN_MODEL](docs/DOMAIN_MODEL.md) | Domain concepts, glossary, typing strategy |
| [API_DESIGN_PRINCIPLES](docs/API_DESIGN_PRINCIPLES.md) | Public API rules |
| [NUMERICAL_CORRECTNESS](docs/NUMERICAL_CORRECTNESS.md) | Floating-point policy, tolerances, invariants |
| [TESTING_STRATEGY](docs/TESTING_STRATEGY.md) | Testing with the standard library only, and differential validation |
| [ROADMAP](docs/ROADMAP.md) | Milestones M0–M5 in detail, then provisional M6–M16 |
| [CONTRIBUTING_GUIDELINES](docs/CONTRIBUTING_GUIDELINES.md) | Contribution, review, release and governance |
| [LICENSING_AND_ATTRIBUTION](docs/LICENSING_AND_ATTRIBUTION.md) | Licence analysis and attribution policy |
| [IMPLEMENTATION_READINESS](docs/IMPLEMENTATION_READINESS.md) | Readiness gates, decision register, risk register |
| [M2_READINESS](docs/M2_READINESS.md) | M2 authorization, delegated decisions and prerequisite CI evidence |
| [M2_VALIDATION](docs/M2_VALIDATION.md) | M2 coverage, test evidence and remaining acceptance gates |
| [M3_LOCAL_READINESS](docs/M3_LOCAL_READINESS.md) | Staged local M3 scope and delegated engineering decisions |
| [M3_CORE_VALIDATION](docs/M3_CORE_VALIDATION.md) | Core calendar/day-count evidence |
| [M3_EXTENSION_VALIDATION](docs/M3_EXTENSION_VALIDATION.md) | Easter/TARGET and contextual day-count evidence and remaining gates |
| [M4_LOCAL_READINESS](docs/M4_LOCAL_READINESS.md) | Delegated rate-domain, compounding and numerical decisions |
| [M4_NUMERICAL_VALIDATION](docs/M4_NUMERICAL_VALIDATION.md) | Independent high-precision rate evidence and conditioning analysis |
| [M4_REFERENCE_VALIDATION](docs/M4_REFERENCE_VALIDATION.md) | Attested public QuantLib comparisons and classified differences |
| [M4_DATE_CONTRACT](docs/M4_DATE_CONTRACT.md) | Owned day-count contexts, date conversion and valuation contracts |
| [M4_BOUNDARY_VALIDATION](docs/M4_BOUNDARY_VALIDATION.md) | Conservative numerical range policy and exact boundary anchors |
| [ADRs](docs/adr/README.md) | Architecture Decision Records ADR-0001 to ADR-0011 |

## Building and testing

SabiQuant needs Rust 1.99 or later and nothing else: it has no dependencies.

```sh
cargo test                                  # unit, integration and documentation tests
cargo clippy --all-targets -- -D warnings   # lints, as run in CI
cargo fmt --all --check                     # formatting
cargo doc --no-deps --open                  # API documentation
```

`sabiquant::error` provides typed errors and `sabiquant::time` provides checked
Gregorian dates, tenors, weekdays, months and frequencies.
`sabiquant::calendar` provides immutable core calendars and checked business-day
operations, Gregorian Easter and TARGET with strict source coverage by default.
`sabiquant::daycount` provides five stateless conventions plus explicit-period
ICMA, termination-sensitive 30E/360 ISDA and explicit-calendar Business/252.
`sabiquant::rates` provides five compounding methods, explicit frequency/context
quotes, compound and discount factors, implied/equivalent rates, single-amount
present values and named percent/basis-point conversions. Reference tests
read committed fixtures offline; regenerating them is documented in
[validation](validation/README.md).

## Relationship to QuantLib

SabiQuant uses [QuantLib](https://www.quantlib.org/) (BSD 3-Clause) as an
external behavioral and numerical reference. The current baseline is QuantLib
1.43. SabiQuant contains no QuantLib code. **SabiQuant is not affiliated with
or endorsed by the QuantLib project.**

## Licence

SabiQuant is licensed under either of

- the [Apache License, Version 2.0](LICENSE-APACHE), or
- the [MIT license](LICENSE-MIT),

at your option ([ADR-0010](docs/adr/ADR-0010-licensing.md)).

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in SabiQuant by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) first: it
covers the Developer Certificate of Origin sign-off, the provenance rules, and
the rules for code. Everyone who takes part follows the
[Code of Conduct](CODE_OF_CONDUCT.md). Report security problems privately as
described in [SECURITY.md](SECURITY.md).
