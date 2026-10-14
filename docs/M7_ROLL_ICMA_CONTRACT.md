# Local M7 ICMA references for civil rolls

Adopted on 2026-10-14 under the continuing local implementation authority,
following civil-roll baseline `af4fa73`. This extends the
[generated ICMA contract](M7_ICMA_REFERENCE_CONTRACT.md) to the four
[civil roll definitions](M7_ROLL_CONTRACT.md), using the reviewed
[design](M7_ROLL_ICMA_DESIGN.md). Historical contracts, reference observations
and their SHA-pinned validation reports remain snapshots of their own tranches.
This analytical extension does not define a standard CDS premium leg.

## Public behavior and compatibility decision

The existing `Schedule::build_icma_context(frequency, max_periods)` now accepts
ThirdWednesday, ThirdWednesdayInclusive, Twentieth and TwentiethIMM schedules.
Frequency remains explicit and periodic. Its Period must normalize equal to
the retained stored tenor. Wednesday and monthly twentieth rolls can match
Monthly, Bimonthly, Quarterly, EveryFourthMonth, Semiannual or Annual;
TwentiethIMM can match Quarterly, Semiannual or Annual. Stored 12M and 1Y can
produce equal contexts while the source schedules retain distinct definitions.
Once, other mismatched frequencies and Zero retain their typed rejections.

Remove the unreleased temporary `RollIcmaUnsupported` variant/kind rather than
retain an impossible current limitation or a dormant public error. The root
catalog decreases from 93 to 92 variants; no new error kind is needed. This
is an explicit API change before the first release. Existing ordinary
date/EOM reference outputs, error causes and precedence retain their contract.
No change is made to the immutable context's frequency, equality, accrual
formula, query work bound or four-ULP rational-oracle profile.

## Same-roll minimal hull

Let q(k) be the third Wednesday or day 20 in
`anchor_month + k * month_stride`, for signed integer k. The original immutable
roll provenance supplies the anchor year/month. Quarter membership follows
from the checked IMM anchor and quarter-multiple stride. Recalculate the day
in each candidate month; never use a clamped Wednesday day as the next seed.

Use the schedule's resolved raw contractual start/end. The greatest q(L) at
or before raw start and the least q(R) at or after raw end delimit the minimal
hull. Return every adjacent determination period between them. Exact endpoints
need no extra reference date. Requested endpoints and adjusted payments do
not extend this hull. Source requested/resolved provenance and calendar guards
remain on the original Schedule.

All four current builders guarantee q(0) lies inside this minimal hull.
Wednesday q(0) shares the requested start month; a twentieth start lies after
the preceding eligible roll and at or before its first eligible anchor.
The shared bounded construction can therefore extend below q(0) first, reverse
that sequence once, then extend above q(0). For these four rules at most one
negative step is needed. This proof is specific to the current builders and
does not authorize arbitrary future anchors.

Independent hand definitions for Monthly Wednesday q(0)=2024-01-17:

| Raw range | Minimal reference boundaries | Exact full-range year fraction |
|---|---|---|
| Jan01..Jan02 | 2023-12-20, 2024-01-17 | 1/336 |
| Jan25..Jan26 | 2024-01-17, 2024-02-21 | 1/420 |
| Jan16..Jan18 | 2023-12-20, 2024-01-17, 2024-02-21 | 3/560 |

## Errors, caps and retained calendar policy

The precedence is valid cap, periodic source, periodic frequency, normalized
tenor match, required bounded generation, full extended calendar coverage,
then determination-period/context construction. The existing valid cap range
is 1..=3,652,058 final positive reference periods. Both extension directions
consume one budget. Before a lower extension, q(0)<raw_end proves at least one
upper period; include that lower bound in the cap check. An inadequate cap
fails before attempting the disallowed required date. No partial result is
returned and no required date can be omitted as an unused schedule candidate.

Required roll extensions use `ScheduleError::Date`, with generation-position
index `boundaries.len()` and phase `ICMA reference extension`. The underlying
checked Date cause retains exact q(0), operation `add months` and signed
month offset, including stored Year tenors. Ordinary Year lattices continue
using their original `add years` causes. For year 0001 Jan16..Jan18 Monthly
Wednesday, cap1 fails with required2 before arithmetic; cap2 preserves the
q(0)=0001-01-17, minus-one-month predecessor error at index1.

All scaled offsets fit i64 under the checked tenor and cap limits. Candidate
months strictly progress. Work and storage remain O(reference periods), plus
existing calendar costs; boundary and period vectors reserve fallibly. No
mutable cache, hidden clock, I/O, dependency or test-only public hook is added.

Validate the entire extended reference hull under the source calendar's
strict/projected TARGET policy, including gaps, quasi endpoints, overrides
and joint calendars. Failures retain index None and phase
`ICMA reference coverage`. This complements the original build's
requested/resolved union guard. The generated context owns dates/frequency,
not authorization for later calendar operations.

## Evidence obligations and open work

Require independent civil-month/congruence and exact-Fraction evidence,
before/on/after endpoint controls, same-month hulls, all matching frequencies,
annual aliases, off-stride terminals, exact cap/cap−1, actual civil-boundary
failures, unchanged ordinary references, calendar extension guards, adjustment
invariance, immutable reuse and generic coupon composition. Source coverage
must distinguish public cases from allocation/invariant-only wrappers.

Full tests, strict lint/docs, actual package extraction and compile-only
cross-target checks remain verification obligations. These do not close
native Linux/Windows execution, financial/owner acceptance, standard CDS
rule/product/source work, NI-01 settlement or full M7/release gates.
