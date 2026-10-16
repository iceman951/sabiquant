# Local US 30/360 and AFB implementation contract

Adopted for local implementation on 2026-10-16, Asia/Bangkok, from `1c16daf`.
The owner's continuing authority covers this implementation and validation;
milestone, financial, native-platform and publication acceptance remain
separate. The [API design](M3_US_AFB_API_DESIGN.md) is a historical proposal.
This document freezes the selected semantics and numerical evaluation.

## Source decisions and limits

The [US source review](M3_THIRTY360_US_SOURCE_REVIEW.md) distinguishes a
dated, explicit EOM-qualified definition from a universal interpretation of
the label 30/360. The SSCM author's fourth-edition description confirms an
EOM-cycle change; its public sample omits the detailed day-count chapter.
The reviewed ROC Revised CDE Technical Guidance v4 (December 2025), Annex
Table 4, A001 FIX column, explicitly gives the four ordered rules adopted
below. Other mapping columns are different definitions. This operation pins
that reviewed definition, not an unspecified current FIX release, an ISO/FpML
equivalence or a claim to have retrieved the complete SIFMA book. OQ-15 is
locally decided with explicit context; the unread full SIFMA source and
historical/universal financial acceptance remain open.

The [AFB source review](M3_AFB_SOURCE_REVIEW.md) records direct visual
inspection of ISDA's official seven-page 1999 paper, page 1 §2(c), and the
1998 memo. ISDA explicitly describes backward whole years, the February
28-to-29 treatment and included-start/excluded-end counting. This closes the
previous unread-source gap for ISDA's published AFB interpretation. The
original September 1994 AFB publication was not retrieved. The executable
equality rule and examples below are the adopted operational interpretation,
not assertions that the paper prints those numerical examples.

No QuantLib implementation was inspected. Public reference calls are
behavioral evidence and cannot supply a missing financial definition.

## Explicit US context

Add immutable Copy/Eq/Hash `daycount::Thirty360Us`, with private Boolean
`end_of_month_security`. Its entire public surface is:

```text
new(end_of_month_security: bool) -> Self
end_of_month_security(self) -> bool
canonical_identifier(self) -> &'static str  // "30/360 (US)"
day_count(self, start: Date, end: Date) -> i32
year_fraction(self, start: Date, end: Date) -> Result<f64, DayCountError>
```

Both explicit Boolean choices are valid. There is no Default, zero-argument
constructor, context-free parser, or scalar `DayCount` variant. A name alone
does not provide the security's qualification; it is never inferred from the
calculation dates. Calendar EOM advancement is a different concept.

For an increasing interval, retain original February-end predicates, then
apply these rules in order:

1. If the flag is true and both original dates are the last day of February,
   change the ending day number to 30.
2. If the flag is true and the original start is the last day of February,
   change the starting day number to 30.
3. If the ending day is 31 and the current starting day is 30 or 31, change
   the ending day to 30.
4. If the starting day is 31, change it to 30.

The count is `360*(Yend-Ystart) + 30*(Mend-Mstart) + Dend'-Dstart'`.
Flag-off reduces algebraically to the existing Bond Basis forward formula;
this does not erase their distinct context definitions. Source-year/month/day
arithmetic is exact and bounded in i32. Year fraction is one binary64 division
of the exact integer count by 360, TOL-E1 (zero ULP from the rounded rational).

## Stateless AFB

Add `DayCount::ActualActualAfb`, canonical `Actual/Actual (AFB)`, exact alias
`Actual/Actual (Euro)`. `DayCount::ALL` has six variants. Its integer count is
the actual civil-day difference, independently of its year fraction. Generic
Actual/Actual identifiers remain ambiguous; scalar US parsing still fails
because the security context is required.

On a chronological interval S..E, repeatedly propose moving the current
remaining end back one calendar year. Keep its month/day, clamp February 29
to 28 in a common target year, and promote a resulting February 28 to 29 in
a leap target year. Accept the candidate while it is on or after S. N is the
number of accepted moves and E* the final remaining end. If no move is
accepted, E* is the original E without a February transformation.

