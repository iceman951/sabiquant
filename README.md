# SabiQuant

**SabiQuant** is a planned, independent quantitative finance library written
natively in Rust. Over time it aims to progressively reach functional parity
with the major domains of [QuantLib](https://github.com/lballabio/QuantLib),
with an idiomatic Rust API, financially correct behavior, and rigorously
validated numerics.

> **Project status: Milestone M0 (research, specification, architecture and
> planning).**
> **There is no usable implementation.** The repository contains planning
> documentation only. Nothing here may be used for financial calculations.

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
  "today" (proposed, [ADR-0011](docs/adr/ADR-0011-explicit-state.md)).

## Decisions are still under review

The native-Rust, Standard-Library-only, and behavioral-compatibility
principles are constraints set by the project owner. **Every other
architectural decision is a proposal that has not yet been approved**, and is
recorded as a *Proposed* ADR ([ADR index](docs/adr/README.md)). Open
questions and the formal readiness gates are in
[IMPLEMENTATION_READINESS](docs/IMPLEMENTATION_READINESS.md). Implementation
starts only after the owner approves it explicitly.

## Documentation

| Document | Purpose |
|---|---|
| [VISION](docs/VISION.md) | Mission, users, priorities, landscape |
| [SCOPE](docs/SCOPE.md) | Scope horizons, non-goals, scope control |
| [REQUIREMENTS](docs/REQUIREMENTS.md) | Constraints, non-functional requirements, functional specifications |
| [QUANTLIB_REFERENCE](docs/QUANTLIB_REFERENCE.md) | Pinned QuantLib 1.43 baseline and architecture analysis |
| [FEATURE_MATRIX](docs/FEATURE_MATRIX.md) | Feature inventory with status, priority, milestone, and validation approach |
| [ARCHITECTURE](docs/ARCHITECTURE.md) | Proposed crate and module architecture |
| [DOMAIN_MODEL](docs/DOMAIN_MODEL.md) | Domain concepts, glossary, typing strategy |
| [API_DESIGN_PRINCIPLES](docs/API_DESIGN_PRINCIPLES.md) | Public API rules |
| [NUMERICAL_CORRECTNESS](docs/NUMERICAL_CORRECTNESS.md) | Floating-point policy, tolerances, invariants |
| [TESTING_STRATEGY](docs/TESTING_STRATEGY.md) | Testing with the standard library only, and differential validation |
| [ROADMAP](docs/ROADMAP.md) | Milestones M0–M5 in detail, then provisional M6–M16 |
| [CONTRIBUTING_GUIDELINES](docs/CONTRIBUTING_GUIDELINES.md) | Contribution, review, release and governance (proposed) |
| [LICENSING_AND_ATTRIBUTION](docs/LICENSING_AND_ATTRIBUTION.md) | Licence analysis and attribution policy |
| [IMPLEMENTATION_READINESS](docs/IMPLEMENTATION_READINESS.md) | Readiness gates, decision register, risk register |
| [ADRs](docs/adr/README.md) | Architecture Decision Records ADR-0001 to ADR-0011 |

## Relationship to QuantLib

SabiQuant uses [QuantLib](https://www.quantlib.org/) (BSD 3-Clause) as an
external behavioral and numerical reference. The current baseline is QuantLib
1.43. SabiQuant contains no QuantLib code. **SabiQuant is not affiliated with
or endorsed by the QuantLib project.**

## Licence

**No licence has been selected yet.** The decision is pending with the project
owner ([ADR-0010](docs/adr/ADR-0010-licensing.md)). Until a licence is chosen,
no licence file exists and default copyright applies.

## Contributing

External code contributions cannot be accepted until a licence is selected
and implementation is authorized. Feedback on the planning documents, and
primary sources for financial conventions, are welcome through issues. See
[CONTRIBUTING_GUIDELINES](docs/CONTRIBUTING_GUIDELINES.md).
