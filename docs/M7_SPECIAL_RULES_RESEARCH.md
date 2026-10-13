# Remaining M7 roll-rule research

Research snapshot: 2026-10-13, Asia/Bangkok. This document proposes the next
local tranche for F-SCHED-01's ThirdWednesday, ThirdWednesdayInclusive,
Twentieth, TwentiethIMM, OldCDS, CDS and CDS2015 families. It is outside the
cash-flow tranche and changes no source, test, generator, fixture or shared
requirement. It does not grant financial/owner acceptance, complete M7, or
authorize publication. Existing [core](M7_CORE_VALIDATION.md),
[explicit-coupon](M7_EXPLICIT_STUB_CONTRACT.md),
[ICMA](M7_ICMA_REFERENCE_CONTRACT.md) and
[cash-flow](M7_CASHFLOW_CONTRACT.md) definitions remain in force.

Recommendation: implement the four generic civil roll families first under a
written local contract; keep CDS maturity selection and CDS accrual/payment
contracts separate. The 2015 CDS change concerns on-the-run maturity selection,
while premium payments remain quarterly. A fixed-end CDS/CDS2015 date vector
can therefore coincide without establishing equal trade-tenor maturity rules.

## Source ledger and evidential limits

Only primary publishers support the financial facts below. No QuantLib C++
generation implementation was inspected, copied or translated.

