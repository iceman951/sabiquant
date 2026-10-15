# Proposed local automatic OldCDS schedule

Draft for engineering review, 2026-10-15, Asia/Bangkok. This proposal is not
adopted or implemented. It changes no production code, frozen fixture, modern
CDS contract or financial acceptance status. The proposed operation implements
an automatic local reference-date rule under F-SCHED-01; its date operands
carry no inferred legal trade, effectiveness or protection role.

The [legacy source review](M7_CDS_LEGACY_SOURCE_REVIEW.md) remains authoritative
about the evidence gap. Markit's prose, its effective-date diagram and the
2003 ISDA Definitions establish useful distinctions but do not jointly define
the exact thirty-day equality or identify a library's generic start operand.
The local engineering choice below does not resolve that financial question.
An applicable historical confirmation or explicit contemporary specification
is still required for historical financial acceptance.

## Frozen public observations actually inspected

This audit read the existing
[roll fixture](../validation/fixtures/schedule/roll-probes.fixture), its
[generator](../validation/generator/schedule_roll_probes.py) and the
[attestation report](M7_ROLL_REFERENCE_PROBES.md). No reference runtime was
rerun, no fixture was regenerated, and no QuantLib C++ implementation was read.
The fixture retains SHA-256
`88e2bacdc1ab999c1979f19c5d2f4fa485f3492cceab040cca6cc927e9edf35a`.

There are **98 OldCDS observations: 88 values and 10 constructor errors**:

| Group | OldCDS rows |
|---|---:|
| Original control | 1 |
| Before/on/after roll, 3M/6M/12M | 27 |
| Short/same-month/year-end spans | 6 |
| Tenor domain | 8 |
| EOM | 4 |
| Explicit first/penultimate roles | 5 |
| Ordinary/terminal conventions | 14 |
| First-roll distances | 20 |
| Fixed end versus quoted maturity | 12 |
| TARGET | 1 |

The 20 distance controls use all four 2026 quarterly twentieths and start
distances 0/1/29/30/31 calendar days, with requested end 2026-12-31. For distances
1 and 29, the upcoming quarter date is absent; 30 and 31 retain it. Distance
zero retains the start once and next includes the following quarter. These
are dated software observations, not an established legal cutoff.

The six short-span observations supply a critical terminal-boundary exception:

| Requested start → end, 3M/Null/Unadjusted | Recorded dates | Reference flags |
|---|---|---|
| 2026-03-01 → 03-30 | Mar01, Jun20 | Regular |
| 2026-03-17 → 03-19 | Mar17, Mar20 | Stub |
| 2026-03-19 → 03-20 | Mar19, Mar20 | Stub |
| 2026-03-20 → 03-21 | Mar20, Jun20 | Regular |
| 2026-03-21 → 03-31 | Mar21, Jun20 | Regular |
| 2026-12-30 → 12-31 | Dec30, 2027-Mar20 | Regular |

Consequently, an unconditional rule to drop the first quarter at distance
less than thirty would contradict the second and third rows. The projected
terminal must remain even when it is also that near quarter. The proposal
therefore suppresses only a strictly interior quarter.

Reference regularity also depends on the observed construction shape. For
start 2026-03-20 and end 2026-11-20, the first interval Mar20→Jun20 is reported
Stub; for start Mar20 and end Mar21, that same sole returned interval is
Regular. The zero-distance March/June/September controls report their first
interval Stub, whereas the December control returns one Regular interval.
These flags cannot be adopted as proof of actual quarter-lattice regularity.

The other observed options are broader than the proposed fixed-quarter API:

- Negative 1M is rejected. Zero months returns the two original requested
  boundaries, reports rule Zero and tenor 0 Years, and reports Regular. The
  local named operation has no zero-tenor substitution.
- Positive 1 Day and 1 Week are accepted in the common span and produce 244
  and 37 dates respectively. The daily result ends at original 2026-11-17;
  the weekly result projects to December 20. Thus quarter-end projection is
  not a demonstrated universal statement across arbitrary reference tenors.
- The 1M/2M/4M requests first include March 20, then use their observed month
  strides, with December 20 terminal. The 1Y common control returns
  `[Feb06, Mar20, Dec20]`, flags Stub/Regular. At start March 19/20/21 and end
  November 20, both 6M and 12M observations return `[start, Jun20, Dec20]`,
  flags Stub/Regular. Do not infer a complete arbitrary-tenor algorithm from
  these few vectors.
