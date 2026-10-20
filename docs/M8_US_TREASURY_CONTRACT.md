# Complete local US Treasury contract

Adopted 2026-10-20 on local `feat/m8-us-treasury` from `7bfd94f` under the
owner's continuing local-development authority, after the complementary
[independent reviews and root reproductions](M8_US_TREASURY_CONTRACT_REVIEW.md).
This is the complete local engineering contract; implementation is pending.
New independent general oracles must be authored, frozen and verified before
Treasury Rust. No QuantLib implementation was read. Existing contracts,
generators and fixtures stay frozen. This adoption is not owner financial,
legal, calendar, native foreign-runtime or full M8 acceptance.

This integrates the complete [proposal](M8_US_TREASURY_LOCAL_DESIGN.md),
[source refresh](M8_US_TREASURY_SOURCE_REFRESH.md),
[source examples](M8_US_TREASURY_SOURCE_EXAMPLES.md) and
[research validation](M8_US_TREASURY_RESEARCH_VALIDATION.md). Its exact cash
component is the separate, jointly normative
[cash contract](M8_US_TREASURY_CASH_CONTRACT.md). Both documents are adopted
together, including all four cash operations; the scope is complete price,
yield, all six risk views and exact cash.

## A. Scope, evidence and local interpretations

Implement fixed-principal semiannual notes/bonds with par redemption 100,
regular/short/long first coupons, explicit analytical settlement, price,
bounded yield inversion, six risk views and four exact-money recipes. Coupon
C is nonnegative exact DecimalAmount **per 100** (C=5 means 5%); y is nominal
annual decimal (y=.05 means 5%). Par nominal for cash is a separate exact
positive amount. No arbitrary binary face or implicit quote conversion exists.

The source identifier is exactly
`US-Treasury-31CFR356-AppB-fixed-2026-10-16-local-v1`. It identifies this
retained local formula/snapshot interpretation, not a live regulation fetch,
actual auction eligibility, financial acceptance or settlement enclosure.
Source policy is retained separately and is never hidden inside that string.

Primary authority inspected is 31 CFR Part 356, Appendix B I.A/I.D/II, and
§§356.2/.10/.25/.30, with the pinned consolidated 2026-10-16 XML. The source
refresh records the TIPS-only 2026-10-15 appendix amendment, unchanged
fixed-principal bodies, historical PDF corroboration and source typos. Apply
II.E's worked divisions r′/s″ and C/2, not its malformed headline. The source
zero-yield annuity limit is finite. Its intermediate worked rounding is
illustrative; the API does not expose a six-decimal source price view.

These explicit engineering choices extend or delimit the source:

- Dated date D and original issue date I are distinct. Raw accrual and first
  coupon coefficients are based on D, including short/long D<I and a long
  original issue after the skipped quasi date. The seven examples do not
  discriminate those short/long D<I cases; the interpretation follows dated
  accrual in §§356.2/.30 and is subject to announcement priority under .10.
- The caller supplies the full nominal lattice and EOM policy. Treasury
  month-end cycles need Preserve. Ignore remains a stated mathematical
  alternative and can give non-month-end interior dates for month-end M.
- Settlement I<=S<M excludes payments on S and retains all strictly future
  coupons/principal. No investor-level record date, detached coupon or
  ex-coupon rule is inferred. All source accrued interest is nonnegative.
- No calendar, holiday closure, settlement lag or adjusted remittance date is
  generated. §356.30's delayed payment without extra interest is source
  context, not an implemented Federal Reserve calendar.
- Full nonnegative DecimalAmount C, negative y>-2 and arbitrary positive
  exact cash nominal are local mathematical domains. In particular coupon
  nominal scaling outside $1,000 multiples is a local extension of I.A.3/4;
  denominations, issue minimums and real auction announcements are absent.

Generic bonds, gilt conventions, existing money/evidence meanings, public
calendars and their acceptance records are unchanged.

## B. Complete type, trait and identity census

Implement exactly fourteen public types. The required completed public census
is126; the current implemented census remains112 until implementation:

| Type | Copy | Eq + Hash | Owned 'static |
|---|---|---|---|
| UsTreasury | no | yes | yes |
| TreasuryFirstPeriod | yes | yes | yes |
| TreasuryDailyBasis | yes | yes | yes |
| TreasuryTieRule | yes | yes | yes |
| TreasurySourcePolicy | yes | yes | yes |
| TreasuryPricingPlan<'a> | no | no | no, borrows the instrument |
| TreasuryValuation | yes | no | yes |
| TreasuryYieldReport | yes | no | yes |
| TreasuryRisk | no | no | yes |
| TreasuryError | no | no | yes |
| TreasuryYieldError | no | no | yes |
| TreasuryCashInstruction | yes | yes | yes |
| TreasuryCashDecision | yes | yes | yes |
| TreasuryCashReceipt | yes | yes | yes |

All struct fields are private. Contextual enum fields are only those listed
in the API/error ledgers; no public constructor exposes a private numeric
state. All implement Debug, Clone, PartialEq, Send and Sync. No new type implements
Default, Ord/PartialOrd, a parser/identifier registry, panicking arithmetic or
implicit float-money conversion. Error types implement Display and
std::error::Error; their prose is not stable. All enums and contextual error
variants are non-exhaustive. Risk Debug omits giant retained arrays.

Instrument equality/hash retain D/I/M/F, canonical C, supplied complete Q,
explicit EOM and source policy. The policy remains part of identity even
where its current numerical outputs agree. Derived immutable caches do not
replace the definition. An instrument owns the caller's Q without cloning;
ordinary Clone/Debug carry normal allocation qualifications. Fallible
construction is not a promise about arbitrary Clone or formatting allocation.

Reports own the scalar metadata in D below, including policy and both AI
observations. They do not claim whole-instrument identity or retain Q. Two
different policies can produce equal dirty/risk numbers and unequal reports.
Instruction and determined-receipt identity/provenance is the compact owned
snapshot specified in the cash contract; equal money does not erase different
policies/evidence/definitions. Non-determined decision variants compare/hash
only their listed diagnostic fields; the caller retains the instruction.
Resource caps are not retained financial identity fields.

```text
TreasuryFirstPeriod::{Regular,Short,Long}; ALL:[Self;3]
TreasuryDailyBasis::{ExactRatio,NineDecimal}; ALL:[Self;2]
TreasuryTieRule::{NearestTiesEven,NearestTiesAwayFromZero}; ALL:[Self;2]
TreasuryTieRule::amount_rounding(self) -> AmountRounding
TreasurySourcePolicy::new(daily_basis:TreasuryDailyBasis,
    tie_rule:TreasuryTieRule) -> Self
TreasurySourcePolicy::daily_basis(self) -> TreasuryDailyBasis
TreasurySourcePolicy::tie_rule(self) -> TreasuryTieRule
```

ALL arrays preserve the listed order. SourcePolicy construction, its two
inspectors and the tie mapping are infallible const methods. The tie mapping returns exactly the corresponding
existing AmountRounding variant. It introduces no additional monetary mode.

## C. Complete instrument and plan API

