# ADR-0002: Standard Library-Only Dependency Policy

| Field | Value |
|---|---|
| ADR identifier | ADR-0002 |
| Title | Standard Library-only dependency policy |
| Status | **Accepted — owner constraint** (from the M0 brief). D2–D4 (what is not a dependency, the approach for each area, and the exception process) **Accepted** on 2026-10-10 ([decision record](#decision-record)). **Partially superseded by [ADR-0012](ADR-0012-test-only-dev-dependencies.md)** on 2026-10-27 for development dependencies and developer tools only; the runtime and build constraint is unchanged |
| Date | 2026-10-10 |
| Decision owner | Project owner |

## Context

The owner's brief sets a hard constraint: SabiQuant may use only the Rust
Standard Library. No third-party dependency of any kind is allowed: runtime,
development, test, build, or procedural macro. Commonly used crates such as
`chrono`, `time`, `nalgebra`, `ndarray`, `rand`, `serde`, `thiserror`,
`anyhow`, and `statrs` are excluded, and that list is not exhaustive.

## Decision proposal

**D1 (owner constraint).** The `sabiquant` package manifest declares **no
dependencies**: no `[dependencies]`, `[dev-dependencies]`, or
`[build-dependencies]` entries, and no procedural-macro crates. NFR-09 makes
this checkable in every pull request.

**D2 (proposed). What is not a dependency.**

- *Rust toolchain components* (rustc, Cargo, rustdoc, rustfmt, Clippy, and the
  built-in coverage instrumentation) are developer tools, not dependencies.
- *External research and reference tools* run outside the crate, for example
  QuantLib-Python for generating reference fixtures, or Python's `fractions`
  and `decimal` modules for oracle values. They are allowed if they are never
  needed to build or test the crate. Tests use only committed data files
  (ADR-0008).

**D3 (proposed). How the constraint affects each area, and the chosen
approach.**

| Area | Limitation | Possible approaches | Recommended approach |
|---|---|---|---|
| Civil dates | std has no calendar dates (`SystemTime` is an instant) | In-house serial-day algorithms | Public-domain civil-date algorithms ([Hinnant](http://howardhinnant.github.io/date_algorithms.html)), with exhaustive tests (ADR-0005) |
| Time zones and "today" | std has no time-zone database | Embed time-zone data; accept UTC only; do not provide it | **Do not provide.** Valuation dates are explicit (ADR-0011) |
| Random numbers | No RNG in std | In-house PRNGs (SplitMix64, MT19937, …), quasi-random sequences | Test-only PRNG in M1; production generators in M13 with published algorithms. Sobol direction-number licensing to be checked |
| Linear algebra | No matrices in std | In-house dense routines | M12: only the decompositions that are needed, validated against oracles |
| Special functions | `erf`, `gamma` are nightly-only | In-house implementations of published algorithms | M6: normal CDF and inverse with documented accuracy targets |
| Serialization (fixtures) | No `serde` | Hand-written JSON parser; simple text format | Line-oriented text with tab-separated records (ADR-0008, OQ-13) |
| Error boilerplate | No derive helpers | Manual implementations | Manual `Display` and `std::error::Error` implementations (ADR-0007) |
| Property testing | No `proptest` or `quickcheck` | In-house generators with fixed seeds | Test-only PRNG and input generators, no shrinking ([TESTING_STRATEGY §5](../TESTING_STRATEGY.md#5-property-based-testing)) |
| Benchmarking | No `criterion`; `#[bench]` is unstable | Custom harness | `std::time::Instant` and `std::hint::black_box` ([TESTING_STRATEGY §13](../TESTING_STRATEGY.md#13-performance-benchmarking-strategy-future)) |
| Exact decimals | No decimal type | In-house fixed-point integers (`i128` exists) or a decimal type | Deferred (NI-01) |
| Parallelism | No `rayon` | `std::thread` scoped threads | Deterministic partitioning when needed (M13) |

**D4 (proposed). Exception process.** A relaxation of any part of this policy
needs:

1. A **new ADR** that states the specific crate, version, scope (runtime,
   dev, or build), licence, transitive dependencies, maintenance status, and
   security record.
2. Evidence that the in-house alternative carries **more** correctness or
   maintenance risk than the dependency.
3. **Explicit approval from the project owner**, recorded in the ADR.

Until all three exist, the constraint stands.

## Alternatives considered

| Alternative | Why not chosen |
|---|---|
| Allow well-known crates freely | Breaks the owner constraint |
| Allow only dev-dependencies (for example property testing) | Breaks the owner constraint, which covers test and dev dependencies too. Remains possible later through D4 |
| Vendor (copy) third-party source into the repository | Still a third-party dependency in substance; adds licence obligations; against the spirit of the constraint |

## Advantages

- No supply-chain exposure through crates. Builds are reproducible and
  auditable.
- No version conflicts for users, and a minimal build footprint.
- Every line of numerical code is owned, specified, and tested by SabiQuant.

## Disadvantages

- A large in-house surface over time: random numbers, linear algebra, special
  functions, parsing.
- Slower delivery of advanced domains.
- Testing ergonomics are poorer: no shrinking in property tests, and no
  statistical benchmark tooling.

## Consequences

- The roadmap adds a numerical-foundations milestone (M6) before curves and
  options, and another (M12) before calibration.
- The fixture format is designed to be parseable with std alone.
- Each in-house algorithm cites its published source and is validated against
  an oracle.

## Risks

- R-06: subtle defects in in-house numerical code. Mitigated by oracle
  testing and published algorithms.
- R-11: maintenance load on a solo maintainer.

## Reconsideration triggers

- An in-house component shows a defect rate or maintenance cost that the
  maintainer cannot sustain.
- A domain needs a capability whose correct implementation is unreasonable
  in-house (for example, a large special-function library for advanced
  models).
- Users ask for interoperability that std cannot reasonably provide, such as
  serialization formats. This could be met by **separate companion crates**
  outside the core crate, which would themselves need an ADR.

Each trigger leads to an ADR under D4, never a silent change.

## Related requirements and documents

C-02, NFR-09; [ARCHITECTURE §11](../ARCHITECTURE.md#11-build-configuration-policy-to-apply-from-m1);
[TESTING_STRATEGY](../TESTING_STRATEGY.md); [NUMERICAL_CORRECTNESS §10](../NUMERICAL_CORRECTNESS.md#10-future-numeric-representation-issues).

## Decision record

| Date | Decision | Approval |
|---|---|---|
| 2026-10-10 | D2–D4 accepted as proposed. Toolchain components (rustfmt, Clippy, rustdoc) and external reference tools are developer tools, not dependencies. The table in D3 is the approach for each area. Any relaxation follows the D4 exception process | Project owner, when authorizing M1 ([IMPLEMENTATION_READINESS §5](../IMPLEMENTATION_READINESS.md#5-decision-register)) |
| 2026-10-27 | [ADR-0012](ADR-0012-test-only-dev-dependencies.md) supersedes D1, D2 and D4 for development dependencies and developer tools only: allowlisted test-only crates as `[dev-dependencies]`, and `cargo-deny` and `cargo-nextest` in CI. The library's runtime and build constraint, and the rest of this ADR, are unchanged | Project owner |
