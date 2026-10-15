# Local modern CDS date contract

Adopted for implementation on 2026-10-15 (Asia/Bangkok), from baseline
`1fde1c1`. This specifies an analytical date operation. It is not financial
acceptance, a legal confirmation, a premium leg, or completion of M7.
The proposed [maturity design](M7_CDS_MATURITY_DESIGN.md) and
[date-plan design](M7_CDS_DATE_PLAN_DESIGN.md) are historical review inputs.
This document fixes the choices made for implementation and oracle pinning.

## Evidence and scope

The [Standard CDS Examples](https://www.cdsmodel.com/assets/cds-model/docs/Standard%20CDS%20Examples.pdf)
was directly read. It supplies quarterly twentieths, Following payments,
strict adjusted-payment selection after unadjusted trade plus one calendar
day, adjusted ordinary accrual boundaries, and included raw final maturity.
Its dated numerical examples are independent test anchors.

The [Standard Model news](https://www.cdsmodel.com/news.html) was directly
read. Its December 2015 maturity table and old/new five-year example were
checked against indexed primary text of the final
[December 10 ISDA FAQ](https://www.isda.org/a/vGiDE/amend-single-name-on-the-run-frequency-faq-revised-as-of-12-10.pdf).
Direct FAQ retrieval failed; this is not a complete PDF review. The final
FAQ identifies the single-name corporate/sovereign scope and preserves
quarterly short tenors. Loan/mortgage and index contracts are not inferred
from shared dates. Operational transition wording differs between sources;
the regime is therefore always an explicit input.

Proleptic years 0001–9999, every representable nonnegative quarter-multiple
quote, resource limits, immutable provenance and structured errors are local
engineering definitions. They do not establish historical trading practice,
liquidity, legal protection, effective dates, accrued rebates or settlement.
The [legacy source review](M7_CDS_LEGACY_SOURCE_REVIEW.md) leaves OldCDS's
automatic first-coupon boundary and generic-start role unresolved. OldCDS
remains required by F-SCHED-01 and is not advertised as implemented here.

## Quoted maturity

Public exports live in `schedule`:

- Closed `CdsMaturityConvention::{Quarterly2009, Semiannual2015}`, exact
  canonical names, `ALL`, Display and strict parsing; no default or aliases.
- Closed `CdsMaturityOutcome::{Date(Date), Matured}`.
- Immutable Copy/Eq/Hash `CdsMaturityResolution`, with `trade_date()`,
  `quoted_tenor()`, `convention()` and `outcome()` inspectors.
- `resolve_cds_maturity(trade_date, quoted_tenor, convention)` returns
  `Result<CdsMaturityResolution, ScheduleError>`.

Accept stored nonnegative Years or Months divisible by three. Reject Days,
Weeks, negative lengths and other Months before any zero/range shortcut.
Keep stored length/unit as separate identity fields: 1Y/12M and 0Y/0M retain
different definitions despite normalized Period equality. Convert to months
in i64, including extreme i32 inputs.

Quarterly2009 selects the first quarterly twentieth strictly after trade,
then adds quoted months. Semiannual2015 selects the last March/September
twentieth on or before trade, adds three months, then quoted months. For
Semiannual2015 only, zero quote returns Matured if that abstract base is no
later than trade. No anchor Date is required: a base in year zero can give
Matured or a valid year-one result. For a non-matured result, construct only
the final month with `trade.checked_add_months(target_month - trade_month)`,
then select its twentieth, retaining the actual DateError on failure.

The formula and boundary table in the maturity design define the equivalent
signed month-coordinate arithmetic with Euclidean remainder. Calendar
adjustment is absent. Explicit maturity supplied to a date plan is never
re-resolved, and March/September supplied maturities remain valid in either
regime. A resolution is not implicitly converted into a successful plan.

## Standard Following date plan

`CdsDatePlan::standard_following(trade_date, unadjusted_maturity,
ScheduleCalendar, convention, max_coupons)` returns an immutable Eq/Hash plan.
Its fixed operation selects quarterly twentieths and Following payments;
there is no omitted business-day-convention default. Inspectors are
`trade_date()`, `payment_selection_cutoff()`, `unadjusted_maturity()`,
`calendar()`, `maturity_convention()`, `rule()` and `coupon_dates()`.
Rule is `DateGenerationRule::Cds` / `Cds2015`, canonical `CDS` / `CDS2015`.
The rule catalog grows to nine actually constructible labels. Ordinary
recurrent/explicit builders reject these two plan rules.

Require trade strictly before maturity, and maturity on a March, June,
September or December twentieth. Define cutoff S as unadjusted trade plus
one calendar day. It is named solely for payment selection. Select the
first adjusted quarterly payment strictly after S, subject to its raw
quarter being no later than maturity. Return a typed NoEligibleCdsPayment
when no payment meets that condition; do not create an empty successful
plan or infer invalidity of a legal contract.

Resolve the greatest raw quarter q <= S and adjust it Following. If A(q)
<= S, first raw payment is q + three months (or no eligible payment when
q equals maturity), and q is the accrual precursor. If A(q) > S, first raw
payment is q and precursor is q - three months. Require precursor payment
<= S and strict adjusted ordering throughout. Following is nondecreasing:
if the predecessor also exceeds S it shares the same next business date
and fails ordering. This bounds selection work without backward searching,
deduplication, skipped quarters, or sorting. Historical periods preceding
the required precursor are outside this operation.

`CdsCouponDates` is immutable Copy/Eq/Hash. Its inspectors return:

| Inspector | Role |
|---|---|
| `unadjusted_start()` / `unadjusted_end()` | Consecutive raw quarterly twentieths |
| `payment_date()` | Following-adjusted raw end |
| `accrual_start()` | Following-adjusted raw start, included |
| `accrual_end()` | Adjusted end for ordinary coupons; raw maturity for final coupon |
| `accrual_end_inclusion()` | EndpointInclusion::Exclude ordinarily, Include finally |
| `accrual_day_count()` | u32 actual days respecting the included final endpoint |

Count final days as raw-end minus adjusted-start plus one in the integer
domain. Do not manufacture M+1. Strict Following ordering proves the final
accrual start precedes raw maturity; a start at/after maturity would collide
with its payment. A plan owns trade, raw maturity, explicit regime, calendar,
cutoff and coupons. The cap does not affect identity. It does not retain an
unsupplied quoted tenor. There is no implicit Schedule, ICMA context or
fixed-rate-leg conversion: those APIs accrue on different date roles.

## Resources, coverage and failures

`MIN_COUPONS = 1`, `MAX_COUPONS = 39_996`. This is a safe bound counting all
quarterly twentieths in the civil range; not every count is attainable with
a required representable precursor. Exactly count selected coupons before
reserving one vector. Use fallible reservation; no forced allocation failure
or fabricated invalid private value is required for coverage. Calendar clones
retain ordinary std allocation behavior.

Failure precedence:

1. Validate cap, increasing trade/maturity, then quarter-twentieth maturity.
2. Guard the full original TARGET span trade through maturity.
3. Construct cutoff and its raw cutoff quarter; guard its union with the
   original span and adjust the quarter under the retained calendar.
4. Select first coupon or return NoEligibleCdsPayment. Compute exact required
   count; enforce cap before an additional precursor extension/allocation.
5. Construct precursor and guard the whole extended raw span. Reuse the
   cutoff-quarter adjustment, adjust its neighbor, and enforce strict order.
6. Generate every required quarter through maturity in order, with no unused
   post-maturity date. Guard the union through final adjusted payment.

Every Following search retains its calendar's 366-rejection limit, civil
range failures and per-candidate TARGET checks. Full spans preserve inception,
interior coverage gaps, verified/projection policy, overrides and joint guards.
Plain Null calendars or joint Null members never bypass a retained TARGET
guard. Each failure returns no partial plan.

Extend non-exhaustive ScheduleError; root Error already wraps it. Existing
InvalidRange is used with start=trade/end=maturity; AllocationFailed uses
phase `CDS coupon records`. New variants and stable kinds are:

| Variant | Kind suffix after `schedule.` | Context |
|---|---|---|
| InvalidCdsTenor | invalid_cds_tenor | trade_date, tenor, convention |
| CdsMaturityDate | cds_maturity_date | trade_date, tenor, convention, Date cause |
| InvalidCdsMaturity | invalid_cds_maturity | maturity, convention |
| InvalidCdsCouponLimit | invalid_cds_coupon_limit | requested, minimum, maximum |
| CdsCouponLimitExceeded | cds_coupon_limit_exceeded | maximum, exact required |
| NoEligibleCdsPayment | no_eligible_cds_payment | trade_date, payment_selection_cutoff, unadjusted_maturity, final_payment |
| CdsDate | cds_date | trade_date, unadjusted_maturity, role, input, Date cause |
| CdsCalendar | cds_calendar | trade_date, unadjusted_maturity, phase, optional index, start/end queried span, Calendar cause |
| CdsAdjustedOrder | cds_adjusted_order | zero-based coupon index, both raw and both adjusted boundaries |

Date roles: `payment cutoff`, `cutoff quarter`, `initial accrual precursor`,
`coupon quarter`. Quarter construction uses trade as its input and a signed
month offset to the required quarter; no artificial source anchor is built.
Calendar phases: `original coverage`, `cutoff quarter coverage`,
`cutoff quarter adjustment`, `precursor coverage`, `precursor adjustment`,
`coupon adjustment`, `final coverage`. Coverage uses index None; adjustments
use None until the selected coupon role is known, then zero-based coupon
index (precursor remains None). Single-date adjustment has start=end=raw date.
All Date/Calendar causes remain inspectable through the root source chain.

## Validation obligations

Use exact E0 comparisons for dates, outcomes, counts and inclusion. Retain
primary numerical anchors, full Gregorian-cycle independent enumeration,
MIN/MAX and extreme/stored tenor cases, strict roll and cutoff equality,
calendar closures/collisions/search exhaustion, TARGET whole-span policies,
exact/minus/plus caps, definition identity, thread sharing and complete
existing regressions. Pin deterministic external oracle generators and
fixtures. Run native debug/release, strict Clippy/docs, actual package
extraction and available compile-only platform checks. Coverage claims must
name instrumentation limits and unreachable/proof-excluded error branches.

Engineering validation cannot close owner/financial acceptance, OldCDS,
NI-01 settlement, native Linux/Windows execution or publication gates.
