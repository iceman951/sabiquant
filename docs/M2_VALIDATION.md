# M2 Validation Report

| Field | Value |
|---|---|
| Date | 2026-10-11 |
| Scope | Dates, months, weekdays, periods, frequencies and crate error integration |
| Authorization and decisions | [M2_READINESS](M2_READINESS.md) |
| Local platform | aarch64 macOS, Rust 1.99.0 |
| Reference environment | Python 3.9.6; actual QuantLib-Python 1.43 execution in an external temporary environment |
| Status | Local checks passed; primary-platform CI and owner milestone acceptance remain required |

## Evidence and coverage scope

The suite checks mathematical contracts, boundary behavior, trait laws and
independent reference values. It does not equate the number of test functions
with the number of inputs checked, or code coverage with proof of correctness.

| Contract | Evidence |
|---|---|
| TS-DATE-01/02/06, INV-DATE-1…4 | Every one of the 3,652,059 dates is checked against independently advanced civil and weekday counters: component conversion, signed differences, ordering, day-of-year, month ends, strict ISO display/parse and adjacent days |
| TS-DATE-03 | Unsupported component extremes, every year's/month's first invalid day, leap-day failures, error kind/context and deterministic validation precedence |
| TS-DATE-04 | MIN/MAX transitions; i64 MIN/MAX shifts; 25,000 seeded date campaigns compared with i128 range and target-month calculations |
| TS-DATE-05, INV-DATE-5 | All 119,988 supported month ends: clamping and year/month equivalence; independent Python and actual QuantLib arithmetic fixtures; non-associativity and non-invertibility examples |
| TS-DATE-07 | ISO round-trip for every valid date; ASCII-byte mutations, inserted Unicode/control/whitespace, 50,000 arbitrary ASCII strings and 100 KB malformed inputs |
| FR-DATE-10 | N-th weekday occurrences scanned over a complete 400-year cycle, missing fifth occurrences, inclusive previous/next queries and range limits |
| TS-PER-01, INV-PER-1/2 | Small exhaustive length/unit grids, normalization idempotence, equality/hash consistency, ordering reversal/substitutability/transitivity, canonical-zero family and arbitrary i32 cases modeled with i128 |
| TS-PER-02, INV-PER-4 | Every frequency maps both ways; actual QuantLib Frequency-to-Period values are compared; Once, zero, negative tenors and equivalent day/week forms have explicit discrepancy cases |
| TS-PER-03, INV-PER-3 | Compatible/incompatible arithmetic, exact cancellation before overflow, i32 boundary grids, 30,000 deterministic integer campaigns, chosen-output-unit overflow context and representation-dependent success regressions |
| FR-PER-04 | Stored text round-trips, optional sign/leading zeros, upper/lower units, every two-byte ASCII input, invalid Unicode/composite/long inputs |
| FR-PER-05, date/period integration | Equivalent normalized tenors and zero dispatch over 479,952 selected dates near every supported month's boundaries; extreme lengths and subtraction |
| TS-CORE-01, NFR-02/05/08/09 | Every domain error kind/context, typed source downcasting, failing formatter propagation, public Send/Sync/value traits, docs/lints and zero Cargo dependencies |

Test entry points are [time_validation.rs](../tests/time_validation.rs),
[time_adversarial.rs](../tests/time_adversarial.rs),
[time_arithmetic_semantics.rs](../tests/time_arithmetic_semantics.rs),
[public_api.rs](../tests/public_api.rs), and unit tests in
[date.rs](../src/time/date.rs) and [period.rs](../src/time/period.rs).

## External reference evidence

Seven committed fixture sets contain **159,947 rows**. They include actual
QuantLib date accessor results for all **109,208** days in 1901–2199, Python
date references for every supported year's leap boundaries, arithmetic and
period comparisons. Normal `cargo test` reads them offline with the existing
std-only schema reader. Generation and integrity-check commands are in
[validation/README](../validation/README.md).

Known differences are recorded in [DISCREPANCIES](../validation/DISCREPANCIES.md):
the wider date range, frequency mapping of negative/zero/normalized tenors,
Once having no period, and partial ordering across unit families. Reference
checks validate actual QuantLib outputs as well as the independent expected
values. The generators reject unexpected differences; tests require the
recorded classification and exact known discrepancy inputs/counts. A frequency
discrepancy never disables normalization comparison.

Fixture provenance records the executed reference version, generator revision,
environment, wheel SHA-256 and installed-package attestation. The pinned
QuantLib source commit is labeled as a declared source baseline, separately
from the observed wheel/runtime identity. It is not claimed to be proven by
the wheel version string.

## Native coverage measurement

The reusable [coverage runner](../validation/coverage.py) uses Rust's own
LLVM tools and `-C instrument-coverage`, with no Cargo dependencies. It runs
unit and integration executables and emits JSON/HTML. Doctests run separately
in the normal test suite. Its production-region summary excludes embedded
`#[cfg(test)]` modules and unions regions across generic instantiations.

The final debug suite passes **135 tests**: 70 unit tests, 62 integration tests
and 3 doctests, with no failures or ignored tests. Release passes **134 tests**;
one internal-invalid-interval assertion test exists only with debug assertions.
Formatting, Clippy with warnings denied, public/private rustdoc,
the dependency policy and complete regeneration checks of all seven fixture
sets pass on the declared Rust 1.99 toolchain.

Production code-region coverage, excluding embedded unit tests:

| File | Covered / reported regions |
|---|---|
| `src/time/date.rs` | 520 / 522 |
| `src/time/period.rs` | 301 / 301 |
| `src/time/mod.rs` | 37 / 37 |
| M2 time module total | **858 / 860 (99.77%)** |
| `src/error.rs` | 244 / 247 |
| `src/identifier.rs` | 27 / 31 |
| `src/numeric.rs` | 38 / 38 |

The two unexecuted date regions are error propagation after revalidating
already bounded components in month arithmetic and n-th-weekday construction.
No valid public input can trigger those constructor errors. Shared helper
coverage also contains compile-time-evaluated paths and formatting regions;
the full LLVM report counts embedded unit-test code as well. It reports
97.59% line coverage across library source files including that test code;
this differs from the production-region metric above.

Region/function/line coverage does not prove all inputs, branches or financial
conventions correct. LLVM branch counts are not available in this stable-Rust
configuration. Invariant-protected error paths and compile-time-evaluated code
are identified separately rather than forcing artificial runtime cases.

## Remaining acceptance gates

- The changed code must pass the primary Linux/macOS/Windows CI matrix.
  The successful M1 run supports the prerequisite only, not these local changes.
- A pull request and owner milestone acceptance are needed before M2 is marked
  Complete under [ROADMAP §2](ROADMAP.md#2-definition-of-done-for-implementation-milestones).
- Dates beyond QuantLib's range have independent oracle evidence, not
  QuantLib differential evidence. Serial-number interop remains deferred.
- Calendars, day counts and pricing remain later milestones. No first public
  release or blanket QuantLib compatibility claim is made here.
