# Complete Canadian fixed-coupon local design proposal

Review proposal recorded 2026-10-23 Bangkok. This is not adopted, production, an oracle,
financial/owner acceptance or a current-market certification. It preserves the complete
fixed-coupon objective. Root promoted the immutable private v3 proposal (SHA256
`8da905ca5178844005f67a4288744678f98e85e433bb34aa8f84593923a335e9`) with editorial
formatting and repository links; the private original remains unchanged. Read the
[source review](M8_CANADIAN_SOURCE_REVIEW.md), [research
validation](M8_CANADIAN_RESEARCH_VALIDATION.md), [numeric/test
companion](M8_CANADIAN_NUMERIC_AND_TEST_DESIGN.md) and [independent first-pass numerical
review](M8_CANADIAN_NUMERICAL_DESIGN_REVIEW.md) together. Later [API closure](M8_CANADIAN_API_REVIEW.md),
[conditional compound profile](M8_CANADIAN_COMPOUND_PROFILE_REVIEW.md) and
[exact MM range proof](M8_CANADIAN_MM_RANGE_REVIEW.md) are separate reviewed inputs.
The [bounded compound guard](M8_CANADIAN_COMPOUND_GUARD_DESIGN.md) and
[numerical resource review](M8_CANADIAN_NUMERIC_RESOURCE_REVIEW.md) close the
threshold mechanism and numeric capacity route. The
[complete cash-resource proof](M8_CANADIAN_CASH_RESOURCE_REVIEW.md),
[source/cash/API integration](M8_CANADIAN_CONTRACT_INTEGRATION_REVIEW.md) and
[independent resource/protocol challenge](M8_CANADIAN_RESOURCE_INTEGRATION_REVIEW.md)
now supply the complementary whole-model reviews. Complete contract integration/adoption,
then independent preproduction freeze, remain required before production. These research results alone do not adopt
an implementation contract.

## 1. Proposed complete scope and source status

Recommend the following whole local model, version identifier
`CA-IIAC13-fixed-local-proposal-v1`. The identifier means this explicitly reconciled
historical source interpretation, not that IIAC endorsed these decisions. Adopt the complete local contract only after
complementary source/model/API/cash/numeric/resource review and integration. Then use
that immutable adopted contract to author, independently review and freeze all required
preproduction evidence. No Canadian production may begin until that evidence gate
passes. This ordering avoids making contract adoption depend on evidence whose
immutable contract input does not yet exist.

The intended immutable named instrument is `CanadianBond`, fixed nonnegative exact
annual coupon C per100 par, bullet principal100, original accrual date D, original issue
I, stated maturity M and separately supplied true terminal payment T>=M. Annual yield y
is decimal, e.g.0.05 denotes5%; C=5 denotes5 per100 peryear. Exact positive par nominal
N for cash is separate from C and from analytic prices. There is no binary face
parameter.

Include all of:

- Semiannual, quarterly and monthly fixed coupons, with compound frequency equal to payment frequency. No monthly omission or silent semiannual default.
- Regular periods; short/long first and last periods; a single actual coupon whose start/end are both off the nominal cycle; arbitrarily many complete virtual periods inside an odd end period up to the public date cap. Long last and the fully composite single-period construction are explicitly local extensions of the source's developed regular/short/long-first/short-last cases.
- Explicit compound, explicit money-market-equivalent (MM), and an explicit caller-selected strategy that uses MM only when one actual coupon/principal bundle remains. No hidden yield convention.
- Source valuation coupon amounts versus actual payable coupon amounts; valuation accrued A versus settlement accrued AI; all six independent analytical risk views; bounded yield inversion; four exact cash recipes.
- Full existing DecimalAmount input domain C>=0, scale0..38, including values whose C/f needs scale39/40 or a nonterminating decimal. Signed/zero caller price evidence is admitted in cash and in typed targets, subject to positive model-dirty target for inversion.

Separate instrument families remain outside this fixed-principal model:
amortizers/sinking funds, floating rates, inflation indexing, optional calls/puts and
foreign currency conversion. This is a substantive model boundary, not omission of
fixed-coupon frequencies or odd endpoints. An explicit calendar/auction/record-date
system is not manufactured from the PDFs.

The retained source ledger and original byte hashes are
`/private/tmp/sabiquant-canadian-source-readiness-final-v1/{review.md,evidence.json}`;
the independent complete source/cash challenge is
`/private/tmp/sabiquant-canadian-sourcecash-challenge-v4/review.md`
(SHA71b9fef82fc1bb61613e4b47797eda8b77a92232ed8c37a517be55e03eb43cfa). Retained
IIAC1.1/1.2/1.3 and BoC bytes were fetched2026-10-21; the greatest retained version is
not asserted latest/current. No QuantLib C++ or other pricing implementation was used.

## 2. Explicit source reconciliation policy

Every row below is a PROPOSED local decision. These are intentionally versioned together
as one `CanadianSourcePolicy::CanonicalIiac13LocalV1`, not an undocumented selection of
whichever formula is convenient. A later different policy must get a different enum
variant/version and retain its identity even on equal outputs.

| Issue | Source status | Recommended local decision and discriminator |
|---|---|---|
| CPN units | IIAC annual decimal multiplied100; BoC writes annual coupon per100 | Public C per100, y decimal; normalize formulas algebraically; C5,y.05 controls |
| Valuation vs payable coupons | IIAC actual/actual valuation differs from odd payable Canadian365 | Cache separate exact definitions; never price the compound model with payable odd coupons |
| Valuation A vs settlement AI | Repeated distinction in general sections | Retain both; compound clean subtracts A, MM clean subtracts AI; cash always uses AI |
| Monthly settlement AI | General §§5/6 prescribe Actual/Actual; referencep42 instead prints C*d/365 | Prefer repeated general monthly exception: AI=A for f12; mark departure fromp42; C12,d15,s31 distinguishes15/31 vs36/73 |
| Completed quasi pieces | Full payable periods explicitly C/f; literal partial-AI at181/182 days does not necessarily equal C/f | A completed full virtual period is exactly C/f in coupon and AI sums; use partial formula only on proper fragments |
| Interior late Canadian365 branch | Literal historical expression decreases for s184,d182→183 | Preserve the literal proper-fragment branch; do not cap, smooth or replace by Actual365Fixed; record -1/146 for C5 |
| Long-first DIC1 | p33 definition conflicts with already-added full coupon; general quasi construction andp35 differ | Use direct interval/quasi intersections, not the p33 full-first-payment interpretation; missing payment never becomes an actual coupon |
| MM generalized sum | p49 sums through NC and separately adds terminal coupon | Sum each actual coupon once, terminal coupon and principal once; NC1,y0,C12,f12 gives101, not102 |
| Worked numerical errors | p16 5*48/365 answer; p17 extra full coupon; Appendix83.298 | Preserve originals as explicit reference divergences; follow declared formulas/reconciled interval model, not mistaken worked cells |
| Bond with MM quote vs MM security | Source rounding paragraphs classify instruments, not every yield label | This remains a bond under either quote model; no3dp price prequantization merely because MM yield is selected |
| Rounding and aggregation | Appendix examples support full-price principal and component rounding, but leave ties/other quanta unresolved | Expose all7 existing signed modes and both aggregations; no default monetary mode/quantum; source-like q.01 component cases remain explicit caller choices |
| True payment maturity | BoC scoped operation and IIAC MM note distinguish actual terminal payment | Caller supplies T>=M; only MM horizons use T. Coupon geometry and compound exponents remain contractual/unadjusted |