| Source and inspection on 2026-10-13 | Verified scope | Limit |
|---|---|---|
| [ISDA-supported Standard CDS Examples](https://www.cdsmodel.com/assets/cds-model/docs/Standard%20CDS%20Examples.pdf), public PDF, pp.1–2 | Quarter-twentieth unadjusted maturities; Following-adjusted payments; first-payment selection after unadjusted T+1; ordinary accrual based on adjusted payment dates and final inclusion of unadjusted maturity | A standard-CDS product example, not a specification of QuantLib's seven generator labels, historical OldCDS cutoff or universal calendar |
| [ISDA December 10, 2015 FAQ](https://www.isda.org/a/vGiDE/amend-single-name-on-the-run-frequency-faq-revised-as-of-12-10.pdf), Q1–4 and Q10–11 | Indexed primary text distinguishes March/September on-the-run rolls, continued quarterly coupons, June/December maturity buckets, and explicit short-end tenors | Direct PDF fetch returned 403; these passages were inspected through the primary document's search index, not a complete PDF/legal-text review. Final FAQ excludes Loan/Mortgage CDS; the earlier October proposal differs |
| [ISDA CDS Standard Model news](https://www.cdsmodel.com/news.html), December 20, 2015 entry | Directly read historical maturity table; reported switch on December 21, 2015 and December 22 five-year example | Distinguish the operational-date wording from the FAQ's December 20 go-live wording. Do not infer a universal legal effective instant or all current product conventions |
| [FpML 5.12 recommended RollConventionEnum](https://www.fpml.org/spec/fpml-5-12-4-rec-1/html/pretrade/schemaDocumentation/schemas/fpml-enum-5-12_xsd/simpleTypes/RollConventionEnum.html) | Indexed primary schema describes roll convention combined with frequency/regular start, IMM as third Wednesday of delivery month, and day-20 monthly roll | Direct page returned 403. It does not prescribe this project's endpoint projection, stub policy or regularity flags |
| [CME SER 9308, December 26, 2023](https://www.cmegroup.com/notices/ser/2023/12/SER-9308.pdf), pp.2–3 | Direct PDF: quarterly IMM Wednesdays; specified US Government Securities calendar adjustment; June 19, 2024 moves to June 20 | This is a particular futures/OTC convention. TARGET or weekends-only is not that market calendar; no automatic calendar or adjustment follows from the weekday alone |
| [ICMA Primary Market Handbook A5, March 2022](https://www.icmagroup.org/assets/documents/ICMA-PMH-Appendix-A5-March-2022.pdf), §§1.1–1.3 | Direct PDF: supplied normal determination dates/frequency define stub accrual; payment adjustment and adjusted interest periods are distinct | Neither this source nor an inspected legacy ISMA text defines all seven QuantLib rule names |
| [Pinned QuantLib v1.43 schedule.hpp](https://raw.githubusercontent.com/lballabio/QuantLib/v1.43/ql/time/schedule.hpp) | Public interface inventory: effective/termination dates, tenor, rule, ordinary/termination adjustments, EOM and coupon-role inputs; separate previous-twentieth helper | Header inventory is not financial authority. Its generation implementation was not read. Pinned dategenerationrule.hpp could not be retrieved via browser or sandbox DNS; no new inspection of that enum header is claimed |

The [July 2015 recommendation](https://www.isda.org/2015/07/08/isda-publishes-recommendation-for-an-amendment-to-the-single-name-cds-roll-frequency/)
and [October FAQ](https://www.isda.org/a/fGiDE/amend-single-name-on-the-run-frequency-faq-revised-as-of-10-13.pdf)
are dated proposals. Their indexed contents must not replace the revised
December FAQ. In particular, the original proposal's broad product scope is
not evidence that the final convention included every loan/mortgage product.
No pre-standardization confirmation was found establishing OldCDS's exact
short-first-coupon cutoff; that remains a source gap.

## Existing public observations: seven cases, not seven specifications

The retained [46-row attested probe](M7_REFERENCE_PROBES.md) and
[fixture](../validation/fixtures/schedule/probes.fixture) were read unchanged.
Each specialized rule has only ONE common input: 2026-02-06..2026-11-17,
3 Months, NullCalendar, both adjustments Unadjusted, EOM false, no coupon roles.
Wheel/runtime attestation belongs to that earlier artifact; no new runtime
execution or wheel-source-commit attestation occurred in this research.

| Rule | Recorded raw-looking Unadjusted output | Flags |
|---|---|---|
| ThirdWednesday | Feb06, May20, Aug19, Nov17 | R R Stub |
| ThirdWednesdayInclusive | Feb18, May20, Aug19, Nov18 | R R R |
| Twentieth | Feb06, Feb20, May20, Aug20, Nov20 | Stub R R R |
| TwentiethIMM | Feb06, Mar20, Jun20, Sep20, Dec20 | Stub R R R |
| OldCDS | Feb06, Mar20, Jun20, Sep20, Dec20 | Stub R R R |
| CDS | 2025-Dec20, Mar20, Jun20, Sep20, Dec20 | R R R R |
| CDS2015 | 2025-Dec20, Mar20, Jun20, Sep20, Dec20 | R R R R |

The returned endpoints can move outside the caller's range. TwentiethIMM's
quarter-twentieth dates are distinct from the financial IMM Wednesday dates;
the former name is a reference-library label. Identical OldCDS/TwentiethIMM or
CDS/CDS2015 vectors here do not prove general equivalence. Reference flags
also need a declared meaning: ThirdWednesday's first R begins off its actual
Wednesday roll. A true consecutive-roll-point regularity definition would
classify that interval differently. Do not silently inherit those flags as
proof of a financially regular coupon or of valid ICMA references.

Fixture SHA-256:
`6e4a2ebbac1a20367c931de3413adf39cce1a63b3b77f44555449bdfd49bfbd7`.
Generator SHA-256:
`2c48705c96a6311922e8483cc47ae383f032555fd56f4c3f53567ca605725d6d`.
The existing 46 fixture bytes and generator were not modified or regenerated.

## Proposed first implementation: explicit civil rolls

These are engineering decisions for review, not assertions of universal market
practice or adopted parity with QuantLib.

Reuse the immutable ScheduleBuilder's explicit calendar and two adjustments.
Add a separate `build_roll(tenor, rule, max_boundaries)` accepting a closed
`SpecialRollRule::{ThirdWednesday,ThirdWednesdayInclusive,Twentieth,TwentiethIMM}`.
No artificial EOM, directional rule, automatic StubPolicy or coupon-role
arguments apply. Existing recurrent/explicit methods keep their contracts.
Unsupported combinations therefore have no implicit financial default; later
roll-stub/explicit-role support requires its own method and contract.

Require positive stored Months/Years, convert to an i64 month stride while
retaining original length/unit in definition identity. TwentiethIMM additionally
requires a stride divisible by three. Reject Days/Weeks, zero and negative
tenors explicitly. A year is twelve months for this coordinate system, while
stored 1Y/12M remain distinct Schedule metadata. EOM Preserve cannot coexist
with a specified twentieth/Wednesday roll; no business-EOM inference applies.

For civil month coordinate m, define W(m) by independently locating the third
Wednesday, and D(m) as that month's twentieth. Derive every point from one
retained anchor month and stride; never repeatedly advance the previous date
or call Calendar::advance to obtain contractual dates.

| Proposed rule contract | Endpoint and lattice behavior |
|---|---|
| ThirdWednesday | Retain requested start/end. Interior points are W(anchor_month+k*stride), k>=1, strictly inside the requested range. No rolled candidate outside the terminal boundary is retained |
| ThirdWednesdayInclusive | Replace both endpoints with W(their respective original months); retain congruent interior month points from the rolled start's month. Reject collapsed/reversed resolved range even when original start<end |
| Twentieth | Retain requested start; select first D on/after start as roll anchor. Resolve terminal to first monthly D on/after requested end. Include anchor-stride points strictly before that terminal; a terminal off that stride is a raw stub |
| TwentiethIMM | Same operation on D in March/June/September/December, with a stride divisible by three. Resolve terminal to next eligible quarter-twentieth independently of stride |

These endpoint choices reproduce the seven-case table's four date vectors,
but the broader contract still needs new distinguishing probes: after-roll
start, 6M/12M stride, end before/on/after the roll, same-month inputs and
off-lattice terminal. A date-vector match must not be described as proven full
rule compatibility. If new probes disagree, classify and review the policy;
do not reconstruct a C++ algorithm or silently change a financial claim.

Propose Regular only when both contractual boundaries are adjacent points of
the retained roll lattice; all retained off-lattice outer intervals are Stub.
No specialized generated interval is NonPeriodic. This deliberately differs
from the recorded off-Wednesday first R; choosing reference-style generator
flags instead is an explicit alternative decision that must not claim actual
roll-point regularity. Long-stub removal is deferred initially.

## Provenance, ICMA and cash-flow composition

Existing Schedule stores output dates and a date seed, but not separately the
original requested range. That is sufficient for the present endpoint-preserving
core and insufficient for endpoint-projecting rolls. Add immutable provenance
with requested start/end, selected rule, stored tenor, anchor month and resolved
roll endpoints, plus borrowed inspectors. Original inputs remain distinct from
contractual output and business-adjusted payment dates. Eq/Hash must include
this provenance; two different input ranges projected to the same vector are
not silently the same defined value. Do not describe a month coordinate as an
ordinary Date::checked_add_months lattice anchor.

The present `build_icma_context` uses clamped civil shifts of its Date seed.
It MUST NOT be reused unchanged for Wednesday rolls: a third Wednesday's
same day three months later is generally not another third Wednesday.
Initial rolled schedules should return a typed unsupported-ICMA-rule error
until a same-roll signed lattice/minimal-hull oracle is supplied. Later support
must keep explicit matching frequency, every required quasi-date, full extended
market coverage and existing E2 rational comparisons. No normal frequency or
reference period may be guessed from projected endpoints or returned flags.

Generic scalar fixed coupons can use declared contractual roll intervals with
explicit quotes. A standard CDS product must not automatically reuse the
current raw-period Leg::fixed_rate factory: the consulted product example
uses adjusted-payment accrual boundaries and special final-day inclusion.
A roll-date plan is not protection coverage, accrued rebate, upfront settlement
or a standard CDS premium leg. These product adapters require separate source,
contracts and tests; analytical cash-flow acceptance does not approve them.

## CDS families: separate date plan and maturity resolver

For CDS/CDS2015, propose a distinct immutable `CdsDatePlan` rather than silently
passing a CDS roll vector into the ordinary raw-accrual Schedule/coupon factory.
Take explicit regime and already-resolved unadjusted maturity; retain all caller
trade/effective/accrual-role dates separately. A separate maturity resolver
must take trade date, quoted CONTRACT tenor and explicit
`CdsMaturityConvention::{Quarterly2009,Semiannual2015}`. No date-based regime
default and no implicit today is appropriate. Coupon frequency remains a
separate 3M concept; a quoted five-year tenor is not the schedule's 3M stride.
The consulted Standard CDS Examples expressly separates its T+1 risk-model
assumption from contractual effectiveness. A generic T+1 date operation is
therefore not a definition of protection commencement, lookback or settlement.
An initial source-backed maturity resolver should state its single-name
corporate/sovereign scope explicitly; index, loan, mortgage and other product
definitions cannot inherit that scope merely by sharing quarter dates.

The directly inspected 2015 news table gives useful exact maturity anchors:
December 22, 2015 + quoted 5Y maps to December 20, 2020 under the new convention,
versus March 20, 2021 under the earlier convention. It also records 3M/6M/9M
short-end examples and 0M windows marked matured. The FAQ gives December 20
go-live while the news describes the market switch December 21. Keep both
provenances and use an explicit regime; do not silently settle that distinction.

Zero quoted tenor is not Schedule::Zero: it can mean an already matured
contract, so a resolver needs a typed outcome rather than manufacturing a
positive schedule or a sentinel Date. Reject negative/day/week contract tenors
in an initial narrowly defined resolver. Pin the exact 0M boundary and supported
quarter-multiple short tenors against the final primary table before coding.
March/September maturity buckets can remain valid off-the-run choices; never
force a supplied maturity into June/December merely because a regime is named
CDS2015. Historical data also require the selected regime to be retained.

OldCDS remains reference-only research: characterize public first-coupon
behavior around 0/1/29/30/31 calendar days before each quarter roll, but do not
attribute a guessed threshold to ISDA. Implementing a narrowly labeled legacy
reference-date algorithm can be a later local decision after attested probes;
financial OldCDS contract acceptance needs the missing historical source.

## Coverage, resources and error policy

Keep the original positive input-range validation and final-boundary cap
2..=3,652,059. Proposed precedence: limit; rule/tenor/domain; original TARGET
span; required endpoint resolution/range; whole extended span; lattice/resource
generation; ordinary/terminal adjustment; strict raw/adjusted order. The guarded
span is min(requested_start,raw_first)..max(requested_end,raw_last), not merely
the output points. This prevents a projection from skipping an original gap or
adding unverified/pre-launch history unnoticed. Overrides and null-member
joints cannot waive that guard. Adjustment candidates keep their existing
coverage/search errors; final raw endpoint uses the explicit terminal convention.

Every required projected endpoint is checked and cannot be dropped to fit a
cap. An unused out-of-range NEXT lattice point can terminate generation only
when civil coordinates prove it is beyond the already-resolved endpoint.
Fallible vectors and monotone month-coordinate progress bound generation and
storage by final boundary count, plus existing calendar/span costs. The civil
range bounds roll lattices; no unbounded calendar search or allocation proof
by forced system OOM is appropriate. Resource caps are per operation and not
financial metadata.
With stored i32 tenors, month stride is at most 12*i32::MAX; multiplying by
3,652,059 possible steps stays below 9.42e16 and fits i64. Required civil-date
conversion still checks 0001..9999 rather than assuming an integer fit makes
the date representable.

Add precise typed contexts for unsupported roll tenors/options, resolved-range
collapse, required roll-date role/cause and unsupported rolled ICMA, alongside
existing cap/allocation/calendar/order failures. Preserve requested and resolved
dates and stored tenor/rule in errors. A required start/end roll error should
name its role rather than present a pre-generation index as a final array index.
All new types follow immutable Clone/Debug/value/Send/Sync/static traits,
closed-option expectations or non-exhaustive extensible errors. No contextual
identifier string is a complete serialized plan.

## Meaningful independent validation before a supported claim

| Layer | Required matrix / independent truth |
|---|---|
| Civil roll points | All 4,800 months in a 400-year Gregorian cycle; enumerate day 15..21 for Wednesday independently, count exactly two earlier Wednesdays; construct monthly/quarter twenties from civil coordinates. Century classes 1900/2000/2100 plus Date MIN/MAX |
| Endpoint projections | Before/on/after each roll, day 1 and month end, same-month collapse, input start/end off different strides, whole endpoint shifts both directions; requested/raw/adjusted/provenance and flags separate |
| Tenor domains | Positive 1/2/3/4/6/12/18M and year aliases, zero/negative/i32::MAX, day/week rejection, quarter-family nonmultiples. Include large unused next candidate versus required overflowing endpoint |
| Anchor and flags | Independent span selection by month congruence, not recurrence loops; genuine adjacent-roll predicate; initial/final short and exact-lattice controls; two input ranges mapping to one output remain unequal definitions |
| Calendars | All 49 ordinary/terminal pairs, custom closed month, 366 rejected search candidates, Nearest ties, raw/adjusted collisions/reversals; CME's June 2024 holiday can be a named custom-calendar control without claiming a market calendar implementation |
| TARGET | Strict/projected policies, 1999 inception and both verified interval boundaries, prior-quarter 2025 expansion, beyond-2028 terminal, interior gaps, overrides and all joint rules; union span and adjustment candidates checked even Unadjusted |
| ICMA/coupons | Initial typed rolled-ICMA exclusion; later exact same-roll minimal hull, off-roll outer spans, long quasi periods, explicit frequency aliases and exact Fraction accrual. Standard-CDS adjusted accrual/final inclusion remain separate from ordinary coupon tests |
| CDS maturity | Final December FAQ/news anchors, every quarter date ±1 day around 2015 switch, March/September on-the-run versus June/December maturity, 0M matured outcomes and 3/6/9M cases; compare fixed-maturity CDS/CDS2015 schedule versus DIFFERENT trade-tenor maturity resolver outcomes |
| CDS product controls, later | Primary first-payment strict inequality: trades 2009-Mar18 versus Mar19, and Jun20 versus Jun21; first payment can change at adjusted-payment/T+1 equality. Pin ordinary adjusted accrual versus final unadjusted maturity inclusion separately |
| Legacy/reference | New attested bounded public probes for every special family and rejected option, especially OldCDS front-distance controls. Record raw outputs/errors/flags unchanged, including unordered or collapsed observations; classify differences instead of relaxing E0 |
| Value/resource | Exact final cap and cap±1, all structured causes/root chains, clone/Hash/threads, no partial results. Independent Python civil enumeration fixtures and Rust hand/cycle tests; generators remain outside package |

Dates, flags and statuses require exact E0. No date tolerance can hide lost
boundaries or endpoint roles. Future ICMA retains current E2; later CDS amounts
need their own independent source/rational/precision model rather than relying
on equality with a reference engine. Source financial claims, public runtime
observations and locally chosen civil algorithms remain separate evidence.

The actionable next decision is the four-rule generic contract, particularly
regularity and endpoint projection, then attested distinguishing probes and
independent cycle/span fixtures. CDS date/maturity plans and historical OldCDS
remain subsequent scoped work. Existing owner/financial/API, native primary
platform, market maintenance, NI-01 and full M7 release gates remain open.
