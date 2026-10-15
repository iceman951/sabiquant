# Proposed modern CDS date plan and maturity integration

Draft for local engineering review, 2026-10-15, Asia/Bangkok, against baseline
`1fde1c1`. This is a design proposal, not an adopted contract, implementation,
financial acceptance or completion of F-SCHED-01. It owns no production change.
The remaining automatic OldCDS rule is explicitly retained below.

The [special-rule research](M7_SPECIAL_RULES_RESEARCH.md),
[CDS source addendum](M7_CDS_SOURCE_ADDENDUM.md) and
[retained public observations](M7_ROLL_REFERENCE_PROBES.md) distinguish financial
sources from reference-library behavior. The coordinating agent additionally
reports direct review of the complete
[Standard CDS Examples](https://www.cdsmodel.com/assets/cds-model/docs/Standard%20CDS%20Examples.pdf)
and [Standard Model news table](https://www.cdsmodel.com/news.html), and primary
indexed text of the final
[December 2015 FAQ](https://www.isda.org/a/vGiDE/amend-single-name-on-the-run-frequency-faq-revised-as-of-12-10.pdf).
This draft uses the stated date-role findings; its author did not independently
repeat that new source inspection. Adoption must link the final source ledger
and exact maturity-table transcription. Public QuantLib outputs do not replace
those sources.

## Proposed public surface

Keep the modern date plan separate from `Schedule`. A small public surface is
sufficient; a builder, public mutable request object, protection-date model and
second calendar wrapper are unnecessary for this operation:

```rust
impl CdsDatePlan {
    pub fn standard_following(
        trade_date: Date,
        unadjusted_maturity: Date,
        calendar: ScheduleCalendar,
        convention: CdsMaturityConvention,
        max_coupons: u32,
    ) -> Result<Self, ScheduleError>;

    pub fn trade_date(&self) -> Date;
    pub fn payment_selection_cutoff(&self) -> Date;
    pub fn unadjusted_maturity(&self) -> Date;
    pub fn calendar(&self) -> &ScheduleCalendar;
    pub fn maturity_convention(&self) -> CdsMaturityConvention;
    pub fn rule(&self) -> DateGenerationRule;
    pub fn coupon_dates(&self) -> &[CdsCouponDates];
}

pub fn resolve_cds_maturity(
    trade_date: Date,
    quoted_tenor: Period,
    convention: CdsMaturityConvention,
) -> Result<CdsMaturityOutcome, ScheduleError>;
```

`CdsMaturityConvention::{Quarterly2009, Semiannual2015}` is an explicit closed
choice of the two researched modern regimes. `CdsMaturityOutcome` has a real
date outcome and a distinct `Matured` outcome; it is not a sentinel Date, an
empty schedule or `Schedule::Zero`. Its exact variant spelling and resolver
errors belong to the integrated maturity contract. The resolver has no calendar
argument: it selects an unadjusted contractual maturity. The plan takes that
resolved date, or an independently supplied existing-contract maturity. Calling
the resolver and constructing the plan are two separately fallible operations.

The explicitly named `standard_following` constructor selects one source-backed
date contract: quarterly March/June/September/December twentieths, Following
payments and the accrual rules below. Following and quarterly frequency are
part of the selected operation, not defaults for omitted options. There is no
generic business-day-convention argument accepting unsupported financial
variants. Existing ordinary schedules continue to provide their independently
specified convention choices.

To avoid an extra public rule enum for two currently constructible plans,
propose `DateGenerationRule::Cds` and `DateGenerationRule::Cds2015`, with exact
canonical strings `CDS` and `CDS2015`. `rule()` maps the explicit modern regime
to the corresponding variant. Recurrent/explicit ordinary builders reject
these variants through their existing incompatible-rule path, after their
normal cap check. Their constructors cannot return a `CdsDatePlan` accidentally.
An eventual `OldCds` variant has canonical string `OldCDS`; add it to the
supported-rule catalog when its automatic operation is implemented. This
staging preserves the requirement rather than advertising a wholly rejected
rule as supported. A nested CDS rule enum is an alternative representation for
review; it does not change the date-role contract.

There is no `From<CdsDatePlan> for Schedule`, `Deref<Target = Schedule>`, implicit
`Leg::fixed_rate` conversion or generated ICMA context. The existing generic
leg factory accrues on raw schedule intervals. A later explicitly specified
CDS coupon adapter must consume these separately represented accrual dates.
This proposal constructs date data, not cash amounts, protection coverage,
accrued rebates, upfront settlement or a priced credit product.

## Inputs, maturity scope and definition identity

Require `trade_date < unadjusted_maturity`. Equality and reversal are typed
input-range errors for this operation. They are not automatic selection of a
historical valuation mode. Require the supplied maturity to be day 20 of
March, June, September or December. Reject another supplied day instead of
silently projecting it or treating it as a quoted-tenor request. This is the
researched standard-plan domain; reference-constructor endpoint projection is
a different behavior and must remain classified in comparison evidence.

Under `Semiannual2015`, explicitly supplied March/September maturities remain
valid. The June/December selection convention applies to the resolver's
specified on-the-run tenor buckets, not to every outstanding contract or every
short tenor. The regime is never selected from the trade year or current date.
The initial resolver's financial scope is the researched single-name
corporate/sovereign convention, with the final FAQ's product exclusions. Sharing
quarterly dates is insufficient evidence to extend it to index/loan/mortgage
products.

`CdsDatePlan` retains the complete supplied trade date, unadjusted maturity,
regime and immutable calendar, together with the computed cutoff and coupon
records. Two regimes can yield equal vectors while remaining unequal defined
plans. The caller's resource cap is not financial identity. A quoted 1Y and 12M
request can select the same maturity; the resolver must preserve original
stored tenor in any error, but a plan built only from the resolved Date does
not pretend to retain an unsupplied tenor or the resolver's provenance.

`trade_date + 1 calendar day` is named `payment_selection_cutoff`. It is
unadjusted and is not a legal-protection commencement, contractual effective
date, settlement date or accrued-rebate date. Those roles require their own
future explicit inputs and source contracts. They must not be fabricated as
extra plan fields. For valid `trade < maturity`, the cutoff is representable
and no later than maturity, including the final supported civil year; checked
arithmetic still preserves any real Date cause.

## Exact quarterly selection and closure behavior

Let `T` be trade date, `S = T + 1 calendar day`, and `M` the supplied unadjusted
maturity. Let `q` range over unadjusted quarterly twentieths and let `A(q)` be
Following adjustment using the one explicit calendar. All required quarter
dates are obtained from civil month coordinates, not repeated adjusted-date
advancement. There is no adjustment of `S`.

The first payment must be strictly after `S`, and its raw quarter must not be
later than `M`. The preceding adjusted quarterly date supplies its accrual
start. This can be selected with a bounded operation even for custom closures
spanning quarters:

1. Resolve `q`, the greatest raw quarterly twentieth on or before `S`, and
   compute `A(q)` under the required coverage guard.
2. If `A(q) <= S`, the first raw coupon is the next quarterly twentieth. Its
   predecessor is `q`. If `q == M`, return `NoEligibleCdsPayment`; there is no
   allowed later raw coupon within this maturity.
3. If `A(q) > S`, the first raw coupon is `q`. Its required predecessor is the
   preceding quarterly twentieth. Resolve and adjust that predecessor.
4. Require the predecessor adjustment to be no later than `S` and strictly
   less than the selected first payment. Generate every subsequent quarter
   through `M`, with strictly increasing adjusted payment dates.

Following under one immutable calendar is nondecreasing in its raw input. If
the predecessor in step 3 were also after `S`, no business day could lie
between that predecessor and `S`; both quarter dates would adjust to the same
next business date. Reject that collision. Do not search farther back, omit a
coupon, reorder dates or merge intervals. Thus a successful plan's chosen
payment is the earliest eligible quarter payment, and a long closure cannot
hide an earlier eligible payment or cause an unbounded backward search.
Earlier historical payments preceding the selected accrual-start boundary are
outside the constructed plan; there is no claim to audit an entire contract
history before that boundary.

The initial selection needs at most three quarter adjustments (the cutoff
quarter and its selected neighboring quarter/predecessor). Reuse already
computed adjustments. Each adjustment retains the calendar's existing maximum
of 366 rejected candidates, Date-range checks, TARGET checks and joint-calendar
costs. An exhausted calendar search returns its typed cause; it is never
reinterpreted as no eligible coupon.

`NoEligibleCdsPayment` is a typed operation outcome represented as an error,
retaining `trade_date`, `payment_selection_cutoff`, `unadjusted_maturity` and
the computed final payment. It does not declare a financial contract invalid,
and is distinct from the quoted-maturity resolver's `Matured` outcome. A valid
`T < M` request can produce it: with a business-day maturity, `T = M - 1` gives
`S = M` and final payment equal to the strict cutoff. Conversely, when maturity
is a holiday and its Following payment is after `S = M`, the final coupon can
still be eligible; retain it if all other checks pass. Never manufacture an
empty successful plan or a quarter beyond `M` to make this operation succeed.

## Coupon records and included final day

The result contains a nonempty chronological slice of immutable
`CdsCouponDates`, one record per selected coupon. For raw quarter boundaries
`q0, q1, ..., qN = M`, set `pi = A(qi)`. The initial boundary is an accrual
precursor, not a selected payment; `p0 <= S < p1` and every `pi < p(i+1)`.

Each record exposes these exact roles without a second public interval type:

| Inspector | Coupon i, numbered 1 through N |
|---|---|
| `unadjusted_start()` | `q(i-1)` |
| `unadjusted_end()` | `qi` |
| `payment_date()` | `pi` |
| `accrual_start()` | `p(i-1)`, included |
| `accrual_end()` | `pi` for i < N; `M` for i = N |
| `accrual_end_inclusion()` | Existing `EndpointInclusion::Exclude` for i < N; `Include` for i = N |
| `accrual_day_count()` | Actual integer days under those endpoint roles |

Ordinary intervals are `[p(i-1), pi)`; the final interval is `[p(N-1), M]`.
The final payment remains `A(M)` even when it lies after the included final
accrual date. No payment-date adjustment extends final accrual. A successful
Following plan with strict adjusted order has `p(N-1) < M`: if its predecessor
adjusted to or beyond `M`, it would share `A(M)` and already fail the collision
check. This proof also covers a single-coupon plan using `p0 <= S < p1`.

Represent the final endpoint as the real Date `M` plus explicit inclusion.
Calculate its actual day count as `M.days_since(start) + 1` in the widened
integer domain, then checked-convert to the declared nonnegative count type.
Do not require an exclusive `M + 1` Date or an out-of-range sentinel. Ordinary
day counts are `end.days_since(start)`. This does not select Actual/360 or
another monetary accrual convention, create a year fraction, or permit an
arbitrary existing day counter to ignore the final inclusion flag.

## Caps, coverage and deterministic failure order

`max_coupons` counts final coupon records, not boundaries or adjustment probes.
Propose the valid configuration range `1..=39_996`, the number of quarterly
twentieths in civil years 0001 through 9999. This is a safe upper bound; not
every cap value is an attainable plan length because a valid precursor is
also required. There is no separate mutable work-budget object. Once the
first raw coupon is selected, its inclusive quarterly distance through `M`
gives the exact required record count. Check that count before allocating the
result or resolving a further precursor solely for an already-over-budget
plan. A failed operation returns no partial value.

Use fallible reservation for the one result vector, with O(N) storage and
O(N) quarter work, plus bounded calendar searches and whole-span TARGET costs.
No independent raw/payment vectors or construction-time per-period heap
objects are necessary. Clone of an already owned calendar retains its std
allocation behavior; this is not a promise to recover every allocation
failure. Month-coordinate arithmetic fits easily in i64 within the civil
range; required Date construction remains checked. There is no need to
construct a next quarter after `M`.

For this direct fallible constructor, the proposed precedence is:

1. Valid coupon cap; increasing `trade < maturity`; quarter-twentieth maturity
   domain. The order differs from ordinary builders only because those have
   a separate constructor that already validates their range.
2. Validate the original inclusive TARGET span `T..M` before projection.
3. Compute cutoff and its required raw cutoff quarter. Validate the union of
   the original span and that raw quarter before its adjustment. Keep each
   adjustment's checked candidate-coverage behavior.
4. Select the raw first coupon, or return the explicit no-eligible outcome.
   Compute the exact final coupon count and check the caller's cap before
   any additional required precursor extension or vector reservation.
5. Resolve the required raw precursor. Validate the whole extended raw span
   `min(T, q0)..M` before remaining adjustment/generation. Then adjust the
   precursor/selected first coupon and check their order and cutoff relation.
6. Generate and adjust remaining quarter coupons chronologically; preserve
   the first calendar/date/collision failure encountered. Before returning,
   validate the complete union through the final adjusted payment as well.

The covered span includes the original input, raw precursor, raw maturity and
all adjusted endpoints. Every forward adjustment candidate is checked under
the same retained calendar policy. In particular, prior-quarter 2025 accrual
for a 2026 trade is not silently authorized by a guard covering only 2026.
Original or extended coverage gaps, TARGET inception, overrides and every
joint rule remain enforceable; Null members do not bypass a TARGET guard.
Near civil MIN, a required previous-quarter date is an actual role-specific
Date failure. Near civil MAX, a final Following adjustment may fail even
though the unadjusted maturity is representable.

## Error and value conventions

Extend the existing non-exhaustive `ScheduleError` and root wrapper rather
than introduce a parallel root error hierarchy. New data-bearing variants
remain non-exhaustive. Use stable schedule-domain kinds and typed Date or
Calendar causes, with their complete `source()` chain. A minimal proposed
inventory is:

| Error | Required context |
|---|---|
| Invalid CDS trade/maturity range | Both supplied dates |
| Non-quarter-twentieth CDS maturity | Supplied maturity and selected modern regime |
| Invalid CDS coupon cap | Requested cap and documented minimum/maximum |
| CDS coupon cap exceeded | Original caller cap and exact required final coupon count |
| `NoEligibleCdsPayment` | Trade, cutoff, unadjusted maturity and final payment |
| Required CDS Date failure | Trade, maturity, role, source input Date and original Date cause |
| CDS calendar failure | Trade, maturity, role/phase, optional coupon index, queried raw Date or guarded span, and original Calendar cause |
| CDS adjusted-order failure | Index of second offending coupon/boundary, both raw quarter dates and both adjusted dates |

Reuse `ScheduleError::AllocationFailed` with a specific `CDS coupon records`
phase. Reuse existing `AdjustedOrder` only if its index documentation is
truthful for the precursor/first-coupon pair; otherwise a CDS-specific
context is preferable. Roles such as `payment cutoff`, `cutoff quarter`,
`initial accrual precursor`, `coupon payment` and `final payment` are static
diagnostic labels, not serialized identifiers or a new public enum for each
internal phase. A precursor failure must not pretend it has a final coupon
index before that index exists. Do not expose mathematically unreachable
invariant failures merely to increase the error catalog.

Plans and coupon records have private fields and borrowed inspectors, and
implement immutable Clone/Debug/PartialEq/Eq/Hash/Send/Sync/'static behavior.
Regimes and catalog enums follow the crate's existing closed-option policy;
extensible failures follow its non-exhaustive error policy. Display/FromStr
on labels cannot reconstruct a contextual date plan or calendar.

## Maturity resolver and automatic OldCDS obligations

The resolver and plan are one implementation tranche, with separate contracts
and fixtures. The exact primary table must specify every accepted quoted
tenor, the date-boundary equality cases, 0M matured windows and short 3M/6M/9M
behavior. Preserve explicit regimes across the December 20 FAQ go-live and
December 21 operational wording. Pin the December 22, 2015 +5Y results:
2020-12-20 under the new convention and 2021-03-20 under the old one. Negative
and day/week quoted tenors require explicit rejection; unsupported month
tenors must not be rounded into supported ones. Do not infer a general
resolver formula from a single date pair or from reference-runtime outputs.

F-SCHED-01's **automatic OldCDS rule remains required**. This proposal does not
remove it, complete that feature, or substitute an explicit-first-coupon
constructor for its automatic behavior. The recorded reference transition
omits the next quarter at distances 1/29 days and retains it at 30/31 days,
but the exact historical financial equality and start-date role remain a
source gap. A future source-backed historical contract, or a clearly labeled
local reference-date contract with separate financial acceptance, must decide
that automatic behavior and validate it independently.

Legacy generation must take a distinctly named `generation_start` unless
historical evidence establishes its contractual role. It must not share a
modern constructor that silently treats that operand as trade date, applies
the modern T+1 selection rule or starts full-quarter accrual. A future legacy
plan may share internal quarter-coordinate and resource helpers; any shared
public result representation must still preserve distinct provenance. The
three named families remain visible in requirement/status tracking, with only
actually implemented operations listed as supported.

## Required evidence before adoption and implementation claims

Use exact E0 for dates, counts, statuses and endpoint inclusion. Meaningful
independent tests must include:

- Primary examples on 2009 March 18/19 and June 20/21, demonstrating strict
  adjusted-payment/cutoff equality; exact ordinary and final day counts.
- Every quarter roll minus one/on/plus one day, weekday/holiday maturities,
  single-coupon and no-eligible outcomes, `trade == maturity`, reversed range,
  and explicit March/September maturities under both modern regimes.
- All source-table maturity rows, both regimes, 0M windows and short tenors;
  identical fixed-maturity plan vectors alongside different quoted-tenor
  resolution, with independent source and reference classifications.
- A closure moving the cutoff quarter after the trade cutoff, a closure
  spanning two quarters causing a precursor/first collision, later payment
  collisions, all-weekday closure and the existing 366-rejection failure.
  No oracle may sort/deduplicate the expected output.
- TARGET original and precursor coverage, adjusted final expansion, inception,
  both verified edges, interior gaps, projection policy, overrides and joints.
- Valid cap minimum/maximum, cap zero/outside range, every oracle's exact cap
  and cap minus/plus one, cap failure before a disallowed precursor extension,
  actual required civil-boundary causes and full root-source chains.
- Independent civil quarter enumeration and integer day counting, including
  leap/century classes; no translated reference recurrence or production
  output used to generate expected values.
- Equality/hash including trade, explicit regime and calendar; shared vectors
  with distinct definitions; immutable cross-thread queries; existing ordinary
  schedules, roll ICMA and generic cash-flow regressions remaining exact.
- Separately retained legacy 0/1/29/30/31-day cases in all quarters, with the
  unresolved automatic rule and financial source gaps explicitly reported.

Run meaningful local debug/release, strict lint/docs, package extraction and
available platform checks under the project workflow. These engineering
results cannot close native-platform execution, owner/financial acceptance,
NI-01 settlement, full M7 or publication gates.