Primary page locators (zero-based PDF indices):1.3 scope4; quasi/coupon11–17; A18–19;
AI20–21; exponents22–23; ordinary/odd compound29–41; contradictory monthly42; MM47–49;
rounding54 (printed53). These are source locators, not new market certification.
[IIAC1.3](https://iiac.ca/wp-content/uploads/Canadian-Conventions-in-FI-Markets-Release-1.3.pdf),
[IIAC1.1](https://iiac.ca/wp-content/uploads/Canadian-Conventions-in-Fixed-Income-Markets-1.1.pdf),
[IIAC1.2](https://iiac.ca/wp-content/uploads/2013/05/Canadian-Conventions-in-FI-Markets-Release-1.2-Feb-2015.pdf),
[scoped BoC note](https://www.bankofcanada.ca/cal_clean_price/).

## 3. Dates, explicit schedule and full endpoint classification

Recommend a separate immutable owned `CanadianCouponSchedule`, so callers can review the
complete contractual cycle once. It owns supplied Q and actual coupon dates; neither is
silently generated or recursively rolled. Its constructor accepts
D,M,f,Q,actual_dates,EOM,max_dates. Bond construction then adds I,C,T,source policy
without a giant collection of unrelated floating inputs.

- Q has2..20,000 dates, strictly increasing. Its last supplied date is the explicit direct cycle anchor. For stride m=12/f months, every Q[i] must equal anchor minus m*(len-1-i) months using the supplied EndOfMonth policy. Preserve means calendar month-end iff the anchor is month-end; otherwise preserve/clamp its day. Ignore clamps the anchor day. Date failures are typed, never a retry with another cycle.
- Minimal outer coverage: Q0<=D<Q1 and Q[len-2]<M<=Qlast. D<M. Thus an off-cycle stated maturity retains the forward complete quasi endpoint beyond M, required to value a short final period. No redundant extra outer periods are accepted.
- Actual coupon dates are a nonempty strictly increasing owned vector, first>D and last=M. Every nonterminal date is a member of Q. Interior actual periods (those that are neither first nor last) join adjacent Q dates. Only first/last periods may skip full virtual periods. Zero-valued actual coupons remain dates.
- The first interval begins at D, other intervals at the preceding actual coupon. A complete adjacent-Q interval is Regular. A one-coupon interval with both endpoints off Q is Composite. All other nonregular intervals are Short or Long according to their positive exact quasi length being less/greater than1. First and last inspectors refer to the same interval for a single coupon. No duplicate coupon or principal is created for that case.
- I must satisfy D<=I<F, where F is first actual coupon, so a reopening/original issue after a skipped virtual boundary can still be admitted. This is a labelled local dated-accrual interpretation, not an announcement/auction eligibility claim.
- T>=M; all nonterminal payment-date inspectors return their contractual coupon date, and the terminal payment-date inspector returns T. No calendar is verified, no delay interest added and no holiday/settlement lag inferred. Supplied T is definition evidence, not a legal true-date certification.
- Prepared trading settlement is explicitly I<=S<M even if T>M. Post-stated-maturity debt transfers, earlier rolled coupon ownership and ex-coupon/record-date entitlements require a separate contract; they are not guessed here. This local trading-domain boundary must be visible in docs/errors. Within it, coupon dates<=S are Ceased and strictly future dates BuyerEntitled; ExCouponDetached is never returned. Cash coupon distribution assumes the caller's held nominal is entitled.

Define exact cycle coordinate chi(Q_i)=i, and inside one period
chi(t)=i+days(Q_i,t)/days(Q_i,Q_i+1). Its difference across any interval is full virtual
periods plus at most two proper fragments. Avoid repeated addition of floating times.
When both endpoints lie in one Q period use a single fragment, not two overlapping ones.

This construction handles source short/long first and short final; source
first+short-last combinations; a long first and long last around regular interiors; an
entirely off-cycle single interval; and maximum-length virtual decompositions. The last
two/long-last generalizations remain labelled local rather than represented as explicit
IIAC worked cases.

## 4. Exact source amounts and caches

Let c=C/f. For an actual accrual interval [a,b), split its intersection with Q into full
periods and proper fragments of integer length d in full quasi length s. f∈{2,4,12};
respectively s∈181..184,89..92,28..31. The prospective primitive bounds may use the
conservative union28..184; no claim that every cross-frequency combination is reachable.

Exact valuation coupon:

`V(a,b) = (C/f) * (full_count + sum(d/s))`.

Exact payable coupon uses each full period C/f and, for each proper fragment, the
reconciled Canadian365 function:

`CB(d,s;C,f) = C*d/365` if f*d<365;

`CB(d,s;C,f) = C*(1/f-(s-d)/365)` if f*d>365.

Always handle d=s as a full period first. d=0 contributes exact0. Equality f*d=365 is
impossible for integer d and f2/4/12; do not extend that statement to arbitrary f. A
doubly partial single interval applies CB once to its actual fragment if both endpoints
lie within the same quasi; otherwise sum its two proper fragments and full pieces. That
last/composite generalization is explicit local policy.

At S choose the latest actual coupon<=S, or D before F. Valuation A is
V(accrual_start,S). At an actual coupon date the start becomes that date and A=AI=0. At
a skipped virtual boundary in a long period the earlier accrued pieces persist. For
f2/4, settlement AI is the reconciled sum of C/f for completed pieces and CB for proper
fragments; for f12 it is A under the selected monthly policy. Coupon-cash uses payable
coupon regardless of frequency; monthly odd payable coupons therefore need not equal
valuation coupons even though monthly settlement AI=A.

There is no daily9, accrued5, payable6dp or other publication stage here. The historical
source's displays are not promoted into an undisclosed universal quantizer. Retain
separate exact valuation/payable first/last caches and the shared regular C/f cache. C/f
itself may not fit DecimalAmount. Public q/m source views quantize only on request and
cannot gate pricing, inversion or otherwise representable cash.

Source ratio bound: with C magnitude<2^127, k<20,000 and at most two s<=184, a common
valuation form has numerator<2^160 and denominator<2^148 (including decimal scale<=38
and f). Canadian365 forms are smaller; adding par100 still stays within the160-bit
numerator allowance. Five u64 words=320 bits suffice for stored positive source
rationals in these declared forms. Canonical zero is0/1. This is a magnitude bound;
helper call liveness must also be audited before adoption/implementation.

## 5. Two yield models, three price bases and all six risks

Introduce explicit `CanadianYieldConvention::{Compound,MoneyMarket}` and caller
selection `CanadianYieldSelection::{Explicit(convention),LastCouponMoneyMarket}`. The
strategy resolves once during plan preparation to MoneyMarket iff exactly one actual
coupon/principal bundle remains, otherwise Compound. Explicit MM is permitted for all
prepared tenors as a labelled local mathematical extension of the short-term source
presentation. An MM-labelled bond is still the fixed-coupon bond, not a Treasury
bill/security-rounding preset.

### Compound

For each retained actual coupon i, lambda_i=chi(contractual_coupon_i)-chi(S)>0. The
coefficient B_i is its exact VALUATION coupon, plus exact100 at the terminal coupon.
With q=1+y/f>0:

`D_C(y)=sum B_i*q^(-lambda_i)`, `y>-f`.

`P_C(y)=D_C(y)-A`.

This is the fractional-power model. It is not Treasury's rational first-period factor.
Do not round lambda, annual time, first coupons, or par-plus-coupon before factor/PV
observation.

For W_i=B_i*q^-lambda, ideal derivatives/time moment are:

`P'_C=-sum W_i*lambda_i/(f+y)`;

`P''_C=sum W_i*lambda_i*(lambda_i+1)/(f+y)^2`;

`M_C=sum W_i*lambda_i/f`.

### Money market

Set tau=(T-S)/365. For a nonterminal coupon use its contractual payment date t_i;
terminal uses T. Let rho_i=(T-t_i)/365 and h_i=(t_i-S)/365=tau-rho_i, all h_i>0.
Coefficient B_i is its exact PAYABLE coupon, plus100 once terminal. On exact
`1+y*tau>0`:

`D_MM(y)=sum B_i*(1+y*rho_i)/(1+y*tau)`;

`P_MM(y)=D_MM(y)-AI`.

Do not admit `y*tau` or either large affine numerator separately as f64; exact rational
affine arithmetic can produce a finite quotient when such an intermediate overflows.
Terminal rho=0 is included only once. This directly reduces to the one-flow source and
avoids p49's double terminal.

For ideal K0=sum B_i, K1=sum B_i*rho_i, define H=sum B_i*h_i>0 directly as a positive
sum, not by cancellation `tau*K0-K1`:

`P'_MM=-H/(1+y*tau)^2`;

`P''_MM=2*tau*H/(1+y*tau)^3`;

`M_MM=sum [B_i*(1+y*rho_i)/(1+y*tau)]*h_i`.

That M is an explicitly local actual-year cashflow moment for this yield convention. Do
not derive it from periodic duration identities. True T changes MM horizons, not
original coupon accrual/payable geometry.

### Six views and price bases

Both conventions expose independently fallible raw price derivative, second derivative,
time-weighted amount, modified duration `-P'/D`, Macaulay duration `M/D`, and convexity
`P''/D` with no1/2. The denominator is positive model-dirty D, not the potentially
signed all-in settlement price. Derivatives are of the chosen ideal source formula, not
of a discontinuous floating program.

A Canadian `Dirty` value must not silently change meaning between pricing and
settlement. Propose dedicated `CanadianPriceBasis::{Clean,ModelDirty,SettlementDirty}`
and finite immutable `CanadianPrice`; no implicit BondPrice conversion:

| Basis | Compound ideal | MM ideal |
|---|---|---|
| ModelDirty | D_C | D_MM |
| Clean | D_C-A | D_MM-AI |
| SettlementDirty | D_C-A+AI | D_MM |

Model dirty is positive on admitted source states. Clean and settlement dirty may be
zero or negative and remain valid signed quote values. A valid negative compound all-in
example is independently proved in
`/private/tmp/sabiquant-canadian-proposal-cash-review-v1/negative-settlement-dirty-proof-v2.json`
(SHA1df460efdb0fad4afa2f30aaa8fc19c8cfb684700f102fb9395c6ce0c417b34d): C5,
D=I2025-09-01, coupons2026-03-01/2026-09-01, S2025-12-01, y2^500. Both PVs are normal,
yet AI-A<-1/128 overwhelms positive D.

Recommended report observations retain independently rounded exact A_hat and AI_hat and
choose quote_accrued=A_hat for compound, AI_hat for MM. ModelDirty is cached D_hat.
Clean is ONE exact_sum `[D_hat,-quote_accrued]`; SettlementDirty is ONE exact_sum
`[D_hat,-quote_accrued,AI_hat]`, not a clean-then-add sequence. MM returns D_hat
directly for its settlement basis. These are binary64 observations, not exact-money
ingredients or ideal enclosures. A bridge failure does not invalidate model dirty or a
risk view.

Inversion target is a `CanadianPrice` with all three bases supported. Resolve T_dirty
once before allocation/trials: ModelDirty uses its value; Clean is one exact_sum
`[value,quote_accrued]`; SettlementDirty is one exact_sum
`[value,-AI_hat,quote_accrued]`. Retain original target and the exact chosen observed
shift. Reject nonpositive resolved T_dirty at zero evaluations. There is no price-basis
or report/invoice round-trip theorem across separate rounding stages.

## 6. Proposed public type and signature census

Prospective addition:18 public types, not a changed implemented census:

`CanadianBond`, `CanadianCouponSchedule`, `CanadianFrequency`, `CanadianPeriodKind`,
`CanadianSourcePolicy`, `CanadianYieldConvention`, `CanadianYieldSelection`,
`CanadianPriceBasis`, `CanadianPrice`, `CanadianPricingPlan<'a>`, `CanadianValuation`,
`CanadianRisk`, `CanadianYieldReport`, `CanadianError`, `CanadianYieldError`,
`CanadianCashInstruction`, `CanadianCashDecision`, `CanadianCashReceipt`.

All have Debug/Clone/PartialEq/Send/Sync. Bond/Schedule are owned, Eq+Hash, not Copy.
The6 metadata enums plus CanadianPrice are Copy+Eq+Hash; Price equality/hash uses
canonical finite bits and basis. Plan borrows Bond and owns bounded bundles, not
Copy/Eq/Hash. Valuation/YieldReport are owned Copy but not Eq/Hash. Risk is owned, not
Copy/Eq/Hash, and Debug excludes retained large arrays. Cash
instruction/decision/receipt are compact owned Copy+Eq+Hash snapshots. Errors are
Clone/Debug/PartialEq/Display/StdError, not Copy/Eq/Hash. No new type has Default,
numeric ordering, parser/identifier registry, panicking arithmetic or implicit
float-money conversions. Metadata enum variants/contextual errors are non-exhaustive.
Struct fields are private.

```text
CanadianFrequency::{Semiannual,Quarterly,Monthly}; ALL:[Self;3]
payments_per_year(self)->u32; months(self)->u32
CanadianPeriodKind::{Regular,Short,Long,Composite}; ALL:[Self;4]
CanadianSourcePolicy::{CanonicalIiac13LocalV1}; ALL:[Self;1]
CanadianYieldConvention::{Compound,MoneyMarket}; ALL:[Self;2]
CanadianYieldSelection::{Explicit(CanadianYieldConvention),LastCouponMoneyMarket}
CanadianPriceBasis::{Clean,ModelDirty,SettlementDirty}; ALL:[Self;3]
CanadianPrice::new(value:f64,basis:CanadianPriceBasis)->Result<Self,CanadianError>
CanadianPrice::value(self)->f64
CanadianPrice::basis(self)->CanadianPriceBasis
```

Price::new accepts every finite signed value, canonicalizes either zero to+0 and rejects
NaN/infinity before any financial model/basis interpretation. Reusing a scalar target in
a different convention is explicit caller definition, not an inferred quote conversion.
Display quantization uses existing `DecimalAmount::from_binary64_rounded` explicitly; no
new display rounding stage affects cash or inversion.

```text
CanadianCouponSchedule::MAX_QUASI_DATES:usize=20_000
CanadianCouponSchedule::MAX_COUPONS:usize=20_000
CanadianCouponSchedule::new(accrual_start:Date,stated_maturity:Date,
  frequency:CanadianFrequency,quasi_dates:Vec<Date>,coupon_dates:Vec<Date>,
  end_of_month:EndOfMonth,max_dates:usize)->Result<Self,CanadianError>
CanadianCouponSchedule::accrual_start(&self)->Date
CanadianCouponSchedule::stated_maturity(&self)->Date
CanadianCouponSchedule::frequency(&self)->CanadianFrequency
CanadianCouponSchedule::end_of_month(&self)->EndOfMonth
CanadianCouponSchedule::direct_cycle_anchor(&self)->Date
CanadianCouponSchedule::quasi_dates(&self)->&[Date]
CanadianCouponSchedule::coupon_dates(&self)->&[Date]
CanadianCouponSchedule::coupon_count(&self)->usize
CanadianCouponSchedule::first_coupon_date(&self)->Date
CanadianCouponSchedule::last_coupon_start_date(&self)->Date
CanadianCouponSchedule::coupon_start_date(&self,index:usize)->Option<Date>
CanadianCouponSchedule::coupon_period_kind(&self,index:usize)->Option<CanadianPeriodKind>
CanadianCouponSchedule::first_period(&self)->CanadianPeriodKind
CanadianCouponSchedule::last_period(&self)->CanadianPeriodKind

CanadianBond::new(original_issue:Date,coupon_per_100:DecimalAmount,
  schedule:CanadianCouponSchedule,true_maturity:Date,
  source_policy:CanadianSourcePolicy)->Result<Self,CanadianError>
CanadianBond::source_version(&self)->&'static str
CanadianBond::source_policy(&self)->CanadianSourcePolicy
CanadianBond::original_issue_date(&self)->Date
CanadianBond::coupon_per_100(&self)->DecimalAmount
CanadianBond::schedule(&self)->&CanadianCouponSchedule
CanadianBond::true_maturity_date(&self)->Date
CanadianBond::first_coupon_date(&self)->Date
CanadianBond::last_coupon_start_date(&self)->Date
CanadianBond::coupon_payment_date(&self,index:usize)->Option<Date>
CanadianBond::valuation_coupon_per_100(&self,index:usize,q:AmountQuantum,
  mode:AmountRounding)->Result<DecimalAmount,CanadianError>
CanadianBond::payable_coupon_per_100(&self,index:usize,q:AmountQuantum,
  mode:AmountRounding)->Result<DecimalAmount,CanadianError>
CanadianBond::coupon_cash(&self,index:usize,nominal:DecimalAmount,
  rounding:SettlementRounding)->Result<CanadianCashInstruction,CanadianError>

CanadianPricingPlan::MAX_BUNDLES:usize=20_000
CanadianPricingPlan::new(bond:&'a CanadianBond,settlement:Date,
  selection:CanadianYieldSelection,max_bundles:usize)->Result<Self,CanadianError>
CanadianPricingPlan::bond(&self)->&'a CanadianBond
CanadianPricingPlan::source_version(&self)->&'static str
CanadianPricingPlan::source_policy(&self)->CanadianSourcePolicy
CanadianPricingPlan::settlement_date(&self)->Date
CanadianPricingPlan::yield_selection(&self)->CanadianYieldSelection
CanadianPricingPlan::yield_convention(&self)->CanadianYieldConvention
CanadianPricingPlan::bundle_count(&self)->usize
CanadianPricingPlan::bundle_coupon_index(&self,index:usize)->Option<usize>
CanadianPricingPlan::coupon_entitlement(&self,index:usize)->Option<CouponEntitlement>
CanadianPricingPlan::valuation_accrued_per_100(&self)->f64
CanadianPricingPlan::settlement_accrued_per_100(&self)->f64
CanadianPricingPlan::quote_accrued_per_100(&self)->f64
CanadianPricingPlan::valuation_accrued_at_quantum(&self,q:AmountQuantum,
  mode:AmountRounding)->Result<DecimalAmount,CanadianError>
CanadianPricingPlan::settlement_accrued_at_quantum(&self,q:AmountQuantum,
  mode:AmountRounding)->Result<DecimalAmount,CanadianError>
CanadianPricingPlan::evaluate(&self,yield_rate:f64)->Result<CanadianValuation,CanadianError>
CanadianPricingPlan::risk(&self,yield_rate:f64)->Result<CanadianRisk,CanadianError>
CanadianPricingPlan::yield_from_price(&self,target:CanadianPrice,bracket:RootBracket,
  options:SolverOptions)->Result<CanadianYieldReport,CanadianYieldError>
CanadianPricingPlan::accrued_cash(&self,nominal:DecimalAmount,
  rounding:SettlementRounding)->Result<CanadianCashInstruction,CanadianError>
CanadianPricingPlan::clean_accrued_cash(&self,nominal:DecimalAmount,
  clean:SettlementEvidence,aggregation:CashAggregation,
  rounding:SettlementRounding)->Result<CanadianCashInstruction,CanadianError>
CanadianPricingPlan::dirty_cash(&self,nominal:DecimalAmount,
  settlement_dirty:SettlementEvidence,rounding:SettlementRounding)
  ->Result<CanadianCashInstruction,CanadianError>
```

Optional index accessors return None; selected source/cash operations return
InvalidCouponIndex. `coupon_entitlement(index)` uses the ORIGINAL actual coupon
ordinal in the full instrument schedule, including ceased coupons, and returns None
only outside that full coupon count. In contrast, `bundle_coupon_index(index)` takes
the compressed future-bundle position and returns its original actual ordinal.
For settlement on the first coupon, bundle position0 maps to original ordinal1,
while entitlement at original ordinal0 is Ceased. Instrument/schedule coupon
selectors also use original actual ordinals. Numeric errors use original actual
ordinals, never compressed positions or fabricated Q indices for off-cycle maturity.
Date-cycle errors use original Q indices. No public private-kernel hook is introduced.

Every numerical report supplies the following owned scalar metadata methods (Copy
reports self; Risk &self):

```text
source_version()->&'static str; source_policy()->CanadianSourcePolicy
accrual_start()->Date; original_issue_date()->Date
stated_maturity_date()->Date; true_maturity_date()->Date
first_coupon_date()->Date; last_coupon_start_date()->Date
first_period()->CanadianPeriodKind; last_period()->CanadianPeriodKind
frequency()->CanadianFrequency; coupon_per_100()->DecimalAmount
end_of_month()->EndOfMonth; direct_cycle_anchor()->Date
settlement_date()->Date; yield_selection()->CanadianYieldSelection
yield_convention()->CanadianYieldConvention
valuation_accrued_per_100()->f64; settlement_accrued_per_100()->f64
quote_accrued_per_100()->f64; yield_rate()->f64
model_dirty_per_100()->f64
```

These scalars are compact definition-reconstructing provenance as proved below; no full
vector is retained. Reports need no borrowed vector or source requery.

```text
CanadianValuation::model_dirty_price(self)->CanadianPrice
CanadianValuation::clean_price(self)->Result<CanadianPrice,CanadianError>
CanadianValuation::settlement_dirty_price(self)->Result<CanadianPrice,CanadianError>
CanadianRisk::price_derivative(&self)->Result<f64,CanadianError>
CanadianRisk::price_second_derivative(&self)->Result<f64,CanadianError>
CanadianRisk::time_weighted_amount(&self)->Result<f64,CanadianError>
CanadianRisk::modified_duration(&self)->Result<f64,CanadianError>
CanadianRisk::macaulay_duration(&self)->Result<f64,CanadianError>
CanadianRisk::convexity(&self)->Result<f64,CanadianError>
CanadianRisk::model_dirty_price(&self)->CanadianPrice
CanadianRisk::clean_price(&self)->Result<CanadianPrice,CanadianError>
CanadianRisk::settlement_dirty_price(&self)->Result<CanadianPrice,CanadianError>
CanadianYieldReport::target(self)->CanadianPrice
CanadianYieldReport::model_dirty_target_per_100(self)->f64
CanadianYieldReport::initial_bracket(self)->RootBracket
CanadianYieldReport::options(self)->SolverOptions
CanadianYieldReport::solve_report(self)->SolveReport
CanadianYieldReport::residual_scale_exponent(self)->u8
CanadianYieldReport::sensitivity(self)->BondYieldSensitivity
CanadianYieldReport::amount_residual(self)->Result<f64,CanadianError>
CanadianYieldReport::model_dirty_price(self)->CanadianPrice
CanadianYieldReport::clean_price(self)->Result<CanadianPrice,CanadianError>
CanadianYieldReport::settlement_dirty_price(self)->Result<CanadianPrice,CanadianError>
```

No getter re-evaluates yield, allocates a new vector or depends on another raw getter's
representability. Ordinary Clone/Debug/formatting have their standard allocation
qualifications; fallible construction does not promise arbitrary Clone or formatting is
allocation-free. In particular, normalized convexity may underflow while the raw second
derivative is normal; no Treasury-specific exclusion is copied into this fractional
model.

## 7. Complete exact cash semantics and public receipt API

Let R be the existing seven-mode signed quantum quantizer at the caller's
SettlementRounding. Let U be selected exact payable coupon, AI exact settlement accrued.
Clean evidence X and settlement-dirty evidence Z preserve existing
exact-decimal/exact-supplied-binary64/continuous-declared-interval meanings.

```text
PayableCouponPer100:               R(N*U/100)
SettlementAccruedPer100:           R(N*AI/100)
SettlementDirtyPricePer100:        R(N*Z/100)
CleanPriceAndSettlementAccruedPer100:
  RoundTotal:                     R(N*(X+AI)/100)
  RoundComponentsThenSum:         exact_sum(R(N*X/100), R(N*AI/100))
```

No preceding q6/q3 price rounding, displayed per100 AI, coupon-cent publication or
approximate binary AI is inserted. Coupon U excludes principal; redemption cash is not
silently added to a coupon distribution. N>0 even when source, price or cash is zero.
Negative cash reverses buyer/seller for trade recipes; positive coupon cash is
issuer-to-holder. Negative N does not encode direction. No currency/unit is inferred.

Dirty cash ALWAYS means ALL-IN SETTLEMENT price evidence, never automatically compound
ModelDirty evidence. A caller may explicitly convert an observed CanadianPrice into
supplied-binary64 evidence, but the price ingredient remains that supplied point. A
binary settlement bridge uses observed AI_hat, while clean invoice uses exact source AI.
Therefore do not promise equality between dirty_cash(observed_bridge) and
clean_accrued_cash(observed_clean) at arbitrarily fine quanta; do not claim either point
encloses an ideal model price.

Required stage order:

1. coupon_cash validates actual coupon ordinal before N>0; other factories validate N>0 first. Source/evidence/rounding types are already valid. Capture immutable owned provenance; no source DecimalAmount or float range gate and no new allocation/query.
2. Pure coupon stage `cash coupon`; AI stage `cash accrued`; dirty/total-invoice stage `cash total`.
3. Component invoice completes `cash capital`, then `cash accrued`, then exact `cash net` with no additional quantizer. Intermediate rounded coefficients stay wide; later net cancellation can rescue final range but cannot waive an earlier Exact domain failure.
4. A required Exact quantizer on a point uses divisibility and returns point Inexact if needed. On a nonpoint continuous stage image, return ExactStageUnresolved even if endpoint values are quantum multiples. Do not enumerate only binary64 interior points.
5. Only final amounts are normalized/admitted, lower before upper when the reported stage input image is nonpoint. Endpoint labels are `point/lower/upper` from that stage image. Component quantization may collapse a later net to a point. Equal declared bounds are points while retaining declared provenance.
6. Equal admitted final extrema give Determined; unequal extrema give QuantumAmbiguous with ordered bounds. No clipping or suggested midpoint; representable extrema do not certify every interior decimal amount across precision holes.

```text
CanadianCashInstruction::evaluate(&self)->Result<CanadianCashDecision,CanadianError>
CanadianCashInstruction::source_version(&self)->&'static str
CanadianCashInstruction::source_policy(&self)->CanadianSourcePolicy
CanadianCashInstruction::recipe_identifier(&self)->&'static str
CanadianCashInstruction::nominal(&self)->DecimalAmount
CanadianCashInstruction::cash_rounding(&self)->SettlementRounding
CanadianCashInstruction::price_evidence(&self)->Option<SettlementEvidence>
CanadianCashInstruction::aggregation(&self)->Option<CashAggregation>
CanadianCashInstruction::coupon_per_100(&self)->DecimalAmount
CanadianCashInstruction::frequency(&self)->CanadianFrequency
CanadianCashInstruction::accrual_start(&self)->Date
CanadianCashInstruction::original_issue_date(&self)->Date
CanadianCashInstruction::stated_maturity_date(&self)->Date
CanadianCashInstruction::true_maturity_date(&self)->Date
CanadianCashInstruction::first_coupon_date(&self)->Date
CanadianCashInstruction::last_coupon_start_date(&self)->Date
CanadianCashInstruction::first_period(&self)->CanadianPeriodKind
CanadianCashInstruction::last_period(&self)->CanadianPeriodKind
CanadianCashInstruction::end_of_month(&self)->EndOfMonth
CanadianCashInstruction::direct_cycle_anchor(&self)->Date
CanadianCashInstruction::settlement_date(&self)->Option<Date>
CanadianCashInstruction::yield_selection(&self)->Option<CanadianYieldSelection>
CanadianCashInstruction::yield_convention(&self)->Option<CanadianYieldConvention>
CanadianCashInstruction::source_coupon_index(&self)->Option<usize>
CanadianCashInstruction::source_coupon_date(&self)->Option<Date>
CanadianCashInstruction::source_payment_date(&self)->Option<Date>
CanadianCashInstruction::source_interval(&self)->Option<(Date,Date)>
CanadianCashInstruction::source_quasi_endpoints(&self)->Option<((Date,Date),(Date,Date))>
CanadianCashInstruction::source_full_period_count(&self)->Option<u32>
CanadianCashInstruction::source_day_ratios(&self)
  ->Option<((u32,u32),Option<(u32,u32)>)>
CanadianCashInstruction::source_value_per_100(&self,q:AmountQuantum,
  mode:AmountRounding)->Result<Option<DecimalAmount>,CanadianError>
CanadianCashDecision::{Determined(CanadianCashReceipt),
  QuantumAmbiguous{lower_amount:DecimalAmount,upper_amount:DecimalAmount},
  ExactStageUnresolved{stage:&'static str}}
CanadianCashReceipt::amount(self)->DecimalAmount
CanadianCashReceipt::instruction(&self)->&CanadianCashInstruction
CanadianCashReceipt::price_evidence_guarantee(self)->Option<SettlementGuarantee>
```

Coupon snapshot: selected actual coupon/start/end/payment date, full/partial source
geometry, exact U; no S/yield selection. AI/invoice snapshot: S, actual accrual
start/next actual coupon and its payment date, geometry, exact AI, chosen
selection/convention. Its source_interval is exactly(latest actual coupon<=S or D,S),
not the whole next-coupon interval. At S=D or an actual coupon date retain Some((S,S)),
full_countSome(0), day_ratiosNone and the containing right-hand Q pair repeated.
Selected NEXT actual coupon/date/payment remains present. C=0 preserves the actual
nonempty day geometry rather than replacing it by empty geometry merely because source
value is zero. Dirty-only snapshot: S/selection/convention and common bond scalars, no
selected source coupon/geometry/value. On dirty-only rows source_value returns Ok(None)
with no quantization. None is genuine absence, not a fake0 date/index.

Geometry is compact: interval endpoints, first/last quasi pairs, full-count and at most
two nonzero proper-fragment ratios. For a nonempty interval the first pair contains its
start on the right and the last pair contains its end on the left (an end exactly on Q
uses the preceding pair). An empty accrued interval uses the pair containing S in both
positions, with full count0 and no fragments. When there is only one containing quasi,
return it in both endpoint-pair positions but only one fragment. When there are no
proper fragments source_day_ratios is None, independently of full count. No full Q or
coupon vector is cloned into cash; exact source ratio plus these fields retain the
selected definition. Public q/m source views are independent calls, not retained invoice
policies.

Every common bond/report/cash snapshot additionally retains L=last_coupon_start_date (D
for the one-coupon case), even if the selected cash recipe is the first coupon or
dirty-only. This field is not optional and is part of Eq/Hash wherever identity traits
apply. It prevents an equal-cash snapshot collision between actual
dates[2024-07-01,2025-01-01,2027-01-01] and[2024-07-01,2025-01-01,2025-07-01,2027-01-01]
on the same D=I2024-01-01, semiannual Q, M=T2027-01-01,C5. Both have Regular first/Long
last and equal first coupon cash, but L differs2025-01-01 versus2025-07-01.

Definition reconstructibility proof: explicit anchor/f/EOM gives every direct nominal Q
date. Minimal outer coverage with D and M uniquely selects its supplied finite extent.
If F=M, strict actual-date ordering implies exactly one actual coupon[M] and L=D.
Otherwise F<M, F and L are nonterminal Q members; every intervening actual period is
regular, forcing exactly the contiguous Q slice F..L followed by M. F=L is the
two-coupon case. First/last long periods merely omit Q dates outside that slice and do
not weaken uniqueness; off-cycle terminal M is appended once. Thus
D,I,M,T,C,F,L,anchor,f,EOM,policy determine the admitted full definition without cloning
vectors. Derived first/last kinds remain exposed consistency metadata.
`/private/tmp/sabiquant-canadian-identity-controls-v1/evidence.json`
SHA4983d69731a8adf9afc8b04e3ced3af9b2ef70994b234bddb5f6b5ba9aa33d79 records516 valid
abstract schedule reconstructions plus the exact dated equal-cash counterexample; the
general proof, not finite enumeration alone, establishes injectivity.

Instrument identity is complete supplied definition I,C,T,source policy and complete
schedule (D,M,f,Q,actual dates,EOM). Caps are resource controls, not financial identity.
The caps bound vector lengths. Moving supplied vectors preserves their capacities;
caller-supplied spare capacity and allocator overhead have no promised byte ceiling.
They are excluded from the separate live-numeric-payload limits. Equal financial
definitions remain equal regardless of spare capacity. Prepared reservations specify
requested element slots, not actual allocator-granted bytes or capacity.
Compact instruction Eq/Hash includes every retained field, exact source ratio, nominal,
rounding, evidence kind/bits/bounds and aggregation. Different
policies/selection/evidence can be unequal despite equal cash. Receipt adds final amount
and ingredient guarantee. Non-determined decisions contain only their listed
diagnostics; callers retain the instruction for provenance.

Pure source recipes have no price ingredient guarantee. Price receipts return
declared-enclosure guarantee for a declared kind (even equal bounds), otherwise
supplied-binary64 or supplied-decimal guarantee. These labels apply ONLY to price
evidence. They do not certify source compliance, ideal prices, legal calendars or
ownership. No old SettlementReceipt is forged with a new rational-source meaning.

## 8. Complete error ledger and precedence

Propose33 CanadianError variants. Structured contextual variants are non-exhaustive;
kind prefix `canadian.` plus suffix below. Each typed cause is the immediate
StdError::source; other variants have None. There are no string-erased causes,
sentinels, panics or recursive allocation.

| Variant and fields | suffix / cause |
|---|---|
| InvalidDateCap{cap,minimum,maximum:usize} | invalid_date_cap / none |
| TooManyQuasiDates{count,cap:usize} | too_many_quasi_dates / none |
| TooFewQuasiDates{count,minimum:usize} | too_few_quasi_dates / none |
| TooManyCoupons{count,cap:usize} | too_many_coupons / none |
| NoCoupons | no_coupons / none |
| AccrualNotBeforeMaturity{accrual_start,stated_maturity:Date} | accrual_not_before_maturity / none |
| QuasiDateOrder{index:usize,previous,date:Date} | quasi_date_order / none |
| QuasiCoverage{accrual_start,stated_maturity,first,next,penultimate,last:Date} | quasi_coverage / none |
| QuasiDateMismatch{index:usize,supplied,expected,anchor:Date,frequency:CanadianFrequency,end_of_month:EndOfMonth} | quasi_date_mismatch / none |
| CouponDateOrder{index:usize,previous,date:Date} | coupon_date_order / none |
| CouponBoundary{last_coupon,stated_maturity:Date} | coupon_boundary / none |
| CouponBeforeAccrual{index:usize,date,accrual_start:Date} | coupon_before_accrual / none; includes equality |
| CouponOffCycle{index:usize,date:Date} | coupon_off_cycle / none |
| IrregularInteriorPeriod{index:usize,previous,date:Date} | irregular_interior_period / none |
| Date{index:Option<usize>,phase:&'static str,cause:DateError} | date / DateError |
| NegativeCoupon{coupon_per_100:DecimalAmount} | negative_coupon / none |
| DatedAfterIssue{accrual_start,original_issue:Date} | dated_after_issue / none |
| IssueNotBeforeFirstCoupon{original_issue,first_coupon:Date} | issue_not_before_first_coupon / none |
| TrueMaturityBeforeStated{true_maturity,stated_maturity:Date} | true_maturity_before_stated / none |
| InvalidBundleCap{cap,maximum:usize} | invalid_bundle_cap / none |
| TooManyBundles{count,cap:usize} | too_many_bundles / none |
| SettlementBeforeIssue{settlement,original_issue:Date} | settlement_before_issue / none |
| SettlementAtOrAfterMaturity{settlement,stated_maturity:Date} | settlement_at_or_after_maturity / none |
| InvalidCouponIndex{index,count:usize} | invalid_coupon_index / none |
| MoneyMarketDomain{yield_rate:f64,days_to_true_maturity:u32} | money_market_domain / none |
| Value{index:Option<usize>,phase:&'static str,cause:ValueError} | value / ValueError |
| Rate{index:Option<usize>,phase:&'static str,cause:RateError} | rate / RateError |
| Sum{phase:&'static str,cause:SumError} | sum / SumError |
| AllocationFailed{phase:&'static str,requested:usize} | allocation_failed / none |
| InternalCapacity{index:Option<usize>,phase,operation:&'static str} | internal_capacity / none |
| ZeroDirtyPrice{measure:&'static str,yield_rate:f64} | zero_dirty_price / none |
| NonPositiveNominal{nominal:DecimalAmount} | nonpositive_nominal / none |
| CashArithmetic{stage,endpoint:&'static str,cause:AmountError} | cash_arithmetic / AmountError |

MoneyMarketDomain tests exact365+y*days>0, avoiding a false f64 Interval boundary for
the non-dyadic floor -365/days. A nonfinite quote first gives Value `rate`; a finite
compound quote first tests y>-f. The new Rate wrapper is for the proposed compound
elementary/boundary profile, not a change to Treasury's rational kernel. ZeroDirtyPrice
is defensive for valid positive-par admitted observations, not permission to forge
report state.

CanadianYieldError has5 variants and kind prefix `canadian_yield.`:

```text
Target{target:CanadianPrice,phase:&'static str,cause:CanadianError} #target
Preparation{phase:&'static str,cause:CanadianError}              #preparation
Solver(SolverError)                                             #solver
Callback{stage:EvaluationStage,point:f64,evaluations:EvaluationCounts,
  cause:CanadianError}                                          #callback
NonPositiveTarget{target:CanadianPrice,model_dirty_target_per_100:f64,
  quote_accrued_per_100:f64,settlement_accrued_per_100:f64,
  source_policy:CanadianSourcePolicy,convention:CanadianYieldConvention,
  evaluations:EvaluationCounts}                                 #nonpositive_target
```

Immediate cause is present for the first4 only. The last records zero
function/derivative evaluations. Prospective root Error wrappers Canadian/CanadianYield
transparently forward kind/Display/source and have From impls; do not count wrappers as
domain variants again. Prospective domain addition is33+5=38; implemented catalogs are
unchanged.

Fixed operation order:

1. Schedule: max_dates in2..20,000; Q count<=cap; Q count>=2; coupon count<=cap; nonempty coupons; D<M; Q strict order by original index; minimal outer coverage; direct-cycle date/mismatch by increasing original Q index; last actual=M; actual strict order; first>D; nonterminal Q membership; regular interior adjacency by increasing actual ordinal; derive exact first/last roles/caches. Own supplied vectors, no cloning or separate coupon-vector allocation.
2. Bond: C>=0; D<=I; I<F; T>=M; derive separate first/last/regular exact source caches. Already valid Schedule is moved, not revalidated or mutated.
3. Plan: cap<=20,000 (zero allowed); S>=I; S<M; count all future actual coupons including zeros/terminal; count<=cap; resolve selection; derive exact A/AI/time metadata; observe A_hat and AI_hat once; reserve/fill one bundle vector fallibly. No allocation or index compression before these checks.
4. evaluate/risk: validate finite quote then branch domain before fallible PV workspace; observe factor then direct exact-source PV then risk terms, if requested, for each original actual ordinal in order; dirty exact reduction before immutable report; no eager clean/all-in/source-view checks. Unused failures do not leak into independent operations.
5. Inversion: typed target resolution/basis shift, positive target gate, residual scale, one PV workspace reservation; then shared fallible Brent lower-first callbacks. Quote/domain checks occur only on invoked points. No hidden final callback or per-trial allocation/date/source work. Failed invocation counts itself and retains exact stage/point/cause.
6. Cash: selector then nominal as above; required stages, Exact semantics and final lower/upper admission as §7. Source diagnostic views use `source valuation coupon`, `source payable coupon`, `source valuation accrued`, `source settlement accrued`, endpoint point, AmountError quantize semantics.

Stable numeric phases: `yield quotation`, `discount factor`, `source present value`,
`dirty price reduction`, `clean price reduction`, `settlement dirty price reduction`,
`yield target shift`, `yield amount residual`, and the six getter names. Allocation
phases: `pricing bundles`, `present value terms`, `risk present value terms`, `yield
present value terms`. Cash stages: source view names above and `cash
coupon/accrued/capital/total/net`. CashArithmetic uses AmountError::Inexact(operation
quantize) or OutOfRange(operation quantize, except final net sum). A nonzero result
rounded to signed0/inf is an independent signed range failure; exact zero is+0. Proposed
fixed internal-capacity operation vocabulary: `derive source`, `decode binary`,
`construct affine`, `normalize rational`, `round rational`, `round source present
value`, `round logarithmic exponent`, `guard discount range`, `round scaled term`, `accumulate risk`, `round
risk value`, `normalize risk value`, `derive residual`, `decode evidence`, `scale cash`,
`combine cash`, `cash rounding`, `exact domain`, `normalize cash`. These are
proof-excluded invariant failures, not ordinary financial overflow. Source/flow work
uses Some(original actual coupon ordinal), global
AI/shared-log/denominator/reduction/getters use None; Q-cycle Date errors alone use
original Q ordinal and phase `quasi cycle`.

| Value/Rate quantity | Outer phase/index |
|---|---|
| `rate` finite/compound-domain admission | `yield quotation`, None |
| `canadian price` finite CanadianPrice construction | `price quotation`, None |
| `canadian scaled logarithm` shared primitive | `discount factor`, None |
| `canadian logarithmic exponent` per-source rational admission | `discount factor`, Some(actual ordinal) |
| `discount factor` observed factor/boundary | `discount factor`, Some(actual ordinal) |
| `canadian present value` nonzero IEEE PV | `source present value`, Some(actual ordinal) |
| Each of the6 getter names | matching phase/quantity, None |
| `yield amount residual` | matching phase/quantity, None |

RateError::NumericalBoundary retains quantity `discount factor` and the observed
logarithm; ordinary RateError::Value preserves its immediate ValueError. The bounded guard uses closed intersection with either certified threshold box,
including touching, before any exp. Disjoint intervals, including certainly out-of-range
ones, call exp once and validate its actual result. Shared g is observed after PV
reservation. The eventual profile freeze must keep these phases/quantities, operation
`guard discount range` and complete context instead of inventing an untyped retry. Source AI IEEE view failures are magnitude-proof-excluded; a
failed bounded helper there remains InternalCapacity `derive source`/`round rational`
with phase `source valuation accrued` or `source settlement accrued`, never a spurious
ordinary amount-view error.

## 9. Adoption/evidence order and remaining implementation gates

This document recommends a coherent entire source/API/cash model. It does not adopt it.
Numeric execution/storage and broad independent test plan are in the [numeric/test
companion](M8_CANADIAN_NUMERIC_AND_TEST_DESIGN.md); those contain explicit acceptance
limits/candidate paths rather than claims of an existing implementation.

Before any Canadian production: independent review of this complete scope and source
deviations; final API/error/resource/numeric observation contract; pinned genuine
primary bytes; independent exact source/cash corpus and hand controls; separate
certified ideal price/risk and numerical boundary corpora; complete static schema/input
identity and attested actual--check; freeze accepted artifacts. Only then implement the
full chosen model, private actual-kernel adapter, public consumers, semantic mutants and
complete native/package/coverage battery. No prior frozen generic/gilt/Treasury
expectation is repinned to make a Canadian claim pass.
