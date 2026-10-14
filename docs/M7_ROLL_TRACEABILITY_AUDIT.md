# Independent civil-roll requirement and evidence audit

Reviewer: `/root/roll_audit`, 2026-10-14. Scope is FR-SCHED-18 through
FR-SCHED-21 in [REQUIREMENTS](REQUIREMENTS.md), the
[adopted four-rule contract](M7_ROLL_CONTRACT.md), and the existing schedule
invariants touched by this tranche. The [API review](M7_ROLL_API_REVIEW.md)
records the source findings and exact production hashes. This audit introduces
no changed requirements and makes no full-M7 or release acceptance claim.

## Requirement mapping

| Requirement | Inspected implementation and meaningful evidence | Audit result |
|---|---|---|
| FR-SCHED-18: four explicit rules, positive stored month/year tenor, quarter-multiple IMM stride, deterministic projections/lattice and genuine adjacent-roll regularity | `build_roll`, required roll resolution and `regularity` in `src/schedule/roll.rs`; literal endpoint/flag vectors, same-month and off-stride controls, stored-domain/huge-tenor tests; independent civil generator and consumer; exact classified public-reference distinctions | Implemented for the adopted four civil definitions. Exact dates/flags/statuses use E0. No CDS, market-calendar or universal reference parity inference. |
| FR-SCHED-19: immutable requested/resolved provenance, original stored tenor, anchor month, definition Eq/Hash, truthful rule and absent artificial metadata | Private `RollProvenance` fields and eight borrowed/value inspectors; Schedule equality/hash explicitly include provenance and stored tenor; oracle checks every inspector; same-vector/different-input and 12M/1Y hash-set tests; caps leave identity unchanged; canonical enum registry, 66-type trait census and 512 concurrent builds | Implemented. Shared projected output does not erase the original requested range. Cap is an operation budget. |
| FR-SCHED-20: original TARGET guard, union guard, bounded resources, typed required-date failure and safe unused next coordinate | Original `validate_span` then union `validate_span_for`; checked roll-date mapping; coordinate comparison before candidate construction; exact cap controls, all-insufficient-cap precedence test, real whole-civil-range schedules, civil overflow/root-source tests, strict/projected coverage, overrides/joints and candidate-adjustment failures | Implemented. Arithmetic and progress have explicit bounds. Vector allocation errors are catalogued without forced OOM. |
| FR-SCHED-21: other builders reject Roll after cap; generated ICMA checks cap then typed unsupported rule before frequency | `build_recurrent`/`build_explicit` dispatch, `build_icma_context` early roll branch; all four rules against every named Frequency and invalid caps; combined invalid input/role/TARGET controls; old-builder metadata and full regression suite | Implemented as the declared temporary exclusion. Same-roll ICMA remains required follow-on work; this requirement does not turn that missing feature into completed M7 scope. |

The changed API also preserves the affected existing invariants:

| Existing invariant | Evidence at this snapshot |
|---|---|
| FR-SCHED-06 adjustment pairing and order | The unchanged shared finalizer remains the only adjustment path. A custom Twentieth lattice checks all 49 ordinary/terminal pairs against hand-pinned dates. Separate collision, reversal, closed-month, search-limit and civil-extreme cases check exact offending context. The 60 compatible reference calendar controls span all four rules, seven ordinary conventions and two terminal choices, plus four TARGET requests. |
| FR-SCHED-07 market span and candidate guards | Public tests exercise original gaps, inception, strict/projected policy, overrides and both joint modes; forward projection into 2029 fails in `roll coverage`, and an adjustment search entering 2029 fails in `adjustment`. Inclusive shrinking retains original range validation. |
| FR-SCHED-08 final cap and bounded growth | Each successful civil-oracle row tests exact count, cap−1 and cap+1; all insufficient interior/terminal caps are separately tested before adjustment. Four actual minimum-stride schedules span the supported civil range. Required date errors stay errors even with a small cap. |
| FR-SCHED-09 definition identity and immutable reuse | Schedule/raw/adjusted/flags/metadata are inspected independently, original stored tenor is preserved, distinct requested ranges remain distinct, and concurrent reuse produces equal values/hashes without changing the builder. |
| FR-SCHED-10 typed errors and root exposure | Four new stable kinds enter the exhaustive 93-variant root catalog. Public required-overflow cases inspect actual signed month counts, role/input/tenor/rule and nested DateError; calendar cases inspect ScheduleError→CalendarError→DateError. |

The 49-pair hand test uses one custom Twentieth lattice; it is not claimed as
the full four-rule × every-calendar × every-convention product. The independent
civil fixture uses Null/Unadjusted. Separate adjustment/coverage tests supply
the calendar evidence, and successful tests do not close historical source gaps.

## Independent civil oracle scope

The generator enumerates all in-span civil months and selects by congruence;
it does not increment the Rust recurrence or use Rust results as expected truth.
Python datetime/calendar enumerates each month's Wednesdays. The Rust consumer
additionally counts Wednesdays using a separate Gregorian ordinal formula and
checks adjacent membership, preserving independence from production weekday
and roll construction. No numerical tolerance is involved.

The inspected corpus has 122,233 records, with 122,183 distinct input tuples
and 50 repeated controls. Every month of 2000..2399 is included under every
rule with six endpoint/stride profiles and rotating tenors. Additional controls
cover century years, civil minimum/maximum, huge stored periods and rejected
domains. This is a finite designed corpus, not every possible date/tenor tuple.

