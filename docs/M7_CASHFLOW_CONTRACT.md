# Local M7 analytical cash-flow contract

Adopted on 2026-10-13 under continuing local-only development authority after
`5ef7af9`. This advances F-CF-01 and the flat NPV/outstanding-accrual part of
F-CF-02. It does not close NI-01 exact settlement, bond settlement/ex-coupon
entitlement, later risk/yield analytics, specialized schedule rules or formal
milestone/platform/release gates. Primary financial evidence and public-header
inventory are in [cash-flow research](M7_CASHFLOW_RESEARCH.md); the choices below
are explicit generic analytical definitions, not universal market conventions.

## Values and plain coupon formula

`cashflows` supplies PaymentDateInclusion (Include/Exclude, no default),
SimpleCashFlow, FixedCouponRate, FixedRateCoupon, CashFlow (Simple or
FixedRateCoupon), Leg and CashFlowError. All fields are private, immutable and
Send+Sync. Discrete inclusion is Eq/Hash; values containing binary64 implement
PartialEq only, comparing all retained terms/context, never merely the amount.
There is no contextual parser, clock, global setting or mutable/lazy cache.

`SimpleCashFlow::new(payment_date, amount)` accepts a finite signed amount.
`FixedCouponRate::new(decimal_rate, accrual: impl Into<AccrualConvention>)`
accepts a finite signed annual decimal rate and retains the entire convention
through one internally shared immutable Arc. Cloning a quote/coupon shares this
context in O(1); equality of independent contexts can still scan their content.
Arc construction and ordinary Clone retain std's system-OOM behavior; growing
library flow/reduction vectors reserve fallibly. No claim makes every allocation
recoverable, and no unsafe/nightly allocator workaround is introduced.

`FixedRateCoupon::new(start,end,payment_date,notional,coupon_rate)` requires
start<end, finite signed notional, valid complete accrual coverage and a finite
nonnegative year fraction, in that order. Exact input zeros are canonical +0.
Its eager amount is N*r*YF(start,end), with SIMPLE annual accrual independently
of the separately supplied discount InterestRate's compounding. Zero notional
or rate cannot waive a required convention/coverage query. A zero fraction on
distinct dates is valid. No relation between payment and raw accrual dates is
inferred: early, ordinary or delayed payment is explicit caller data.

The private triple product multiplies the exact supplied binary64 significands
in at most 159 bits, then rounds once, nearest/ties-even. It recovers finite
MAX*2*0.5 and min_sub*0.5*2 without intermediate overflow/underflow. Nonzero
exact products rounding to zero or infinity are typed numeric errors. This is
correct rounding of the supplied rounded YF, not exact financial accrual or
decimal currency settlement. Coupon amount/fraction and all source terms have
borrowed/infallible inspection.

## Legs, inclusion and outstanding accrued amount

`Leg::new(flows:Vec<CashFlow>,max_flows:u32)` checks valid cap, count, then the
first payment-date reversal. Caps 0..=Leg::MAX_FLOWS (3,652,059) are supported;
zero permits an empty leg. Duplicate dates are retained, never sorted, merged
or dropped. This maximum is a resource policy; arbitrary duplicate-date flows
are not bounded by a civil-date theorem. The accepted cap is not value metadata.

`Leg::fixed_rate(schedule,notional,&coupon_rate,max_flows)` checks cap/count
before coupon work, then builds one coupon per raw contractual interval. Its
payment date is the corresponding adjusted END. No adjusted-accrual substitution,
extra payment adjustment/lag, implicit redemption, guessed frequency, rebuilt
termination context or trimmed ICMA provenance is introduced. Zero supplies one
nonperiodic interval when the caller supplies an appropriate convention.
The leg retains flow terms; retain the original Schedule separately for its
calendar/generation provenance. Schedule intervals are disjoint: ICMA reference
overlap work is O(n+M), plus partition searches, and Business/252 visited days
are bounded by the original complete raw span.

At cutoff v, a flow is eligible iff payment>v or payment==v with Include.
Simple flows have zero accrued amount. A fixed coupon's outstanding accrued
amount is +0 when ceased or v<=start; N*r*YF(start,v) when start<v<end;
the cached full amount when v>=end and still eligible. Partial fractions query
the original convention, rather than scaling full amount by elapsed-day ratio.
Include at an early payment reports clipped earned interest, not necessarily
the full payment due. Delayed completed coupons can coexist with later active
coupons. This generic unpaid-coupon analytic does not define a bond clean price.

