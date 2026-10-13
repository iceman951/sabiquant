# Local M7 contractual ICMA reference research

Research snapshot: 2026-10-13, by the independent period/schedule reviewer.
This is a proposal for the next delegated local tranche, separate from the
saved explicit-coupon tranche. It adds no implementation, fixtures or tests;
it does not accept M7, approve a financial product or authorize remote writes.

## Primary evidence and its limits

[ICMA Primary Market Handbook, Appendix A5, March 2022, A5-2–A5-3](https://www.icmagroup.org/assets/documents/ICMA-PMH-Appendix-A5-March-2022.pdf#page=2)
was re-opened directly. Version 2 §1.3 apportions accrual across determination
periods using actual overlap days divided by actual reference days and the
normal annual frequency. Its concluding note permits more than two segments.
The March note distinguishes contractual interest periods from payment-day
adjustments and does not prescribe adjusted accrual. The determination-date
definition supplies the financial basis for a contractual coupon lattice.

The appendix does **not** define this Rust builder, resource/error precedence,
every calendar roll or every day/week frequency. Daily/weekly divisors below
are explicit local conventions, not a claim that A5 certifies all ten named
rolls for every market. The three-page appendix contains no worked numerical
examples. The primary [ISDA 1999 paper](https://www.isda.org/a/pIJEE/The-Actual-Actual-Day-Count-Fraction-1999.pdf)
could not be fetched on this review (HTTP 403); no unseen examples are claimed.

The pinned [QuantLib 1.43 public schedule header](https://raw.githubusercontent.com/lballabio/QuantLib/v1.43/ql/time/schedule.hpp)
was inspected only for API inventory. It exposes tenor, rule, EOM and first/
penultimate metadata, while its supplied-vector constructor leaves metadata
unchecked. This does not establish SabiQuant's reference-generation semantics.
No QuantLib implementation was read, copied or translated for this proposal.

Local constraints come from [M7 research](M7_LOCAL_RESEARCH.md),
[explicit coupon contract](M7_EXPLICIT_STUB_CONTRACT.md),
[existing ICMA contract](M3_ICMA_CONTRACT.md), [schedule](../src/schedule.rs),
[ICMA](../src/daycount/icma.rs) and [frequency](../src/time/period.rs).

## Implementable candidate contract

`Schedule::build_icma_context(frequency: Frequency, max_periods: u32)` returns
`Result<ActualActualIcma, ScheduleError>`. It consumes no schedule, adjusts no
dates and never infers a frequency from coupon lengths or regularity flags.

| Decision | Proposed precise behavior | Evidence / qualification |
|---|---|---|
| Periodic domain | Reject Zero, Once and a frequency whose `period()` is not normalized-equal to the stored positive schedule tenor | Explicit local API decision; M3 requires periodic frequency |
| Accepted names | Annual 1Y/1; Semiannual 6M/2; EveryFourthMonth 4M/3; Quarterly 3M/4; Bimonthly 2M/6; Monthly 1M/12; EveryFourthWeek 4W/13; Biweekly 2W/26; Weekly 1W/52; Daily 1D/365. Each pair is tenor/annual divisor | Existing closed Frequency catalog, including its conventional day/week counts |
| Equivalent units | 12M with Annual, 7D with Weekly, 14D with Biweekly and 28D with EveryFourthWeek are valid; month/day families cannot substitute merely because one observed interval has equal length | Period normalized equality; stored unit still governs the original lattice |
| One named lattice | Use `lattice_anchor()`, stored tenor and retained EOM. Signed indices extend the same anchor in both directions; never seed from a clamped neighbor, payment date or another explicit role | Existing directional provenance; this is delegated roll policy |
| EOM | Preserve activates only when that original anchor is calendar month-end. Keep the activation fixed during extension | Existing M7 policy, not Calendar::advance business-EOM behavior |
| Minimal coverage | Select the largest lattice date `qL <= raw_start` and the smallest `qR >= raw_end`; retain every lattice interval between them. Exact raw endpoints require no extra outward interval | Sufficient contractual references without gratuitous range/coverage errors |
| Long/equal-role spans | Restore suppressed Long boundaries and extend over every outer span, however many tenors it covers; equal first/penultimate roles do not imply a one-period reference context | A5 permits multi-period accrual; roles locate the anchor, not a frequency or reference count |
| Required dates | Every required quasi date must fit Date's range. An unavailable classification-only neighbor may label an outer coupon Stub, but cannot supply a missing ICMA denominator | Typed checked-Date cause required; unused-candidate `Ok(None)` is unsuitable here |
| Calendar | Validate the **entire extended** reference span using the owned TARGET guard, including joints and projection policy. Do not adjust reference dates | Existing maintenance/coverage policy plus contractual-date scope |
| Owned result | Construct ordered, positive, contiguous DeterminationPeriods and pass them with the explicit frequency to ActualActualIcma | Existing constructor revalidates structure; no subsequent calendar queries occur |

The result stores only the immutable reference boundaries and frequency. It
does not preserve calendar provenance or promise adjusted-date coverage.
Later `year_fraction` requests retain M3's closed outer coverage checks,
including reversed and equal-date queries; `day_count` remains unrestricted.
Definition equality of the context can therefore coincide for two schedules
with different adjustment/calendar metadata. This follows from the ICMA value's
existing fields rather than an assertion of financial product equivalence.

## Resource, error and numerical policy to pin before coding

Require `1 <= max_periods <= 3_652_058`, measured against final reference
**periods**, not schedule boundaries or only newly added intervals. Reuse
fallible vector growth and return no partial context. Generation can visit
each signed lattice date once, O(p) time/storage plus the retained calendar's
whole-span validation cost; it must stop at the caller's cap. The hard ceiling
follows from the supported Date range, not a heuristic annual-frequency bound.

Recommended precedence is: invalid period limit; nonperiodic schedule; Once;
tenor/frequency mismatch; bounded reference generation (Date/cap/allocation);
whole extended TARGET coverage; ICMA structural construction. Thus a cap error
need not discover a later uncovered reference. Root should adopt or amend this
order explicitly before consumers pin it. Preserve the original cap in errors.
Do not reuse `BoundaryLimitExceeded` with a derived private boundary allowance
and accidentally report that allowance as the requested period maximum.

New domain failures need structured frequency/tenor/rule or requested limit.
Reference Date failures should retain anchor, signed step and the Date cause,
or a clearly documented existing index/phase interpretation. Coverage uses
the existing Calendar cause with a reference-coverage phase; constructor
failure should retain typed IcmaError as a source. Precise new variant/kind
names remain root's API decision. No default, clock, I/O, cache or new crate
dependency is required.

Reference dates, period count, coverage and statuses use TOL-E0. Fraction
evaluation inherits existing ICMA TOL-E2 and its positive compensated-sum
bound; generation introduces no floating-point arithmetic or tolerance change.
This does not establish universal libm accuracy or platform acceptance.

## Independent validation proposal

1. Build reference expectations from civil-coordinate **congruence membership**
   around the named anchor, not by calling Date recurrence or production
   Schedule helpers. Scan 400 Gregorian years over every monthly residue,
   all ten frequencies, both directions, inactive/active EOM, stored aliases,
   automatic None/Short/Long and all explicit role combinations. Check the
   minimal hull, every intermediate lattice date and the original cap at p−1,
   p and p+1. Shape/status/date comparisons are exact.
2. Independently count overlaps using Python datetime ordinals and Fraction.
   Add `overlap_days / (F * reference_days)` as reduced integers and round
   once for the binary64 oracle. Rust consumers should independently verify
   reduced rational metadata with checked i128 arithmetic and pre-reduction,
   rather than treating parsed numerator/denominator strings as verified.
   Large-prefix checks can weight each civil day by the LCM of actual generated
   denominator lengths; determine that LCM from the independently verified
   references rather than assuming a fixed month-length table.
3. Exercise Date MIN/MAX when raw schedules succeed but required references
   do not. Example: backward monthly EOM from 0001-02-28 covering Date MIN
   needs an unavailable preceding December end. Conversely an exact lattice
   endpoint must succeed without computing an unnecessary year-0/10000 date.
4. Check extended TARGET inception/gap failures even though the raw schedule
   passed. A 2026-01-15 start with first 2026-01-31, monthly Preserve, needs
   2025-12-31 as its preceding reference; strict TARGET must reject that date.
   Include joint-null shortcuts, projection, overrides and reference-only
   weekends/holidays; they must not alter contractual denominators.
5. Validate source chains, all input precedence combinations, same/reversed
   queries, immutable reuse, context Eq/Hash and frequency mismatch despite
   identical adjusted vectors. Different conventions on the same raw lattice
   must produce the same context when both schedules can be built.

Formula-derived anchors, **not claimed handbook worked examples**:

| Accrual and frequency | Required references / exact rational |
|---|---|
| 2003-11-01–2004-05-01, Semiannual | Same full reference: 182/(2×182) = 1/2 |
| 1999-02-01–1999-07-01, Annual, backward July-1 anchor | 1998-07-01–1999-07-01: 150/365 = 30/73 |
| 2002-08-15–2003-07-15, Semiannual, July-15 anchor | References 2002-07-15–2003-01-15 and 2003-01-15–2003-07-15: 153/368 + 181/362 = 337/368 |
| 2023-12-15–2024-03-15, Monthly, equal explicit role 2024-02-01 | Four references December-1 through April-1: 17/372 + 1/12 + 1/12 + 14/372 = 1/4 |

Actual QuantLib comparison should supply identical explicit reference periods
per component and attest the declared divisor with a full-period probe. Its
schedule-assisted defaults, inferred month counts or different roll anchors
cannot serve as this helper's independent oracle. Day/week cases need exact
rational evidence even when the public reference cannot express the intended
frequency. Specialized rules, adjusted accrual, nonuniform/multiple-tenor
contracts and cash-flow construction remain outside this candidate.
