# ADR-0004: Single-Crate vs Multi-Crate Architecture

| Field | Value |
|---|---|
| ADR identifier | ADR-0004 |
| Title | Single-crate vs multi-crate architecture |
| Status | **Proposed** |
| Date | 2026-10-10 |
| Decision owner | Project owner |
| Must be decided before | M1 ([OQ-01](../IMPLEMENTATION_READINESS.md#5-decision-register)) |

## Context

SabiQuant will eventually span many financial domains. Rust allows one crate
with modules, or a Cargo workspace of several crates. Starting with too many
small crates is a known source of release and maintenance overhead. The
initial scope is four foundational modules (time, calendar, day count, rates)
plus shared error and numeric code. The project starts with a single
maintainer (A-01).

## Decision proposal

1. **One library crate named `sabiquant`.** The name was unclaimed on
   crates.io on 2026-10-10. It is not reserved, and nothing will be published
   before M5 (ADR-0009).
2. Domain modules as in [ARCHITECTURE §4](../ARCHITECTURE.md#4-proposed-crate-and-module-structure):
   `error`, internal `numeric`, `time`, `calendar`, `daycount`, `rates`. Future
   modules are added on top.
3. Boundaries are enforced through module privacy, the dependency rules in
   [ARCHITECTURE §6](../ARCHITECTURE.md#6-dependency-rules), and review.
4. **No workspace** for now. The repository root holds exactly one package
   (from M1), plus non-Rust validation tooling excluded from the package
   (ADR-0008).

## Alternatives considered

| Alternative | Description | Assessment |
|---|---|---|
| **A. Single crate with modules** | One package and one version | **Proposed.** Simplest; enough for the known scope |
| B. One crate per domain (`sabiquant-time`, `sabiquant-calendar`, …) | Fine-grained crates | Premature. Every cross-domain change spans crates; versioning coordination; more release overhead for a solo maintainer |
| C. Core crate plus a calendar-data crate | Separates code from holiday data | Worth keeping for later: holiday data may change more often than code. Premature while only one pilot calendar exists |
| D. Single crate with Cargo features for optional parts | Feature-gate large components | Compatible with A. Use only if size becomes a measured problem |

## Advantages

- One version, one changelog, one documentation site, one MSRV.
- Refactoring across domains is cheap while the API is unstable (0.x).
- The dependency graph stays empty, which fits ADR-0002.

## Disadvantages

- The compiler does not enforce module boundaries between crates. Discipline
  and review do.
- All domains share one version number. A breaking change in any module is a
  breaking change for the crate.
- Compile times grow with the codebase. This is not expected to matter at
  foundation size.

## Consequences

- Module privacy is the main encapsulation tool. Internal helpers stay
  crate-private.
- Moving to a workspace later is possible. It is a **breaking** change for
  paths unless the main crate re-exports the moved items.

## Risks

- Accidental coupling between modules. Mitigated by the dependency rules and
  review.

## Reconsideration triggers

- One component needs an **independent release cadence**. The most likely
  case is market-calendar data updated several times a year.
- **Measured** compile-time or binary-size problems that Cargo features cannot
  solve.
- An optional component would otherwise force an ADR-0002 exception on all
  users. It could instead be a separate companion crate with its own ADR.
- More maintainers join and want independent ownership of domains.

## Related requirements and documents

NFR-09, NFR-10; [ARCHITECTURE](../ARCHITECTURE.md); [ADR-0002](ADR-0002-standard-library-only.md);
[ADR-0009](ADR-0009-api-stability.md).