| Pinned outcome or check | Count |
|---|---:|
| Valid defined schedules | 116,811 |
| Inclusive collapsed resolved ranges | 5,290 |
| Required projected date overflows | 54 |
| Invalid stored tenor/rule combinations | 78 |
| Exact successful boundaries checked | 354,671 |
| Regular / Stub intervals | 114,065 / 123,795 |
| Successful schedules with exact/cap−1/cap+1 checks | 116,811 |

The ordinary oracle spans are at most 25 months. The public resource test
constructs all four full-supported-range minimum-stride schedules, rather than
inferring resource behavior from short oracle rows:

| Rule | Actual retained boundary count | Stub count |
|---|---:|---:|
| ThirdWednesday, 1M | 119,989 | 2 |
| ThirdWednesdayInclusive, 1M | 119,988 | 0 |
| Twentieth, 1M | 119,989 | 1 |
| TwentiethIMM, 3M | 39,997 | 1 |

Their exact endpoint values, strict ordering, interior roll membership and
cap−1 failure are checked. Huge i32::MAX Years controls separately exercise
unused coordinates beyond the supported Date range. All generation arithmetic
remains bounded by valid month coordinates plus one positive stride.

## Reference evidence and fidelity

The new consumer checks all 538 preserved public constructor observations:
466 returned values and 72 RuntimeErrors, including five non-strict returned
vectors. It checks the byte-attested QuantLib 1.43 wheel/module metadata while
retaining the explicit absence of exact source-commit attestation. It does not
repair or silently discard unsupported reference results.

Of 248 requests compatible with the local four-rule/options API, 110 agree in
dates and flags, 100 differ only in flags, 13 differ in date vectors, and 25
receive a deliberate local typed rejection. The 13 date distinctions comprise
six additional local Wednesday interiors and seven retained projected
Twentieth terminal boundaries. The test pins those actual vectors; aggregate
counts alone are not their proof. The 25 rejections are 19 invalid tenors and
six collapsed inclusive ranges.

The remaining 290 observations are explicitly outside this API: 254 are
OldCDS/CDS/CDS2015 and 36 request EOM or explicit coupon roles. Forty legacy
cutoff cases and 144 separately quoted maturity calls are retained as reference
observations, not adopted contract behavior. The
[reference report](M7_ROLL_REFERENCE_PROBES.md) owns their collection,
attestation, bounded subprocess design and dated interpretation.

## Final snapshot and validation evidence

The reviewer inspected final v2 logs with 508 native debug and 507 native
release passes, each containing 56 result groups and zero failures, ignored
tests or warning lines. The root reports both commands terminated with exit
zero. These comprise 80/79 unit tests, 400 integrations and 28 doctests, including
three compile-fail cases. The new public hand/oracle/reference harnesses
contribute 20/1/4 integration tests. Passed large suites were not rerun solely
to duplicate evidence.

The new types bring the independently counted public declarations and central
value/copy trait assertions to 66 each, with no missing or extra names. The
exhaustive 93-variant root classifier and catalog cover all four new kinds.
Actual archive bytes and all 34 original members were independently compared
again; extracted execution evidence is recorded by the separate package
reviewer. Compiler target checks remain compile-only evidence for Linux/Windows.

| Audited artifact | SHA-256 |
|---|---|
| `tests/schedule_roll.rs` (20 tests) | `c10278d17fa6772d684f03f0ca06b8f1528acef7d1d0f040844fb6317bbd5c63` |
| `tests/schedule_roll_oracles.rs` | `3063a2080a47b90d46e0192d01cd8edc07f5d83a3669b488b90ff1c3d7792cca` |
| `tests/schedule_roll_reference.rs` | `37ded5f0ca8a21cd789cbdc3ae732cbdb0cf9b50f5983f9f7f296903e707332e` |
| `tests/public_api.rs` | `07298bd9042e6d526af4f2c22777a0722cbe61f57c0c7134b4f450f03a72626a` |
| `docs/REQUIREMENTS.md` | `ecf8f0c6de209dc00a55e5122e0341a63e69956b6759c901d1b6c4841a9e1039` |
| Final debug v2 log | `142d5856aa32470b186d5ea18d1a07d1ebca65175de8a932bb7d393275d7a4bd` |
| Final release v2 log | `f1a8f7391caf876ca25d4ea7f73918e186344e4ae4e81fbcf955818c7dae4a6b` |

The fresh v2 native coverage report was independently rehashed and inspected:
`/private/tmp/sabiquant-m7-roll-coverage-final-v2/production-regions.json`, SHA-256
`debb39ab30857308b8ce3698aacb129cfd45a23562daf5c93ed138dedd3497a3`.
The root reports terminal exit zero. Its 54 instrumented executable logs contain
480 passes with zero failures or ignored tests. Parent schedule source has
1,036/1,074 covered production regions; roll source has 285/296, giving a union
of 1,321/1,370 across those two files (96.42%). Cash-flow source coverage remains
688/703. The newly observed public interior-cap error path is covered.

All 11 uncovered roll regions were mapped to source: three same-month Wednesday
error propagations, six regions of the valid-Date-to-Month error adapter, one
initial allocation-failure propagation, and one interior-date error propagation
whose coordinate is already proven valid. The [API review](M7_ROLL_API_REVIEW.md)
records those structural arguments. This is not a reason to fabricate invalid
private state or force system OOM. The metric excludes embedded test source and
doctests; no branch/input completeness or financial accuracy claim follows.

Complete lint/docs/cross-target command results are scoped in the
[parent validation report](M7_ROLL_VALIDATION.md); compile-only targets do not
prove native platform parity. Same-roll ICMA, OldCDS/CDS/CDS2015, CDS maturity/
product semantics, missing historical sources and all formal release gates
remain open.