```text
UsTreasury::MAX_QUASI_DATES:usize = 20_000
UsTreasury::new(original_dated_date:Date,original_issue_date:Date,
    maturity_date:Date,coupon_per_100:DecimalAmount,first_coupon_date:Date,
    quasi_dates:Vec<Date>,end_of_month:EndOfMonth,
    source_policy:TreasurySourcePolicy,max_dates:usize)
    -> Result<Self,TreasuryError>
UsTreasury::source_version(&self) -> &'static str
UsTreasury::original_dated_date(&self) -> Date
UsTreasury::original_issue_date(&self) -> Date
UsTreasury::maturity_date(&self) -> Date
UsTreasury::coupon_per_100(&self) -> DecimalAmount
UsTreasury::first_coupon_date(&self) -> Date
UsTreasury::first_period(&self) -> TreasuryFirstPeriod
UsTreasury::first_coupon_quasi_index(&self) -> usize
UsTreasury::quasi_dates(&self) -> &[Date]
UsTreasury::coupon_dates(&self) -> &[Date]
UsTreasury::end_of_month(&self) -> EndOfMonth
UsTreasury::source_policy(&self) -> TreasurySourcePolicy
UsTreasury::coupon_count(&self) -> usize
UsTreasury::raw_coupon_per_100(&self,index:usize,quantum:AmountQuantum,
    mode:AmountRounding) -> Result<DecimalAmount,TreasuryError>
UsTreasury::source_coupon_per_100(&self,index:usize,quantum:AmountQuantum,
    mode:AmountRounding) -> Result<DecimalAmount,TreasuryError>
UsTreasury::coupon_cash(&self,index:usize,nominal:DecimalAmount,
    rounding:SettlementRounding) -> Result<TreasuryCashInstruction,TreasuryError>

TreasuryPricingPlan::MAX_BUNDLES:usize = 20_000
TreasuryPricingPlan::new(treasury:&'a UsTreasury,settlement_date:Date,
    max_bundles:usize) -> Result<Self,TreasuryError>
TreasuryPricingPlan::treasury(&self) -> &'a UsTreasury
TreasuryPricingPlan::source_version(&self) -> &'static str
TreasuryPricingPlan::source_policy(&self) -> TreasurySourcePolicy
TreasuryPricingPlan::settlement_date(&self) -> Date
TreasuryPricingPlan::next_quasi_index(&self) -> usize
TreasuryPricingPlan::remaining_quasi_periods(&self) -> u32
TreasuryPricingPlan::remaining_days(&self) -> u32
TreasuryPricingPlan::quasi_period_days(&self) -> u32
TreasuryPricingPlan::bundle_count(&self) -> usize
TreasuryPricingPlan::bundle_quasi_index(&self,index:usize) -> Option<usize>
TreasuryPricingPlan::coupon_entitlement(&self,coupon_index:usize)
    -> Option<CouponEntitlement>
TreasuryPricingPlan::raw_accrued_per_100(&self) -> f64
TreasuryPricingPlan::accrued_per_100(&self) -> f64
TreasuryPricingPlan::evaluate(&self,yield_rate:f64)
    -> Result<TreasuryValuation,TreasuryError>
TreasuryPricingPlan::risk(&self,yield_rate:f64) -> Result<TreasuryRisk,TreasuryError>
TreasuryPricingPlan::yield_from_price(&self,target:BondPrice,bracket:RootBracket,
    options:SolverOptions) -> Result<TreasuryYieldReport,TreasuryYieldError>
TreasuryPricingPlan::accrued_cash(&self,nominal:DecimalAmount,
    rounding:SettlementRounding) -> Result<TreasuryCashInstruction,TreasuryError>
TreasuryPricingPlan::clean_accrued_cash(&self,nominal:DecimalAmount,
    clean:SettlementEvidence,aggregation:CashAggregation,
    rounding:SettlementRounding) -> Result<TreasuryCashInstruction,TreasuryError>
TreasuryPricingPlan::dirty_cash(&self,nominal:DecimalAmount,
    dirty:SettlementEvidence,rounding:SettlementRounding)
    -> Result<TreasuryCashInstruction,TreasuryError>
```

All arguments are already valid domain values; caller argument-expression
evaluation is not ordered by these methods. Coupon dates are the Q suffix
from F; zero coupons remain actual coupons. Optional index inspectors return
None outside range, with no allocation; value/cash selectors use
InvalidCouponIndex. Valid coupon entitlement is Ceased for payment<=S,
BuyerEntitled otherwise. ExCouponDetached is never returned by this model.

The two coupon views quantize their respective exact point only at the
caller-requested quantum/mode. They exclude principal and are not promised
exact DecimalAmount representations. Regular C=1e-38 gives C/2 at scale39;
raw fractional coupons can be nonterminating. Source cash policy does not
change raw pricing coefficients. Dirty prices and all risk values are
numerically policy-independent; clean prices, clean targets and cash can
depend on policy. No source six-place P getter or ideal-price enclosure is
fabricated.

## D. Owned numerical report API

Every Valuation, Risk and YieldReport supplies the following exact methods.
The Copy reports take self; Risk uses &self:

```text
source_version() -> &'static str
source_policy() -> TreasurySourcePolicy
original_dated_date() -> Date
original_issue_date() -> Date
maturity_date() -> Date
coupon_per_100() -> DecimalAmount
first_coupon_date() -> Date
first_period() -> TreasuryFirstPeriod
end_of_month() -> EndOfMonth
settlement_date() -> Date
next_quasi_date() -> Date
remaining_quasi_periods() -> u32
remaining_days() -> u32
quasi_period_days() -> u32
raw_accrued_per_100() -> f64
accrued_per_100() -> f64
yield_rate() -> f64
dirty_per_100() -> f64
```

The accrued method is the cached **published** AI observation, not raw AI.
Both views round their independently retained exact values once to IEEE
nearest/even. Reports retain both observations and the policy, so a clean
quote/target can be interpreted without a borrowed plan.

```text
TreasuryValuation::dirty_price(self) -> BondPrice
TreasuryValuation::clean_price(self) -> Result<BondPrice,TreasuryError>

TreasuryRisk::price_derivative(&self) -> Result<f64,TreasuryError>
TreasuryRisk::price_second_derivative(&self) -> Result<f64,TreasuryError>
TreasuryRisk::time_weighted_amount(&self) -> Result<f64,TreasuryError>
TreasuryRisk::modified_duration(&self) -> Result<f64,TreasuryError>
TreasuryRisk::macaulay_duration(&self) -> Result<f64,TreasuryError>
TreasuryRisk::convexity(&self) -> Result<f64,TreasuryError>

TreasuryYieldReport::target(self) -> BondPrice
TreasuryYieldReport::dirty_target_per_100(self) -> f64
TreasuryYieldReport::initial_bracket(self) -> RootBracket
TreasuryYieldReport::options(self) -> SolverOptions
TreasuryYieldReport::solve_report(self) -> SolveReport
TreasuryYieldReport::residual_scale_exponent(self) -> u8
TreasuryYieldReport::sensitivity(self) -> BondYieldSensitivity
TreasuryYieldReport::amount_residual(self) -> Result<f64,TreasuryError>
TreasuryYieldReport::dirty_price(self) -> BondPrice
TreasuryYieldReport::clean_price(self) -> Result<BondPrice,TreasuryError>
```

Dirty-price construction is infallible from an admitted value. Clean is one
exact_sum([cached dirty,-cached published AI]) and finite signed BondPrice
construction, with no callback or factor/date work. No rounded-price display
stage is inserted. Risk returns its six Results independently; neither a raw
getter's range failure nor an unused clean quote can invalidate another view.
Yield report sensitivity is MathematicallyStrictlyDecreasing for the ideal
positive model, with the observation/attainability qualifications below.