- All four EOM=true observations reject the combination. All five supplied
  first/penultimate-role combinations reject it, including reversed roles.
- The 14 weekday-calendar convention controls use start February 6, requested
  end November 21 and fixed 3M. Every Unadjusted terminal is December 20;
  every Preceding terminal is December 18. Ordinary Following/modified
  Following/half-month Following give June 22 and September 21; Preceding/
  modified Preceding give June 19 and September 18; Nearest gives June 19
  and September 21. The first input is a business day in every such control,
  so these rows do not establish how the reference adjusts a holiday start.
- The TARGET Following/Following common control returns
  `[Feb06, Mar20, Jun22, Sep21, Dec21]`. It supplies no permission to bypass
  the local calendar's verified-coverage policy.

Exactly **63 rows** have fixed 3M, EOM=false and no explicit coupon roles and
therefore fit this proposed operation's expressible input scope. An independent
read-only Python civil-quarter calculation reproduced all 63 date vectors
under the proposed terminal-preserving cutoff. With the genuine adjacent-roll
flags proposed below, **47 also agree in flags and 16 differ only in flags**.
The 14 adjustment controls and one TARGET control use the hand-pinned
February/quarter dates above; their candidate neighborhoods do not intersect
the six 2026 TARGET closures recorded in the existing roll report. This is an
exploratory comparison of a proposal, not a production test or universal
parity claim. The 35 other rows remain explicitly outside the fixed-quarter
surface; their values and errors must remain preserved unchanged.

## Recommended operation and exact local date contract

Adopt a distinctly documented operation only after review:

```rust
impl ScheduleBuilder {
    pub fn build_old_cds(&self, max_boundaries: u32)
        -> Result<Schedule, ScheduleError>;
}
```

For this operation the existing builder's `start` is named **generation_start**
in documentation and error context. Its `end` is the **requested end**. Neither
is automatically a trade date, effective date, protection date or already
resolved legal maturity. There is no T+1 calculation, maturity-tenor resolver,
modern full-first-coupon convention or business-day cutoff calculation here.

The selected operation declares a fixed quarterly twentieth lattice. This
is a deliberate first local OldCDS domain, not proof of every accepted
reference tenor. Do not add unused tenor/EOM/stub/explicit-role arguments.
The operation automatically decides the first retained quarter; an explicit
first-coupon constructor is not a substitute for this requirement.

For `S = generation_start`, `E = requested_end`, retain the builder invariant
`S < E`. Define `Q(x)` as the first March/June/September/December twentieth on
or after x. Let `A = Q(S)`, `R = Q(E)` and `d = A - S` in calendar days.

1. Retain S exactly as the first raw boundary; resolve R independently of
   the front cutoff and retain it exactly as the final raw boundary.
2. Include every quarterly twentieth strictly between S and R, except omit
   A when `0 < d < 30` and `A < R`.
3. At `d = 0`, S already is A; do not append a duplicate or suppress the
   following quarter. At distances 30 and 31, retain A if it is interior.
4. If A equals R, retain R regardless of the cutoff. Do not extend R another
   quarter solely to obtain a first interval of at least thirty days.
5. Generate every point from checked civil quarter coordinates. An adjusted
   date or a repeatedly clamped start is not a recurrence anchor.

This positive domain always has at least two distinct raw boundaries because
`R >= E > S`. The only omitted point is a possible first interior quarter.
No periodic endpoint is dropped to fit a cap or repair a later collision.

Apply the builder's explicit ordinary convention to S and every interior
boundary, and its explicit terminal convention to R. This is the existing
generic Schedule contract, now selected expressly for this local operation.
The frozen controls do not prove its holiday-start parity; add new attested
holiday-start probes before making any broader reference claim. All seven
conventions remain expressible engineering options without implying that
they are historical CDS market conventions. Reject adjusted collisions or
reversals with the ordinary typed error; do not sort, deduplicate or merge.

Define Regular only when both raw boundaries are quarterly twentieths three
calendar months apart. Every other interval is Stub, including a retained
short terminal, an off-quarter first boundary and a first interval spanning
an omitted quarter. None is NonPeriodic. This matches the existing civil-roll
meaning and deliberately preserves the 16 observed flags-only distinctions.

## Return Schedule, with truthful provenance

