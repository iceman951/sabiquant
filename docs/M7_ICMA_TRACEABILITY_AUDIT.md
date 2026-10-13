# Local generated-ICMA traceability audit

Snapshot: 2026-10-13, Asia/Bangkok, after explicit schedules were saved at
`d570b45`. This independent agent audit owns only this document. It inspects
the [adopted contract](M7_ICMA_REFERENCE_CONTRACT.md), source, generator,
fixture, exact-rational consumer and hand/property suites. Final aggregate
and package evidence belongs to [M7_ICMA_VALIDATION](M7_ICMA_VALIDATION.md).
This is not formal financial/owner/API approval, native primary-platform CI,
release authorization or completion of M7.

Reviewed schedule source SHA-256:
`5e92f66d8435b21aa4bb31727ef4eb4376a01498c5f625befa4a393bfd5fbf30`.
No source, tests, generators, other documents, Git or account settings were
changed by this audit. No reference implementation or guessed QuantLib
periods supply its expected dates/fractions.

## Oracle independence and minimality

The [generator](../validation/generator/schedule_icma_oracles.py) uses the
frozen [civil membership helper](../validation/generator/schedule_explicit_oracles.py).
It uses direct civil shifts to CHOOSE input endpoints and a safe envelope,
not to generate the expected reference sequence. Expected points are selected
across the complete bounded civil envelope: ordinal-day congruence for
day/week units, and month-coordinate congruence plus independently clamped or
calendar-EOM day matching for month/year units. This is separate from the
Rust negative/positive extension loop and its shared counter/cap stopping path.

Among selected points it chooses the greatest date no later than raw start
and the least no earlier than raw end, then retains every lattice point
between those extrema. This proves minimality for each supplied case: no
extra predecessor/successor is added at an exact endpoint. The directional
seed remains first-or-start for Forward and penultimate-or-end for Backward,
including automatic Short/Long and shared explicit roles. Stored 12M/1Y,
7D/1W, 14D/2W and 28D/4W aliases use the same civil membership predicate;
they do not replace the retained stored unit.

Fraction expectations sum exact integer overlap days divided by explicit
annual count times reference-period length, then reduce using Python Fraction.
The [Rust consumer](../tests/schedule_icma_oracles.rs) independently reconstructs
the reduced rational with checked i128 products/GCD cancellation; it does
not share production's rounded-term compensated accumulation. Every recorded
numerator/denominator pair was independently recounted: maximum 20/18 bits,
well inside both i128 and exact binary64 integer-conversion ranges. Every
recorded float also matches nearest binary64 conversion of its exact Fraction.
The existing **four-ULP E2** assertion remains unchanged.

## Counts, representations and query scope

Independent fixture parsing and generator `--check` pass with these counts:

| Evidence | Count |
|---|---:|
| Accrual rows / contexts | 36,092 / 18,046 |
| Automatic / explicit contexts | 7,734 / 10,312 |
| Each role: None, Short, Long, first, penultimate, both, equal | 2,578 |
| Forward / Backward contexts | 9,023 / 9,023 |
| Ignore / Preserve contexts | 11,438 / 6,608 |
| Forward / reverse / zero query rows | 27,069 / 4,512 / 4,511 |
| Full-hull query rows | 8,158 |
| Contexts whose second query repeats their first query | 998 |
| Reference-period counts | 178 with 2; 2,934 with 3; 12,534 with 4; 2,400 with 6 |
| Consumer period comparisons, including both rows per context | 147,388 |
| Fixture content size | 8,253,708 bytes |

All ten named periodic frequencies are present. Their explicit event counts
are 1, 2, 3, 4, 6, 12, 13, 26, 52 and 365; they are not inferred from leap
years, weekly-year lengths or schedule dates. Frequency context counts are:
Annual 1,904; Semiannual 2,212; EveryFourthMonth 2,212; Quarterly 2,212;
Bimonthly 2,296; Monthly 2,380; EveryFourthWeek 1,190; Biweekly 1,190;
Weekly 1,204; Daily 1,246. Canonical and stored aliases are both represented.

Rows are observations, not distinct contexts or financial examples. Repeated
full-hull controls and two query rows per context are intentional. A Daily
Short/Long label may still lie on an exact one-day lattice; label counts must
not be presented as actual residual-stub counts.

The generator requires every point in its ±5-step envelope to be representable
before emitting any role case. This deliberately biases its successful civil
edge coverage: it does not prove required-extension behavior at MIN/MAX.
Generated explicit outer spans involve at most two quasi-periods per side,
and references have 2/3/4/6 periods; one-period hulls, larger outer spans,
error/market guards and complete-range stress come from separate tests.
Seed-year classes are varied, not a claim of the full Cartesian product of
every frequency with every civil edge. NullCalendar/Unadjusted is the fixture
scope; payment adjustment, TARGET and failures are not inferred from its rows.

## Requirements and complementary executable evidence

