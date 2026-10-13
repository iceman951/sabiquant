# Local M7 cash-flow and coupon research

Research snapshot: 2026-10-13. This document proposes the next local M7
analytical tranche: simple cash flows, plain fixed-rate coupons, immutable
legs, flat-rate net present value and outstanding accrued interest. It creates
no implementation, tests or fixtures and is separate from the saved ICMA
tranche. Recommendations below are proposed delegated engineering decisions
for root to record in a contract before implementation; they do not record
formal owner, milestone, financial-product or release acceptance.

During this research, root adopted the plain SIMPLE coupon formula, explicit
payment inclusion/outstanding accrual, direct factor-required NPV, exact
supplied triple product and rounded-term reduction, and internally immutable
Arc with the stated std-OOM qualification. Root subsequently selected direct
contextual queries with caller `max_accrual_work` below; no optimized batch is
part of the first tranche. The concrete API is recorded in the root-owned
[cash-flow contract](M7_CASHFLOW_CONTRACT.md); this research does not claim
completed implementation or replace its separate validation evidence.

## Scope and NI-01

[ROADMAP M7](ROADMAP.md#4-future-milestones-provisional) and F-CF-01/F-CF-02
in [FEATURE_MATRIX](FEATURE_MATRIX.md#7-future-domains-grouped) include these analytics.
The [domain model](DOMAIN_MODEL.md#8-outlook-concepts-of-future-domains) directs
cash-flow kinds into values/enums and pure analytics. Existing
[FS-DISC](REQUIREMENTS.md#11-fs-disc--rate-based-discounting),
[M4 date contract](M4_DATE_CONTRACT.md), [rates](../src/rates.rs),
[generated ICMA contract](M7_ICMA_REFERENCE_CONTRACT.md) and
[API principles](API_DESIGN_PRINCIPLES.md) supply the composition constraints.

Recommend **analytical binary64 staging** under continuing local authority:
amounts are finite signed f64 in one caller-chosen consistent unit, without
currency, minor-unit rounding, accounting postings or a settlement-payment
instruction. This continues the analytics side of
[ADR-0006](adr/ADR-0006-numeric-policy.md) and [SCOPE](SCOPE.md); it does not
resolve [NI-01](NUMERICAL_CORRECTNESS.md#10-future-numeric-representation-issues).
NI-01 remains the decision for settlement-exact amounts before any settlement
feature is accepted. Record scoped F-CF progress without changing F-CORE-09
to implemented or declaring full M7 complete. Full M7 also retains the seven
specialized schedule rules and its existing acceptance gates. Bond settlement,
clean/dirty pricing and ex-coupon entitlement stay in M8; BPS, yield, duration
and convexity are the later part of F-CF-02; floating coupons stay in M10.

## Primary evidence and reference inventory

The [Department of Finance Canada bond pricing guide](https://www.canada.ca/en/department-finance/programs/financial-sector-policy/securities/securities-technical-guide/determining-bond-treasury-bill-prices-yields.html)
describes fixed coupon payments and valuation as the sum of discounted future
cash flows, and distinguishes accrued interest from quoted prices. Its
Canadian quotation/day-count details are market-specific: they do not define
this generic API or select a universal settlement convention.

[ICMA PMH Appendix A5, March 2022, A5-2–A5-3](https://www.icmagroup.org/assets/documents/ICMA-PMH-Appendix-A5-March-2022.pdf#page=2)
was reopened directly. Version 2 apportions an accrual interval over actual
determination-period lengths and explicit annual frequency; it permits an
extension beyond two determination periods. The March note distinguishes
unadjusted contractual interest periods from delayed business-day payment,
with market exceptions. This supports retaining separate accrual/payment
dates. It does not prescribe this Rust API, all day/week divisors, inclusion
at payment or every market's adjusted-accrual policy.

The following **QuantLib 1.43 public headers were inspected for inventory
only**. No .cpp implementation was read or translated, and header inline
bodies were not used as an implementation recipe.

| Primary header | Public concepts observed; limits |
|---|---|
| [cashflow.hpp](https://raw.githubusercontent.com/lballabio/QuantLib/v1.43/ql/cashflow.hpp) | Date, undiscounted amount, occurrence/inclusion and ex-coupon inspection; Leg is a sequence of shared cash-flow objects. Its optional date/inclusion settings and lazy/global machinery are not a local design requirement. |
| [simplecashflow.hpp](https://raw.githubusercontent.com/lballabio/QuantLib/v1.43/ql/cashflows/simplecashflow.hpp) | Predetermined amount/date, plus named redemption/amortization subclasses. No local implicit redemption is required. Directly fetched bytes SHA-256 `b87ae585c50d3258c1d6120818dfc6c5b142bda73cbbe9cbd05adc9dcfa67d8f`. |
| [coupon.hpp](https://github.com/lballabio/QuantLib/blob/v1.43/ql/cashflows/coupon.hpp) | Separate payment, accrual and reference-period dates, nominal/rate/day-counter inspectors, accrual and accrued-amount methods. The constructor does not adjust a supplied payment date. |
| [fixedratecoupon.hpp](https://raw.githubusercontent.com/lballabio/QuantLib/v1.43/ql/cashflows/fixedratecoupon.hpp) | Rate/day-counter and InterestRate constructor forms, plus a schedule-based leg builder with explicit payment/rate/notional options. That inventory is broader than the proposed plain simple-accrual coupon. Directly fetched bytes SHA-256 `37c84fdb89b02e935a8274fcc1be64d81fb10ecc442313734597dfb402d7497f`. |
| [cashflows.hpp](https://raw.githubusercontent.com/lballabio/QuantLib/v1.43/ql/cashflows/cashflows.hpp) | Constant-rate and curve NPV, accrued-interest inspectors and later yield/risk analytics, with settlement/inclusion/NPV-date inputs. Header declarations alone do not establish its algorithm or local behavioral parity. |

Some raw-header browser fetches missed their cache. The two SHA-pinned headers
above were subsequently fetched by bounded read-only GETs from the same
official v1.43 repository paths; no reference source files were saved locally.
The coupon header was read from its official tag page. No reference executable
probes were run for this research; future differential fixtures must attest
the installed wheel/module bytes and give every relevant date/context/flag.

## Proposed public values and operations

Use `cashflows` with private fields, non-exhaustive CashFlow/Error enums and
the following surface, now recorded in root's local contract. These are
signatures, not implementation or completed validation evidence.

```text
PaymentDateInclusion::{Include, Exclude}       // no default, no Boolean
SimpleCashFlow::new(payment_date, amount) -> Result<SimpleCashFlow, CashFlowError>
FixedCouponRate::new(decimal_rate, accrual: impl Into<AccrualConvention>)
    -> Result<FixedCouponRate, CashFlowError>
FixedRateCoupon::new(start, end, payment_date, notional, coupon_rate: FixedCouponRate)
    -> Result<FixedRateCoupon, CashFlowError>
CashFlow::{Simple(SimpleCashFlow), FixedRateCoupon(FixedRateCoupon)}
Leg::new(flows: Vec<CashFlow>, max_flows: u32) -> Result<Leg, CashFlowError>
Leg::fixed_rate(schedule: &Schedule, notional, coupon_rate: &FixedCouponRate,
    max_flows: u32) -> Result<Leg, CashFlowError>
Leg::net_present_value(valuation_date, discount_rate: &InterestRate,
    inclusion: PaymentDateInclusion, max_accrual_work: u64) -> Result<f64, CashFlowError>
Leg::accrued_amount(as_of, inclusion: PaymentDateInclusion,
    max_accrual_work: u64)
    -> Result<f64, CashFlowError>
FixedRateCoupon::accrued_amount(as_of, inclusion: PaymentDateInclusion)
    -> Result<f64, CashFlowError>
```

Amounts, notionals and decimal coupon rates may be negative, positive or zero;
all must be finite, and both zero signs become +0. Signed notional/rate are
analytical inputs, not a currency or an implicit payer/receiver direction.
The quote wrapper reduces bare-scalar argument confusion and owns complete
accrual context. It is separate from `InterestRate`, which owns a discount
quote's compounding and may use a different accrual convention.

Define a **plain fixed-rate coupon explicitly by simple accrual**:

```text
coupon_amount = notional × annual_decimal_coupon_rate × YF_accrual(start, end)
```

Do not silently evaluate `notional × (compound_factor - 1)` or borrow discount
compounding. A broader compounded-coupon return needs a separate formula and
numerical contract; the QL InterestRate overload is not evidence that this
plain formula covers it. The proposed basic coupon meets the scoped plain
fixed-rate use case, without claiming compatibility with every reference
constructor form.

Require `start < end`. Validate the complete supplied accrual interval and
its finite nonnegative fraction, including possible zero accrual on distinct
dates, before a zero notional/rate shortcut. Construct and retain the full
amount/fraction eagerly. Thus `amount()` is infallible inspection of a valid
value. Preserve the original quote/context, raw dates and payment date in
definition comparison; equal amounts alone do not define equal coupons.
Provide borrowed getters for the source terms/context. Values containing f64
have PartialEq only; no Eq/Hash/ordering or contextual string parser. All
values are immutable Send+Sync, with no lazy calculation or mutable cache.

A manual coupon's payment date is an explicit supported Date. Do not impose
`payment_date >= end`: Preceding can pay before raw accrual end. This generic
analytical value also need not impose an unsubstantiated payment/start rule;
early/advance payment is caller-supplied contractual data, without a market
approval claim. No calendar adjustment or business-day assertion is inferred
by the manual constructor.

`Leg::fixed_rate` makes exactly one coupon from each raw Schedule interval and
uses the corresponding **adjusted interval end** as payment date. It never
uses adjusted starts/ends to compute coupon accrual, advances payment again,
adds a payment lag, inserts principal or changes reference periods. Zero can
provide one nonperiodic coupon when its accrual context is supplied explicitly;
it still cannot generate an ICMA context itself. For ICMA, callers may create
the quote from `Schedule::build_icma_context` with explicit matching frequency
and reference cap; neither this builder nor a label guesses those inputs.
Retain supplied 30E/360 ISDA termination and Business/252 calendar definitions,
rather than rebuilding them separately for each coupon. A caller may supply
different coupon-accrual and payment calendars. The resulting Leg retains
flow terms; it does not purport to serialize the Schedule's payment-calendar
provenance. Keep that original Schedule when that provenance is needed.

## Inclusion, accrued interest and valuation dates

For a cutoff date v, a flow is eligible when payment > v, or payment == v and
Include was explicitly selected. Past payments are excluded. Duplicate
payment dates are valid, including coupon plus redemption; all are retained.
`Leg::new` requires nondecreasing payment order and rejects the first reversal
with original positions/dates. It does not sort, deduplicate or coalesce.
Empty legs are valid. Simple cash flows contribute no accrued interest.

For an eligible fixed coupon, define its **outstanding accrued amount** using
the raw interval, not a day-proportional scaling of the cached full coupon:

```text
as_of <= start:                      +0
start < as_of < end:                 N × r × YF_accrual(start, as_of)
as_of >= end, payment still eligible: cached full coupon amount
payment no longer eligible:          +0
```

The same Include/Exclude policy controls payment-date cutoff for both NPV and
outstanding accrued amount. At an ordinary payment after accrual end, Include
retains the full unpaid coupon and Exclude removes it. A delayed payment can
retain a completed coupon while a following coupon has begun accruing; sum
each explicitly represented outstanding coupon. With an early payment, the
included payment-date accrual is the clipped earned fraction, not necessarily
the full amount due. This is a generic unpaid-coupon analytic, not a bond
clean-price, ex-coupon entitlement or settlement convention.

Eligibility and interval clipping are structural selections. Before-start or
ceased coupons return zero without inventing a contextual query outside their
validated raw interval. Every real partial interval is checked before zero
scalar shortcuts; the full raw interval was checked at construction. Thus
clipping does not enlarge ICMA coverage or waive a Business/252 guard for an
interval actually accrued. Leg accrued amount sums those finite rounded
coupon results and returns canonical +0 for an empty/cancelled result.

For NPV, validate the discount quote's equal-date query `(v,v)` first, including
for an empty leg or a leg with no eligible flows. This preserves M4's
context-before-identity discipline. Then compute each eligible contribution:

```text
PV_i(v) = A_i × P_discount_quote(v, payment_i)
NPV(v)  = sum of the eligible PV_i(v)
```

Each horizon begins at the explicit valuation date. Do not multiply discount
factors between successive payments: Simple and hybrid methods are not
semigroups, and day-count contexts can also change interval behavior. No
separate settlement cutoff, default evaluation date, inferred discount curve
or forward-factor ratio is introduced. Coupon amount accrual and discount
quote accrual are distinct inputs. A coupon payment beyond its accrual ICMA
hull is valid data; the discount quote must independently cover its horizon.

An included zero amount still validates the complete date horizon and factor.
Excluded payments require no hypothetical reversed discount query, but do not
bypass the valuation anchor check. Require finite positive representable
compound/discount factors under the **existing M4 factor-required domain**.
Reject an individually overflowing or zero-underflowing nonzero PV even when
another flow could cancel it. Preserve NumericalBoundary and typed RateError
causes instead of replacing them with log-only amount-scaled valuation.
For example, `min_sub × exp(745)` can be mathematically finite while the
existing continuous negative-rate quote rejects its unavailable factor;
accepting it would be a distinct future valuation operation/domain.

## Numerical choices and independent profiles

Recommended coupon arithmetic is a private correctly rounded **three-factor
product of the supplied binary64 N, r and computed binary64 YF**. Finite
significands have at most 53 bits, so their exact product needs at most 159
bits: three u64 limbs with u128 carries, signed exponent sum and one final
nearest/ties-even rounding suffice. This is a bounded original bit-arithmetic
design, not arbitrary-precision decimal settlement or an NI-04 claim.
Validate all inputs/context first. Exact input zero yields +0; a nonzero exact
product rounding to zero or infinity is a typed numeric error. Cover normal,
subnormal, carry/sticky, midpoint and sign cases. Do not use log/exp or
`powi`/`exp2` scaling for a correctly rounded arithmetic claim.

Naive left association wrongly rejects recoverable `MAX × 2 × 0.5 = MAX`,
and loses `min_sub × 0.5 × 2 = min_sub`. Reordering just these examples does
not prove a universal range/rounding contract. If root chooses a simpler
staged product instead, its range and additional rounding profile must be
explicitly narrower and independently validated before implementation.

For leg NPV/accrued reduction, prefer existing
[`exact_sum`](../src/math/sum.rs), with a fallibly reserved bounded vector of
finite per-flow results. This strengthens the current NP-09 reduction policy
and must be named in the adopted cash-flow contract. It recovers overflowing
prefixes, unlike Neumaier's documented checked-intermediate domain, and rounds
the exact sum of the **supplied rounded terms** once. The primary
[Neal superaccumulator paper, pp. 3–4](https://www.cs.toronto.edu/~radford/ftp/xsum.pdf#page=3)
supports the general finite-exponent accumulation principle; the existing
local sum implementation remains its own source, not a translation of that
paper's optimization.

This is not an exact dot product or a recovery of individually failed terms.
For a=1+2^-27 and b=1−2^-27, rounding a×b first then summing −1 gives zero,
while the exact product-sum is −2^-54. Similarly, separately unavailable
`+MAX×2`, `−MAX×2`, `+1` cannot become a valid leg merely because the ideal
total is one. Four individually zero-underflowing `min_sub×0.25` contributions
are also outside this proposed per-flow domain. Reduction permutation
invariance applies to fixed supplied term bits, not to independent reevaluation
of unspecified-precision elementary factors.

Record proposed cash-flow profiles separately from existing M4 tolerances:

| Quantity | Proposed evidence/acceptance profile |
|---|---|
| Dates, order, eligibility, stored normalized input bits, error kinds | Exact structural/identity checks; no numerical tolerance for wrong metadata. |
| Triple product given binary64 operands | 0 ULP against Fraction's exactly multiplied binary64 operands, including independently decided final overflow/zero-underflow status. |
| Coupon versus exact financial YF | Propagate the existing YF error through `abs(N*r)×abs(delta_YF)`, plus final product rounding. A correctly rounded product does not make the previously rounded YF exact. |
| PV given a returned discount | One correctly rounded binary64 product; Fraction verifies that product separately. Ideal-factor comparison also includes `abs(A)×abs(delta_discount)` and any coupon amount error. |
| NPV versus ideal analytical cash flows | Absolute propagated budget: sum of per-flow amount/factor/product errors plus final exact-sum rounding. Condition on gross magnitude `G=sum(abs(PV_i))`; relative-to-net/ULP-only tolerances are inappropriate when signed flows cancel. |

For date-based discounts, propagate time error with derivatives
`abs(r)*D` (Continuous), `abs(m*ln(1+r/m))*D` (periodic), or `abs(r)*D^2`
(Simple), with the existing M4 boundary/switch conditioning. Near a nonpositive
Simple factor or rounded hybrid threshold, use the established active-domain
and NumericalBoundary contract, not a blanket few-ULP assertion. Compute
oracle budgets in Fraction/high precision or a bounded scaled form: overflow
of a tolerance expression is not permission to accept arbitrary error.
Elementary functions still have Rust's
[unspecified precision](https://doc.rust-lang.org/std/primitive.f64.html);
calibrated scenario evidence is not a universal libm certificate.

## Ownership, work and errors to adopt before coding

Recommend an internally shared immutable `Arc<AccrualConvention>` inside
FixedCouponRate, with borrowed context inspection and no public pointer/refcount
semantics. Moving a convention into one quote and cloning that quote for n
coupons gives O(context size+n) storage, retaining full ICMA/calendar definition
identity. Equality between independently allocated equal contexts can still
cost O(context size). Deep-cloning an M-period context into each coupon costs
O(nM) storage; silently trimming each coupon changes coverage/definition
inspectors, even though a correct partition merge could bound trimmed copying
by O(M+n). Business/252 has no equivalent calendar-slicing contract.

The [Rust 1.99 Arc documentation](https://doc.rust-lang.org/std/sync/struct.Arc.html#method.try_new)
confirms fallible Arc construction remains nightly-only. Thus an ergonomic
internal Arc::new needs an explicit standard system-OOM qualification, as
current owned Calendar Clone already has: financial invalid inputs remain
typed errors, and growing flow/PV vectors use fallible reservation, but every
system allocation is not promised recoverable. Pre-reserving a Vec or
converting from Box does not fix Arc allocation. If root requires absolutely
fallible library allocation instead, accept a caller-created Arc explicitly,
or use one central owned context in a distinct FixedLeg with index-only coupon
records; do not add unsafe allocation glue or nightly features.

Propose `Leg::MAX_FLOWS = 3_652_059` and valid caller budgets `0..=MAX_FLOWS`,
where zero permits only an empty leg. This accommodates every supported
one-day coupon plus an explicit final principal flow. It is a **resource
policy**, not a civil-date theorem: duplicate payment dates can otherwise
create arbitrarily many flows. Check budget validity and known final flow
count before building coupons or reserving buffers. No callback/iterator with
an unknown unbounded length is necessary. Library-owned vector growth is
fallible; a supplied Vec's earlier allocation and normal Clone behavior are
not reclassified as recoverable errors. Return no partial leg or valuation.

NPV needs O(n) factor evaluations and O(n) transient reduction storage, plus
accrual-context work. A direct ICMA query per payment can cost O(sum k_i),
potentially O(nM); do not advertise that as unconditional O(n). **Root adopted
an explicit caller-selected `max_accrual_work: u64` for direct multi-flow
valuation/accrued queries. Every value `0..=u64::MAX` is a valid budget; zero
means no accrual work, not unbounded work.** Charge the next required query
before performing that query or its coverage validation:

| Required query context | Work charge |
|---|---|
| Scalar day count or termination-sensitive 30E/360 ISDA | 1 |
| ICMA | 1 plus the number k of overlapping reference periods, estimated by partition points over the immutable ordered period slice; an equal-date query has k=0. |
| Business/252 | 1 plus the absolute civil-day span, a conservative date-work estimate regardless of faster particular calendar cases. |
| Equal-date valuation anchor in any context | 1, including ICMA and Business/252. |
| Excluded flow, cached complete accrued coupon, or before-start accrued selection | 0; these require no fresh accrual query. |

The valuation anchor is still required for empty/all-excluded NPV, so budget
zero stops before it. Included zero amounts/rates consume their full required
horizon charge and retain contextual/factor validation. Account for one shared
budget across the entire operation. If a next charge exceeds it, preserve the
original flow index (None for the valuation anchor), phase, original caller
cap, work already used and next required lower bound; return before that
disallowed query. Do not prevalidate future flows. With adequate budget,
original per-flow Rate/coverage/error order is retained exactly. Small budgets
can deliberately stop before the next otherwise reachable coverage error.

The accepted operation costs O(n + charged work), plus bounded ICMA binary
search and the supplied calendar definition/classification factors; O(n)
eligibility scans and reduction storage remain bounded by the Leg cap. Work
units are a deterministic resource policy, not CPU-time promises for arbitrary
calendar graphs. Even the conservative total
`1 + 3_652_059*(1 + 3_652_058) = 13_337_534_939_482` is below 1.34e13 and fits
u64, so used+charge cannot overflow under the defined flow/date bounds even
when the caller supplies u64::MAX. A finite large budget does not promise a
fast pathological case; callers choose their explicit accepted work.

An independent period review found a feasible future ICMA optimization:
maintain compensation for **completed** periods from valuation date and
evaluate a copy plus the current final partial period for each payment. Do
not commit transient partial payments or subtract rounded large prefixes.
An induction on completed periods can preserve each original chronological
operation sequence in O(n+M). It still needs source proof and bitwise
comparison with existing ICMA year_fraction before adoption. Business/252
integer counts are additive over consecutive half-open spans, but using only
incremental guards can change the original Date diagnostic: TARGET endpoint
validation can precede a gap check. Any future batching must validate each
original horizon/guard in source order and preserve its actual cause, not
replace it with a different segment failure. No batch, mutable cache or
adjusted-date accrual is implemented or claimed here.

The adopted first-tranche error order checks each work charge before its
required query, checks the valuation anchor, then visits eligible flows in
original order; each required accrual/factor/product error carries the original
Leg index and phase, with no amount-zero bypass. Coupon construction checks raw
interval, finite notional, required accrual and final amount in that order;
leg construction checks cap/count then payment order. Suggested kinds cover
invalid accrual interval, numeric input/result, flow cap/count, payment order,
allocation, rate/accrual and final reduction failures. Preserve typed
ValueError, RateError and SumError, with crate Error -> CashFlowError -> cause
source layering. Per-flow indices refer to original input positions, not
positions in a filtered PV buffer. Proof-excluded structural/input wrappers
and OOM are catalog checks rather than fabricated public failures.

## Mandatory meaningful validation matrix

| Area | Required independent cases |
|---|---|
| Value domain and identity | Every sign/zero sign; NaN and both infinities for amount/notional/rate; raw interval equal/reversed; finite extremes; retained complete context; PartialEq distinctions despite equal amounts; clone/Send+Sync/thread reuse; canonical +0. |
| Coupon financial accrual | Exact Act/360, Act/365F and 30-day anchors; leap/century ISDA; zero accrual on distinct dates; termination-sensitive February; independent Business/252 weekday/holiday prefix; supplied ICMA regular/Short/Long/equal-role multi-period references. Keep payment shift separate. |
| Schedule composition | Forward/Backward/Zero with applicable EOM and automatic/explicit stubs; all 49 ordinary/termination BDC combinations; preceding/late payments; one coupon per raw interval; coupon plus explicit redemption on the same date; no implicit payment adjustment, principal, frequency or termination rewrite. |
| Inclusion and accrued amount | Day before/start/interior/end/day after/payment/day after payment; both payment-date policies; delayed coupon plus next active coupon; early/advance payment; multiple unpaid coupons; Simple flows contribute zero. Compare exact clipped convention fractions, not `full_amount×elapsed_days/full_days`. |
| Coverage and precedence | Invalid contextual constructor even with zero rate/notional; partial/full supplied ICMA coverage; zero amount/rate/equal valuation outside discount coverage; TARGET gap/inception/joint/override/projected policy; payment beyond coupon hull; excluded past flows versus included zero future flows; original error indices after filtering and complete source chains. |
| Triple product | Fraction from exact binary64 operands, exponent bins/signs/zeros, normal/subnormal carries, guard/sticky/ties, `MAX×2×0.5`, `min_sub×0.5×2`, zero-underflow midpoint and neighbors, actual overflow midpoint and neighbors. Financial fixtures must additionally propagate independently exact YF. |
| Per-flow discounts | All 32 supported compounding configurations and all current accrual forms; same-day identity, tiny/negative/large rates, Simple domain boundary, hybrid switches, existing NumericalBoundary cases. Two independent Decimal precisions for ideal factors; exact Fraction for basic arithmetic and returned-factor products. |
| Direct valuation definition | Two/three payments under Simple/hybrids that distinguish direct valuation-date factors from chained factors; separate coupon/discount day counts; zero-rate NPV equals exact_sum of eligible stored amounts; covered same-day Include preserves each amount before reduction. |
| Cancellation and range | Fixed terms `[MAX,MAX,-MAX]` and `[MAX,MAX,-MAX,-MAX,min_sub]`; tiny residuals/gross conditioning; signs, permutations and duplicate dates; exact product-sum counterexample; separately invalid PV/factor cases whose ideal total could be finite; zero-underflowing terms; final overflow midpoint and finite MAX rounding. |
| Resource/algorithm | Empty/zero budget; exact count, cap−1 and unsupported caps; fail before disallowed build work; maximum scalar leg without quadratic context copying; required vector allocation context; ICMA batch versus existing individual horizons, repeated/clipped/boundary payments and long outer periods; contextual work upper bound and guarded calendar complexity. |

Generators must never obtain expected values from Rust or the reference's
internal code. Use Python datetime/civil coordinates plus Fraction for dates,
products/reductions and rational accrual; preserve binary64 inputs with
Fraction.from_float/Decimal.from_float; compare independent high precisions
for elementary-factor outputs. Reference fixture generation remains external,
wheel/module-attested, bounded and exactly reproducible with --check; Rust
tests remain offline/std-only. Record exact row/context/status counts, numerical
distributions, conditioning and discrepancies; do not count labels or repeated
queries as additional independent cases. Unsupported market day-count variants
remain unsupported rather than substituted silently.

## Prerequisites and evidence boundary

Root should record the adopted plain-coupon formula, outstanding-accrual
definition, payment inclusion, factor-required PV domain, exact product/reduction
policy, shared ownership/OOM qualification and direct contextual work/error
precedence in one concrete local contract, then update requirements/catalogs
for implementation.
These are feasible delegated local engineering decisions under existing
authority; this research does not require renewed owner permission to continue
authorized preparation. Implementation should progress through values/coupons,
bounded leg composition, then contextual flat NPV/accrued analytics with
independent oracles and source review. Preserve all existing source guards and
regression evidence, then run affected/full debug/release, strict lint/docs,
coverage and extracted-package checks under the project's existing workflow.

Local research inputs were rates SHA-256
`5778f81cb9c997163e34a95d6f1a9ee25b47b49df39512fa68097153ec12a2d7`,
sum SHA-256 `c0b794bf43276f17bcfd080e71aa3e70ca996d0750e364ef1622de1e6907d935`,
M4 date contract SHA-256
`47216e578082eb56daaffb6b49c45951a416f21fe36372baede2bd0a29cdea98`
and generated ICMA contract SHA-256
`abdc7dd61de1683399fbfc57403def012b251f72dd7ae734e6e322cc0eb25a74`.
This document reports no implemented cash-flow behavior or passing cash-flow
tests. NI-01 settlement, seven specialized rules, native primary-platform CI,
formal owner/financial/milestone acceptance and existing maintenance/release
gates remain open; scoped analytical progress does not erase them.