Returning ordinary `Schedule` is appropriate for this expressly local
raw-interval operation. It provides the established raw/adjusted vectors,
bounded strict-order checks and optional generic analytical composition.
A separate `LegacyDatePlan` would duplicate those invariants without resolving
the missing historical date roles. Prefer a separate product plan later if a
source-backed historical contract needs adjusted accrual, protection or
settlement dates that differ from this local raw schedule.

Add `DateGenerationRule::OldCds`, canonical `OldCDS`, only with the implemented
automatic builder. Existing recurrent/explicit builders reject that label
with their normal incompatible-rule error. No alias redirects it to
TwentiethIMM, CDS or CDS2015.

The returned metadata should be:

| Inspector | Proposed value |
|---|---|
| `generation_rule()` | `OldCds` |
| `tenor()` | `Some(Period::new(3, Months))`, declared by the named operation |
| `stub_policy()` | None; the local cutoff is not an ordinary Long/Short option |
| `end_of_month()` | None |
| `explicit_stub_dates()` | None |
| `roll_provenance()` | None; this remains the four existing civil-rule definition |
| `lattice_anchor()` | None; this inspector retains its ordinary clamped-date-seed meaning |
| `old_cds_provenance()` | Borrowed immutable definition described below |

Update `tenor()` documentation from exclusively "supplied" to "declared"
stored tenor so that a fixed named operation is truthful. Do not pretend the
caller supplied EOM Ignore, a directional rule or a generic stub policy.

A small immutable `OldCdsProvenance` is needed because current Schedule lacks
an original requested end once it is projected. Proposed private fields and
borrowed/value inspectors retain `generation_start`, `requested_end`,
`initial_quarter = A`, `resolved_end = R`, and `omitted_quarter: Option<Date>`.
The optional value is present only for an actually omitted interior A; it is
absent when A is the required terminal or duplicates S. This is enough to
reconstruct the complete fixed rule without another public options enum.

Schedule equality/hash must include this provenance. Two requested ends
projecting to the same R remain distinct defined schedules. The cap is an
operation resource limit, not financial identity. Existing Schedule values
have no OldCDS provenance. All new values retain private immutable fields,
Clone/Debug/Eq/Hash/Send/Sync/'static behavior; the small provenance can be Copy.

Generic `Leg::fixed_rate` composition remains possible precisely because this
operation declares ordinary raw accrual intervals. Documentation and examples
must call it generic analytical composition, not a validated historical CDS
premium leg. The modern `CdsDatePlan` remains distinct and cannot be implicitly
converted to this Schedule to erase its adjusted-accrual/final-inclusion roles.

## ICMA and full automatic-rule scope

Recommend implementing the fixed-quarter ICMA lattice in this same tranche.
The named operation is periodic, and returning `NonPeriodicIcma` for it would
misdescribe the result. The known quarterly lattice also provides a small,
independently testable extension of current minimal-hull machinery.

Require explicit matching Quarterly frequency under the existing normalized
tenor/frequency contract. The minimal determination hull starts at the greatest
quarterly twentieth on or before raw S and ends at R. Include every adjacent
quarter, including any omitted first coupon boundary. Business adjustment and
the original E do not replace these raw reference endpoints. The retained A
is a real quarter point; an internal signed 3M lattice from that day-20 anchor
is exact and can share existing arithmetic without inventing public Forward,
EOM or TwentiethIMM metadata.

For example, S=2026-02-19, E=2026-11-17 produces raw first interval Feb19→Jun20.
Its quarterly ICMA determination dates include 2025-Dec20, Mar20 and Jun20;
the first interval's exact fraction is `29/(4*90) + 1/4 = 119/360`. The omitted
March coupon still supplies a required quasi-boundary. This is analytical
ICMA arithmetic, not a claim that historical CDS premiums use ICMA.

Reuse the existing ICMA cap/frequency/domain precedence, minimal-hull coverage,
fallible allocation and checked source chain. A schedule beginning in early
year 0001 can succeed while its required prior-quarter ICMA hull fails;
similarly a 2026 raw schedule can succeed under strict TARGET while its
2025-December quasi-reference fails coverage. Both are expected independent
operation outcomes. If ICMA is deliberately staged separately, use an explicit
unsupported-rule error and record the remaining work; do not label a periodic
schedule nonperiodic or claim that stub omission removes quasi-periods.

