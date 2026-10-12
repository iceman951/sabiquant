# Local M7 explicit-coupon traceability audit

Audit snapshot: 2026-10-13, Asia/Bangkok. This independent agent audit inspected
the adopted requirements, public surface, executable assertions, fixture bytes,
aggregate logs and local documentation targets. It changes only this document.
It is not formal owner/API/financial approval, native primary-platform CI or
M7 acceptance. The [validation report](M7_EXPLICIT_VALIDATION.md) records the
completed local tranche; the [contract](M7_EXPLICIT_STUB_CONTRACT.md) and
[source/API review](M7_EXPLICIT_API_REVIEW.md) retain separate responsibilities.

Reviewed schedule source SHA-256:
`be91b84dcaecd03b9ab7c667a742c3dffc5d27b5a04961a2badc49017e54ba06`.
It matches the independently verified extracted package and the final report.
No source, test, other document, Git or network mutation was performed here.

## Requirement-to-executable mapping

The [FS-SCHED requirements](REQUIREMENTS.md#14-fs-sched--local-m7-core-schedules)
are supported by the following actual assertions, not only proposed tests.
All date, flag, metadata and error comparisons use exact E0 expectations.

| Requirement | Executable evidence inspected | Scope/limit |
|---|---|---|
| FR-SCHED-01 | [Core](../tests/schedule_core.rs) range/option and public-trait tests; [public API](../tests/public_api.rs) thread-safe values | Explicit owned calendar and both conventions; no implicit date or financial default |
| FR-SCHED-02 | Core civil oracle and 65,988 Gregorian monthly preparations; calendar-day/week goldens; [explicit oracle](../tests/schedule_explicit_oracles.rs) all-unit/direction checks | Anchor-based calendar recurrence; schedule identity still distinguishes stored tenor units |
| FR-SCHED-03 | Core calendar-EOM regression and cycle; [explicit](../tests/schedule_explicit.rs) `clamp_and_eom_outer_flags_come_from_the_named_directional_seed` | Preserve rejects day/week units; calendar EOM differs deliberately from business-only EOM |
| FR-SCHED-04 | Core initial/final None/Short/Long goldens, exact-lattice and no-prior-regular failures; final-cap interactions | Automatic stubs only; explicit construction has no automatic policy |
| FR-SCHED-05 | Core `zero_metadata_and_recurrent_option_errors_are_explicit`; [reference](../tests/schedule_reference.rs) exact Zero metadata distinction | Two boundaries, NonPeriodic interval, absent recurrence/explicit metadata |
| FR-SCHED-06 | Core 49 convention pairs and exact collision/reversal context; explicit `all_adjustment_pairs_keep_role_correspondence_or_report_the_first_conflict` | No sort, deduplication or payment/accrual inference |
| FR-SCHED-07 | Core strict/projected TARGET, overrides/joints and calendar failures; explicit whole-span/candidate/Easter/source-chain tests; [properties](../tests/schedule_properties.rs) candidate coverage escape | Raw span and every visited adjustment candidate retain coverage guards, including identity-like paths |
| FR-SCHED-08 | Core exact limit ±1, Long fit and unused huge candidates at MIN/MAX; properties complete supported daily span | Final count bounded at 3,652,059; system allocation failure is not manufactured |
| FR-SCHED-09 | Core immutable rebuild/hash checks; properties equal vectors/distinct definitions; explicit collection/thread tests | Equality is retained definition/context, not financial equivalence |
| FR-SCHED-10 | [Root catalog](../src/error.rs) all-variant display/kind/source assertions; core/explicit reachable failure paths and nested typed sources | All variants cataloged; allocation/proof-excluded date propagation is not presented as an executed public failure |
| **FR-SCHED-11** | Explicit `explicit_value_absence_order_and_equal_roles_are_checked` and `explicit_build_input_precedence_and_both_strict_interior_roles_are_contextual`; oracle retained-role assertions; reference domain classifications | Absence/order/interior precedence, equal-role reuse, absent automatic policy |
| **FR-SCHED-12** | Explicit 230,400 cycle preparations, clamp/EOM/implicit-middle tests; 21,883 civil-coordinate rows and exact irregular context; reference clamp/EOM predicates | One named directional lattice; classification-only unsupported quasi-dates imply Stub, not missing required ICMA references |
| **FR-SCHED-13** | Explicit definition collections and concurrent rebuilds; every oracle success checks exact cap, cap−1 and larger-cap equality; properties full-span explicit construction | Explicit roles and raw anchor retained; original caller cap preserved; resource cap excluded from contractual identity |

The explicit API/property file contains 13 tests, its civil fixture consumer
one test, and the reference consumer two tests. The cycle asserts exactly
230,400 preparations, including 227,709 values and 2,691 irregular-middle
errors. The fixture consumer checks every one of its 17,789 successful rows
and 4,094 contextual errors. These sets are different evidence; their counts
are not described as independent financial conventions or a proof over all
possible calendars.

## Public types, methods and errors

A direct declaration scan finds **57 public struct/enum declarations**.
`public_api.rs` asserts Clone/Debug/PartialEq/Send/Sync/`'static` (Copy where
applicable) for 49. The remaining eight are explicitly covered by
[ICMA traits](../tests/icma.rs), [stateful day-count traits](../tests/daycount_extended.rs)
and [TARGET traits](../tests/target.rs): DeterminationPeriod, ActualActualIcma,
IcmaError, ThirtyE360Isda, Business252, TargetCalendar, TargetPolicy and
TargetCoverage. Stronger discrete Eq/Hash assertions and collections are
present in those suites and the schedule suites. This does not impose Eq/Hash
or ordering on floating numerical values.

The eight schedule declarations are DateGenerationRule, StubPolicy,
PeriodRegularity, ScheduleCalendar, ExplicitStubDates, ScheduleBuilder,
Schedule and ScheduleError. Inspection matches the executable public API:
`ExplicitStubDates::new/first/penultimate`; explicit builder range/context
inspectors; separate `build_recurrent`, `build_explicit`, `build_zero`;
read-only contractual/adjusted/regularity slices; recurrence/role/context
inspectors and optional `lattice_anchor`. Private fields, no mutation API,
standard value traits and typed fallibility agree with the adopted contract.

The root catalog's exhaustive variant dispatch and uniqueness assertion sample
**74 nested variants**. The schedule kind function has these **16 distinct
kinds**, each represented in that catalog:

| Kind suffix (`schedule.` prefix) | Evidence category |
|---|---|
| invalid_range, invalid_boundary_limit, invalid_rule, invalid_tenor, incompatible_end_of_month | Core and explicit public domain/precedence cases |
| empty_explicit_stub_dates, reversed_explicit_stub_dates, explicit_stub_outside_range | Explicit constructor/interior tests and reference classifications |
| irregular_explicit_middle | Every error oracle row; independent cycle and contextual reference cases |
| stub_required, long_stub_without_regular_period, boundary_limit_exceeded | Core stub/resource cases and explicit final-cap tests |
| calendar, adjusted_order | Real guarded calendar, nested date and collision/reversal failures |
| allocation_failed, date | Complete display/kind/source catalog plus source proof/propagation review; no forced OOM or corrupted private state |

`Error::Schedule` preserves ScheduleError as its source, with calendar/date
layers underneath when applicable. The public conversion/StdError assertions,
root unit catalog and explicit nested failure tests inspect that distinction.

One concrete documentation overstatement was reported: FR-SCHED-10's initial
validation cell could imply an executed public failure for every variant.
The root qualified it to all-variant catalog checks plus reachable public
failure paths, explicitly retaining allocation/proof-excluded date limits.
The correction is present. No other unresolved traceability defect was found
in this snapshot.

## Fixture and scoped-reference accounting

Independent parsing of every repository fixture gives **982,698 rows in
46 files**. Schedule files contribute 18,916 core oracle rows, 46 original
public probes, 21,883 explicit oracle rows and 29 new public probes.

All **59 previously tracked fixture/generator files** were compared byte-for-byte
with base `86d5bb8`; none changed. In particular:

| Frozen original artifact | SHA-256 |
|---|---|
| schedule_oracles.py | `20dbc0ab2bc2044a5e134ff5ed1cdf38eea3c7ac2bd13f0070871eb1b92a16ac` |
| oracles.fixture | `4c162e36ee6c1884b0a98a7e9ddefcde8cea4ea51ca3164d432eae5c38a9a1db` |
| schedule_probes.py | `2c48705c96a6311922e8483cc47ae383f032555fd56f4c3f53567ca605725d6d` |
| probes.fixture | `6e4a2ebbac1a20367c931de3413adf39cce1a63b3b77f44555449bdfd49bfbd7` |

The original reference test has **39/46 supported inputs**, comprising 30
shared values, four EOM distinctions, four collision rejections and one Zero
metadata distinction; seven specialized rules remain unsupported. The new
29-row test asserts 13 exact shared values, two reversed-role rejections,
three irregular-middle rejections, eight strict-interior rejections, one
named-clamp flag distinction and two business-only-EOM distinctions.
The four reference constructor errors are a subset of the eight interior
domain rows, not four extra cases. All 29 outputs/errors are accounted for.

Five quarterly examples occur in both sets. Their dates/flags are shared
exactly, but they are **five duplicated examples**, not ten independent
financial examples. [The discrepancy ledger](../validation/DISCREPANCIES.md)
records exact predicates rather than tolerances. The probe attestation binds
the installed wheel/module bytes; it does not attest an upstream source
revision or turn observed QuantLib behavior into market authority.

## Local execution, links and remaining gates

The final aggregate logs were independently recounted: 424 debug and 423
release passes in 46 result groups, with zero failures/ignored tests/warning
lines. The native coverage JSON agrees with 816/846 schedule regions (96.45%)
and 30 uncovered regions. This is source-region coverage; it is not branch or
exhaustive-input coverage. The extracted-package evidence independently
records 73 unit tests, 25 doctests and successful strict Clippy/public/private
documentation checks. Its 32-file allowlist and all 30 original-file byte
comparisons remain consistent with this source snapshot. This document is
excluded from the archive and changes no package byte.

Local file targets in README and all M7 documents were checked. GitHub
`blob/main` links for this repository were mapped to existing local paths;
that verifies the intended target file only. It does not verify remote
availability, publication or a remote branch. Other external links were not
fetched by this audit.

The named-lattice contract deliberately differs from endpoint-relative clamp
flags, hidden internal stubs, business-EOM rewriting and permissive reversed
or endpoint coupon roles observed in QuantLib. It defines local schedule
provenance, not every market convention. Specialized Wednesday/twentieth/CDS
rules, automatic ICMA references, accrual/payment choice, cash flows and exact
settlement remain separate work. Formal owner/financial/API/milestone review,
Linux/Windows native execution, external CI and the retained TARGET/source
maintenance and release gates remain open. Cross-compilation is not native
runtime evidence. No push or publication is inferred from this audit.
