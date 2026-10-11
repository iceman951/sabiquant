# M3 Core Batch Validation

| Field | Value |
|---|---|
| Date | 2026-10-11 |
| Authority and scope | [M3_LOCAL_READINESS](M3_LOCAL_READINESS.md) |
| Status | Local core batch passes; full M3 scope and milestone acceptance remain outstanding |

## Scope and references

Core immutable calendars, seven business-day conventions and four day counts
are the first slice. No market-calendar coverage is claimed. Primary evidence
and exclusions are recorded in [calendar research](M3_CALENDAR_RESEARCH.md)
and [day-count research](M3_DAYCOUNT_RESEARCH.md).

Calendar fixtures use actual QuantLib 1.43 public interfaces and independent
enumeration, with bounds on any scenario that could make QuantLib search
indefinitely. Pathological never-open/long-closure calendars are tested on the
Rust side without executing unbounded reference calls. The generator must
stop on unexpected differences; labels cannot turn defects into accepted
divergences.

Day-count oracles count leap-year and common-year days independently, then
form exact integer/rational expected results. E1 results require the correctly
rounded single quotient; ISDA E2 uses the declared ULP bound, with no global
epsilon. Equal and reversed endpoints, leap/century windows and full-range
integers are required. The existing M2 regression suite remains enabled.

## Specifications clarified by testing

- Unadjusted advancement with EOM uses calendar month end only. A custom
  calendar's business month end on 2024-03-28 does not cause a two-month
  backward Unadjusted shift to become 2024-01-31; the correct result is
  2024-01-28. Actual QuantLib generation rejected the earlier specification's
  conflicting expectation, and FR-CAL-05 was corrected.
- Closed calendars remain valid values for classification and composition.
  A reachable explicit business-date override must not be hidden by a
  structural no-business-weekday rejection.
- A normalized Period's equality does not substitute for choosing the correct
  advancement operation: Days count business days in a calendar; Weeks count
  calendar weeks followed by adjustment (FR-PER-05).

## Check record

The full local suite passes **184 debug tests** and **183 release tests**,
including all M2 regression tests. The one-test difference is the debug-only
invalid-internal-interval assertion. There are no failed or ignored tests.

Calendar tests include all 128 weekend masks, every pair of weekend definitions
under both joint rules, a full leap-year classification grid, all seven
conventions, signed endpoint flags, override immutability, 365/366/367 search
runs, empty months, range edges, extreme tenors and joint-depth limits. Actual
QuantLib calendar fixtures have **75,227 rows**, including exactly five finite
documented intentional differences.

Day-count tests include a complete 400-year adjacent-date cycle, all supported
month ends, 24,000 deterministic full-range date pairs, century/year-boundary
grids and an independent weighted-day rational oracle. TS-DCC-07 also checks
every ordered date pair in 1999–2002 and 2099–2101: **3,333,546 pairs** against
all four implemented conventions (13,334,184 convention/pair checks).
**72,776 actual
QuantLib day-count fixture rows** compare the four conventions with independent
Python `Fraction` values. M2 and M3 together have **307,950 fixture records**.

Two specifically classified reference issues are tested in
[DISCREPANCIES](../validation/DISCREPANCIES.md): 8,372 same-year ISDA
cancellation cases use a justified reference-to-oracle absolute bound while
Rust keeps its original tight oracle contract; 452 nonzero 2199 intervals
document the reference's invalid year-2200 intermediate. Unknown differences
or exceptions still fail generation and testing.

Native production-region coverage, excluding embedded test modules:

| Source | Covered / reported regions |
|---|---|
| `calendar.rs` | **612 / 618 (99.03%)** |
| `daycount.rs` | **148 / 148 (100%)** |

The unexecuted calendar paths are invariant-protected revalidation/fallbacks,
the already-handled Unadjusted match arm, and empty endpoint iteration. The
coverage runner does not claim branch coverage or exhaustive arbitrary inputs.

Rust 1.99 formatting, strict Clippy, public/private rustdoc, zero-dependency
checks and fixture regeneration checks pass locally. Linux GNU and Windows
MSVC **all-target compile checks** pass using their standard-library targets
on this Mac. These checks do not execute tests on Linux or Windows and do not
replace native platform CI. No M3 success is inferred from the earlier M2-only
checks in [M2_VALIDATION](M2_VALIDATION.md).

## Outstanding milestone scope

Easter, TARGET, market provenance/coverage, other day-count variants and their
source ambiguities remain outside this first slice. Primary-platform CI,
review and milestone acceptance remain outstanding. The user selected
local-only engineering; no external GitHub writes are authorized.