## Resources, guards and errors

Reuse `ScheduleBuilder::MIN_BOUNDARIES = 2` and `MAX_BOUNDARIES = 3_652_059`.
The final cap includes S and R after the possible one-point omission. It does
not count the suppressed point or adjustment candidates. A full representable
quarter plan can have 39,997 raw boundaries: Date MIN followed by all 39,996
quarter twentieths through 9999-12-20. The generic cap remains a uniform API
bound even though this particular lattice cannot approach its maximum.

After required endpoint resolution, quarter-coordinate counts determine the
exact final size; check it before allocation/adjustment. Generate at most that
many dates, reserve fallibly, and do not construct an unused quarter after R.
Generation/storage are O(final boundaries), plus calendar and TARGET span
costs. An omitted quarter can make cap N succeed where the unsuppressed
quarter vector would need N+1; it cannot justify deleting the required R.

Proposed deterministic precedence:

1. Existing builder construction validates S<E. `build_old_cds` first validates
   its configured boundary cap; there are no tenor/EOM/role options to validate.
2. Guard the entire original inclusive span S..E under the retained calendar.
3. Resolve required terminal R, then initial quarter A. Retain checked Date
   causes for any required quarter outside the supported range.
4. Guard the complete expanded raw span S..R. Because R only moves forward,
   this is also the union of original and resolved spans.
5. Determine the actual omission and exact final count, enforce the caller's
   cap, then allocate/generate raw dates and regularity.
6. Use ordinary Schedule adjustment/order checks with explicit final convention.

Do not bypass original/expanded TARGET gaps, inception, overrides or joint
guards. Retain every adjustment candidate's existing search/coverage errors.
There is no preliminary previous-quarter dependency for raw generation; only
the separate ICMA operation can require that backward extension.

Reuse existing InvalidBoundaryLimit, BoundaryLimitExceeded, AllocationFailed,
Calendar and AdjustedOrder failures where their indices and phases remain
truthful. A new non-exhaustive `OldCdsDate` variant can retain generation_start,
requested_end, role (`terminal quarter` or `initial quarter`), original role
input and underlying DateError. The quarter's original role input supplies
the signed month displacement; required projection failures must not present
a fabricated final array index. Terminal-first resolution makes an end after
9999-12-20 fail as a required terminal, rather than quietly truncate the plan.
Use the existing root ScheduleError wrapper and complete source chain.

## Required independent evidence and remaining gates

Before an implemented claim, add a written adopted contract and separate
immutable validation artifacts without altering the original 98 observations.
Required tests include:

- All four quarters across a 400-year cycle, start distances 0/1/29/30/31 and
  before/on/after roll controls; independent civil enumeration, exact flags,
  actual omitted-quarter provenance and terminal-preserving short spans.
- Requested ends immediately before/on/after quarters, same-quarter inputs,
  December crossings, different ends projecting to one vector, and the
  maximum 39,997-boundary civil span.
- Exact cap and cap±1 after omission, required overflow after 9999-12-20,
  unused post-terminal coordinates, all typed causes and immutable reuse.
- All 49 ordinary/terminal convention pairs with holiday starts and terminals,
  custom closures, all-weekday closures, 366-search exhaustion, collisions and
  reversals. No sorting/deduplication in either production or expected output.
- TARGET original/forward-expanded spans, both verified intervals, gaps,
  inception, projection policies, overrides and every joint rule.
- Exact classification of the 63 expressible frozen requests: dates all equal,
  47 dates/flags equal and 16 flags-only differences; preserve the other 35
  out-of-scope observations. New attested probes should cover holiday starts,
  short terminal/cutoff combinations, leap-year cutoffs and all terminal
  conventions; observations must not replace the declared local contract.
- Quarterly ICMA minimality and omitted quasi-dates, rational first-stub
  fractions, explicit frequency mismatches, shared caps and backward civil/
  TARGET failures; generic raw-coupon composition clearly identified.
- Identifier/catalog changes and unchanged modern CDS plans, maturity
  resolution, ordinary/explicit schedules, civil rolls and cash-flow behavior.

The automatic local rule advances the remaining named F-SCHED-01 work. It does
not substitute supplied coupon roles for automatic generation, erase broader
reference-tenor differences, resolve historical financial equality/start roles,
or close owner/financial/native-platform/NI-01/full-M7/publication gates.
