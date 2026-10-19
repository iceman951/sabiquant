# Local conventional gilt contract

| Field | Value |
|---|---|
| Status | **Adopted for complete local implementation; frozen** |
| Baseline | Local `0d1c987`, `feat/m8-conventional-gilt` |
| Date | 2026-10-19, Asia/Bangkok |
| Scope | Complete conventional gilt definition, preparation, price, yield, all six risk views, published-dividend cash, exact-source accrued cash and both invoice aggregation policies |
| Decision | All operations in this contract are adopted together; implementation and all independent acceptance gates remain pending |

Adopted on 2026-10-19 under the owner's continuing authority for autonomous
local engineering. Root reviewed the complete [draft](M8_GILT_CONTRACT_DRAFT.md)
(SHA-256 `2d113405b8aadfcf478fb967ac12e0e0d53d5cde81bdf7dcfe426565f49bb4ed`)
and [independent pre-adoption challenge](M8_GILT_CONTRACT_REVIEW.md), resolving
all source/API/numerical/cash findings before adoption. Authoritative
`FR-GILT-01..24` are recorded in [requirements](REQUIREMENTS.md#21-fs-gilt--local-conventional-gilt).
Independent oracles must be authored from this frozen contract before new gilt
production. This decision does not accept an implementation, financial model,
maintained calendar, owner API, native foreign platform, release or full M8.

Section S fixes exact source/date meanings. Sections A-J fix the complete
public interface, validation/failure order and bounded lifecycle. Sections
N1-N9 select numerical staging, proofs and accuracy qualifications. Sections
C1-C8 fix complete cash semantics. All sections are adopted together; a partial
price-only implementation cannot satisfy this contract. Earlier model,
numerical and test proposals are historical inputs, and their alternatives
do not override this contract. All earlier frozen contracts and evidence
remain unchanged.

## S. Primary evidence and complete source formulas

The primary source is the [UK DMO fourth edition, 18 December 2024](https://www.dmo.gov.uk/media/334d05fo/yldeqns_v4.pdf).
The inspected 29-page local PDF has SHA-256
`bd752ac495ec340c0d34e7d6d1e6fe0af2e1d25819364935312f53a9f5037745`.
The [model proposal](M8_GILT_MODEL_DESIGN.md) records the complete pages visually
inspected and source versus engineering distinctions. The historical [DMO worked examples](https://www.dmo.gov.uk/media/ftjpyv1z/yldconv.pdf)
have SHA-256 `8f491b8567c6f3a75ee2a7cbc02ccc4275810257df5533ed1fd349992465e018`;
complete PDF pages 1, 2 and 5 were visually inspected. The independent
[source checker](M8_GILT_SOURCE_EXAMPLES.md) reproduced eight displayed prices,
eight displayed inverse yields, 214 daily AI cells, three separate AI anchors
and the first-dividend display at their actual printed precisions. Those are
source checks, not executed Rust gilt tests or full-precision market oracles.

Use unadjusted civil dates and the caller-supplied, validated maturity-anchored
semiannual vector `Q[0..m]`, ending at `Q[m]=M`. Minimal issue coverage is
`Q[0]<=I<Q[1]`. Let `B=C/2` as an exact rational. The supported first roles are:

| Role | Required issue and first actual dividend F | Exact first dividend |
|---|---|---|
| Regular | I=Q0, F=Q1 | B, with no six-place publication rounding |
| Short | Q0<I<Q1, F=Q1 | Round6(B*r1/s1) |
| Long | Q0<I<Q1, F=Q2 | Round6(B*(1+r1/s1)) |

Here `s1=days(Q0,Q1)`, `r1=days(I,Q1)`. Round6 means exact nonnegative nearest
multiples of 1e-6 with half ties upward; publication is separate from cash
settlement rounding and has no public i128 intermediate admission. The full
exact annual coupon C is retained. Every later actual dividend is exactly B;
the actual dividend-date suffix starts at F. A Long first period has no
payment at Q1. Redemption is fixed par100 on M, independent of dividend
entitlement. There is no Schedule adapter, arbitrary redemption or adjusted
payment-date argument: these other definitions cannot enter through this API.

For an actual dividend ordinal i at date D_i with supplied cutoff E_i,
`dividend_entitlement(i)` is exactly:

| Condition, checked in order | Result |
|---|---|
| D_i<=S | CouponEntitlement::Ceased |
| D_i>S and S>E_i | CouponEntitlement::ExCouponDetached |
| Otherwise | CouponEntitlement::BuyerEntitled |

Out-of-range ordinal returns None. Equality S=E_i remains cum-dividend.
Missing Long-first Q1 is not an actual dividend ordinal. Entitled actual zero
coupons remain bundles and validate their factor; detached/missing dates are
absent without renumbering quasi indices. On any actual or skipped quasi date,
pricing chooses the strictly next Q date. Thus an actual dividend on S is
excluded from buyer PV, while the skipped Long-first quasi date never creates
a payment or an AI reset.

For the unique k with `Q[k]<=S<Q[k+1]`, define
`next_quasi_index=k+1`, `next_quasi_date=Q[k+1]`,
`r=days(S,Q[k+1])`, `s=days(Q[k],Q[k+1])` and `n=m-k-1`.
Thus local j ranges from0 through n, the maturity bundle has j=n, and the
last quasi interval has n=0. These definitions fix every remaining-period
inspector; neither a detached dividend nor a skipped payment changes n.

The exact source accrued amount A per100 uses the next actual dividend's
strict-after ex status. Its full piecewise definitions are:

| Current lifecycle | Cum (S<=next actual cutoff) | Ex (S>next actual cutoff) |
|---|---|---|
| Regular current period, or after actual first dividend F | B*t/s | B*(t-s)/s |
| Short first, I<=S<F | B*t_star/s1 | B*(t_star-r1)/s1 |
| Long first, I<=S<Q1 | B*t_star/s1 | Unreachable under compatible cutoff window |
| Long first, Q1<=S<F | B*(r1/s1+r2/s2) | B*(r2-s2)/s2 |

For a regular current interval `[Qk,Q(k+1))`, `t=days(Qk,S)` and
`s=days(Qk,Q(k+1))`. In the Short and first Long interval,
`t_star=days(I,S)`. In the second Long interval, `r2=days(Q1,S)` and
`s2=days(Q1,F)`. At S=Q1 in Long, use the second-interval Long formula with
r2=0: AI=B*r1/s1, even though pricing's next quasi interval has changed.
At an actual F<M, switch to the ordinary current period with t=0; AI resets
to zero. S=M is outside the named trading domain. No later auction date
replaces original I or changes the first dividend's source ratios.

AI uses raw source ratios, never cum AI minus a six-place rounded first
publication. For the source example C=6,I=1999-05-07,Q1=1999-06-07,
F=1999-12-07, the raw long dividend is639/182 and published3.510989;
source AI on 1999-12-06 is -1/61. Subtracting the rounded published dividend
from raw cum AI changes the model by1/91000000 per100. Preserve that
published-versus-raw distinction in both analytics and cash.

The unadjusted cycle, dividend and AI formulas and strict-after boundary are
source-based. The supplied complete Q vector, explicit EndOfMonth rule,
minimal coverage, compatibility windows and active trading domain are explicit
local engineering choices. They admit a bounded mathematical domain without
claiming every admissible input was issued or verified against a maintained
UK calendar. Long issue exactly on Q0 with F=Q2 is deliberately unsupported
until source eligibility is resolved. Signed final cash ties and invoice
aggregation remain caller policies; no legal market choice is silently made.

## A. Scope, module, units and source provenance

Expose the new types from `bonds`, implemented in a private child module.
Conventional-gilt coefficients and quoted/returned prices are per100 par;
annual coupon C is exact DecimalAmount in those units (C=6 denotes6%), whereas
yield y is annual decimal (.05 denotes5%). Redemption is the named model's
fixed par100. Exact traded/held nominal is independent of an analytical face.
There is no arbitrary face field or per100 conversion inside inversion.

The source identifier returned by every `source_version` inspector is exactly
`UK-DMO-Conventional-v4-2024-12-18`. It identifies the retained formula contract,
not a runtime source fetch, UK calendar assurance, financial certification,
price enclosure or claim that every admissible mathematical input describes
an issued gilt. Generic Bond and all previous settlement contracts remain
unchanged. No currency conversion, today, settlement lag, implicit calendar,
business-day adjustment, default EOM, bracket, tolerance, nominal, rounding,
aggregation or yield model is inferred.

## B. Complete selected public types and traits

Add exactly eleven public types:

1. ConventionalGilt
2. GiltFirstPeriod
3. GiltPricingPlan<'a>
4. GiltValuation
5. GiltYieldReport
6. GiltRisk
7. GiltError
8. GiltYieldError
9. GiltCashInstruction
10. GiltCashDecision
11. GiltCashReceipt

The required implemented public census is112, from baseline101 plus eleven.
Adoption alone does not claim that these new types already exist in production. All fields are private except explicit contextual enum fields.
All types implement Debug, Clone, PartialEq and Send+Sync. Instrument and cash
definition identity have Eq+Hash; floating valuation/risk/yield/error/plan
values do not gain Eq, Hash or numerical ordering. GiltFirstPeriod and the
three compact owned cash types are Copy+Eq+Hash; GiltValuation and GiltYieldReport
are Copy, GiltRisk is not. ConventionalGilt owns vectors and is not Copy;
GiltPricingPlan borrows it and owns bounded prepared bundles, hence is not
unconditionally 'static. The other new types are owned 'static values.

ConventionalGilt equality/hash retain original issue, maturity, canonical C,
first actual dividend, supplied complete Q vector, explicit EOM policy and
all supplied ex-dates. Derived exact caches do not erase definition identity.
Cash identity retains recipe, compact source snapshot, nominal, exact evidence
kind/bounds and all policies; equal numeric cash does not erase provenance.
Cash source snapshots are owned and contain only required dates/ratios/C,
never a full instrument clone. Ordinary Clone of vectors retains the standard
allocation/OOM qualification; fallible-construction guarantees do not extend
to Clone or arbitrary Debug formatting allocation. Risk Debug omits giant arrays.

GiltFirstPeriod is non-exhaustive with `Regular`, `Short`, `Long`, in that order;
`ALL:[Self;3]` lists them. It is inferred source-role metadata with no parser,
Display registry, default or user-selected policy constructor. Other new
contextual enums/variants are non-exhaustive. Both error types implement
std::error::Error and Display; wording is unstable. No new type has Default,
Ord/PartialOrd, panicking arithmetic, implicit From<f64> or an implicit money
conversion. No generic public callback/private-number type is exposed.

## C. Full instrument and plan signatures

All names below are public signatures, not illustrative pseudocode. Existing
argument types retain their complete contracts.

```text
ConventionalGilt::MAX_QUASI_DATES: usize = 20_000
ConventionalGilt::new(issue_date:Date, maturity_date:Date,
    coupon_per_100:DecimalAmount, first_dividend_date:Date,
    quasi_dates:Vec<Date>, ex_dividend_dates:Vec<Date>,
    end_of_month:EndOfMonth, max_dates:usize) -> Result<Self,GiltError>
ConventionalGilt::source_version(&self) -> &'static str
ConventionalGilt::issue_date(&self) -> Date
ConventionalGilt::maturity_date(&self) -> Date
ConventionalGilt::coupon_per_100(&self) -> DecimalAmount
ConventionalGilt::first_dividend_date(&self) -> Date
ConventionalGilt::first_period(&self) -> GiltFirstPeriod
ConventionalGilt::first_dividend_quasi_index(&self) -> usize
ConventionalGilt::quasi_dates(&self) -> &[Date]
ConventionalGilt::dividend_dates(&self) -> &[Date]
ConventionalGilt::ex_dividend_dates(&self) -> &[Date]
ConventionalGilt::end_of_month(&self) -> EndOfMonth
ConventionalGilt::ex_coupon_boundary(&self) -> ExCouponBoundary
ConventionalGilt::dividend_count(&self) -> usize
ConventionalGilt::dividend_per_100(&self,index:usize,
    quantum:AmountQuantum,mode:AmountRounding) -> Result<DecimalAmount,GiltError>

GiltPricingPlan::MAX_BUNDLES: usize = 20_000
GiltPricingPlan::new(gilt:&'a ConventionalGilt,settlement_date:Date,
    max_bundles:usize) -> Result<Self,GiltError>
GiltPricingPlan::gilt(&self) -> &'a ConventionalGilt
GiltPricingPlan::source_version(&self) -> &'static str
GiltPricingPlan::settlement_date(&self) -> Date
GiltPricingPlan::next_quasi_index(&self) -> usize
GiltPricingPlan::remaining_quasi_periods(&self) -> u32
GiltPricingPlan::remaining_days(&self) -> u32
GiltPricingPlan::quasi_period_days(&self) -> u32
GiltPricingPlan::bundle_count(&self) -> usize
GiltPricingPlan::bundle_quasi_index(&self,bundle_index:usize) -> Option<usize>
GiltPricingPlan::dividend_entitlement(&self,dividend_index:usize)
    -> Option<CouponEntitlement>
GiltPricingPlan::accrued_per_100(&self) -> f64
GiltPricingPlan::evaluate(&self,yield_rate:f64) -> Result<GiltValuation,GiltError>
GiltPricingPlan::risk(&self,yield_rate:f64) -> Result<GiltRisk,GiltError>
GiltPricingPlan::yield_from_price(&self,target:BondPrice,
    bracket:RootBracket,options:SolverOptions) -> Result<GiltYieldReport,GiltYieldError>
```

Date inputs are already valid Date values; quantum/mode/price/options inputs
are already their valid domain types. Argument expressions are evaluated by
the caller before entry; this contract orders work only inside the operation.
`dividend_dates()` is the Q suffix beginning at first_dividend_quasi_index;
it includes every actual dividend through maturity, not the skipped first-long
quasi date. `ex_coupon_boundary()` is always After. No UK-calendar-generated
cutoff provenance is implied. Zero-valued published actual coupons remain
actual dividends. Out-of-range optional inspectors return None without
allocation; value operations use InvalidDividendIndex.

`dividend_per_100` is explicit quantization of the exact source dividend. Its
quantization failures are CashArithmetic at stage `source dividend`, endpoint
`point`, with AmountError::Inexact or OutOfRange and operation `quantize`. It
does not promise an exact DecimalAmount point: C=1e-38 has standard dividend
5e-39, and first-dividend microunit coefficients may need147 bits. Mode Exact
can therefore fail. The full C>=0 DecimalAmount domain is retained internally;
there is no C cap to make an inspector or intermediate addition fit i128.

`remaining_days` is r, `quasi_period_days` is s, and
`remaining_quasi_periods` is n. Local j=0..n corresponds to original absolute
Q index next_quasi_index+j; lambda=(r+j*s)/s. Missing/detached dividends do not
renumber that lattice. All numeric error `index` fields below identify this
absolute Q index; a cash/source InvalidDividendIndex instead identifies the
actual-dividend ordinal explicitly named by that operation.

## D. Full owned analytic result accessors

Every one of GiltValuation, GiltRisk and GiltYieldReport provides these metadata
inspectors (GiltRisk uses &self; the two Copy reports take self):

```text
source_version() -> &'static str
issue_date() -> Date
maturity_date() -> Date
coupon_per_100() -> DecimalAmount
first_dividend_date() -> Date
first_period() -> GiltFirstPeriod
end_of_month() -> EndOfMonth
settlement_date() -> Date
next_quasi_date() -> Date
remaining_quasi_periods() -> u32
remaining_days() -> u32
quasi_period_days() -> u32
yield_rate() -> f64
dirty_per_100() -> f64
```

They retain owned scalar formula/context metadata, not the full original Q/ex
vectors. The original instrument/plan remains authority for that complete
definition; report equality does not claim whole-instrument identity.

Additional exact signatures:

```text
GiltValuation::accrued_per_100(self) -> f64
GiltValuation::dirty_price(self) -> BondPrice
GiltValuation::clean_price(self) -> Result<BondPrice,GiltError>

GiltRisk::price_derivative(&self) -> Result<f64,GiltError>
GiltRisk::price_second_derivative(&self) -> Result<f64,GiltError>
GiltRisk::time_weighted_amount(&self) -> Result<f64,GiltError>
GiltRisk::modified_duration(&self) -> Result<f64,GiltError>
GiltRisk::macaulay_duration(&self) -> Result<f64,GiltError>
GiltRisk::convexity(&self) -> Result<f64,GiltError>

GiltYieldReport::target(self) -> BondPrice
GiltYieldReport::dirty_target_per_100(self) -> f64
GiltYieldReport::accrued_per_100(self) -> f64
GiltYieldReport::initial_bracket(self) -> RootBracket
GiltYieldReport::options(self) -> SolverOptions
GiltYieldReport::solve_report(self) -> SolveReport
GiltYieldReport::residual_scale_exponent(self) -> u8
GiltYieldReport::sensitivity(self) -> BondYieldSensitivity
GiltYieldReport::amount_residual(self) -> Result<f64,GiltError>
GiltYieldReport::dirty_price(self) -> BondPrice
GiltYieldReport::clean_price(self) -> Result<BondPrice,GiltError>
```

Raw price derivatives are per100 per annual-decimal yield and its square;
moment is per100 times annualized quasi years. Modified/Macaulay are those
years and convexity their square; there is no convexity half. Valid plans
always return MathematicallyStrictlyDecreasing sensitivity, conditional on the
real source model. The numerical sections separately proves which signed
range/zero outcomes are reachable versus defensive.

Cached AI is RN-even of the exact source AI rational, independently of any
cash rounding. Clean analytic views use exact_sum([cached_dirty,-cached_AI])
and no callbacks. Dirty-only evaluation, inversion and risk never eagerly
request clean. Report getters do no date/factor/solver work. The mathematical
source AI remains retained separately for exact-source cash instructions.

## E. Cash signatures and compact provenance

Sections C1-C8 supply the exact cash arithmetic and stage definitions;
the following public interface is part of this same adopted tranche:

```text
ConventionalGilt::dividend_cash(&self,index:usize,nominal:DecimalAmount,
    rounding:SettlementRounding) -> Result<GiltCashInstruction,GiltError>
GiltPricingPlan::accrued_cash(&self,nominal:DecimalAmount,
    rounding:SettlementRounding) -> Result<GiltCashInstruction,GiltError>
GiltPricingPlan::clean_accrued_cash(&self,nominal:DecimalAmount,
    clean:SettlementEvidence,aggregation:CashAggregation,
    rounding:SettlementRounding) -> Result<GiltCashInstruction,GiltError>
GiltPricingPlan::dirty_cash(&self,nominal:DecimalAmount,
    dirty:SettlementEvidence,rounding:SettlementRounding)
    -> Result<GiltCashInstruction,GiltError>

GiltCashInstruction::evaluate(&self) -> Result<GiltCashDecision,GiltError>
GiltCashInstruction::source_version(&self) -> &'static str
GiltCashInstruction::recipe_identifier(&self) -> &'static str
GiltCashInstruction::nominal(&self) -> DecimalAmount
GiltCashInstruction::cash_rounding(&self) -> SettlementRounding
GiltCashInstruction::price_evidence(&self) -> Option<SettlementEvidence>
GiltCashInstruction::aggregation(&self) -> Option<CashAggregation>
GiltCashInstruction::coupon_per_100(&self) -> DecimalAmount
GiltCashInstruction::issue_date(&self) -> Date
GiltCashInstruction::maturity_date(&self) -> Date
GiltCashInstruction::first_dividend_date(&self) -> Date
GiltCashInstruction::first_period(&self) -> GiltFirstPeriod
GiltCashInstruction::end_of_month(&self) -> EndOfMonth
GiltCashInstruction::issue_quasi_dates(&self) -> (Date,Date)
GiltCashInstruction::settlement_date(&self) -> Option<Date>
GiltCashInstruction::settlement_quasi_dates(&self) -> Option<(Date,Date)>
GiltCashInstruction::source_dividend_index(&self) -> Option<usize>
GiltCashInstruction::source_dividend_date(&self) -> Option<Date>
GiltCashInstruction::ex_dividend_date(&self) -> Option<Date>
GiltCashInstruction::is_ex_dividend(&self) -> Option<bool>
GiltCashInstruction::source_value_per_100(&self,quantum:AmountQuantum,
    mode:AmountRounding) -> Result<Option<DecimalAmount>,GiltError>

GiltCashDecision::Determined(GiltCashReceipt)
GiltCashDecision::QuantumAmbiguous {
    lower_amount:DecimalAmount,upper_amount:DecimalAmount }
GiltCashDecision::ExactStageUnresolved { stage:&'static str }
GiltCashReceipt::amount(self) -> DecimalAmount
GiltCashReceipt::instruction(&self) -> &GiltCashInstruction
GiltCashReceipt::price_evidence_guarantee(self) -> Option<SettlementGuarantee>
```

No old SettlementReceipt or guarantee is fabricated. Pure source dividend/AI
receipts have None price-evidence guarantee: their exact-source arithmetic is
identified by their retained source recipe. For price recipes, Some labels
only the supplied price ingredient, including the caller's enclosure claim;
it does not turn empirical analytic error into a proved ideal-price enclosure.
Dirty recipe's source_value_per_100 is None. Other source values use exact
point rational quantization with explicitly requested quantum/mode.

## F. Constructor order, direct cycle and lifetime

Instrument construction performs exactly this order:

1. max_dates must be2..=MAX_QUASI_DATES.
2. Reject Q.len()>max_dates, then Q.len()<2.
3. Reject negative C; zero is allowed. Then require issue<maturity.
4. Require Q.last()==maturity, then strictly ascending Q in increasing original
   index order (first nonincrease wins).
5. Require minimal issue coverage Q[0]<=I<Q[1]. Thus issue on the cycle is
   Q[0]=I; redundant earlier prefixes are rejected rather than creating
   arbitrary irrelevant work or first-period ambiguity.
6. For index k increasing from0, expected Q[k] is a direct six-month multiple
   from original maturity: -6*(Q.len()-1-k) months, never recursive clamping.
   With Ignore, clamp maturity's day to that target month. With Preserve,
   preserve calendar month-end iff original maturity is calendar month-end;
   otherwise use ordinary clamping. No business calendar participates.
   A checked date arithmetic failure is Date with index=Some(k),
   phase=`quasi cycle`; a different valid expected date
   is QuasiDateMismatch. Caller-supplied EOM is retained, not UK-compliance.
7. Infer Regular iff I=Q0 and F=Q1; Short iff Q0<I<Q1 and F=Q1; Long iff
   Q0<I<Q1 and F=Q2. Other choices are FirstDividend. In particular I=Q0,F=Q2
   is rejected pending separate source evidence, rather than silently admitted
   by the mathematical full-year limit.
8. Require ex-date count equal to actual dividend count. In dividend order,
   check each cutoff inside its inclusive compatible window: first Regular
   lower I, first Short lower I, first Long lower Q1; every later lower bound
   is its previous quasi date; upper bound is that actual dividend date.
   The semantic cutoff rule is fixed After, never OnOrAfter.
9. Reserve one derived source-dividend vector fallibly, then form exact private
   source coefficients/publication data in original order. Full-C intermediates
   are retained in bounded wide form; no public Amount view is requested.
   Return the immutable owned definition only after complete success.

Input Vec allocations were made by the caller; successful construction owns
them without a hidden clone. Dates/C/EOM/cutoffs and derived coefficient data
are immutable. No partially valid instrument escapes.

Plan preparation checks max_bundles<=MAX_BUNDLES (zero is a valid cap value),
then S>=I, then S<M. It resolves the strictly next Q and (r,s,n), counts future
actual buyer bundles without allocation, and rejects required>max_bundles.
Principal shares the last entitled-date bundle or creates that last bundle
when its dividend is detached. A published zero actual coupon is retained;
missing long-first Q and detached payments are absent, without changing j.
Only then derive exact AI/canonical cached binary AI, reserve the bundle
vector once, and fill source coefficients and exact L/s coordinates in order.
No date/calendar query budget is hidden: this named model uses no DCC query or
calendar scan. Its finite input/global caps and deterministic work are below.

## G. Numerical calls and yield ordering

Evaluation/risk validate finite y>-2 before workspace reservation. They reserve
exactly bundle_count PV slots with try_reserve, then traverse bundle Q indices
in original order: factor and its NumericalBoundary policy -> one exact-source-
coefficient times accepted discount IEEE PV rounding -> risk terms if requested.
Actual zero coupons still observe their factor. Small coefficients, source
zero, or later sums never rescue an inadmissible factor. No intermediate
compound-factor admission is implied: the numerical sections defines direct
discount observation. Dirty reduction occurs after traversal; any dirty Sum
error precedes returning a value/risk. Risk extended sums are subtracted only
after successful price traversal/reduction. Raw getter admission is deferred.

Inversion order is target already-valid BondPrice -> native per100 target
(no face conversion) -> Clean optional exact_sum([target_value,cached_AI]) ->
fixed residual scale -> reject resolved T<=0 as NonPositiveTarget with zero
evaluations -> PV workspace reservation -> normal lower-first bounded Brent.
Dirty-target rejection is a proof of positive-price model incompatibility,
not an unannounced change to generic Bond's signed targets. No sensitivity-key
vector/sort is required: positive principal and positive lambda already supply
the conditional decreasing certificate.

Only actual callback points undergo yield/factor validation. Preserve lower
one-call exact-zero success without checking upper, checked caps before every
invocation, increment including a failed callback, derivative count0, exact M6
trial trace/order/tolerance/precision safeguards and cached endpoint payload.
No final repricing, last-result latch, uncounted closed-form shortcut, automatic
bracket expansion, epsilon residual termination or NaN error signal is allowed.
Successful getters/cash factories/evaluation views make zero further callbacks.

Residual R=RN((Dhat-T)*2^-s), s=0 for abs(T)<2^970 else1. Exact operand
difference precedes output exponent adjustment/one rounding. Reports retain
native T, cached Dhat and s; amount_residual separately rounds their unscaled
exact difference. Because admissible T and Dhat are positive, its overflow is
proof-excluded here, while the generic helper's complete signed policy remains.
Computed zeros/small brackets do not establish a mathematically exact root or
a rigorous financial enclosure. Numerical component/corpus qualifications apply.

## H. Complete structured GiltError ledger

All contextual variants below are non-exhaustive. `kind()` returns the listed
stable spelling. source() is the immediate typed cause where shown, otherwise
None. Float range causes preserve exact signed infinity/zero and Interval
fields; Display prose is not a test contract.

| Variant and fields | kind | source |
|---|---|---|
| InvalidDateCap{cap:usize,minimum:usize,maximum:usize} | gilt.invalid_date_cap | None |
| TooManyQuasiDates{count:usize,cap:usize} | gilt.too_many_quasi_dates | None |
| TooFewQuasiDates{count:usize,minimum:usize} | gilt.too_few_quasi_dates | None |
| NegativeCoupon{coupon_per_100:DecimalAmount} | gilt.negative_coupon | None |
| IssueNotBeforeMaturity{issue_date:Date,maturity_date:Date} | gilt.issue_not_before_maturity | None |
| MaturityBoundary{maturity_date:Date,last_quasi_date:Date} | gilt.maturity_boundary | None |
| QuasiDateOrder{index:usize,previous:Date,date:Date} | gilt.quasi_date_order | None |
| QuasiCoverage{issue_date:Date,first_quasi_date:Date,next_quasi_date:Date} | gilt.quasi_coverage | None |
| QuasiDateMismatch{index:usize,supplied:Date,expected:Date,maturity_date:Date,end_of_month:EndOfMonth} | gilt.quasi_date_mismatch | None |
| Date{index:Option<usize>,phase:&'static str,cause:DateError} | gilt.date | Some(cause) |
| FirstDividend{issue_date:Date,first_dividend_date:Date,next_quasi_date:Date,second_quasi_date:Option<Date>} | gilt.first_dividend | None |
| ExDateCount{count:usize,expected:usize} | gilt.ex_date_count | None |
| ExDateWindow{dividend_index:usize,cutoff:Date,lower:Date,dividend_date:Date} | gilt.ex_date_window | None |
| InvalidBundleCap{cap:usize,maximum:usize} | gilt.invalid_bundle_cap | None |
| TooManyBundles{count:usize,cap:usize} | gilt.too_many_bundles | None |
| SettlementBeforeIssue{settlement_date:Date,issue_date:Date} | gilt.settlement_before_issue | None |
| SettlementAtOrAfterMaturity{settlement_date:Date,maturity_date:Date} | gilt.settlement_at_or_after_maturity | None |
| InvalidDividendIndex{index:usize,count:usize} | gilt.invalid_dividend_index | None |
| Value{index:Option<usize>,phase:&'static str,cause:ValueError} | gilt.value | Some(cause) |
| Rate{index:Option<usize>,phase:&'static str,cause:RateError} | gilt.rate | Some(cause) |
| Sum{phase:&'static str,cause:SumError} | gilt.sum | Some(cause) |
| AllocationFailed{phase:&'static str,requested:usize} | gilt.allocation_failed | None |
| InternalCapacity{index:Option<usize>,phase:&'static str,operation:&'static str} | gilt.internal_capacity | None |
| ZeroDirtyPrice{measure:&'static str,yield_rate:f64} | gilt.zero_dirty_price | None |
| NonPositiveNominal{nominal:DecimalAmount} | gilt.nonpositive_nominal | None |
| CashArithmetic{stage:&'static str,endpoint:&'static str,cause:AmountError} | gilt.cash_arithmetic | Some(cause) |

`GiltError::kind(&self)->&'static str` is the only new error inspector method;
typed fields remain pattern-matchable. No source string flattens causes.
There are26 domain variants in this ledger. The five below give31 additions
to the baseline139 error census, hence170 for the complete implementation. Root enum
wrappers Error::Gilt(GiltError), Error::GiltYield(GiltYieldError), corresponding
From impls and transparent source/kind/Display are not counted again.

Stable phases: Date `quasi cycle`; CashArithmetic `source dividend` or
`source accrued` with endpoint `point` for exact-source quantized views; allocation `source dividends`, `pricing bundles`,
`present value terms`, `risk present value terms`, `yield present value terms`;
Rate quote `yield quote` with indexNone, factor `discount factor` with absolute
Q index; Value PV `source present value`, quantity `gilt present value`.
Risk getter Value phase/quantity is exactly `price derivative`, `price second
derivative`, `time weighted amount`, `modified duration`, `macaulay duration`,
or `convexity`, with indexNone. Value amount residual uses `yield amount residual`;
clean Sum uses `clean price reduction`, dirty Sum `dirty price reduction`,
and target shift Sum `yield target shift`.

There is no GiltError::Amount wrapper. Both public source views use the
CashArithmetic point policy: instrument dividend_per_100 and a dividend
instruction's source_value_per_100 use `source dividend`; AI instructions use
`source accrued`. The dirty recipe returns Ok(None) without quantization.

InternalCapacity index is absolute Q index for per-bundle operations, otherwise
None. For exact cash/source views, a Some index instead identifies the original
actual-dividend ordinal, retained by the compact cash snapshot; there is no
implied discounted-flow index on cash. Capacity phases/operations are the
complete numerical/cash component tables, including `round source present
value` and `round quasi exponent`, not an unspecified dynamic string. Quote
ValueError parameter is exactly `rate`; its phase remains `yield quote`.
Factor Value/NumericalBoundary quantity is exactly `discount factor`; a
defensive nonfinite scaled log uses `gilt scaled logarithm` in that same
factor phase. PV quantity is exactly `gilt present value` as above.
CashArithmetic endpoint is point/lower/upper as fixed by the cash sections;
stage/order and ExactStageUnresolved labels are its full normative ledger.

## I. Complete GiltYieldError ledger

```text
GiltYieldError::Target { target:BondPrice,phase:&'static str,cause:GiltError }
GiltYieldError::Preparation { phase:&'static str,cause:GiltError }
GiltYieldError::Solver(SolverError)
GiltYieldError::Callback { stage:EvaluationStage,point:f64,
    evaluations:EvaluationCounts,cause:GiltError }
GiltYieldError::NonPositiveTarget { target:BondPrice,dirty_target_per_100:f64,
    accrued_per_100:f64,evaluations:EvaluationCounts }
GiltYieldError::kind(&self) -> &'static str
```

Kinds respectively `gilt_yield.target`, `.preparation`, `.solver`, `.callback`,
`.nonpositive_target`, with the full prefix on every spelling. The first four
source() return their immediate cause; NonPositiveTarget returns None, counts
function0/derivative0. Target phase is `target clean shift`; Preparation phase
is `yield present value terms`. Callback point is the canonical finite
attempted coordinate and contains actual counts including that invocation.
No recursively boxed GiltError or public callback surface is introduced.

Target shift range failure, ZeroDirtyPrice and some raw/normalized range
categories are proof-excluded on valid full-C positive-par plans; retaining a
defensive typed variant does not authorize fabricated invalid-state tests.
There are no NoYieldSensitivity/ConstantPriceMismatch variants for this
strictly positive, positive-time named plan. Generic Bond keeps those cases.

## J. Work, memory, operation isolation and traceability

Q and actual dividends are each globally bounded by20_000. Source construction
is O(Q.len()) fixed-cost checked civil-month arithmetic plus fixed-width source
coefficient work; input vectors are already owned, one fallible derived vector
reservation follows validation. Plan uses O(log Q) period lookup plus O(Q)
bounded entitlement/count/fill passes and one fallible bundle vector. No ICMA,
Business252, maintained-calendar or per-day scan is performed. Hence a generic
accrual-work budget is neither consumed nor silently defaulted. Caller caps
max_dates/max_bundles remain resource arguments, not financial equality fields.

AllocationFailed.requested always counts requested vector element slots,
not bytes or an allocator's resulting capacity. The allocation phase in H
identifies the operation: source-dividend count for instrument construction,
retained bundle count for plan preparation and f64 PV slots for evaluation,
risk or yield. A reservation may physically allocate extra capacity; the
reported request and finite contractual count do not depend on that.

An evaluation/risk traversal is O(bundle_count) and reserves one reusable PV
workspace; yield reserves it once per call, no allocation per callback. All
numeric scalar loops have the numerical sections' fixed-width bounds;
extended sums occupy its fixed stack arrays. Cash instruction/decision/receipt
are compact fixed storage; evaluate allocates no heap. Maximum callbacks are
exactly bounded by u32 SolverOptions, including failed calls. The product of
the global bundle cap and u32::MAX is below2^47; this is an explicit finite
bound, not a speed promise. Immutable concurrent reuse has no global cache,
I/O, clock, dependency, unsafe or mutable rounding environment.

Every failure returns before a partial value/report/receipt escapes. Raw
getter and cash instructions never mutate a plan or one another. Repeated
elementary observations retain the numerical profile; no universal identical-
bits assumption is introduced by a same-host operational regression.

The authoritative `FR-GILT-01..24` rows in the requirements register adopt
these sections together. Each new test cites the relevant IDs. The complete
source/API/cash, exact staging and conditional ideal-comparison obligations
are independent; passing only one ledger is insufficient. Full generic
regression and all broader named-market and acceptance gates remain required.

UK maintained-calendar generation, signed cash ties/aggregation's named-market
interpretation, rigorous ideal-price enclosures, other named markets and
owner/financial/platform/release acceptance remain open. Selecting an explicit
local engineering policy must not be mislabeled source certification.

## N1. Exact model, invariant inputs and indices

The source ideal is the conventional semiannual quasi model. Source coupon C
is a nonnegative exact DecimalAmount per100, including its entire existing
scale0..38 and coefficient domain. Buyer source dividends are exact regular
C/2 or the separately published exact six-decimal short/long first dividend.
At each retained buyer payment date, combine its exact dividend with par100
when that date is redemption. There is one source coefficient A per such
bundle; redemption is never detached. Missing long-first or detached dividend
dates remain in the quasi lattice but do not create artificial buyer bundles.
An entitled actual coupon remains a bundle even when its published amount is
zero. Its factor is still observed. A zero non-redemption source coefficient
has canonical zero PV and exact-zero risk contributions. At maturity a zero
dividend contributes zero while the grouped coefficient still includes par100.
Only missing or detached dividends are absent.

Sections A-J validate the unadjusted semiannual cycle, I<=S<M and
first-period/entitlement roles. It retains absolute source quasi-vector
indices. All numerical per-bundle errors below use that absolute index,
never a newly compressed position. Let the strictly following quasi date have
remaining/full days r,s, with 1<=r<=s<=184. There are fewer than20,000 quasi
dates across the supported dates. For a retained bundle j semiannual steps
beyond that next date:

```text
L = r+j*s                    lambda = L/s
1/184 <= lambda < 20000      L < 20000*184 < 2^22
L+s < 20001*184 < 2^22
q = 1+y/2                   delta = 2+y
P(y) = sum A_j*q^(-lambda_j), y>-2.
```

The pricing coordinate uses exact L,s. There is no intermediate RN(lambda),
RN(r/s)+j, rounded annual time, payment-date substitution, or actual-days/365
reinterpretation. On-cycle settlement has r=s and starts from the following
date. Price/risk/yield reuses these prepared integers and exact source
coefficients without date/calendar queries.

## N2. Quote validation and scaled-log staging

Validate y using the existing periodic2 finite/rate>-2 validator, preserving
its ValueError parameter `rate`; canonicalize either zero to+0. This occurs
before risk/pricing workspace allocation. Yield inversion validates only
queried coordinates in its counted callback, after target/preparation work.

For each actual buyer bundle, observe the discount directly. There is no
intermediate compound-factor admission or reciprocal. This is deliberately a
new named-model observation policy; generic InterestRate/Bond behavior stays
unchanged. Observe these operations in this exact order:

```text
if y == 0:
    discount = 1                      # explicit identity, no libm call
else:
    if abs(y) < 2^-52:
        g = y                         # explicitly qualified tiny identity
    else if y <= -1:
        b = (2+y)/2                   # exact subtraction/division here
        g = 2*ln(b)
    else:
        x = y/2                       # exact power-of-two scaling here
        g = 2*ln_1p(x)
    require finite g
    z = RN_even((-g*L)/(2*s))          # exact rational of represented g
    if z == 0:
        discount = 1                  # explicit rounded-exponent identity
    else:
        apply N3 range-uncertainty guard
        discount = exp(z)
    require finite discount > 0
```

`2*ln(...)` is exact power-of-two scaling of the accepted finite primitive
result. Its ordinary validated result magnitude is below1420. Exact-ratio
exponent multiplication admits directly to IEEE binary64 once; do not first
form g*L, L/s or an extended53 rounded value and then round again. A nonzero
exponent may correctly round to either signed zero; canonicalize that internal
zero and use discount1. It is not a financial underflow error.

The negative branch has b>=2^-53 at next_up(-2). Sterbenz makes 2+y exact in
this branch, and halving is exact. Direct 1+y/2 also preserves the near-floor
distance for these binary64 inputs; the reason for this chosen branch is an
explicit reproducible argument to ln, not a claim that direct subtraction
necessarily loses it. The other non-tiny branch never halves into underflow.

The tiny branch avoids erasing MIN_SUB before an integer-L amplification.
For example, y=MIN_SUB,L=184,s=184 gives exact exponent -MIN_SUB/2, which
rounds to zero; L=368,s=184 gives -MIN_SUB. Either discount rounds to1, but
the internal staging and proof remain specified. In general the scaled-log
ideal g*=2ln(1+y/2) has deterministic remainder

```text
abs(g*-y) <= y^2 / (4*(1-abs(y)/2)).
```

This is a mathematical bound, separate from libm error. Tiny-path exponents
have abs(z)<2^-38, so cannot approach a discount overflow/zero boundary.

A nonfinite observed g is a defensive `GiltError::Rate`, index=absolute quasi
index, phase=`discount factor`, with RateError::Value quantity
`gilt scaled logarithm`. An observed discount uses the same outer phase/index
and inner quantity `discount factor`. Positive infinity is ResultNotFinite;
zero is ResultOutOfDomain with +0 and Interval::greater_than(0). Preserve
other unexpected observed nonfinite/domain values through the same existing
typed validation. No retry, hidden alternate branch or silently clipped value
is introduced.

## N3. Full-quote range classification and conditional empirical profile

The new operation-specific **conditional** primitive comparison hypotheses
apply to finite observed g on non-tiny branches and admitted finite positive
exp observations:

```text
non-tiny: abs(g-g*) <= 4*u*abs(g*)
exp:      abs(exp_observed(z)-exp(z)) <= 4*u*exp(z)+MIN_SUB/2
u=2^-53, MIN_SUB=2^-1074.
```

The first is measured separately for the ln and ln_1p branches. The second
includes IEEE subnormal absolute rounding. Neither follows from Rust's
unspecified transcendental precision nor from the old M4 expression's
profile. Observed zero, infinity, NaN or negative exp results receive the
ordinary typed runtime validation in N2. Zero and overflow observations are
recorded separately against exact-reference IEEE range classification; no
absolute-error claim is made for infinity. Runtime does not return an
analytical error interval or force libm to meet these hypotheses. Section N9
records actual finite native measurements and their limits. Any future
violation is classified before changing a bound. Cross-target compilation
alone supplies no measurement.

For the non-tiny exponent, multiplication by exact lambda introduces no
coordinate representation error. Its single normal RN and the 4u log
hypothesis give

```text
abs(z-z*) <= (5u+4u^2)*abs(z*) < 6u*abs(z),
z*=-lambda*ln(1+y/2).
```

Thus the deliberately conservative guard budget 8u*abs(z) covers this
conditional exponent error. For the tiny branch retain the explicit
remainder plus single-RN radius `u*abs(-lambda*y/2)+MIN_SUB/2`; do not label
the discarded log remainder libm error. No exponent overflow is possible:
lambda<20000 and abs(ln(q))<710 give abs(z)<2^24 over every finite valid quote.

Discount zero-admission and overflow midpoints are exp-domain thresholds
`Z=ln(2^-1075)` and `H=ln(2^1024-2^970)`. Their fixed nearest binary64 images
are, respectively,

```text
Zhat bits c0874910d52d3052
Hhat bits 40862e42fefa39ef
abs(Zhat-Z), abs(Hhat-H) <= 2^-44.
```

The half-MIN_SUB and overflow midpoint are exact mathematical values; they
are not first rounded as binary64 operands. Do not replace zero boundary by
ln(MIN_SUB). The constants above were independently checked at160/240 Decimal
digits during drafting. Future frozen oracle generation must pin/reproduce
their nearest-bit decisions.

For every non-tiny nonzero z, compute an outward guard as follows. `next_up`
and `next_down` are exact adjacent-float bit operations, with explicit zero
handling; no environment rounding-mode change is used.

```text
a = abs(z)*2^-50                        # exact power-of-two scaling
G = next_up(next_up(a+2^-44)+2^-50)       # upward over both additions
for B in [Hhat, Zhat], in that order:
    d = abs(z-B)
    d_lower = 0 if d==0 else max(0,next_down(d))
    if d_lower <= G:
        return RateError::NumericalBoundary {
            quantity: "discount factor", logarithm: z }
```

Outer error is `GiltError::Rate`, index=absolute source index,
phase=`discount factor`. The a term is8u*abs(z), threshold uncertainty is
2^-44, and the final2^-50 is8u, a conservative logarithmic allowance for a
4u relative exp discrepancy. These are distinct contributions. Near the zero
midpoint, solving the finite-value inequality for an observed MIN_SUB bounds
a discrepant finite crossing by ln(1+4u)<-ln(1-4u)<8u in logarithmic coordinates. A
zero result is always rejected separately by runtime validation. Near H the
same relative allowance bounds finite-value crossing, while infinity is
always rejected by ordinary validation. The guard does not assert a uniform
libm classification theorem. Non-tiny z
is normal, with magnitude above2^-62 and below2^24, so a is exact, normal
and finite; both next_up additions remain finite. The threshold floor is
never erased by an underflowed error calculation. d_lower cannot exceed the
exact distance, including rounded subtraction. The comparison may reject a
slightly larger set than the exact real band, intentionally.

The guard is an explicit conditional range policy, not a theorem about libm.
Outside it, call exp exactly once and validate the observed result; far
overflow/zero results remain ordinary typed range failures. Within it, never
guess a finite factor. Tiny z and explicit identities bypass it only by the
range proof above. There is no eager factor check at an unqueried yield.

For financial comparisons define a new profile `TOL-GILT-ELEMENTARY`, rather
than importing TOL-E3 unchanged. For main paths use
`E_z=(5u+4u^2)*abs(z*)`; for tiny paths use the explicit remainder propagated
by lambda/2 plus the single-IEEE-RN radius. A conditional absolute factor
allowance, evaluated outward with scaling/high precision, is

```text
E_d = d_ideal*expm1(E_z) + 4u*d_ideal*exp(E_z) + MIN_SUB/2.
```

This follows from exponential sensitivity plus the separately measured exp
profile. It covers subnormal factors without claiming relative accuracy
there. It is not a settlement enclosure producer. Bounds that overflow or
cannot exclude a range/conditioning boundary are recorded as uninformative;
they do not silently excuse every difficult fixture row.

## N4. Exact source amount times observed discount: one IEEE rounding

Each nonzero source coefficient is retained as a bounded positive rational N_A/D_A,
not first converted to DecimalAmount or binary64. This preserves C/2 at scale39
and large six-decimal publication coefficients. At redemption combine source
dividend and100 exactly before this operation; source distributions retain
their separate dividend/principal metadata.

For a nonzero accepted binary64 discount `m*2^e`, form exact positive integers
`N=N_A*m`, `D=D_A`, retaining e separately. Round `N/D*2^e` directly once to
IEEE nearest/ties-even. For a zero source amount, observe and validate its
factor first, then return canonical+0 PV and exact-zero risk contributions
without decomposing a zero significand. A nonzero source amount rounding to
zero is a typed positive underflow, not zero coupon.

Use this exact integer algorithm for both source PV and quasi-exponent
admission (with the exponent's sign retained):

1. Determine k=floor(log2(N/D)) by bit lengths plus an exact aligned compare.
2. Set K=k+e. K>1023 yields infinity; K<-1075 yields zero. These are
   mathematical IEEE outcomes, not integer-capacity errors.
3. Otherwise set output unit v=max(K-52,-1074). Align numerator or denominator
   by e-v. Divide exactly, retaining quotient q and remainder r.
4. Increment q iff r>D-r, or equality with odd q. This avoids doubling a
   remainder. Handle the53-bit carry and IEEE overflow/subnormal crossover.
5. Encode the IEEE magnitude. Apply sign only after classification; exact
   zero is canonical+0. Internal exponent zeros are allowed, PV nonzero-to-zero
   is rejected, and raw signed risk zeros follow N6.

Do not round to unbounded53 bits and subsequently admit that rounded value
to binary64: this can double-round at subnormal ties. For example exact
`(100.5+5e-39)*MIN_SUB` rounds to101 MIN_SUB directly, while an initial53-bit
extended rounding gives exactly100.5 MIN_SUB and then100 MIN_SUB (even).
The exact coefficient is a valid source regular coupon plus par for
`C=1+1e-38`. This is a valid numeric primitive control; a public-plan case
must additionally establish its actual accepted discount, not forge it.

PV range errors are `GiltError::Value`, index=absolute source index,
phase=`source present value`, quantity=`gilt present value`:
ResultNotFinite PositiveInfinity, or ResultOutOfDomain +0/>0. An unexpected
capacity failure is `InternalCapacity` with this index/phase and operation
`round source present value`. Quasi exponent capacity uses phase
`discount factor`, operation `round quasi exponent`. It is proof-excluded,
not an ordinary financial overflow.

Pricing validates quote, reserves the bounded binary64 PV vector fallibly,
traverses retained source bundles in source order through factor then PV,
and admits one existing exact_sum reduction, phase=`dirty price reduction`.
No bad factor/PV is rescued by omitted clean views or subsequent reduction.
No per-bundle allocation, repeated date query, coefficient clone or hidden
source-price rounding occurs. The same accepted PV observation seeds risk.

## N5. Scalar capacity and absolute-shift proof

Selected scalar storage is five u64 words=320 bits, shared in convention
with the generic kernel without changing its existing instantiation.

For regular C=Ccoef/10^a, the combined coupon/par numerator is
`Ccoef+200*10^a<2^135`, denominator `2*10^a<2^128`. Published first-dividend
micro-unit coefficient is below2^147, and combining par in that basis stays
below2^148; denominator10^6<2^20. The source-publication component separately
proves its exact rounding arithmetic. Thus source PV has N<2^201,D<2^128.
Exponent multiplication has N<2^75,D<2^9 (g significand times L and2s).

An aligned compare at the initial bit-length difference needs at most one
more bit than the larger input. For normal53-bit quotient alignment,
if the numerator is shifted its result is below D*2^53; if the denominator
is shifted it stays below the original numerator. For PV the working bound
is therefore202 bits; exponent admission needs76 bits. For subnormal output,
the clamped unit reduces the positive shift from its normalized53-bit value.
Since input binary e>=-1074, subnormal admission never needs a negatively
shifted denominator. Early K classification prevents huge absolute shifts
for values already proved zero/infinite. All remaining shifts, quotient,
remainder, carry and complement fit320 bits. The final encoded quotient has
at most53 bits; no division by a cast/rounded source coefficient is used.

Risk exact multipliers use L and L+s below2^22, s below2^8, and at most53-bit
PV/delta significands. Their second-derivative numerator is below2^97 and
denominator below2^122; first derivative below2^75/2^61; moment below2^75/2^9.
Normalized53-bit relative alignment needs at most176 bits, including a spare
correction bit. Integer factors in these bounds are explicit; the old generic
five-binary-numerator/two-binary-denominator proof is not substituted for this.

## N6. Analytic risk, extended terms and direct normalization

The ideal source derivatives are with respect to annual decimal y:

```text
P'  = -sum W*lambda/(2+y)
P'' =  sum W*lambda*(lambda+1)/(2+y)^2
M   =  sum W*lambda/2
W=A*q^(-lambda)
Modified=-P'/P; Macaulay=M/P; Convexity=P''/P.
```

Convexity has no half. M is source quasi-coordinate time weighting. Derivatives
refer to this ideal model, not to the discontinuous floating program.

For numerical risk validate quote before its fallible PV workspace reservation.
Traverse every retained bundle once in source order; factor, exact-source PV
and extended-risk work for one bundle precede the next. Set delta_hat=RN(2+y),
positive finite for every finite valid y: 2^-52<=delta_hat<=MAX. Form

```text
P' term = -Vhat*L/(s*delta_hat)
P'' term=  Vhat*L*(L+s)/(s*s*delta_hat*delta_hat)
M term  =  Vhat*L/(2*s).
```

Each exact represented rational is rounded once to53 normalized significant
bits, ties-even, with a separately retained integer exponent. L+s stays exact;
neither lambda nor lambda+1 becomes a rounded float. There is no early IEEE
range admission of risk terms. A53-bit significand carry renormalizes and
increments the exponent. Retain exact sign (P' negative; other terms positive).

Use the existing2^-8192 basis and243-u64-word width for the three retained
signed risk sums. Fixed-sign unsigned accumulation is permitted; the existing
positive/negative-pair convention is also permitted internally. Dirty exact
reduction precedes immutable risk construction and accumulator subtraction.
There is no cancellation-based rescue or partial report.

For nonzero contributions, bounds using1/184>2^-8, lambda<20000<2^15 and
lambda(lambda+1)<20000*20001<2^29 are:

```text
2^-2106 < abs(unrounded P' term) < 2^1091
2^-3130 < unrounded P'' term    < 2^1157
2^-1083 < unrounded M term      < 2^1038.
```

Zero source-coefficient bundles have exact-zero contributions after their
factor/PV checks and do not enter a normalized-significand routine. After53-bit rounding, every
nonzero term is below2^1158 and has no retained bit below
2^-3182. Fewer than20000<2^15 terms give basis coefficients below2^9365.
The243-word width has15552 bits. Direct division by finite binary64 dirty
uses its <=53-bit significand and exponent[-1074,971]; the worst shifted
denominator has10187 bits. Binade compare, numerator alignment, quotient,
remainder and final carry fit the same width. Thus160 words would suffice
mathematically, but this complete contract selects243 consistently with the
established generic kernel to avoid unnecessary width variants.

Raw getters round the retained signed sum*2^-8192 once to IEEE. Normalized
getters divide that retained value directly by observed positive dirty, then
round once; they do not depend on a raw getter's representability. Keep all
six independent Result getters and the existing raw sign/underflow policy.
All getter Value errors have index=None and phase/quantity one of `price
derivative`, `price second derivative`, `time weighted amount`, `modified
duration`, `macaulay duration`, `convexity`. Nonzero-to-infinity preserves
signed NonFinite; nonzero-to-zero preserves signed0 and strict corresponding
Interval. Capacity operation labels reuse `decode binary`, `normalize
rational`, `round scaled term`, `accumulate risk`, `subtract risk sums`,
`round risk value`, `normalize risk value`. No invalid private report is
constructed merely to hit these guards.

## N7. Reachable risk errors in this positive trading model

Let S=sum of accepted nonnegative PVs, including the positive principal
bundle, so dirty=RN(S)>0 whenever construction succeeds. It cannot be negative
or zero. Each extended contribution's relative RN error is at most u, and
S/dirty is between1/(1+u) and1/(1-u); subnormal dirty is exact because S is
a MIN_SUB integer. These algebraic facts do not require accurate libm factors.
They imply, with loose margins including final getter rounding:

```text
2^-10   < Macaulay < 2^14
2^-1033 < Modified < 2^68
0       < exact retained normalized Convexity < 2^134.
```

Therefore normalized Macaulay and Modified cannot overflow or underflow to
zero; convexity cannot overflow. ZeroDirtyPrice is defensive/unreachable in
a successful source-positive plan. Exact zero risk sums are unreachable too.
Do not fabricate signed/zero-dirty models to cover those branches.

Positive normalized convexity underflow is reachable, including when raw P''
remains finite: very large y, a short remaining fraction and large valid C
give a large dirty denominator and a tiny lambda(lambda+1)/(2+y)^2. A public
fixture must exhibit the actual source plan and typed positive0 error. Raw
P' can have negative overflow or negative0 underflow; raw P'' can have positive
overflow or positive0 underflow; raw M can overflow positively. Those require
actual valid source controls, not unrestricted generic kernel states.

Raw M underflow is also excluded **conditional on the new factor profile**,
unlike the algebraic normalized exclusions. If redemption lambda>=1, its
accepted PV is at least100 MIN_SUB and its extended moment at least roughly
50 MIN_SUB. If redemption lambda<1, finite y implies ideal discount>2^-1023;
the admitted conditional factor profile keeps it above2^-1024. Its par100
moment then remains above2^-1028, far from a zero-rounding boundary. Positive
coupon terms cannot lower this sum. This conditional exclusion must not be
promoted to an unconditional Rust libm theorem or tested with forged states.

Exact source AI has magnitude<2^127 over all admitted lifecycle branches;
its cached nearest binary64 view has magnitude<=2^127. Hence a finite target
plus cached AI or successful dirty minus cached AI has exact magnitude at
most MAX+2^127, strictly below the IEEE overflow midpoint MAX+2^970.
Target clean shifts and analytic clean views therefore cannot overflow on
valid inputs. These defensive Sum wrappers and Value amount-residual guards
do not justify constructing impossible financial states to cover a branch.

Plan construction admits cached binary AI with the direct one-IEEE-round
algorithm of N4, preserving its sign and canonicalizing exact zero. With
`A=Ccoef*K/(2*10^a*dayD)`, abs(K)<2^17 and dayD<=184^2, its numerator and
denominator are each below2^144; normal quotient alignment plus a spare
comparison/carry bit needs at most198 bits in the selected five-word scalar.
Every nonzero AI magnitude is at least1/(2*10^38*184^2)>2^-143, so cached AI
cannot underflow to zero or overflow. Rational derivation capacity uses
index=None, phase `source accrued`, operation `derive source`; binary admission
capacity uses the same index/phase with operation `round source accrued`.
Instrument dividend derivation instead uses index=Some(actual dividend ordinal),
phase `source dividend`, operations `derive source` and, for the first Short
or Long publication only, `round publication`. These are proof-excluded
capacity guards, without an intermediate DecimalAmount range operation.

Risk accuracy propagates exact-source PV error and multiplier error separately.
Moment has no multiplier representation error. For delta_hat relative error
<=u, first-derivative multiplier error is bounded by
`abs(g_ideal)*u/(1-u)` and second by
`abs(g_ideal)*((1-u)^-2-1)`; use zero when delta subtraction is exact. Add
extended-term rounding `u*abs(Vhat*g_hat)` before summing gross allowances.
For source PV, `E_V=A*E_d+IEEE_rounding_radius`; there is no coefficient RN
error because A was exact. Add final dirty reduction rounding once. For a
normalized view use the existing conditioning formula only if dirty uncertainty
excludes zero; otherwise record the lack of an informative ideal comparison.
Exact-staging E0 checks and conditional ideal-model checks are separate ledgers.

## N8. Yield callbacks, reports and bounded resources

Units are per100 throughout this named model; there is no arbitrary face
normalization. Resolve the caller's finite clean/dirty BondPrice once. Dirty
target is its value; clean target is one exact_sum of that value and cached
source-AI binary view. That binary AI was already prepared at plan construction
and is reused here; only the optional target shift precedes yield workspace
preparation and callbacks. An unused clean view must not invalidate dirty evaluation.
Keep signed/zero targets governed by Section G’s explicit choice,
without silently inheriting a source no-root certificate from a failed bracket.

Reuse the frozen exact residual and private payload-carrying Brent algorithm:
scale exponent0 for abs(T)<2^970, else1; one exact signed binary difference
followed by output exponent adjustment and one IEEE RN.34 words suffice
for the2099-bit maximum MIN_SUB-basis difference. Residual is finite, sign
preserving, zero iff computed dirty==T. No separately halved operands or
trial-dependent scale is permitted. Report retains target, resolved T, scale,
cached selected dirty and full SolveReport. An amount residual independently
admits the unscaled cached difference, phase=`yield amount residual`; it
makes no callback. Its generic signed range policy remains defensive; in this
named model Dhat>0 and T>0 imply abs(Dhat-T)<=MAX, excluding overflow.

Residual capacity errors have index=None, phase `yield residual` in callbacks
or `yield amount residual` in the independent getter. Both use operation
`residual difference` for exact operand decode/add/subtract and `round residual`
for final IEEE admission. The callback wraps its actual attempted point,
stage and counts normally. These capacity paths are proof-excluded, with no
new dynamic string labels or scalar range sentinel.

Only actual callback invocations consume function count; derivative count0.
Preserve every frozen M6 lower-first trace/count/safeguard/tolerance rule,
cached endpoint/history payload identity, lower-zero one-call success,
no eager unqueried-upper validation and no final repricing. Callback failure
retains point, EvaluationStage, actual counts and immediate typed GiltError.
The API's separate GiltYieldError avoids recursive boxed generic errors.
An n=0 algebraic inverse is an independent oracle, not an uncounted special
success path. Positive source coefficients/par imply ideal strict decrease
on y>-2; this proves conditional uniqueness, not existence, rounded monotonicity,
solver root accuracy or a universal slope lower bound.

No sensitivity-key sorting/vector is needed: the source domain already proves
strict decrease. Reserve one PV vector of retained bundle count before the
first callback; risk reserves its own once. Capacity is fewer than20000 f64s,
below160,000 payload bytes. Reuse it across callbacks without allocation,
date/holiday queries or source coefficient reconstruction. Scalar division
loops have at most320 bit steps and wide loops at most15552, with fixed limb
caps; no precision retry or arbitrary-precision dependency is introduced.
At most options.max_evaluations()<=u32::MAX callbacks means fewer than2^47
bundle observations in the most expensive configured solve.

Numeric storage payloads are explicit: one320-bit scalar40 bytes;34-word
residual272 bytes per magnitude;243-word risk1944 bytes per magnitude;
three retained risk magnitudes5832 bytes; six construction sign accumulators
11664 bytes if pairs are reused. Checked scalar/source operations and getters
must remain nonrecursive. Budget at most24 simultaneous full-risk-width value
slots across a helper call chain (46656 bytes), eight residual magnitudes
(2176 bytes), and16 scalar slots(640 bytes), below50KiB numeric payload before
small metadata. Implementation review must census helper temporaries and
ordinary Copy argument values against this budget or reduce their lifetimes.
This is a bound on explicit logical numeric storage, not a promise about
compiler-generated stack frame size, cloning a report, a caller's stack limit
or an OOM-free allocator. Validate actual supported builds without claiming
an unmeasured machine-stack guarantee.

## N9. Evidence and remaining gates

Required exact E0 evidence includes source publication/rational retention,
single-IEEE PV normal/subnormal ties and carries (including double-rounding
controls), exact-ratio exponent MIN_SUB amplification,53-bit extended terms,
all risk reductions/direct normalization, residual thresholds/signs and cached
solver traces. Cases must be derived from valid numeric inputs or actual
public source plans; proof-excluded financial statuses are listed honestly.

New ideal-price/risk references use independently formulated high precision
from exact C,y,L,s and source publication. Justify two precision budgets over
the full fixture domain, distinguish agreement from analytical certification,
and derive inverse locations from the actual resolved target. Root enclosure
extension to computed zeros needs a uniform conditional price-error bound;
planting a price from an arbitrary seed yield is not an independent root oracle.

Pre-adoption evidence actually observed: native scaled-log profile16943 distinct
quotes,10695 tiny identities and6248 primitive branches, Decimal160/240 RN
reference agreement, worst scaled-log error1.1666666558012366 units of
u*abs(g*) at y bits be4fffffffffffff. Scratch evidence:
`/private/tmp/sabiquant-gilt-log-profile-v1/evidence.json`, source SHA
48b7e53c4208256f42b88fccbdce5713235b92cdf02db654459702df83a297ed,
checker SHA8db986ab1bdc3757b768ecc22e0c06899c4ae7b61795bd05a0e81fc49051e3ae.
This supports the proposed4u profile on that native finite corpus only.
The separately reviewed exp measurement used12459 observed inputs:12390
finite-positive results satisfy the stricter4u*exp(z)+MIN_SUB/2 criterion;
36 observed zeros and33 overflows match separate160/240 reference IEEE
classification. The worst finite-positive fraction of that budget was
0.9999999999999003 at z bits c0874910d52d3051. The source, input and observed
output bytes are unchanged from the prior looser scratch check; this stricter
checker reused them without rerunning the primitive. Evidence
`/private/tmp/sabiquant-gilt-log-profile-v1/exp-evidence-numeric-review-v3.json`
has SHA-256 `5ac343ffecd7da27e16f2405d3e0d068383bca0da9ee52ed4e8298e8e1b38a12`;
checker `b77bd766abef08ec91b39d8e6f57166df93800fab813964c432aeea4e8ba8775`.
The empirical checks do not establish a uniform primitive theorem, foreign
runtime behavior, correctly rounded libm or an ideal-price enclosure.

Independent guard-arithmetic challenge covered10913 nonzero normal exponents,
including4097 neighboring floats about each threshold. Exact Fraction checks
confirmed outward G, lower distances and rejection of all17 certified inner
band cases;26 observations were rejected in total. Decimal160/240 reproduce
both constant bits and their2^-44 error limits. Scratch evidence
`/private/tmp/sabiquant-gilt-guard-review-v1.json`, SHA-256
`888f444b06c8ba787033c4737fc4710cb1ba32fbba6280c511e22cfe9382ef25`,
is a finite arithmetic challenge accompanying the proof, not a production
test or universal primitive claim.

The pre-adoption review resolved the complete source/API/order semantics and
independently reviewed the capacity/rounding proofs and conditional primitive
hypotheses. Author new independent oracles before production. Require
scalar/flow/ideal-reference profiles, unchanged generic M3/M4/M6/M7/bond/
settlement regressions and the complete acceptance ledger before implementation
acceptance. FR-GILT-01..24 retain the complete obligations.
Formal elementary enclosures, ideal-price quantum certificates, maintained
UK calendars, source-negative cash ties/invoice aggregation, other markets,
index-linked/strip models and full M8 financial/owner acceptance remain their
existing explicit gates. Exact rounding of supplied computed binary64 values
must not be called correctly rounded settlement of the ideal source price.

## C1. Scope, input meaning and exact source values

There are four cash operations, all implemented together:

1. Dividend cash from one actual published dividend and strictly positive held
   par nominal. The selected dividend excludes principal/redemption.
2. Accrued cash from the exact source accrued formula at a prepared settlement,
   with strictly positive traded par nominal.
3. An invoice combining caller price evidence for clean price per100 with that
   exact source AI per100, under an explicit aggregation policy.
4. A final trade payment from caller dirty-price evidence per100, with one final
   cash quantizer and no accrued-price shift.

No operation converts an exact source AI into a nearest float, a DecimalAmount
point or a binary enclosure before deciding final cash. Nonterminating source
rationals and values whose public decimal coefficient would be too large stay
exact and private. A quantized public source view is independent: its failure
cannot invalidate an otherwise representable final cash result.

The instrument's annual coupon `C` is the full admitted nonnegative
`DecimalAmount` per100; all coefficient/scale combinations admitted by the
instrument contract are supported. There is no additional six-decimal input
restriction. Regular dividends are exactly `C/2`. Short/long first dividends
alone use the fixed positive nearest/ties-away six-decimal publication stage:

```text
short D = Rplus(C*r1/(2*s1); 1e-6)
long  D = Rplus(C*(s1+r1)/(2*s1); 1e-6)
```

`Rplus` is exact positive half-up: quotient/remainder determine ties and the
micro-quantum count. It has no caller-selectable mode and no public decimal
range admission. Zero coupon gives exact zero publication. The private
published micro coefficient may need147 bits. A regular `C/2` may need scale39
and is not rounded to six decimals as a workaround.

AI uses the unrounded source coupon and exact civil ratios, never the published
rounded first dividend. Under the instrument/ex-date compatibility rules:

```text
regular cum:          A = C*t/(2*s)
regular ex:           A = C*(t-s)/(2*s)
short first cum:      A = C*t_star/(2*s1)
short first ex:       A = C*(t_star-r1)/(2*s1)
long first interval:  A = C*t_star/(2*s1)
long second cum:      A = C*(r1*s2+r2*s1)/(2*s1*s2)
long second ex:       A = C*(r2-s2)/(2*s2)
```

All formulas also divide by `10^C.scale()` when expressed using its coefficient.
The named contract supplies the precise source date/lifecycle classification.
Ex means strictly after the supplied compatible cutoff. At the skipped quasi
date in a long first period, AI does not reset merely because a quasi boundary
was crossed. At an actual payment-date roll, the plan selects the new source
interval. Exact coupon and AI rationals are retained independently of the
pricing kernel's own binary observation views.

Every `SettlementEvidence` supplied to these APIs retains its frozen meaning:
exact canonical decimal; exact finite represented binary64; or the closed set
of all real values between declared exact endpoints. A nondegenerate interval
between adjacent floats is continuous, not a two-point set. Constructors have
already validated evidence and its finite/order rules before a cash factory
is called. Price evidence may be signed or zero; this does not assert that
every supplied quote is an actual conventional gilt market price.

Nominal is exact par, not the binary pricing face or a bond count. `N>0` is
mandatory even when coupon, AI, price or final cash is zero. Quantum and mode
are always explicit via existing `SettlementRounding`, whose actual mode type
is `AmountRounding`. There is no `RoundingMode` alias or new rounding registry.

Positive trade/accrued/invoice cash is payable buyer to seller; negative cash
reverses them. Positive dividend cash is payable issuer to holder. Dividend
cash assumes the caller's held nominal is entitled; it does not generate or
certify record-date ownership. The source permits nonnegative dividend cash;
signed final rounding applies wherever the exact recipe is signed. Negative
nominal does not encode transaction direction.

## C2. Public interface and identity

Section E is the complete cash interface; its selected accessor spelling is
`cash_rounding()`. Section B fixes the traits. Factories capture a compact
owned source snapshot rather than the full instrument or a borrowed plan.
Instruction equality/hash is complete snapshot equality, including the exact
rational source value, source version, EOM policy, original coupon, lifecycle
and relevant dates, evidence kind/bits/bounds, nominal and all cash policies.
Unrelated future cutoffs are not retained. Receipt identity adds the final
amount and ingredient guarantee to this instruction. Neither claims complete
instrument equality merely because its cash inputs or outputs coincide.

## C3. Inspector roles, lifecycle and guarantees

`source_version()` is the immutable named source/model identifier adopted in
the containing gilt contract. It performs no URL fetch, date inference,
legal-calendar verification or current-market certification. The containing
contract uses the same exact identifier on the instrument, plan and snapshot.

| Factory | Recipe ID | Source value per100 | price / aggregation | Optional context |
|---|---|---|---|---|
| dividend_cash | `PublishedDividendPer100` | Selected exact regular or six-decimal published first dividend | None / None | Selected actual dividend index/date; no settlement/ex fields |
| accrued_cash | `SourceAccruedPer100` | Exact source AI | None / None | Prepared settlement/quasi pair, next actual dividend index/date and its ex cutoff/status |
| clean_accrued_cash | `CleanPriceAndSourceAccruedPer100` | Exact source AI | Some(original clean evidence) / Some(caller aggregation) | Same AI context |
| dirty_cash | `DirtyPricePer100` | None | Some(original dirty evidence) / None | Prepared settlement/quasi pair; no selected source dividend/ex fields |

All rows retain the original annual coupon, issue/maturity/first-dividend
dates, first-period classification, EOM policy and issue quasi pair. The issue
pair is the validated `(Q0,Q1)` with `Q0<=I<Q1`; the settlement pair is the
validated containing `(Qk,Qk1)` with `Qk<=S<Qk1`. Settlement at a quasi date
uses the interval beginning there. A selected dividend index is an ordinal
among **actual dividends**, excluding skipped quasi dates and redemption;
it is not the pricing flow's quasi ordinal. The selected dividend on AI rows
is retained even when detached from the buyer. Absence means None, not a
manufactured default date/index or ex status.

`source_value_per_100(q,m)` performs one explicit quantization of the exact
source value, without nominal scaling. It returns Some for dividend/AI rows
and Ok(None) for the dirty recipe. It is a diagnostic view, never an earlier
stage of `evaluate`. It can fail Inexact/OutOfRange independently. Its chosen
q/m are call arguments and are not silently substituted for cash_rounding or
retained as an instruction policy.

For a determined receipt:

* Pure dividend/AI recipes return None from `price_evidence_guarantee`.
  Their exact-source cash provenance is the private validated source snapshot
  and recipe, not an absent or unknown price guarantee.
* Price recipes return Some of the existing `SettlementGuarantee`, scoped
  **only to the price-evidence ingredient**. Declared interval kinds always
  give `UniqueForDeclaredEnclosure`, including equal bounds; otherwise a
  supplied binary64 gives `ForSuppliedBinary64`; otherwise exact decimal gives
  `ForSuppliedDecimals`.

The complete determined cash consequence is exact for the retained source
definition, nominal, price-evidence meaning where present, and explicit recipe
policies. The ingredient label does not say that source AI is a supplied
DecimalAmount, that the supplied price equals the gilt model's ideal price,
or that a declared enclosure actually contains that unknown ideal price.
It establishes no legal ownership/calendar/market claim. Existing
SettlementReceipt/Instruction constructors or private fields are never forged
to attach these new source-rational claims to an old receipt.

## C4. Four exact recipes and universal Exact semantics

Write `R(x)` for the existing seven-mode signed quantum function at the
instruction's cash rounding, retaining the wide result. `N` is par nominal;
`D` is the selected exact source dividend per100; `A` exact source AI per100;
`C_p` and `P` are each admissible real value of the original clean/dirty price
evidence respectively.

```text
PublishedDividendPer100: cash = R(N*D/100)
SourceAccruedPer100:     cash = R(N*A/100)
DirtyPricePer100:        cash = R(N*P/100)

CleanPriceAndSourceAccruedPer100:
    capital = N*C_p/100
    accrued = N*A/100
    RoundTotal:              cash = R(capital+accrued)
    RoundComponentsThenSum:  cash = exact_sum(R(capital),R(accrued))
```

There is no intermediate source-price/source-AI quantization in an invoice,
and no extra final quantizer after separately rounded components. Pure AI and
dividend source values remain exact points, even when their shortest exact
decimal expansion does not exist or cannot fit DecimalAmount. Do not create
a surrounding binary interval and then return unresolved when the rational
itself decides the quantum.

Every non-Exact mode is monotone. The source is fixed, and nominal scaling
is strictly positive. Thus one price-evidence point gives a point stage
image; a nondegenerate declared interval gives continuous variation through
each unrounded required price stage. A cash quantizer can collapse that
variation to one output, but this never changes original evidence provenance.

At a required Exact quantizer:

1. A point image uses exact rational divisibility. A nonmultiple returns
   CashArithmetic with AmountError::Inexact{operation:"quantize"} at point.
2. A nonpoint image containing continuous variation returns
   GiltCashDecision::ExactStageUnresolved at this required stage, even when
   both endpoints are multiples. Universal Exact-definedness fails on interior
   real values. This is not a midpoint calculation or a point Inexact error.

There is no uncertain source quantizer in this recipe family and hence no
discrete source-count image requiring a later base/step test. In component
aggregation, rounded capital counts may vary, but there is no later quantizer:
the terminal sum uses exact endpoint images. Do not import the old staged
recipe's source quanta or silently enumerate only binary64 interior points.

## C5. Complete construction/evaluation order

Construction:

1. For dividend_cash, validate actual dividend index first. Invalid index
   precedes nominal, matching the instrument's dividend-view selector. Then
   require N>0. No source arithmetic or allocation is needed for these checks.
2. The other three factories require N>0 first; plan/source/evidence/rounding
   arguments are already valid immutable values. Argument constructors' own
   evaluation order is not changed by this rule.
3. Capture exact source and public provenance into a compact owned snapshot.
   There is no source DecimalAmount/f64 range admission, heap reservation or
   new accrual query. Source invariant/capacity failures remain typed.

For an existing valid instruction, evaluate only the required stages:

1. Pure dividend uses `cash coupon`; pure AI uses `cash accrued`; dirty uses
   `cash total`; a total invoice combines exact unrounded ingredients then
   uses `cash total`.
2. For a component invoice, completely execute `cash capital`, including its
   universal Exact test/endpoint quantization. Then execute point `cash accrued`.
   Retain both rounded results wide, even outside DecimalAmount's range.
3. Component aggregation exact-sums those results at stage `cash net`; there
   is no extra quantum operation. A later net cancellation cannot waive an
   earlier component Exact failure.
4. Normalize/admit only the returned final lower amount, then upper if the
   reported final stage input image was nonpoint. Single/total/coupon stage
   pointness is measured before its cash quantizer. For cash net use the exact
   net image after component quantization. If a price range collapses at the
   capital quantizer, a subsequent net image can be a point.
5. Equal admitted final extrema return Determined with a receipt. Unequal
   extrema return QuantumAmbiguous with strictly ordered amounts.

The stable numeric endpoint labels are `point`, `lower`, `upper`, determined
by the reported stage's input image, not by whether the constructor was a
declared-interval kind. Equal declared bounds are point images but preserve
declared provenance. A nonpoint total/coupon input whose cash outputs collapse
to one unrepresentable value still fails at lower, not point. At cash net a
collapsed net image fails at point.

Earlier Exact/error outcomes precede later stages and all final range tests.
Do not prevalidate accrued Exact or final cash ranges before the required
capital stage. Intermediate source/cash magnitudes and rounded component
coefficients need not fit public DecimalAmount. Final cancellation may rescue
range, but cannot rescue an earlier required Exact domain. Do not clip an
evidence interval or return a numerical suggestion for unresolved/ambiguous
cash. Distinct representable extrema do not certify representability of every
interior amount across i128 precision holes.

## C6. Errors, sources and stable strings

Use the common GiltError, adding/counting these domain variants once:

| Variant / kind | Fields / source |
|---|---|
| InvalidDividendIndex / `gilt.invalid_dividend_index` | index:usize, count:usize; None |
| NonPositiveNominal / `gilt.nonpositive_nominal` | nominal:DecimalAmount; None |
| CashArithmetic / `gilt.cash_arithmetic` | stage:&'static str, endpoint:&'static str, cause:AmountError; Some(cause) |

The shared InternalCapacity variant has fields
`index:Option<usize>,phase:&'static str,operation:&'static str` and kind
`gilt.internal_capacity`; it is not duplicated by a cash-only error type.
All variants are contextual/non-exhaustive under the common error contract.
Root Error::Gilt exposes exact GiltError, followed by AmountError for a
CashArithmetic cause. Existing AmountError and SettlementError catalogs are
unchanged. There is no new GiltCash root wrapper or recursive error allocation.

Stable cash stage strings are `source dividend`, `source accrued`,
`cash coupon`, `cash accrued`, `cash capital`, `cash total`, `cash net`.
The first two label explicitly requested quantized source views and source
construction; they are not invoice quantization stages.

CashArithmetic causes are only AmountError::Inexact and
AmountError::OutOfRange. The former always uses operation `quantize`.
Final admission after single/total/coupon/AI quantization and public source
views uses operation `quantize`; only final cash-net admission uses operation
`sum`. Preserve the final sign in OutOfRange. Quantum scales are already<=38,
so ResultScale is not a reachable quantized cash/source-view cause.

Capacity phase is the required stage; operations are `derive source`,
`round publication`, `decode evidence`, `scale cash`, `combine cash`,
`cash rounding`, `exact domain`, `normalize cash` as applicable. For selected
dividend source/view/coupon work, index is Some(actual dividend ordinal).
Other cash work uses None. CashArithmetic carries its own stage/endpoint,
not a manufactured per-price-flow index. InternalCapacity is proof-excluded
for valid finite inputs; no forced private-state test is requested.

## C7. Complete input-space and width proof

The actual half-year interval validation supplies `1<=s1,s2<=184`, hence
`dayD=s1*s2<2^16`. Source AI can be written exactly as

```text
A = Ccoef*K / (2*10^Cscale*dayD), abs(K)<2^17
```

with optional cancellation of unused period factors; `abs(A)<2^127`.
Regular source dividends are Ccoef/(2*10^Cscale); published first dividends
have exact six-decimal micro coefficient `<2^147`. GCD reduction is optional,
not a substitute for these unreduced bounds. Nominal coefficient `n` and
quantum coefficient `Q` are positive and `<2^127`; each scale is0..38.

### Pure exact-source cash and source views: eight limbs

Combine all decimal powers, including the per100 divisor, before constructing
the quantization ratio. For source AI and the raw regular dividend:

```text
e = qscale - Nscale - Cscale - 2
num = n*abs(Ccoef)*abs(K)*10^max(e,0)
den = 2*dayD*Q*10^max(-e,0)
```

`max(e,0)<=36`, so num `<2^391`; `max(-e,0)<=78`, so den `<2^404`.
After any rounding mode, the count times Q is at most num+Q `<2^392` because
den>=Q. Retain a carry for comparisons; every scalar is below405 bits and
eight u64 limbs provide512. Pure source views omit nominal and per100 factors
and are smaller. Zero K uses an exact zero shortcut only after required public
selector/nominal checks.

For a six-decimal published micro coefficient m `<2^147`, combine
`e=qscale-Nscale-8`; num `<2^374`, den `<2^280` and rounded count times Q
`<2^375`. Publication itself rounds Ccoef times a bounded day numerator with
a decimal exponent difference at scale6; it is smaller than these bounds.
All source-cash evaluation accepts the full coupon/nominal/quantum scale domain
without requiring an intermediate Amount conversion.

### Price-evidence invoices and dirty trades: forty limbs

Do not route a binary64/interval invoice through the eight-limb source path.
Set `R0=2^1074*5^38<2^1163`. Every finite binary64 and every decimal with
scale<=38 is an integer multiple of1/R0. For an invoice choose
`Dc=R0*dayD<2^1179`; for a dirty-only recipe dayD=1. This denominator also
represents exact source AI because `2*10^Cscale*dayD` divides Dc. There is no
extra powers-of-five multiplier after constructing this common basis.

Each finite price endpoint has magnitude `<2^1024`, so its signed common
coefficient is `<2^2203`. Source AI's coefficient is `<2^1306`.
Their signed sum, including a carry before cancellation, is `<2^2204`.
Multiplication by nominal coefficient `<2^127` and reciprocal-quantum scale
`10^qscale<2^127` gives quantization numerator `<2^2458`. Its denominator,
including `Dc`, `10^Nscale`,100 and Q, is `<2^1440`.

Since den>=Q, rounded count times Q is at most num+Q `<2^2459`. Under component
aggregation, store each rounded signed coefficient directly at the **same
final quantum scale**, then exact-sum them; the net coefficient is `<2^2460`.
Do not reconstruct those coefficients on Dc using a gratuitous extra multiply.
Normalize final coefficient at qscale<=38 and admit only its canonical signed
i128 result. Forty u64 limbs provide2560 bits, covering every path with at
least100 bits of headroom. Negative i128::MIN output magnitude2^127 is handled
by unsigned magnitude/sign, without signed abs or negation overflow.

The 40-limb proof covers all four price-evidence kinds, signed endpoints,
full finite binary64 range, all nominal/quantum scales, total/component
aggregation, exact quotient/remainder ties and wide net cancellation. It is
not an economic price bound or a claim that every resulting cash fits i128.
Ordinary final range failures remain CashArithmetic, not capacity failures.

Cash evaluation has one source point and at most two price endpoints, two
mandatory component quantizers and one terminal signed sum. Universal Exact
continuous-image checks are constant structural tests; no interval enumeration
or precision-refinement loop is necessary. Loops are fixed-width/scale bounded.
Factories, arithmetic, inspectors and immutable reuse allocate no heap and
perform no I/O, calendar lookup, callback, clock/environment access, unsafe
code or global/interior mutation. The instrument/plan's separately documented
construction allocation does not become a cash-time allocation guarantee.

## C8. Required complete evidence before implementation acceptance

Use new independent integer/Fraction oracles, authored from the final adopted
contract before production. Exact cash and all decisions use TOL-E0. Include:

* All39 scales, C zero/full range, nominal positive minimum/maximum, arbitrary
  quanta, all seven signed modes, algebraic ties/parity, each side of both
  final signed limits, regular scale39 and147-bit first-publication values.
* Long first dividend639/182 published3.510989; exact ex-AI -1/61; its
  nondecimal rational cash must not become a binary enclosure ambiguity.
  Demonstrate a quantum boundary resolved differently by exact AI and its
  rounded nearest observation, without selecting expectations from production.
* Fixed positive half-up publication ties, source/nominal scaling rescue,
  zero published coupon versus missing quasi cash flow, all AI branches and
  strict-after cutoff equality. Published dividend and raw AI remain distinct.
* Direct dirty, pure dividend, pure AI, total invoice and component invoice;
  final cash, every inspector and exact source-view quantization/error, including
  a failing source view with a successful final cash receipt.
* Exact decimal versus supplied binary price differences, both binary zero
  signs, subnormal/MAX endpoints, declared point provenance, continuous
  adjacent-float intervals, threshold ownership, unique/ambiguous collapse,
  point/lower/upper error labels and precision-hole qualifications.
* Exact point Inexact versus continuous Exact-unresolved; capital-before-AI,
  no net rescue of required Exact; component wide range cancellation with
  final admission only; invalid dividend index before nominal and other
  factory nominal-first rules, all typed/root causes and stable strings.
* Independent valid eight-/40-limb carry/division/normalization extremes and
  algebraic width review, without impossible private states or OOM stress.
  Proven capacity failures are documented separately from ordinary range.
* Receipt/source snapshot Eq/Hash distinctions, canonical zeros and exact
  original evidence; Copy/Send/Sync/repeated/threaded reuse, zero new date work,
  no per-evaluation allocation and unchanged old fixtures/contracts/results.

The broader source, numerical price/risk/inversion and lifetime requirements
remain part of the same adopted tranche. Passing this cash component alone
does not accept a partial implementation. No current UK calendar, auction/record-date
ownership, signed legal tie/aggregation convention, ideal-price enclosure,
financial/owner/platform/release or full M8 acceptance is implied.