`FixedRateCoupon::accrued_amount(as_of,inclusion)` performs at most one bounded
partial query. `Leg::accrued_amount(as_of,inclusion,max_accrual_work:u64)` sums
eligible coupon results. Structural zero/cached paths create no extra queries.

## Flat NPV, contextual work and failure order

`Leg::net_present_value(valuation_date,&discount_rate,inclusion,max_accrual_work)`
first charges one unit and validates the quote's equal-date valuation anchor,
even for empty/all-excluded legs. Each eligible flow is then discounted DIRECTLY
from that same valuation date with the existing present_value_between operation.
No adjacent factor chaining is validly inferred for Simple/hybrid methods.
Included zero amounts still validate their complete horizon/factors; past or
excluded flows create no reversed query. Coupon and discount accrual conventions
are independent. The existing M4 factor-required domain and NumericalBoundary
errors remain authoritative, even if amount scaling could rescue an unavailable
factor or later flows could cancel a failed individual PV.

The caller-selected u64 work budget is explicit, and every value including zero
is valid. Before EACH required query charge 1 plus the number of overlapping ICMA
reference periods (found by immutable partition searches), or 1 plus absolute
calendar-span days for Business/252. Scalar/termination forms charge 1. Equal
date queries charge 1; clipped/ceased/cached accrual paths charge zero. ICMA work
count uses overlap of supplied periods, even when a later coverage check fails.
Charges bound interval-scanning work; binary-search, fixed-date operations and
calendar-definition/tree/override costs remain additional declared factors.
Direct repeated ICMA prefixes can cost O(nM) when the caller explicitly permits
that much work; no unconditional linear-time claim is made. Budget bounds this
to O(n+budget) interval work. Future ordered-prefix optimization requires its
own exact-order proof and parity tests; it is not silently included here.

If the next charge exceeds the budget, return AccrualWorkLimitExceeded BEFORE
that query, with original flow index (None for anchor), phase, caller maximum,
used and required-at-least cumulative charge. No future horizon is prevalidated.
With sufficient budget, original eligible-flow order determines the first
accrual/factor/product failure. Maximum input count and supported date/reference
range bound cumulative charges below 1.34e13, safely inside u64. A budget is
per operation, not stored mutable state or a financial convention.

Finite rounded contributions are retained in a fallibly reserved vector and
reduced by exact_sum: exact sum of SUPPLIED rounded terms, one final ties-even
rounding and canonical zero. Overflowing prefixes can cancel, but independently
failed terms cannot. This is not an exact dot product. Errors preserve typed
ValueError, RateError and SumError through CashFlowError and root Error, original
indices/phases and allocation/resource context. No partial leg/valuation escapes.

## Accuracy and executable obligations

Triple product is 0 ULP against independent Fraction of its binary64 operands.
Per-flow PV is one rounded product of returned amount and discount factor.
Accrual/elementary-factor error remains governed by existing profiles. Ideal
financial NPV comparisons use propagated ABSOLUTE per-flow amount/factor/product
error plus final exact-sum rounding, with gross signed-flow magnitude conditioning;
relative-to-net or uniform ULP criteria cannot certify cancellation cases.
No existing day-count/rate tolerance is widened. Elementary functions retain
Rust's unspecified-precision qualification; scenario evidence is not a universal
libm or market certificate.

Tests/oracles must cover all finite signs/zero/nonfinite inputs, raw period
validation, retained context/equality/sharing, plain and contextual day counts,
schedule raw/adjusted composition, all 49 convention pairs, inclusion/clipped/
early/delayed accrued dates, complete guard/source/error precedence including
zero inputs, exact product ties/sticky/range and reduction cancellation,
all 32 discount configurations, direct-versus-chained cases, original indices,
work/count cap boundaries, empty/maximum legs and immutable concurrent reuse.
Independent generators derive expected values from civil dates, Fraction and
two high Decimal precisions; reference probes remain external and byte-attested.
OOM and structural proof-excluded causes get catalogs, not manufactured invalid
private states. Full debug/release, strict lint/docs, coverage and extracted
package checks provide local evidence; Linux/Windows compilation is not runtime CI.