Let r be actual days in `[S,E*)`. Set B=366 iff this residual half-open
interval contains February 29; otherwise B=365. Whole years do not cause
the residual denominator to change. Compute the forward binary64 result in
this exact order: `q = f64(r)/f64(B)`; `fraction = f64(N) + q`. Use TOL-E2,
at most four ULP from independently rounded `(N*B+r)/B`. There is one rounded
division and at most one rounded addition, no cancellation or many-term
summation. Equal and whole-year cases retain exact integer results.

Consequences to test include 2024-02-28 to 2025-02-28: N=1, E*=2024-02-29,
r=1, B=365, result `1 + 1/365`. The excluded residual February 29 does not
select 366. From 2023-03-01 to 2024-02-29, N=0 and r=365 with B=365; a
residual fraction can itself equal one. Simple anniversary and arbitrary
split-additivity invariants are not asserted.

## Constant-time production derivation

A production annual scan is prohibited by the existing scalar O(1) contract.
For every positive number of backward moves, a February 28/29 end becomes
the target year's last February day; other month/day pairs remain unchanged.
Let d=E.year-S.year. For d>0, compare the candidate in S.year with S:
choose N=d when it is on/after S, otherwise N=d-1. For d=0 choose N=0.
Candidate dates decrease strictly with their target year, proving maximality.

Derive the residual end's day-of-year using the original end and target leap
status, with the original-end special case at N=0. Its year is S.year or
S.year+1. Integer day-of-year arithmetic gives r and February-29 membership
using at most those two years. No intermediate Date, out-of-range anchor,
year-zero/year-10000 sentinel, allocation, calendar query or clock is needed.
Whole years are at most 9,998 and all integer arithmetic is bounded. The
independent validation oracle must actually repeat yearly rollback, rather
than duplicate this production selection formula.

## Signed, numerical and downstream contracts

Both operations retain OQ-14: normalize to chronological dates, evaluate
forward, then negate for reversal using the same context. Every zero returns
positive zero. All valid dates from MIN through MAX belong to the domain;
there is no new Date error variant. Existing finite-result validation and
typed `DayCountError::Value` wrapping remain in place.

Add `AccrualConvention::Thirty360Us(Thirty360Us)` and its From conversion;
AFB uses the existing Scalar variant. Complete immutable context participates
in Eq/Hash, rates, equivalent-rate targets, coupons and legs. Neither US choice
is defaulted by an adapter, including zero-rate/zero-amount operations.
Both computations take constant time and consume one existing accrual-work
unit per query. ICMA and Business/252 retain their existing scan charges.

The public type census becomes 73; error variants remain 103. In the existing
hash-context matrix, adding AFB plus both US flags raises eleven definitions
to fourteen. This is definition identity, not a claim that outputs always
differ. The two-term AFB evaluation does not change ICMA compensation or
exact reduction of supplied cash-flow terms.

## Required validation and remaining gates

Retain existing fixtures/generators byte for byte. Add separately named new
independent integer/rational oracles and attested public-reference evidence,
pinning this contract and the finalized source records. Cover every Gregorian
cycle day with documented rotating profiles, all supported years' February
and month-end classes, all endpoint inclusion/equality/reversal paths, exact
MIN/MAX and extreme spans, both US flags and rule-order distinctions, AFB
zero-versus-positive rollback count and leap centuries. Compare AFB's discrete
decomposition to a repeated-year oracle before evaluating its rational.

Hand tests must distinguish each financial rule, preserve actual signed
reference differences, exercise immutable identity and threads, and compose
with every supported compounding/frequency family and raw/adjusted coupon
roles. Check exact and one-short work caps using independently derived times
and cash amounts. Do not generate expected values by calling production.

Run native debug/release, strict Clippy, warning-denied public/private docs,
formatting, meaningful package extraction, available compile-only platforms,
generator checks, complete public traits/errors and source-region coverage.
No forced allocator failure or corrupted private states are needed. Financial
acceptance, unread original/source material, native Linux/Windows execution,
NI-01 settlement, full M3/M7 and publication remain separate gates.