## E. Construction order, coordinates and exact source values

Instrument construction performs exactly:

1. Validate 2<=max_dates<=20,000, then Q.len()<=max_dates, then Q.len()>=2.
2. Reject C<0; require D<=I, then I<M. Decimal signed zero is canonical zero.
3. Require Q.last()==M, then strictly ascending Q in original-index order.
4. Require minimal coverage Q[0]<=D<Q[1]; redundant prefixes are rejected.
5. For increasing original index i, form the direct expected date
   M-6*(Q.len()-1-i) months, with the supplied EOM rule. Preserve applies
   calendar month-end iff M is month-end; otherwise clamp its original day.
   Ignore always clamps M's original day. A checked date failure precedes
   mismatch at that index. No recursive roll, calendar query or adjustment.
6. Require I<F, returning FirstCoupon if not, before role inference. Infer
   Regular iff D=Q0,F=Q1; Short iff Q0<D<Q1,F=Q1;
   Long iff Q0<D<Q1,F=Q2. All other F choices fail FirstCoupon. In particular
   D=Q0,F=Q2 is outside this contract. Long I>Q1 remains admitted if I<F,
   under A's explicitly local dated-date interpretation.
7. Derive compact exact raw/source first-coupon caches and first index.
   Regular later coupons share the C/2 definition. Own Q without cloning.
   There is no derived coupon vector or new heap allocation in construction.

All derived exact cache numerators, denominators and publication counts use
the private five-u64-word Scalar (320bits), never i128/u128/DecimalAmount.
This includes both first coupons, a cached daily9 count, raw plan AI and the
published5 count. Original C remains its valid input DecimalAmount. Cache
zero rationals use numerator0/denominator1; day metadata is retained separately.
Raw first coupons use E's exact one-period forms; raw AI uses at most two
periods. Define C=Ccoef/10^Cscale, d0=days(D,Q1), s0=days(Q0,Q1).
The following table is the single normative source-coupon cache form per
$1,000: `X/(10^dscale*s_part)`. K is the nonnegative integer nearest/even or
nearest/away count of `5*C*10^9/s0` under the retained tie rule, so the
NineDecimal daily amount is K/10^9. Define m=max(9,Cscale).

| Coupon / basis | X | dscale | s_part | X bit upper bound |
|---|---|---|---|---:|
| Regular, either basis | 5*Ccoef | Cscale | 1 | 130 |
| Short, ExactRatio | 5*Ccoef*d0 | Cscale | s0 | 137 |
| Long, ExactRatio | 5*Ccoef*(d0+s0) | Cscale | s0 | 138 |
| Short, NineDecimal | K*d0 | 9 | 1 | 160 |
| Long, NineDecimal | K*d0*10^(m-9)+5*Ccoef*10^(m-Cscale) | m | 1 | 161 |

For K's operands use `5*Ccoef*10^(9-Cscale)/s0` if Cscale<=9, otherwise
`5*Ccoef/(s0*10^(Cscale-9))`. These have the cash proof's bounded integer
widths. Selected X is at most161bits; raw first coupon is at most136/
135bits, raw AI144/143bits and published count147bits. Thus Scalar holds
the caches and their stated unreduced forms, independently of optional
public amount views. Changing these forms requires an updated width proof.

Plan construction validates max_bundles<=20,000 (zero is a valid cap), S>=I,
then S<M. Resolve **first** Q[k]>S, r=days(S,Qk), s=days(Q[k-1],Qk) and
n=Q.len()-1-k. Count future actual coupon bundles, including the final
coupon+principal, without allocation; reject count>max_bundles. The missing
long-first Q1 has no payment. Derive exact raw/published AI and cached IEEE
views, then reserve/fill one bundle vector fallibly in original index order.
No further allocation or public source-amount gate intervenes.

A bundle at absolute Q index i has j=i-k, not its retained-vector ordinal.
Indices on numeric failures are absolute Q indices. Actual coupon-cash/view
selectors instead use actual-coupon ordinals i-first_coupon_quasi_index.
All s lie in181..184; r in1..s; n and j are below20,000. A guard/proof may
use conservative j<=19,999 without claiming that every primitive j-cap cell
is realized by a maximum-length admitted date vector.

Write c=C/2, s0=days(Q0,Q1), d0=days(D,Q1). Exact raw coupon per100 is:

```text
regular or any later coupon: c
short first:                c*d0/s0
long first:                 c*(d0/s0+1)
```

Add par100 to the final raw coupon **exactly** before PV rounding. No first
coupon publication is inserted into this raw analytic coefficient.

For source cash, a selected daily ratio per$1,000 is `5*C/s`. ExactRatio
retains that rational. NineDecimal applies the retained TreasuryTieRule to
its nine-place quantum before multiplying fractional days. The same tie
rule governs **both** daily9 and total accrued5 publication. The 640 table
cells contain no daily9 ties, but general C does (C=23e-7,s=184 gives one).
Regular full coupons remain c, never daily9*s/10. Source long first cash is
its fractional source part plus c, without component quantization.

Before F, raw/source accrued cover D..S split at Q1 if necessary; at/after F
they cover the last actual coupon..S in its regular half-year. Thus:

```text
first single interval:     raw AI = c*days(D,S)/s0
long second interval:     raw AI = c*(d0/s0+days(Q1,S)/s)
later interval:           raw AI = c*days(previous actual coupon,S)/s
```

At a skipped long Q1, the first accrued piece persists. At any actual coupon
date, including F, AI resets to zero. Source AI replaces each fractional
piece by its selected-basis daily-per$1,000 product, sums all pieces, then
rounds the total to five places per$1,000 and divides by10 for per100.
This total-first stage follows I.D example(2); II.E's individually displayed
components cannot supply a different universal component rule. Raw/source
AI stay exact privately. A tiny C can give positive raw AI and zero daily9
and published AI. All AI are nonnegative in this no-detachment model.

## F. TreasuryError and TreasuryYieldError ledgers

All contextual variants are non-exhaustive. The kind prefix is exactly
`treasury.`; the table gives each full suffix. Immediate typed causes are
preserved by source(), never flattened into strings:

| Variant and fields | kind suffix | source |
|---|---|---|
| InvalidDateCap{cap:usize,minimum:usize,maximum:usize} | invalid_date_cap | None |
| TooManyQuasiDates{count:usize,cap:usize} | too_many_quasi_dates | None |
| TooFewQuasiDates{count:usize,minimum:usize} | too_few_quasi_dates | None |
| NegativeCoupon{coupon_per_100:DecimalAmount} | negative_coupon | None |
| DatedAfterIssue{original_dated_date:Date,original_issue_date:Date} | dated_after_issue | None |
| IssueNotBeforeMaturity{original_issue_date:Date,maturity_date:Date} | issue_not_before_maturity | None |
| MaturityBoundary{maturity_date:Date,last_quasi_date:Date} | maturity_boundary | None |
| QuasiDateOrder{index:usize,previous:Date,date:Date} | quasi_date_order | None |
| QuasiCoverage{original_dated_date:Date,first_quasi_date:Date,next_quasi_date:Date} | quasi_coverage | None |
| QuasiDateMismatch{index:usize,supplied:Date,expected:Date,maturity_date:Date,end_of_month:EndOfMonth} | quasi_date_mismatch | None |
| Date{index:Option<usize>,phase:&'static str,cause:DateError} | date | Some(cause) |
| FirstCoupon{original_dated_date:Date,original_issue_date:Date,first_coupon_date:Date,next_quasi_date:Date,second_quasi_date:Option<Date>} | first_coupon | None |
| InvalidBundleCap{cap:usize,maximum:usize} | invalid_bundle_cap | None |
| TooManyBundles{count:usize,cap:usize} | too_many_bundles | None |
| SettlementBeforeIssue{settlement_date:Date,original_issue_date:Date} | settlement_before_issue | None |
| SettlementAtOrAfterMaturity{settlement_date:Date,maturity_date:Date} | settlement_at_or_after_maturity | None |
| InvalidCouponIndex{index:usize,count:usize} | invalid_coupon_index | None |
| Value{index:Option<usize>,phase:&'static str,cause:ValueError} | value | Some(cause) |
| Sum{phase:&'static str,cause:SumError} | sum | Some(cause) |
| AllocationFailed{phase:&'static str,requested:usize} | allocation_failed | None |
| InternalCapacity{index:Option<usize>,phase:&'static str,operation:&'static str} | internal_capacity | None |
| ZeroDirtyPrice{measure:&'static str,yield_rate:f64} | zero_dirty_price | None |
| NonPositiveNominal{nominal:DecimalAmount} | nonpositive_nominal | None |
| CashArithmetic{stage:&'static str,endpoint:&'static str,cause:AmountError} | cash_arithmetic | Some(cause) |

There are24 TreasuryError variants. There is no Rate wrapper, elementary
boundary outcome or public interval refinement: invalid y is Value, while
impossible bounded-integer failures are InternalCapacity. source_version
does not imply use of generic Compounding. Error::Treasury(TreasuryError)
and Error::TreasuryYield(TreasuryYieldError) transparently forward kind,
Display/source and have corresponding From impls, without counting wrappers
again in the domain-variant census.

```text
TreasuryError::kind(&self) -> &'static str
TreasuryYieldError::Target{target:BondPrice,phase:&'static str,cause:TreasuryError}
TreasuryYieldError::Preparation{phase:&'static str,cause:TreasuryError}
TreasuryYieldError::Solver(SolverError)
TreasuryYieldError::Callback{stage:EvaluationStage,point:f64,
    evaluations:EvaluationCounts,cause:TreasuryError}
TreasuryYieldError::NonPositiveTarget{target:BondPrice,dirty_target_per_100:f64,
    accrued_per_100:f64,source_policy:TreasurySourcePolicy,
    evaluations:EvaluationCounts}
TreasuryYieldError::kind(&self) -> &'static str
```

The five kinds respectively are treasury_yield.target/preparation/solver/
callback/nonpositive_target, each with the complete prefix. source() is the
immediate cause except None for NonPositiveTarget. That case has zero
function/derivative evaluations and retains published AI/policy. The required
completed domain error census is170+24+5=199; the current implemented census
remains170 until implementation. No ideal-upper-image or constant-price
yield error is introduced. ZeroDirtyPrice is defensive, proof-excluded for
valid admitted positive-par prices; it does not authorize forged Risk tests.

Stable phases, quantities and capacity operations are completed in N below
and the cash component. Allocation phases are `pricing bundles`,
`present value terms`, `risk present value terms`, `yield present value terms`.
Date phase is `quasi cycle`. A failed yield callback retains the canonical
attempted finite point, actual EvaluationStage and counts including itself.

## G. Complete cash API and arithmetic component

The cash contract specifies every CashInstruction inspector, Decision variant,
Receipt method, recipe identifier, public source-view stage, intermediate
publication stage, quantizer/error order and complete width proof. It is
jointly normative with C's four factories and was reconciled in the joint
pre-adoption reviews. Cash source publication uses TreasuryTieRule; final cash uses the
independent caller SettlementRounding. No default or silent replacement of
the original SettlementEvidence guarantee is permitted.

## H. Adopted requirement/evidence families

These authoritative local IDs have the same complete scope in
[REQUIREMENTS](REQUIREMENTS.md) and the [test design](M8_US_TREASURY_TEST_DESIGN.md).
Their implementation and validation obligations remain pending:

| Requirement | Obligation |
|---|---|
| FR-UST-01 | Retained source version, source inconsistencies and local interpretations |
| FR-UST-02 | Fourteen-type traits, definition/report/cash provenance and identity |
| FR-UST-03 | Original dated/issue roles, constructor order and explicit caps |
| FR-UST-04 | Complete direct maturity cycle, EOM and date boundary handling |
| FR-UST-05 | Regular/short/long first coupon, skipped original index |
| FR-UST-06 | Both daily bases and both explicit publication tie rules |
| FR-UST-07 | Full-C exact raw/source coefficients and explicit quantized views |
| FR-UST-08 | Strict-future plan, entitlement, coordinates and prepared resources |
| FR-UST-09 | Nonnegative raw/published AI, total publication and reset boundaries |
| FR-UST-10 | Deterministic p256 lower-endpoint factor observation and bounds |
| FR-UST-11 | Direct single IEEE source PV and exact dirty reduction |
| FR-UST-12 | Owned reports, both AIs, policy-aware clean views |
| FR-UST-13 | All six ideal-model risk definitions and stub Macaulay discriminator |
| FR-UST-14 | Once-rounded complete extended terms and direct normalization |
| FR-UST-15 | Independent signed getter range admission and proved exclusions |
| FR-UST-16 | Native target/published-AI shift and nonpositive rejection |
| FR-UST-17 | Bounded Brent, exact residual, actual callbacks and cached payload |
| FR-UST-18 | Selected-source coupon cash and nominal-scaling qualification |
| FR-UST-19 | Published accrued cash, mandatory source publication before scaling |
| FR-UST-20 | Clean evidence invoice, both explicit aggregation recipes |
| FR-UST-21 | Dirty evidence trade with one final cash quantizer |
| FR-UST-22 | Universal Exact, continuous evidence, ambiguity and final admission |
| FR-UST-23 | Structured errors, stable kind/stage context and precedence |
| FR-UST-24 | Whole-domain widths, fallible preparation, bounded work and reuse |
| FR-UST-25 | Independent pre-production oracles and discriminating semantic families |
| FR-UST-26 | Full native checks, coverage classification, package and preserved baseline |

## I. Adoption and execution order

The complete main/cash pair, numeric temporary/operation bounds, interfaces
and independent oracle/comparison/test specification were jointly reviewed
and adopted as recorded in the [integration review](M8_US_TREASURY_CONTRACT_REVIEW.md).
Next author and verify independent general oracles and blind hand controls,
freeze their expectations and provenance, then implement the complete scope
and execute the required validation. Native foreign-runtime, current legal,
announcement/calendar, owner financial and wider M8 gates stay open. This
local contract and research checks do not complete those gates.

## N1. Exact model and deterministic observation

For the original quasi ordinal j of each prepared raw amount A_j, define:

```text
alpha=r/s; delta=2+y; b=1+y/2; q=(2*s+r*y)/(2*s)
F_j=1/(q*b^j); V_j=A_j*F_j; D_ideal=sum(V_j)
```

Admit finite represented binary64 y>-2, canonicalizing signed zero. All b/q
and day/source expressions here denote exact rationals, not intermediate
float arithmetic. The ideal derivative is not the derivative of the
discontinuous observed program. Whole powers make this a rational model;
no logarithm, exponential, powf/powi or elementary libm profile is used.
This source-specific integer power is not a general elementary-function API.

Select p=256 endpoint precision. A positive endpoint is `(m,e)` representing
m*2^e, with exactly256 significant integer bits in m and checked i32 e. An
interval has ordered positive low/high endpoints. All exact comparisons,
integer shifts/divisions and normalizations below preserve discarded-bit
information. No silent high-limb truncation is a successful result.

Define point normalization for positive exact `(N/D)*2^E`:

1. Determine k=floor(log2(N/D)) by significant-bit counts and one exact
   aligned comparison. Set output unit U=k-255.
2. Shift N by -U if U<0, or D by U otherwise. Exact quotient/remainder give
   `f=floor(N/D/2^U)` and the discarded fraction, with 2^255<=f<2^256.
3. Down selects f; up selects f+1 iff the remainder is nonzero. If that
   increment reaches2^256, normalize it to2^255 and increment U.
4. Return selected m with exponent E+U. Keep power-of-two carry exactly.

Conversion encloses the exact seed with these two directed selections.
Multiplication encloses the exact low product and exact high product by
normalizing their integer products at the summed exponent. Division uses
low numerator/high denominator for low, high numerator/low denominator for
high, with exact integer-ratio normalization at exponent difference. Positive
monotonicity establishes containment by induction.

At each numeric call, after quote validation and workspace reservation,
construct b and q intervals once from the exact represented y and retain
them. For each j independently, binary-power b left-to-right: j=0 gives
exact1; otherwise start with b at the leading1, then square and multiply
by the original b for each remaining1 bit. Do not use repeated sequential
coupon-to-coupon multiplication or renumber missing payments.

For j=0 use q directly; otherwise multiply q interval by that power. Divide
exact1 by that interval. Round its **low endpoint** once to unbounded53
significant bits nearest/ties-even: discard203 bits, compare the discarded
part with its exact half, use significand parity on a tie, then normalize a
53-bit carry. This is `factor_hat`; no binary64 range admission occurs here.
It is a deterministic bounded observation, not the ideal factor's correctly
rounded value, a lower price or a declared settlement enclosure.

Both endpoints are specified for proof and primitive validation. An
optimization omitting unused computations is permitted only after proving
the selected low endpoint observationally identical for all admitted inputs.
Caching pure seed conversions is specified; changing the power tree is not
an automatically equivalent optimization. The zero-yield case follows the
same exact seeds b=q=1 and gives exact factor1 for every j.

## N2. Whole-domain width, exponent and seed construction proof

Let u=2^-255. Each inexact p-bit seed conversion has width ratio <=1+u;
each outward product/quotient adds at most gamma=(1+u)/(1-u). Assign expanded
tree weights `(c,o)` to conversions/operations, adding both child weights
at a product or quotient. A squared child therefore doubles its weights.
Then `hi/lo <= (1+u)^c*gamma^o <= 1/(1-(c+2o)*u)` whenever `(c+2o)*u<1`.
The last bound uses Bernoulli and exact integer inequalities, not a floating
logarithm. Actual instruction count is O(log j), but width weights are O(j).

For this factor cell, j>=1 has c<=j+1, o=j+1, so a conservative bound is
`N=c+2o<=3*j+4<=60001`. j=0 has c<=1,o=1, within the same bound. Let v=N*u
and t=2^-53. Containment yields `(1-v)*F<=low<=F`; the final extended53
nearest observation gives

```text
(1-t)*(1-v) <= factor_hat/F <= 1+t.
```

No universal ideal-RNE decision is needed. Exact ties and close midpoints
must still be handled by the specified rounding algorithm and independent
primitive controls. A research sample's zero unresolved endpoint pairs
cannot prove a uniform correct-rounding guarantee.

Construct seeds using B=2^1074 and signed exact integer Y=y*B. Normal/subnormal
binary decomposition gives Y without rounded arithmetic. For valid y:

```text
b = (2*B+Y)/2 * 2^-1074
q = (2*s*B+r*Y)/(2*s) * 2^-1074
delta = (2*B+Y) * 2^-1074.
```

Positive-y addition and negative-y magnitude subtraction are checked. The
resulting positive numerators are below2^2107 and fit34 u64 words (2176bits).
The negative lower-domain limit cannot cancel to zero: b/q>=2^-53 and
delta>=2^-52. Seed normalized p256 extraction aligns only within this
34-word space; the exact numerators after cancellation still have at least
1022bits, and denominator at most9bits, so no extra255-bit numerator shift
is needed. Ceil carry has257bits before endpoint renormalization.

Exact b/q and their directed endpoints stay within inclusive power-of-two
envelopes `[2^-53,2^1023]`. That envelope is preserved by directed normalized
rounding because its endpoints are exactly representable. For j<=19,999,
every resulting factor endpoint lies in
`[2^-20460000,2^1060000]`. Its normalized p256 exponent lies in
[-20460255,1059745]; the selected53-bit exponent lies within
[-20460052,1059948]. Products/quotients along the specified tree obey the
corresponding prefix-index envelopes. All checked exponent arithmetic fits
i32; no exponent-size allocation or enormous absolute shift is required.

Store endpoints in a bounded container, permitting nine-word arithmetic
temporaries. Normalized products need512bits. A quotient implementation that
forms `N<<257` before extracting256bits can need **513bits**; eight words are
insufficient for that intermediate. Nine words provide576bits. For generic
normalization at p256, the bit-length aligned comparison and quotient shifts
need at most max(input numerator bits,denominator bits+256)+1bits; use the
34-word container for seed extraction and nine-word container for endpoint
operations. Carry is checked before narrowing. There is no precision ladder.

## N3. Exact source PV, reduction and quantitative comparison policy

Prepared raw coefficients are exact positive/zero N_A/D_A, not public decimal
or float observations. Their chosen unreduced day denominators are bounded
by2*s0*s1*10^Cscale, with unused period factors optionally cancelled; bounded
integer day numerators are below2^17. Adding par100 exactly gives a uniform
`N_A<2^152,D_A<2^144`; raw amount A_j<C+100<2^128. No arbitrary unreduced
multiplier beyond these stated half-year factors is permitted.

For nonzero A and extended factor `(sig,e)`, form exact `N=N_A*sig,D=D_A`
with the binary exponent retained. **Round directly once to IEEE binary64**:
find k=floor(log2(N/D)), K=k+e; K>1023 gives infinity, K<-1075 gives zero.
Otherwise choose unit U=max(K-52,-1074), align by e-U, divide exactly and
nearest/even-round using remainder versus denominator-minus-remainder.
Handle53-bit carry, normal/subnormal crossover and overflow. Never first
round the exact product into extended53 then round again to binary64.

N<2^205,D<2^144. Initial aligned comparison, normalized quotient alignment
and subnormal-clamped alignment need at most206bits, including correction
and carry, inside five words (320bits). Early K classification prevents
absolute factor exponents of millions from becoming integer shifts. At
subnormal U, a negatively shifted denominator is bounded by numerator bits+1;
a positive numerator shift is no larger than its normalized53-bit alignment.

Observe a zero coefficient's factor first, then use canonical+0 PV and
exact-zero risk contributions. A nonzero source coefficient rounding to
zero is a typed positive underflow; infinity is a typed range failure. No
later term or final reduction rescues that failed source PV. Finite subnormal
PVs are admitted. These are PV gates, not binary-factor gates: a tiny/huge
extended factor can be admitted when the exact scaled source PV is finite
and positive.

Traverse source order, collect the accepted finite PVs, then exact_sum once
for dirty. Positive principal makes every successful dirty strictly positive;
overflow in the complete sum is ordinary Sum failure. Dirty reduction
precedes immutable valuation/risk construction. Do not evaluate clean during
dirty price/risk/inversion, and do not use rounded source-cash coupons.

For every accepted PV, with ideal V, h=2^-1075 and t/v as above:

```text
abs(PVhat-V) <= (2*t+t*t)*V + h.
abs(Dhat-D_ideal) <= ((1+t)^3-1)*D_ideal + ((1+t)*m+1)*h,
```

where m is the count of positive accepted PVs. The normal-only relative
PV bound2*t+t*t requires the exact product A*factor_hat>=MIN_NORMAL; a rounded
PV equal to MIN_NORMAL does not establish that condition. If all exact
products and their sum are normal, the h terms can be omitted. At subnormal
PV, relative error can approach100%; use absolute/conditioned comparisons.

These form the selected `TOL-UST-RATIONAL` comparison profile. Primitive
observations are tested TOL-E0 against the prescribed algorithm, while
ideal price/risk/inverse comparisons use outward evaluation of these bounds
and denominator conditioning. An uninformative bound is recorded as such,
never counted as a financial accuracy assertion or widened silently. The
profile is not a public settlement-enclosure producer. Printed-source
decimal display comparisons are a separate literature policy.

An analytical clean view's ideal reference is D_ideal minus the exact
**selected published** AI, not raw AI. Its absolute allowance is the dirty
allowance plus the cached published-AI IEEE error plus one final exact-sum
IEEE rounding radius. Use absolute propagation near cancellation/zero and
for signed clean values. Clean-target inversion similarly includes the AI
cache/target-shift allowance when comparing with an ideal clean-price root.

## N4. Six risks, complete retained ratios and direct normalization

For each ideal V and exact alpha=r/s:

```text
eta_j=j/delta + alpha/(2*q)
V'_j=-V_j*eta_j
kappa_j=j*(j+1)/delta^2+j*alpha/(delta*q)+alpha^2/(2*q^2)
V''_j=V_j*kappa_j
tau_j=(j+alpha)/2; M=sum(tau_j*V_j)
Modified=-D'/D; Macaulay=M/D; Convexity=D''/D.
```

No convexity half is applied. Source quasi time, not a separately queried
day count, defines tau. For a stub Macaulay is not b*Modified; alpha=.5,
j=0,y=.05,A=102.5 gives1/4 versus41/162. The full alpha=1 periodic identity
is recovered. Raw derivative units are per100 per annual-decimal yield and
its square; durations are annualized quasi years and convexity their square.

Risk observes once `delta_hat=RN64(exact(2+y))` and
`q_hat=RN64(exact((2s+r*y)/(2s)))`, using the34-word construction. Both are
positive normal, with delta_hat>=2^-52,q_hat>=2^-53; neither overflows even
at MAX. Use the same accepted IEEE PVs as evaluate, independently of the
factor interval's internal endpoints. For one PV the complete exact positive
ratios are:

```text
first magnitude = PV*(2*s*j*q_hat+r*delta_hat)/(2*s*delta_hat*q_hat)
second = PV*[2*j*(j+1)*s^2*q_hat^2
             +2*j*r*s*delta_hat*q_hat+r^2*delta_hat^2]
               /(2*s^2*delta_hat^2*q_hat^2)
moment = PV*(r+j*s)/(2*s).
```

Round each complete supplied-observation rational once to normalized53bits
nearest/even with a separately retained checked integer exponent. Do not
round the second derivative's three components separately. Preserve exact
r/s and integer j; no rounded alpha/lambda is substituted. Zero PV gives
zero contributions without decomposing a zero significand.

Accumulate three positive-magnitude retained sums exactly on basis2^-8192 in
243 u64 words, then apply a minus sign only to the first derivative getter.
Raw getters round the retained sum once to IEEE. Normalized getters divide
that retained value directly by observed dirty and then round once, without
requesting a raw getter. Every getter is independent and performs no factor,
date, allocation or solver work. Exact zero is canonical where mathematically
zero; nonzero underflow errors preserve the required signed zero in the cause.

## N5. Coupled scalar and accumulator capacity proof

Decode the53-bit positive binary observations as `d*2^ed`, `q*2^eq`.
The normalized exponent difference is universally in[-51,9]; a conservative
[-52,9] is sufficient here. This follows from coupling through the **same**
y,r,s: for y<0 use q>=delta/2 and q<=1, normal-scale-compatible monotone RNE,
and q_hat<1 when y<-1; for y>=0 use exact delta/q in[2,368] and
`368*(1+t)/(1-t)<512`. It is not a premise for arbitrary independent d/q.
The power research gives the full analytical proof and separate samples.

Align the second-derivative three numerator terms on2*min(ed,eq). Before
alignment their integer bounds are below2^153,2^138,2^122. If ed<eq the
shifts are at most104,52,0; if ed>=eq they are0,9,18. Sum then multiply the
PV significand: numerator<2^311, denominator<2^229. First derivative and
moment are smaller. Normalized53 rational extraction requires no more than
max(numerator bits,denominator bits+53)<=311bits, including comparison,
quotient, discarded remainder and carry. Five words suffice. This is a new
Treasury caller proof, not gilt's different176-bit coefficient proof. Shared
integer arithmetic must not manufacture GiltError as a Treasury cause.

For accepted finite positive PV, `2^-1074<=PV<2^1024`, with
delta_hat in[2^-52,2^1024),q_hat in[2^-53,2^1023),alpha>2^-8:

```text
2^-1032 < eta_hat < 2^67
2^-2063 < kappa_hat < 2^134
2^-9 < tau <=10000 <2^14.
```

Thus all positive rounded extended risk terms lie strictly within the loose
envelope[2^-3200,2^1160], with normalized53 exponent>-3300. Fewer than20,000
terms give each sum<2^1175. The selected243-word basis spans15552bits and
reaches2^7360; every exact coefficient, carry and sum fits. Direct division
by finite binary64 dirty uses its <=53-bit significand and exponent in
[-1074,971]; comparison/normalization shifts remain inside the same width.
Explicitly, the accumulator integer has at most9367bits. After early IEEE
K classification choose output unit U=max(K-52,-1074); the normalized
denominator shift is8192+dirty_exponent+U, between6044 and10134. Including
the53-bit dirty significand gives at most10187bits. Raw rounding needs no
larger workspace. Both are below15552bits.
No raw IEEE range gate or growing-precision allocation is needed.

Conditioned ideal-risk comparison adds the PV absolute error in N3 to the
exact positive derivative/moment coefficients, each retained-term rounding,
and final getter/dirty-denominator rounding. Prefix observations satisfy
`eta_hat/eta in[1/(1+t),1/(1-t)]` and
`kappa_hat/kappa in[1/(1+t)^2,1/(1-t)^2]`. Tau is exact. Positive sums preserve
those bounds. Direct normalized comparisons require a strictly positive
lower bound for the ideal/observed denominator; if unavailable, classify
the comparison as uninformative rather than assuming a tiny relative error.

For a general retained risk sum, let ideal positive coefficient g_i be eta,
kappa or tau and let ghat_i be the exact supplied-observation coefficient.
With E_Vi from N3 and E_gi from the coefficient intervals above, a fully
quantitative per-term allowance is
`E_Ri=E_Vi*g_i + PVhat_i*E_gi + t*PVhat_i*ghat_i`.
Sum these positive allowances exactly/outward to E_R. A finite raw getter
adds its final IEEE radius, at most `t*abs(retained sum)+h` universally.
For ideal S_R,D and observed retained Shat_R,Dhat, direct normalized error
before the final IEEE getter is bounded by
`E_R/Dhat + abs(S_R)*E_D/(Dhat*D)`; then add the final IEEE radius.
Use rigorously positive denominator lower bounds for evaluated intervals.
These exact product/absolute forms, not first-order small-t expansions,
govern the independent ideal comparisons.

An important exact cancellation applies when only one PV is positive (C=0,
or only a final bundle remains). Then dirty equals that binary PV exactly,
including when subnormal. Each retained normalized-risk numerator is one
extended53 RN of PV*g_hat, so its direct division by dirty cancels the PV
observation; propagating the full absolute PV error would be unnecessarily
uninformative. For g=eta/kappa/tau and k=1/2/0 respectively, the computed
normalized value Z, when admitted finite, satisfies the absolute interval:

```text
[(1-t)^2/(1+t)^k]*g - h <= Z <= [(1+t)^2/(1-t)^k]*g + h.
```

The final h is omitted when its exact pre-IEEE getter input is normal.
Macaulay is always normal; Modified may be subnormal and Convexity may be
subnormal or fail positive-underflow admission. This profile makes valid
single-term subnormal-price risk comparisons informative without claiming
the price itself has tiny relative error. A sharper multi-term relative
structure lemma is optional research; this contract relies on the rigorous
absolute/conditioned bounds above rather than approximate first-order
constants. No blanket unconditioned ULP tolerance substitutes for them.

## N5a. Reachable getter failures and source-view exclusions

Let S=sum of accepted IEEE PVs, including the strictly positive principal
bundle, and Dhat=RN64(S)>0. For normal Dhat, S/Dhat lies in
[1/(1+t),1/(1-t)]; for subnormal Dhat, S is an exact MIN_SUB integer in
range, so Dhat=S. One extended53 RN changes each positive risk term by at
most t relatively. With the coefficient bounds in N5 this proves:

```text
2^-10 < observed Macaulay < 2^14
2^-1034 < observed Modified < 2^68
0 < exact retained normalized Convexity < 2^135.
```

Thus Macaulay/Modified cannot return zero or infinity, Macaulay is normal,
and Convexity cannot overflow. Finite subnormal Modified/Convexity are
admitted with an absolute final IEEE rounding term; Convexity can underflow
to zero and then returns the typed positive-zero error. The derivative
retained sums cannot be exact zero. ZeroDirtyPrice and opposite-sign outputs
are proof-excluded on valid successful plans.

Raw D' may overflow negatively or underflow to negative zero; raw D'' may
overflow positively or underflow to positive zero; raw M may overflow.
All require actual admitted financial controls, not forged private Risk
objects. Raw M underflow is excluded by principal structure:

- For n>=1, its principal PV>=MIN_SUB and tau_n>=185/368. Its retained
  moment is at least `(1-t)*185/368*MIN_SUB > MIN_SUB/2`; positive other
  moments cannot lower the sum.
- For n=0, A0>=100, q<2^1023 and the factor/PV observation bounds give
  PV>2^-1019. With tau>2^-9, the retained moment exceeds2^-1029, far above
  the zero-rounding threshold2^-1075.

There is no valid normalized-only Convexity underflow with a finite nonzero
raw D''. If Dhat>=1 and y<0, q_hat<=1 gives kappa_hat>2^-17. If y>=0, b>=1,
A_j<2^128 and fewer than2^15 terms give
`Dhat < 2^144/q + 2^-1058` from N3. Hence Dhat>=1 implies q<2^145 and
q_hat<2^146; its positive prefix alone gives kappa_hat>2^-309. Retained
rounding and S/Dhat then keep normalized Convexity>2^-310, excluding zero.
So its zero-rounding case requires Dhat<1. A retained S2/Dhat<=2^-1075
then has S2<2^-1075, so raw D'' also underflows. Do not transplant gilt's
different reachable normalized-only control into Treasury.

Raw AI<C<2^127 when positive. Under NineDecimal, two fractional daily
rounding errors total at most184e-9 per$1,000; the final five-place stage
adds at most.5e-5 per$1,000. Therefore published AI per100<C+1<2^128.
Any positive raw AI is at least `1/(2*10^38*184^2)>2^-143`; positive
published AI is at least1e-6. Their cached IEEE views are finite and cannot
underflow. Cache capacity uses the explicit phases/operations in N6.

Successful dirty minus cached published AI, and any finite target plus it,
have exact magnitude no greater than MAX+2^128, below the IEEE overflow
midpoint MAX+2^970. Clean views/target shifts cannot overflow. Their defensive
Sum/range wrappers are retained, without manufactured source states to
exercise them. The fullC domain is never curtailed to make optional public
Decimal views fit. The cash contract separately admits only final money.

## N6. Error precedence and stable numeric context

Evaluation/risk first validate finite y>-2 (Value, indexNone, phase
`yield quote`, ValueError parameter `rate`), then fallibly reserve exactly
bundle_count PV slots at their respective allocation phase. Build b/q
seeds; risk also builds its two prefix observations. Traverse increasing
absolute Q index: complete factor observation -> direct source PV -> that
bundle's extended risk terms, if requested. Dirty exact reduction follows
the whole traversal. Only then return the immutable result. No conditional
float range band, extra elementary observation or error sentinel is used.

| Work | phase | index | InternalCapacity operation(s) |
|---|---|---|---|
| instrument raw first-coupon cache | raw coupon | Some(0), actual coupon ordinal | derive source |
| instrument source first-coupon cache | source coupon | Some(0), actual coupon ordinal | derive source |
| NineDecimal daily9, constructor | daily decimal | Some(0), actual coupon ordinal | round daily |
| NineDecimal daily9, plan accrued pieces | daily decimal | None | round daily |
| plan total published AI count | accrued publication | None | publish accrued |
| raw AI rational/cache | raw source accrued | None | derive source; round source accrued |
| published AI IEEE cache | published accrued cache | None | round source accrued |
| b/q construction | discount seeds | None | decode yield; derive prefix; round seed |
| risk delta/q observations | risk prefix observations | None | derive prefix; round prefix observation |
| p256 power/product/division | discount factor | Some(absolute Q index) | multiply endpoint; divide endpoint; round endpoint; power exponent |
| final extended factor selection | discount factor | Some(absolute Q index) | round factor observation |
| exact source PV | source present value | Some(absolute Q index) | derive source; round source present value |
| complete risk ratios | price derivative / price second derivative / time weighted amount | Some(absolute Q index) | decode binary; normalize rational; round scaled term |
| retained risk sums | same contributing measure | Some(absolute Q index) | accumulate risk |
| raw getters | corresponding measure | None | round risk value |
| normalized getters | corresponding duration/convexity | None | normalize risk value |
| target/residual | yield residual / yield amount residual | None | residual difference; round residual |

PV Value quantity is exactly `treasury present value`, phase `source present
value`, with positive infinity ResultNotFinite or +0 ResultOutOfDomain in
the strict positive interval. Quote/prefix integer capacity failures cannot
be relabeled ordinary rate errors. Getter Value phase and quantity are both
exactly its measure name from the table/D: first derivative strictly<0,
all other five strictly>0. Retain signed infinity/zero and the corresponding
Interval. ZeroDirtyPrice is only a defensive normalization guard.

Sum phases are `dirty price reduction`, `clean price reduction`,
`yield target shift`. Target wrapper phase is `target clean shift`;
Preparation phase is `yield present value terms`. Amount residual Value
phase/quantity is `yield amount residual`. Cash/source-view context and
publication capacity stages are the separate complete cash ledger.

All capacity paths above are proof-excluded under valid inputs, not economic
overflow categories. Ordinary allocation failure and final IEEE/Decimal
range errors remain their actual typed categories. No invalid private
reports or arbitrary uncoupled derivative inputs are created to hit guards.

## N7. Bounded inversion and attainable-image qualifications

Resolve the finite caller BondPrice in native per100 units. Dirty target is
its value; clean target is one exact_sum of its value and cached published
AI. Raw AI never supplies this shift. Retain target, policy, both AI views
and once-resolved T in the report. Reject T<=0 before workspace/callbacks,
with NonPositiveTarget zero counts. Then reserve one reusable PV vector and
run the existing payload-carrying bounded Brent algorithm.

Only actually invoked points validate y and observe factors/PVs. Preserve
the frozen lower-first order, lower exact-zero one-call success even if an
unqueried upper endpoint would fail, count checks before every invocation,
counts including failed callbacks, derivative count0, actual trial safeguards,
precision-limit/tolerance rules and cached endpoint/history payload identity.
No automatic bracket expansion, hidden final repricing, uncounted closed-form
root, epsilon-residual termination, last-result latch or NaN signal exists.

Use fixed scale a=0 if abs(T)<2^970, otherwise1. Residual is one IEEE RN of
the exact represented difference `(Dhat-T)*2^-a`; never separately halve
the operands.34words suffice for the2099-bit MIN_SUB-basis signed difference.
It is finite, sign-preserving and zero iff Dhat==T. A callback zero and a
small coordinate bracket remain separate successful solver reasons. Report
getters use the cached selected Dhat and unscaled amount difference, with
zero new callbacks. Dhat/T>0 exclude unscaled difference overflow.

The ideal positive model is strictly decreasing but observed values form a
rounding staircase. This proves ideal sensitivity/conditional uniqueness,
not observed strict decrease, continuity, universal attainability or an
inverse accuracy certificate. For n=0,alpha<1 the ideal image has excluded
upper bound A0/(1-alpha); alpha=1 or retained j>=1 principal has unbounded
ideal upper image. **Do not pre-reject positive targets with the ideal upper
bound.** Rounding can meet/exceed it; the selected observation's fully valid
r1/s182,C5,ynextup(-2) counterexample is retained in research validation.
Actual caller-bracket callbacks decide success/failure, including round trips.

That counterexample is specific to N1's factor/PV observation: correctly
rounded ideal PV at those same inputs lies below the ideal supremum. More
generally RN(supremum) itself can lie above a nonrepresentable supremum, so
an ideal-real bound is not an exact observed-value range certificate. Any
future observation change must re-derive its discriminator. Source six-place
P staging is not inserted into the inversion callback.

## N8. Resource, operation isolation and complete validation obligations

Public real outputs have these adopted exactness/profile assignments, also
recorded in [NUMERICAL_CORRECTNESS](NUMERICAL_CORRECTNESS.md):

| Output family | Class/profile | Required comparison |
|---|---|---|
| Raw/published AI binary views | E1 / TOL-E1 | One correctly rounded exact retained rational;0ULP |
| Dirty and policy-aware clean prices | E5 / TOL-UST-RATIONAL | N3 exact-staging E0 plus absolute/conditioned ideal allowance |
| All six raw/normalized risks | E5 / TOL-UST-RATIONAL | N5 and N5a; final subnormal rounding explicit |
| Returned inverse yield | E5 / TOL-UST-INVERSE | Existing bounded-Brent coordinate contract plus N7 residual/model/target conditioning |
| Cached input-yield metadata | E0 snapshot / TOL-E0 | Canonical input bits, without a new arithmetic observation |
| Amount residual | E0 once-rounded exact reduction / TOL-E0 | N7's correctly rounded represented binary difference, following the existing M6 exact-sum convention;0ULP |
| All DecimalAmount source/cash views and decisions | E0 / TOL-E0 | Exact source policy/quantization/evidence semantics |

Private fixed-width integers/extended exponents do not replace public f64
analytics or implement general NI-03 elementary functions. Integer and IEEE
staging are intended bitwise reproducible at R1–R3 under existing basic
arithmetic semantics. Actual native foreign-runtime execution remains a
separate verification gate; compile-only evidence cannot establish it.

Source construction owns supplied Q and compact caches only; it does not
allocate a derived coupon vector. Plan construction reserves one vector of
at most20,000 prepared bundles; each stores a five-word numerator/denominator
and bounded original-index/coordinate fields, with <=96bytes logical payload
per bundle. Numeric calls reserve at most20,000 f64 PV slots (<160,001bytes);
yield reuses them across callbacks. No per-bundle allocation, date/calendar
query, arbitrary source clone or precision refinement occurs during numerics.

Every power endpoint has at most28 directed multiplication steps for j<2^15,
plus its fixed product/quotient. All divisions/normalizations are bounded by
their stated bit width. Solver callbacks<=u32::MAX and bundles<2^15 imply
fewer than2^47 bundle observations. A caller choosing huge explicit budgets
does not obtain an implicit runtime guarantee. There are no hidden callback
or calendar-query budgets.

Logical numeric storage must fit a64KiB payload budget across a helper call
chain: at most24 full243-word slots (46,656bytes), eight34-word seed/residual
magnitudes (2,176bytes),32 nine-word endpoint temporaries (2,304bytes) and
16five-word scalar temporaries (640bytes), plus bounded small metadata.
Implementation review must census actual argument copies/helper lifetimes
or reduce them. This does not promise compiler stack-frame size, a caller's
stack limit, allocation-free ordinary Clone or an OOM-free process.

General fixtures must be new and independently authored after adoption,
before Treasury Rust. Separate exact date/source/cash, prescribed primitive
observations, ideal all-six-risk and target-driven inverse corpora. Freeze
expectations, hashes, source provenance and hand discriminators without
rewriting old artifacts. The cash contract's full scale/quantum/evidence cases
are mandatory. Also include source A-G and I.A/I.D/Table cells, D!=I and
long I>Q1, skipped versus actual AI resets, both EOM/policy/tie choices,
general-C daily ties, tiny daily9-to-zero, all sign/range/underflow/carry and
subnormal direct-rounding cases, solver callback/error order and identity.

Run meaningful mutant controls, strict docs/public-catalog/error-census
checks, native debug/release, fmt/strict Clippy, warning-denied docs, foreign
target compilation, production-region coverage classification and actual
extracted-package checks against final bytes. Record native foreign runtime
as open until executed. Uninformative ideal rows, allocation guards and
proof-excluded financial/private states remain explicit rather than counted
as tested accuracy or manufactured test inputs.
