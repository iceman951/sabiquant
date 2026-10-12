# Local M7 schedule research and readiness questions

Research snapshot: 2026-10-12. This document was requested as local preparation
after M6 work. It proposes contracts and engineering decisions for review;
it does not authorize M7 implementation, accept M5/M6, change the roadmap,
or establish financial/API approval. No M7 production code, tests or fixtures
were added for this analysis. Remote writes and publication remain disabled.

## Scope and prerequisite status

[ROADMAP §4](ROADMAP.md#4-future-milestones-provisional) assigns M7 schedules,
basic cash flows, fixed-rate coupons, legs, flat-rate NPV and accrued interest.
Its dependencies are M5 and M6. Full acceptance includes schedule differential
evidence for all rules and independent analytics oracles. A schedule-only
prototype would therefore be a tranche, not completion of M7.

[F-SCHED-01](FEATURE_MATRIX.md#7-future-domains-grouped) lists ten rule families: Forward,
Backward, Zero, ThirdWednesday, ThirdWednesdayInclusive, Twentieth,
TwentiethIMM, OldCDS, CDS and CDS2015. The current [REQUIREMENTS](REQUIREMENTS.md)
specifies date, period, calendar and ICMA behavior but contains no FS-SCHED
contract defining those families, stub selection or adjustment collisions.
Names in the feature matrix do not resolve those financial semantics.

The roadmap still records M2–M6 as local candidates or in progress. Local
tests do not establish their formal acceptance. [M5 local readiness](M5_LOCAL_READINESS.md)
retains owner API review, native primary-platform CI, unresolved US/AFB source
decisions, TARGET maintenance and release gates. [M6 local readiness](M6_LOCAL_READINESS.md)
records delegated staging while those formal dependencies remain open.
The existing M6 staging exception must not be described as automatic M7
implementation authorization.

| Gate | Current evidence | Needed before the corresponding claim |
|---|---|---|
| Formal dependencies | Reviewed local M5 corrections and local M6 numerical evidence exist | Record dependency acceptance, or a specific authorized local M7 exception; neither follows from this document |
| Schedule specification | Existing date/calendar/ICMA clauses constrain a future design | Write FS-SCHED domains, rule-specific roll/stub/endpoint semantics, error precedence and traceability; obtain the required approval |
| API/value design | Immutable values, private fields, typed options and visible fallibility are adopted principles | Review the proposed builder, calendar ownership, regularity and reference-period views |
| Rule sources | ICMA, ECB and the pinned QuantLib schedule interface were inspected below | Resolve Wednesday/twentieth/CDS rule contracts and sources; inspect and attest the pinned behavioral reference without translating its implementation |
| Exact amounts | OQ-06 permits analytical `f64`; NI-01 remains open before cash settlement | Explicitly bound analytical cash-flow scope or choose an exact settlement representation before adding settlement features |
| Validation | Existing date/calendar/day-count tests and schema-v1 fixture infrastructure are reusable | Implement the mandatory matrix below, independent exact-date oracles and pinned differential fixtures, then run native primary-platform tests |
| Market maintenance | TARGET has an explicit coverage policy and an unassigned maintainer role | Preserve the policy in schedule composition; appoint a maintainer and satisfy the release maintenance gate before market-calendar release |
| Milestone/release | No M7 implementation or acceptance is recorded | Complete the roadmap Definition of Done; release and publication remain separate decisions |

US 30/360 and AFB gaps need not block research or a deliberately restricted
schedule tranche, but excluded conventions must remain excluded. The
foundation acceptance issue cannot be resolved by relabeling that tranche.

## Inspected sources and limits

Only primary external sources support the claims in this section. No
reference implementation was copied or translated.

| Source inspected on 2026-10-12 | What it establishes | Limit |
|---|---|---|
| [QuantLib v1.43 schedule interface](https://raw.githubusercontent.com/lballabio/QuantLib/v1.43/ql/time/schedule.hpp) | The rule-based interface distinguishes ordinary and termination adjustment, first and penultimate dates, tenor, calendar, rule and EOM; an explicit-date constructor accepts unchecked metadata | An API/header is a behavioral-reference inventory, not a financial specification or an independently executed parity result |
| [ICMA Primary Market Handbook Appendix A5, March 2022](https://www.icmagroup.org/assets/documents/ICMA-PMH-Appendix-A5-March-2022.pdf), Version 2 §§1.3(a)–(b) and notes | Stub accrual is the sum of actual overlap days divided by determination-period days and contractual annual frequency; periods beyond two can be accommodated. The note distinguishes payment adjustment from interest-period adjustment | It does not fully specify this project's generation builder, every market roll, or every adjusted-accrual product |
| [ECB dated public holidays](https://www.ecb.europa.eu/ecb/contacts/working-hours/html/index.en.html) | The TARGET-marked 2026 closures include Good Friday 3 April and Easter Monday 6 April; corresponding dated anchors also appear for 2027–2028 | Office holidays without the TARGET marker are not TARGET closures; this inspection does not extend the existing verified intervals |

Fetching the pinned `dategenerationrule.hpp` through the browsing tool did
not succeed. The ten names above are recorded in the repository feature
matrix; this research does not claim a newly verified definition of the seven
specialized rules. Search results from unpinned mirrors are not used as
authority. No current-master algorithm is substituted for the v1.43 baseline.

## Existing APIs and composition constraints

| Existing capability | Relevant contract | Consequence for schedules |
|---|---|---|
| `Date` | 0001-01-01 through 9999-12-31; total ordering; checked day/week/month/year arithmetic; calendar month-end and nth-weekday queries | All contractual, adjusted and quasi-coupon dates must remain representable; generate roll dates directly from an anchor |
| `Period` / `Frequency` | Signed stored units; normalized equality; checked arithmetic; periodic mappings are optional; `Once` has no period | Require a positive recurrence tenor. Do not infer a contractual coupon frequency from arbitrary tenors such as `5M` |
| `Calendar::advance` | Nonzero Days count business days, Weeks count calendar weeks; months/years clamp and then apply calendar advancement EOM semantics | It is not a universal schedule recurrence function. `7D == 1W` does not imply equal calendar advancement |
| `Calendar::adjust` | Seven explicit conventions; range and bounded-search errors; modified conventions can fail in a month with no business day | Generate contractual boundaries first, then adjust explicitly and validate chronological order again |
| `TargetCalendar` | Guarded owned value; checked classification and advancement; disjoint verified intervals; no unchecked plain-calendar escape | The builder must own and retain the guard, including projection policy, overrides and joint-calendar behavior |
| `DeterminationPeriod` / `ActualActualIcma` | Positive, ordered contiguous supplied periods; explicit periodic frequency; no guessed schedule or references | Generate quasi-coupon boundaries from the contractual roll lattice and hand them to the existing validating ICMA constructor |
| `AccrualConvention` / `InterestRate` | Owned day-count context and explicit compounding; checked date-based fractions and discounting | Keep schedule construction independent of rates; cash-flow builders choose accrual/payment semantics explicitly |

Sources for these local API facts are [time](../src/time/mod.rs),
[date](../src/time/date.rs), [period](../src/time/period.rs),
[calendar](../src/calendar.rs), [TARGET](../src/calendar/target.rs),
[ICMA](../src/daycount/icma.rs) and [rates](../src/rates.rs).
The corresponding contracts are FR-DATE-04/05/06, FR-PER-02/03/05,
FR-CAL-04/05, FS-BDC, FS-DCC §9.5,
[explicit ICMA](M3_ICMA_CONTRACT.md) and [TARGET](M3_TARGET_CONTRACT.md).

## Proposed value and builder surface

The following names describe a reviewable design, not settled Rust APIs.
Apply [API principles](API_DESIGN_PRINCIPLES.md) P1–P10: immutable owned
results, private invariant-carrying fields, no implicit today or conventions,
descriptive enums and domain errors. Construction consumes or clones owned
context; inspectors borrow data. No public user-implemented calendar trait
or shared mutation is needed for this tranche.

`ScheduleBuilder` would collect explicit `start`, `end`, generation rule,
calendar, interior/start adjustment, termination adjustment, EOM policy,
stub specification and generation limits. Recurrent rules additionally need
a positive tenor. A distinct Zero configuration needs no artificial zero
tenor or periodic frequency. Construction completes only through a fallible
`build`, which returns a fully validated `Schedule`.

The schedule should retain both `contractual_dates()` and `adjusted_dates()`,
with one-to-one boundary correspondence, plus one interval record per
adjacent boundary pair. An interval's regularity is defined on the original
roll lattice, never by the length of its adjusted dates. Prefer a descriptive
`PeriodRegularity` value to an unexplained vector of booleans. Generated
intervals know their regular/stub status; an explicit-date constructor cannot
invent missing roll provenance. If explicit-date schedules are admitted,
optional generation metadata expresses absence and is validated when supplied.

The builder should use explicit plain-calendar and TARGET entry points over
a private owned sum type, following `Business252`'s existing pattern. This
avoids exposing an unchecked TARGET conversion or adding an open trait.
Read-only calendar/provenance inspection must reveal which guard is retained.
Equality and hashing should compare the defined discrete schedule value and
retained metadata; they must not imply financial equivalence of different
coupon or payment conventions.

A `ScheduleError` should preserve the rejected range, tenor, rule/options,
stub dates or offending interval index. Calendar/date/ICMA causes should be
nested typed errors, with the schedule domain itself retained as the
crate-level error's `source`. Proposed distinct errors include invalid range,
nonpositive tenor, incompatible rule options, invalid stub dates, an
unsatisfied stub policy, an adjustment collision, date-limit exhaustion and
reference dates outside the supported range. Final stable kind names and
precedence belong in FS-SCHED; this document does not reserve them.

## Proposed delegated engineering decisions

These are recommendations for a future explicitly authorized tranche.
They are not recorded as adopted M7 decisions.

| ID | Question | Recommendation and reason |
|---|---|---|
| M7-D01 | Which rules start the first tranche? | Forward, Backward and Zero first, with independent contract-derived date models. Treat the seven specialized families as separate source/contract work. Full ten-rule acceptance remains open unless scope is formally revised |
| M7-D02 | Does a schedule have an empty or reversed range? | Require `start < end` and at least two boundaries for every rule. Equal dates and reversed ranges are typed errors; backward generation still returns chronological dates |
| M7-D03 | How do recurrence units behave? | Days and Weeks count calendar days on the contractual lattice; months/years use anchored clamping. Use a separately named business-day recurrence only if later sourced and authorized. Never dispatch recurrence through generic calendar advancement |
| M7-D04 | How does EOM activate? | Specify activation, seed and endpoint exceptions independently for schedules. Prefer calendar EOM for the contractual lattice; any business-EOM roll needs explicitly named semantics. Reusing `EndOfMonth::Preserve/Ignore` must not imply that `Calendar::advance` defines schedule behavior. Reject EOM for day/week rules and incompatible specialized rolls |
| M7-D05 | How are stubs requested? | Use a typed policy for no stub, short stub, long stub or validated explicit coupon dates. Direction locates the automatic residual: Forward at the end, Backward at the start. Do not silently create a long stub by a heuristic or reinterpret explicit dates |
| M7-D06 | Which dates accrue? | Retain contractual boundaries separately from adjustment. The first ICMA helper uses contractual accrual and quasi-coupon dates. Adjusted-accrual products need their own explicit contract and source; do not let payment adjustment silently alter interest |
| M7-D07 | What if adjustment collapses or reverses adjacent dates? | Reject with raw/adjusted dates and interval indices. Do not sort, deduplicate or drop a coupon. Any alternative consolidation policy would be a financial behavior decision needing its own evidence |
| M7-D08 | Can strict TARGET bridge its coverage gap? | Validate the declared generation span and any required reference extension under the retained policy, including Zero and Unadjusted configurations. Verified endpoints cannot justify an unchecked intervening span. A checked interval operation can supply this validation without exposing TARGET internals |
| M7-D09 | May frequency be guessed from a stub? | No. ICMA helpers require explicit named periodic frequency and verify consistency with the supported recurrence. An arbitrary positive tenor may generate dates but need not support automatic ICMA contexts |
| M7-D10 | How is work bounded? | Add an explicit maximum generated-boundary count with a supported-date-range hard ceiling; use checked arithmetic and fallible reservation. Calendar searches keep their existing bound. A resource failure produces no partially usable schedule |
| M7-D11 | How broad is the raw-date constructor? | If needed, validate chronological order, distinctness, minimum length and matching metadata lengths; retain supplied order. Defer metadata-free constructors if no concrete client requires them |
| M7-D12 | Can analytical `f64` cash flows imply exact settlement? | No. Bound a first tranche to analytical amounts with existing numeric validation. NI-01 remains a prerequisite for exact settlement features; choose that scope explicitly before cash-flow implementation |

Before coding, resolve the details left by M7-D04: whether a business-EOM
seed activates preservation; which seed wins with explicit first/penultimate
dates; whether Following may move a calendar-EOM boundary into the next
month; and how start and termination EOM exceptions differ. Existing
advancement rules and QuantLib parity are inputs to that decision, not
interchangeable definitions of a contractual market roll.

For stub policy, proposed precise behavior is: a short residual retains the
last regular lattice boundary; a long residual merges it with the preceding
regular interval. Without a residual, neither creates an artificial stub.
A total range shorter than one tenor can form a short single interval; it
cannot meet a requested long-stub policy with no regular interval to merge.
Explicit first/penultimate coupon dates must be strictly interior, ordered,
and compatible with the selected middle lattice. If both are provided, the
middle region must be exactly regular rather than inventing another hidden
stub. The meaning of a sole explicit date as the directional lattice anchor
must be pinned in FS-SCHED and tested in both directions.

## Independent bounded algorithm proposal

Generate contractual boundaries in wide integer calendar coordinates from a
fixed anchor. For Forward use `anchor + k*tenor`; for Backward use
`anchor - k*tenor`. Compute each candidate from the same anchor rather than
repeatedly adding to the preceding clamped or adjusted date. For example,
an anchored monthly sequence from 2024-01-31 reaches 2024-02-29 and
2024-03-31; repeated clamped additions would drift to 2024-03-29.

Validate range, positive recurrence, compatible rule/stub/EOM options and
limits before generation. Establish the regular anchor explicitly, detect
strict progress at each step, and stop by comparing wide coordinates with
the declared endpoint before constructing a candidate past the date range.
A regular sequence ending at `Date::MAX` must not fail merely because the
unused next candidate would be outside the supported range. Actual required
quasi-coupon extensions outside that range are errors.

Apply the chosen residual/explicit-stub rule to the contractual sequence,
then construct interval regularity from the unchanged lattice provenance.
Apply the declared endpoint/interior adjustments through the checked owned
calendar adapter, preserving every error. Validate adjusted strict order;
an adjustment collision aborts the build. Do not repair it by mutating the
contractual dates or regularity records.

For ICMA, extend the same regular lattice outward far enough to cover the
actual stub accrual, split long stubs over every intersecting determination
period, and pass those ordered contiguous positive periods to
`ActualActualIcma::new` with the explicit frequency. Short-first and
long-first references extend backward; final stubs extend forward. References
are not manufactured from adjusted payment dates or observed day lengths.
Coverage, EOM, resource and date-range checks apply to these extra dates too.

At most 3,652,059 distinct boundaries and 3,652,058 positive one-day intervals
fit `Date::MIN..=Date::MAX`. A configured limit can be much lower. With `n`
boundaries, recurrence and structural validation are O(n) and storage O(n).
Adjustment additionally depends on calendar-query cost and its existing
366-consecutive-holiday bound; a wide joint calendar must not be advertised
as constant-cost classification. Strict TARGET span validation may visit
coverage/year blocks even when the output has few dates. Date lookup can use
binary search, and the existing ICMA query remains O(log n + k).
There is no clock, I/O, global cache, unbounded search or reference-code
translation in this proposal.

## Mandatory meaningful validation matrix

Date/rule/status comparisons are exact E0; no epsilon excuses a missing,
extra, reordered or silently merged boundary. ICMA and future coupon/NPV
amounts use their existing independently justified numerical policies.
All rows below require public API coverage, structured error context and
an independent model or hand-audited expected schedule.

| ID | Scenario set | Required observation |
|---|---|---|
| TS-SCHED-01 | Equal/reversed range; Zero; zero/negative/i32-extreme tenor; missing/contradictory options | Typed domain errors or exactly two valid Zero boundaries. No implicit today, frequency or convention |
| TS-SCHED-02 | Forward/Backward regular lattice; lengths of one, two and many periods; anchored Jan-31 and Feb-29; 1Y versus 12M and 7D versus 1W | Correct chronological boundaries and regularity. Equivalent regular lattices agree; direction is not assumed equivalent when anchor or residual differs |
| TS-SCHED-03 | Short/long first and final stubs; span shorter than tenor; residual-free Long; explicit first, penultimate and both dates; invalid/equal/reversed/off-lattice explicit dates | Stub position and policy are exact. Middle regularity is preserved; no invented internal stub or removed contractual coupon |
| TS-SCHED-04 | Every month end over a Gregorian 400-year cycle; 1900/2000/2100; calendar/business EOM seeds; Preserve/Ignore; both directions and endpoint conventions | Pin activation, leap/clamp behavior, start/terminal exceptions and next-month adjustments; no repeated-clamping drift |
| TS-SCHED-05 | Null/weekends/custom calendars crossed with all seven adjustment conventions; month boundary; day 15; Nearest tie; a whole closed month | Raw-to-adjusted mapping is exact, including no-business-day-in-month and bounded-search errors. No unsupported monotonicity assumption |
| TS-SCHED-06 | Daily/weekend collision; modified-convention reorder; adjusted start equals end; explicit stub boundary collapses onto another boundary | Reject collision/reversal with both original and adjusted context. Never silently deduplicate, sort or change period flags |
| TS-SCHED-07 | TARGET 2026-04-03/06 through 7 April; strict/projected years; inception; verified endpoints spanning 2003–2025; Unadjusted/Zero; Nearest at 2026-01-01 | Checked policy persists. Easter closures come from TARGET-marked ECB dates. Coverage and candidate-search failures remain visible even on identity-like paths |
| TS-SCHED-08 | TARGET overrides and joint calendars, including a null member under JoinBusinessDays; different build order; clone/reuse across threads | No guard escape, mutable sharing, inferred coverage or calendar-name-only cache. Equal defined inputs reproduce exact dates |
| TS-SCHED-09 | Regular/short/long ICMA references at both ends, EOM and leap boundaries; accrual spanning more than two quasi-periods; arbitrary tenor/frequency mismatch | Exact reference boundaries and overlap-day metadata; positive contiguous periods; explicit frequency; exact rational oracle fractions and existing E2 budget |
| TS-SCHED-10 | Date::MIN/MAX; unused out-of-range next candidate; required out-of-range quasi-date; configured boundary limit ±1; all-closed 366/367-day search cases | Correct stopping, explicit required-reference/resource/range errors and bounded work; no panic, hang or partially returned schedule |
| TS-SCHED-11 | Explicit date vectors: empty/singleton, duplicates, unordered values, metadata length mismatch and unverifiable regularity | Validate supplied order rather than repairing it. Missing provenance is explicit; it never becomes inferred ICMA data |
| TS-SCHED-12 | ThirdWednesday and Inclusive: days around each third Wednesday and both endpoints; Twentieth/IMM around month/quarter transitions; CDS families around their effective regime boundaries | Only after rule-specific sources/contracts exist: exact independent fixtures and pinned v1.43 comparisons for every supported family; reject incompatible stub/EOM options explicitly |
| TS-SCHED-13 | Contractual accrual versus adjusted payment; coupon at start/end; valuation on payment date; signed/zero amounts; near-cancelling multi-flow NPV | Cash-flow ownership and inclusion policies are explicit; no accidental payment-date accrual. Separate independent amount oracles and NI-01 scope decisions are required for the cash-flow tranche |
| TS-SCHED-14 | All value/error traits; builder independence; root/domain/nested error chain; safe optional inspection and boundary lookup | Immutable Send/Sync values, correct equality/hash contract, stable error kinds/context and no unchecked indexing |

Use a cycle-exhaustive exact-date model for month/day/weekday rolls, plus
small exhaustive products of direction, tenor, stub, EOM and adjustment
options. Add seeded non-cycle cases, supported-range endpoints, hostile
calendars and explicit resource exhaustion rather than only familiar bond
examples. Keep raw, adjusted, regularity and quasi-period metadata in separate
fixture columns so a final date vector cannot hide lost contractual meaning.

Differential fixtures must use the attested external QuantLib v1.43 baseline,
including its binary/source version provenance, while the Rust crate remains
std-only and consumes committed data. Predeclare intentional divergences
such as invalid explicit-date rejection, equal-date domain rejection,
collision policy and TARGET coverage guards. Compare every supported rule,
not merely matching happy-path vectors. A discrepancy is investigated and
classified; reference behavior is not automatically the financial oracle.

## Suggested research handoff

First resolve M7-D01 through D10 into an FS-SCHED contract and reviewable API
sketch, with one hand-audited regular and each stub example in both directions.
Next establish independent recurrence/EOM models and a pinned specialized-rule
source ledger. Then reassess the dependency and authorization gates for a
bounded local schedule tranche. Cash-flow and settlement scope needs its own
decision before claiming full M7 work. This document supplies preparation;
it records no newly implemented feature or accepted milestone.