The new [FR-SCHED-14..17](REQUIREMENTS.md#14-fs-sched--local-m7-schedules)
are present. Inspection maps their actual assertions as follows:

| Requirement | Executable evidence | Qualification |
|---|---|---|
| FR-SCHED-14: periodic schedule, explicit frequency, normalized tenor match and event counts | [Hand tests](../tests/schedule_icma.rs) all-frequency aliases/mismatches, Zero/Once and cap precedence; fixed Python frequency tuples and consumer event assertions; property fixed tuples | No named frequency is guessed; ten day/week/month/year frequencies are delegated analytical scope, not universal market convention approval |
| FR-SCHED-15: minimal same-anchor hull, long/clamp/EOM provenance and required dates | Civil-envelope oracle; [properties](../tests/schedule_icma_properties.rs) independent civil-day membership across the complete Gregorian cycle; hand long/equal EOM examples, exact lattice endpoints and actual signed MIN/MAX failures | Removed Long coupon boundaries remain in references; unsupported REQUIRED dates fail, unlike unused schedule candidates |
| FR-SCHED-16: shared final period cap, validated owned context and typed causes | Every oracle exact-cap/cap−1/larger-cap check; hand interior two-direction lower bound, nine-reference long hull and all 3,652,058 daily periods; property equality/hash/thread checks; root catalog | Cap counts periods, not coupons/boundaries. ICMA-constructor and allocation propagation remain catalog/proof-reviewed without fabricating invalid private state or OOM |
| FR-SCHED-17: complete extended TARGET hull and unadjusted references | Hand projected/strict gap and inception checks; properties guarded overrides/joints and exact-stop controls; all 49 convention pairs plus two distinct calendar definitions produce one context | Returned context is a reference/frequency value, not market verification or permission for later TARGET queries |

The six property tests assert exactly **614,400** cycle preparations:
594,618 valid contexts and 19,782 correctly rejected explicit middles;
1,777,260 reference periods and 2,966,496 rational forward comparisons.
Their membership scan and fixed annual-frequency tuples are independent of
the Python fixture and production recurrence loop. Signed reverse checks,
covered zeros and one-term identities add checks beyond those forward totals.
The separate immutable-rebuild test exercises ten frequencies across eight
threads and sixteen rounds: **1,280 generated-context rebuilds**. Dedicated
hand tests cover a 7.5-year equal-role accrual requiring nine references,
including more than two periods on each outer side, and the full daily hull.

## Source and public surface review

`Schedule::build_icma_context(frequency, max_periods)` is a new fallible method
on the existing immutable Schedule; it introduces no type. The source scans
still count **57 public struct/enum declarations**. MIN_ICMA_PERIODS=1 and
MAX_ICMA_PERIODS=3,652,058 match the contract. Validation order is cap,
periodic schedule, named periodic frequency, normalized tenor match, shared
bounded extensions, complete extended market span, positive contiguous
period/context construction. Exact raw endpoints stop extension immediately.

The six additional stable schedule kinds are invalid_icma_period_limit,
non_periodic_icma, invalid_icma_frequency, icma_frequency_mismatch,
icma_period_limit_exceeded and icma (all with `schedule.` prefix). They bring
the schedule kind count to **22** and the exhaustive root catalog to **80**.
The root uniqueness test passes independently. Date/calendar/ICMA wrappers
retain their typed cause chain and extension/coverage phase; the existing
StdError/value/thread traits remain in force. The constructor Icma wrapper
is proof-excluded by valid generation, but preserves its cause instead of
unwrapping it.

The source module overview initially called automatic ICMA references deferred.
This concrete stale description was reported promptly and corrected by the
root. No arithmetic or invariant defect was found in this audit snapshot.
The named lattice determines reference provenance; it does not claim that
every clamp, roll, day/week count or adjusted-accrual product is a universally
approved ICMA market convention.

## Independent execution and reproducibility

Rust 1.99.0/macOS ARM64 execution in a fresh temporary target independently
passes eight hand tests, the 36,092-row consumer and six properties in both
debug and release: fifteen integration tests per profile.
The consumer reports maximum **1 ULP**, under the unchanged four-ULP assertion.
The root catalog uniqueness test passes separately. Final aggregate/package
status belongs to the execution report linked above; no native Linux/Windows
result is inferred from cross-compilation.

| Reproduction artifact | SHA-256 |
|---|---|
| Generated ICMA oracle | `0f30968d42e3d6f18debb2aae1ea6ef7178229f765e754988c135aef09a7c31e` |
| Frozen civil helper | `a082c47a2f0b6c98d312dcc52105395b153e6619d5eff107b3619d2c651448ce` |
| ICMA fixture | `0df58ae610aac332f0c18deb9cd5ebc69460b2202ee3c489a9bfa8495bf22a55` |

`python3 validation/generator/schedule_icma_oracles.py --check` reproduces the
fixture byte-for-byte in the declared local Python environment. These hashes
attest retained inputs/output; row counts and passing tests are not proofs of
every date/context or market convention. Specialized rules, cash flows and
settlement, formal owner/financial/API/milestone acceptance, native primary
platform CI, TARGET maintenance and release gates remain separate work.
