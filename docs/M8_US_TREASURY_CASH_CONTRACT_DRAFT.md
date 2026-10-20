# US Treasury cash contract draft

| Field | Value |
|---|---|
| Status | **Proposed draft for root integration; not adopted, not frozen** |
| Baseline | Local `7bfd94f`, `feat/m8-us-treasury` |
| Date | 2026-10-20, Asia/Bangkok |
| Scope | Source policy types, exact coupon and published accrued values, the four cash recipes, their views, stages, errors, identity, width proofs and resources |
| Owner | Drafted by Claude peer `sabiquant-a2`; root owns the main contract and decides adoption |

Every name below is **proposed**. Nothing is implemented. Root's main
Treasury contract draft defines the instrument, dates, plan, price, yield and
risk. This draft defines only the cash-facing surface and the source values
that cash consumes; the main contract will incorporate or replace it after
independent review. No QuantLib implementation, Treasury Rust, general fixture
or requirement exists or was read.

Inputs: [design proposal](M8_US_TREASURY_LOCAL_DESIGN.md) D3/D8/D9,
[source examples](M8_US_TREASURY_SOURCE_EXAMPLES.md),
[source refresh](M8_US_TREASURY_SOURCE_REFRESH.md), root's alignment choices
of 2026-10-20 and the frozen [gilt contract](M8_GILT_CONTRACT.md) C1–C8 as
structural guidance only. No frozen gilt artifact changes.

## T1. Scope, units and labelled local extensions

The primary source is 31 CFR Part 356 Appendix B, sections I.A (coupons),
I.D (accrued interest) and the Table 2 daily interest decimals, pinned as eCFR
XML SHA-256 `35fa6cb3…699d`. Coupon C is the exact nonnegative annual
`DecimalAmount` per 100 (`6.125` means 6 1/8%). Source cash amounts are per
$1,000 par as in Appendix B. Values per 100 are exactly one tenth of them.
Cash uses explicit `SettlementRounding` and strictly positive exact par
`nominal`.

Positive coupon cash is payable issuer to holder; the caller supplies an
entitled held nominal, without ownership/record-date verification. Positive
trade/accrued/invoice cash is payable buyer to seller (including Treasury
for an auction); negative price-based cash reverses that direction. Nominal
remains positive and does not encode transaction direction. No currency
conversion, denomination enforcement or payment-calendar assurance is implied.

The following are explicit **local engineering choices**, not source facts:

1. **D-based fractional periods.** Fractional coupon and accrued interest are
   measured from the original dated date D. When D < I this extends
   I.A.3/4's literal issue-date wording, as in the design's D2. A long first
   period with I > Q[1] is admitted and needs a dedicated control.
2. **Non-$1,000 coupon nominal.** I.A.3/4 state scaling only for multiples of
   $1,000. Coupon cash for any other positive par scales exactly by `N/1000`.
   I.D.4 does cover both multiples and fractions for accrued interest.
   Denominations and minimum amounts are not modeled.
3. **General coupon.** Table 2 covers 1/8% to 20% in 1/8% steps. Every other
   nonnegative C is a mathematical extension under the same rules.
4. **Tie rule.** "Normal rounding procedures" does not state a tie rule. The
   caller-selected `TreasuryTieRule` is not asserted to be the regulatory one.
5. **Secondary settlement.** Accrued and price recipes accept any prepared
   settlement `I <= S < M`, not only auction issue dates. With no ex-coupon
   or record-date state, accrued interest is always `>= 0`.
6. **Cycle policy.** Official Treasury month-end cycles require
   `EndOfMonth::Preserve`. `Ignore` remains an admitted mathematical variant
   with no Treasury compliance claim.

For accrued cash, this local contract applies the mandatory five-place stage
per $1,000 once, then scales by N/1000 and applies only the caller's final
cash quantizer. It does not insert a second five-place quantizer after a
non-integer nominal multiple. This makes the chosen interpretation of
I.D.4 explicit; the printed integer-multiple examples cannot discriminate
every fractional-nominal alternative.

There is no six-place source price API. Source prints remain test anchors,
and price inputs to cash are caller `SettlementEvidence`.

## T2. Source policy types

```text
TreasuryDailyBasis::{ExactRatio, NineDecimal}; ALL: [Self; 2]
TreasuryTieRule::{NearestTiesEven, NearestTiesAwayFromZero}; ALL: [Self; 2]
TreasuryTieRule::amount_rounding(self) -> AmountRounding
TreasurySourcePolicy::new(daily_basis: TreasuryDailyBasis,
    tie_rule: TreasuryTieRule) -> Self                    // const, infallible
TreasurySourcePolicy::daily_basis(self) -> TreasuryDailyBasis
TreasurySourcePolicy::tie_rule(self) -> TreasuryTieRule
```

Both enums are `#[non_exhaustive]` with variants in the order shown. All three
types are `Debug + Clone + Copy + PartialEq + Eq + Hash + Send + Sync`. None
has `Default`, a parser, `Display` registry, `Ord` or an implicit conversion.
The policy is part of `UsTreasury` definition identity, and every cash
instruction retains it. The tie rule governs **both** source publication
stages of T3: the nine-place daily decimal and the five-place accrued
amount per $1,000. It never governs coupon cents or final cash, which use only
the caller's `SettlementRounding`. Source values are nonnegative, so
`NearestTiesAwayFromZero` rounds ties upward.

## T3. Exact source values

Notation: `C = Ccoef/10^Cscale`, `H = 5C` per $1,000 (the half-year coupon), and
`s = days(Q[j-1], Q[j])` for the half-year containing a fractional period.
The design's direct cycle gives `181 <= s <= 184`; the proofs below use
only `1 <= t <= s <= 184`.

**Daily stage.** The staged daily amount per $1,000 for half-year `s` is

```text
ExactRatio:   δ(s) = H/s                    (exact rational)
NineDecimal:  δ(s) = RN9_T(H/s)             (nearest 1e-9 multiple, tie rule T)
```

For C on the 1/8% grid a nine-place tie is impossible. `5k/(8s)` terminates
only when `k` is divisible by the factor of `s` other than 2 (23, 91, 183 or
181), and it then terminates within six places. For general C ties occur:
`C = 23e-7` at `s = 184` gives `6.2e-8` (even) or `6.3e-8` (away).

**Coupons.** Actual coupon ordinal `i` is paid at `Q[f+i]`, where `f` is 1
for Regular/Short and 2 for Long. Let `s1 = days(Q0,Q1)` and
`r1 = days(D,Q1)`, with `1 <= r1 < s1` for Short/Long.
Here r1/s1 are aliases for main E's d0/s0, respectively. Main E's half-coupon
c=C/2 is never a decimal coefficient; coefficient notation here is Ccoef.

| Coupon | Raw per 100 (exact) | Source per $1,000 (policy-staged, unrounded) |
|---|---|---|
| Any regular coupon (all `i >= 1`; `i = 0` when Regular) | `C/2` | `H` |
| Short, `i = 0` | `(C/2)·r1/s1` | `δ(s1)·r1` |
| Long, `i = 0` (paid at `Q2`; nothing at `Q1`) | `(C/2)·(r1/s1 + 1)` | `δ(s1)·r1 + H` |

A source coupon has no coupon-level quantizer before cash (I.A.3/4
"unrounded"); its fractional daily ingredient may already have the chosen
nine-place publication. The long
coupon combines its fractional piece with the full coupon before nominal
scaling. Under `ExactRatio` the source coupon equals ten times the raw
coupon exactly. The instrument caches only the raw and source first coupons;
regular coupons derive from C, and no coupon vector is allocated.

**Accrued pieces.** For settlement S, let `k` be the first index with
`Q[k] > S`:

| Lifecycle | Pieces `(t, s)` |
|---|---|
| `k > f`, or Regular with `k = f` | `(days(Q[k-1],S), days(Q[k-1],Q[k]))` |
| Short, `k = f = 1` | `(days(D,S), s1)` |
| Long, `k = 1` (S before the skipped `Q1`) | `(days(D,S), s1)` |
| Long, `k = f = 2` | `(r1, s1)` and `(days(Q1,S), days(Q1,Q2))` |

`t = 0` is valid. Accrued interest is zero at an actual coupon date and at
`S = D`. At the skipped long-first `Q1` it is `(r1, s1)` plus a zero second
piece, so it does not reset.

**Raw accrued per 100** is `Σ (C/2)·t_p/s_p`, exact and retained separately.
It is never used by cash.

**Published accrued per $1,000** is `P1000 = RN5_T(Σ δ(s_p)·t_p)`. The pieces
are summed unrounded and rounded once, as in I.D.4 and I.D example (2).
Published per 100 is `P1000/10`, identical to six-place rounding of the
per-100 sum because the factor 10 maps one lattice onto the other.
Per-component rounding, as in II.E's display, is a different recipe and is
not substituted. Plan preparation computes the exact `P1000` count once and
retains it with its pieces. The plan's binary `accrued_per_100` is one
nearest rounding of `P1000/10`; `raw_accrued_per_100` is the raw view.

**Source anchors.** The width check replays these definitions under
`NineDecimal` with both tie rules. Every printed value matches:

* I.A.3: 41.419837004 per $1,000; $20,000 → 828.39674008, cents 828.40.
* I.A.4: 55.210597810 per $1,000; $7,000 → 386.47418467, cents 386.47.
* I.D(1): 16.87500 per $1,000; $150,000 → 2,531.25.
* I.D(2): 36.72798 per $1,000; $11,000 → 404.00778, cents 404.01.

`ExactRatio` reproduces the same cents. Its I.A.3 coupon is `30485/736` per
$1,000, which does not terminate, so an Exact view at `1e-9` is Inexact.

## T4. Public cash interface

```text
UsTreasury::coupon_cash(&self, index: usize, nominal: DecimalAmount,
    rounding: SettlementRounding) -> Result<TreasuryCashInstruction, TreasuryError>
UsTreasury::raw_coupon_per_100(&self, index: usize, quantum: AmountQuantum,
    mode: AmountRounding) -> Result<DecimalAmount, TreasuryError>
UsTreasury::source_coupon_per_100(&self, index: usize, quantum: AmountQuantum,
    mode: AmountRounding) -> Result<DecimalAmount, TreasuryError>

TreasuryPricingPlan::accrued_cash(&self, nominal: DecimalAmount,
    rounding: SettlementRounding) -> Result<TreasuryCashInstruction, TreasuryError>
TreasuryPricingPlan::clean_accrued_cash(&self, nominal: DecimalAmount,
    clean: SettlementEvidence, aggregation: CashAggregation,
    rounding: SettlementRounding) -> Result<TreasuryCashInstruction, TreasuryError>
TreasuryPricingPlan::dirty_cash(&self, nominal: DecimalAmount,
    dirty: SettlementEvidence, rounding: SettlementRounding)
    -> Result<TreasuryCashInstruction, TreasuryError>

TreasuryCashInstruction::evaluate(&self) -> Result<TreasuryCashDecision, TreasuryError>
TreasuryCashInstruction::source_version(&self) -> &'static str
TreasuryCashInstruction::recipe_identifier(&self) -> &'static str
TreasuryCashInstruction::source_policy(&self) -> TreasurySourcePolicy
TreasuryCashInstruction::nominal(&self) -> DecimalAmount
TreasuryCashInstruction::cash_rounding(&self) -> SettlementRounding
TreasuryCashInstruction::price_evidence(&self) -> Option<SettlementEvidence>
TreasuryCashInstruction::aggregation(&self) -> Option<CashAggregation>
TreasuryCashInstruction::coupon_per_100(&self) -> DecimalAmount
TreasuryCashInstruction::original_dated_date(&self) -> Date
TreasuryCashInstruction::original_issue_date(&self) -> Date
TreasuryCashInstruction::first_coupon_date(&self) -> Date
TreasuryCashInstruction::maturity_date(&self) -> Date
TreasuryCashInstruction::first_period(&self) -> TreasuryFirstPeriod
TreasuryCashInstruction::end_of_month(&self) -> EndOfMonth
TreasuryCashInstruction::dated_quasi_dates(&self) -> (Date, Date)
TreasuryCashInstruction::settlement_date(&self) -> Option<Date>
TreasuryCashInstruction::settlement_quasi_dates(&self) -> Option<(Date, Date)>
TreasuryCashInstruction::source_coupon_index(&self) -> Option<usize>
TreasuryCashInstruction::source_coupon_date(&self) -> Option<Date>
TreasuryCashInstruction::source_day_ratios(&self)
    -> Option<((u32, u32), Option<(u32, u32)>)>
TreasuryCashInstruction::source_value_per_100(&self, quantum: AmountQuantum,
    mode: AmountRounding) -> Result<Option<DecimalAmount>, TreasuryError>

TreasuryCashDecision::Determined(TreasuryCashReceipt)
TreasuryCashDecision::QuantumAmbiguous { lower_amount: DecimalAmount,
    upper_amount: DecimalAmount }
TreasuryCashDecision::ExactStageUnresolved { stage: &'static str }
TreasuryCashReceipt::amount(self) -> DecimalAmount
TreasuryCashReceipt::instruction(&self) -> &TreasuryCashInstruction
TreasuryCashReceipt::price_evidence_guarantee(self) -> Option<SettlementGuarantee>
```

Inspector roles by recipe:

| Factory | `recipe_identifier` | `source_value_per_100` | Price evidence / aggregation | Settlement fields | Coupon index/date | `source_day_ratios` |
|---|---|---|---|---|---|---|
| `coupon_cash` | `SourceCouponPer1000` | Source coupon per 100 | None / None | None | Selected ordinal and date | `((r1,s1), None)` for a Short/Long `i=0`; None for regular coupons |
| `accrued_cash` | `PublishedAccruedPer1000` | Published accrued per 100 | None / None | S and `(Q[k-1],Q[k])` | Next actual coupon after S | The one or two accrued pieces |
| `clean_accrued_cash` | `CleanPricePer100AndPublishedAccruedPer1000` | Published accrued per 100 | Clean evidence / caller aggregation | Same | Same | Same |
| `dirty_cash` | `DirtyPricePer100` | `Ok(None)` | Dirty evidence / None | Same | None | None |

`dated_quasi_dates()` is `(Q0, Q1)` with `Q0 <= D < Q1`; together with
`source_day_ratios()` it provides the dated first-fraction metadata that a
long-first accrued value depends on. `source_version()` returns the
instrument's identifier, proposed by root as
`US-Treasury-31CFR356-AppB-fixed-2026-10-16-local-v1`.
`source_value_per_100` quantizes the **selected cash source** once, never the
raw value. Its quantum and mode are call arguments and are not retained.
For a coupon instruction its view stage is `source coupon`, endpoint
`point`, quantization operation `quantize`, with capacity index
Some(selected actual coupon ordinal). For every valid i,N,R,q,m:
`coupon_cash(i,N,R)?.source_value_per_100(q,m)` equals
`source_coupon_per_100(i,q,m).map(Some)`, including identical quantization
errors. Accrued/invoice instruction views use `source accrued` at point and
exact P1000/10, never raw AI; dirty views return Ok(None) without quantizing.
TreasuryCashDecision and its contextual variants are non-exhaustive.

## T5. Recipes and universal Exact

`R` is the existing seven-mode signed quantizer at the instruction's
`SettlementRounding`, retaining wide results. `N` is par nominal, `K_i` the
source coupon per $1,000, `P1000` the published accrued amount per $1,000,
and `C_p`/`P` each admissible real value of the clean/dirty price evidence
per 100.

```text
SourceCouponPer1000:      cash = R(N*K_i/1000)
PublishedAccruedPer1000:  cash = R(N*P1000/1000)
DirtyPricePer100:         cash = R(N*P/100)
CleanPricePer100AndPublishedAccruedPer1000:
    capital = N*C_p/100;  accrued = N*P1000/1000
    RoundTotal:             cash = R(capital + accrued)
    RoundComponentsThenSum: cash = exact_sum(R(capital), R(accrued))
```

Source values are exact points, so coupon and accrued images are always
points; their cash is `>= 0`. Price recipes inherit the gilt C4 semantics
unchanged:

* every non-Exact mode is monotone;
* a point image under `Exact` uses rational divisibility and fails `Inexact`;
* a nonpoint image under `Exact` returns `ExactStageUnresolved` at that stage,
  even when both endpoints are multiples;
* there is no extra quantizer after component rounding, and no rounding of
  `P1000` beyond its publication stage.

## T6. Stages, endpoints and order

| Stage | Kind | Rounding | Can fail |
|---|---|---|---|
| `daily decimal` | Internal publication (NineDecimal) | `RN9_T` | Capacity only |
| `accrued publication` | Internal publication | `RN5_T` | Capacity only |
| `raw coupon`, `source coupon` | Instrument views | Caller quantum/mode | `CashArithmetic` |
| `source accrued` | AI instruction view | Caller quantum/mode | `CashArithmetic` |
| `cash coupon`, `cash accrued`, `cash capital`, `cash total`, `cash net` | Cash stages | Instruction `SettlementRounding` | `CashArithmetic`, Unresolved |

Publication stages always use nearest rounding, so they can never raise
`Inexact`. Their exact results stay wide and private, so they never raise
`OutOfRange`. Their names appear only in capacity errors. Views and cash
stages use the gilt meaning of endpoint labels `point`, `lower` and `upper`.
ExactRatio has no daily quantizer and cannot raise a `round daily` capacity
failure. Instrument raw/source first-cache construction uses respectively
`raw coupon`/`source coupon`, operation `derive source`, index Some(0).
Instrument NineDecimal daily work uses `daily decimal`/`round daily` at
Some(0); plan daily work has index None. Plan total publication uses
`accrued publication`/`publish accrued` at None. The separate IEEE AI caches
use main N6's `raw source accrued` and `published accrued cache` phases;
these are not public quantized-view stages.

Construction order:

1. `coupon_cash` checks `InvalidCouponIndex`, then `NonPositiveNominal`.
2. The plan factories check `NonPositiveNominal` first.
3. The two coupon views check `InvalidCouponIndex`, then the view stage.
4. Factories copy the cached source and publication values into a compact
   snapshot. They allocate nothing and run no date query.

Evaluation follows gilt C5 with these stage names:

* coupon uses `cash coupon`, accrued uses `cash accrued`, dirty and total
  invoices use `cash total`;
* a component invoice completes `cash capital`, including its Exact test,
  before point `cash accrued`, then exact-sums at `cash net`;
* only final amounts are admitted, lower then upper;
* net cancellation cannot waive an earlier Exact failure.

## T7. Errors

The cash surface uses four of root's 24 `TreasuryError` variants. All are
`#[non_exhaustive]` and contextual, with stable kinds:

| Variant and fields | Kind | `source()` |
|---|---|---|
| `InvalidCouponIndex { index: usize, count: usize }` | `treasury.invalid_coupon_index` | None |
| `NonPositiveNominal { nominal: DecimalAmount }` | `treasury.nonpositive_nominal` | None |
| `CashArithmetic { stage: &'static str, endpoint: &'static str, cause: AmountError }` | `treasury.cash_arithmetic` | `Some(cause)` |
| `InternalCapacity { index: Option<usize>, phase: &'static str, operation: &'static str }` | `treasury.internal_capacity` | None |

The cause of `CashArithmetic` is only `AmountError::Inexact { operation:
"quantize" }` or `OutOfRange { operation, negative }`. Its operation is
`quantize`, except final `cash net` admission, which uses `sum`.
`ResultScale` is unreachable because quantum scales are at most 38.

Capacity phases are the stage names of T6. Capacity operations are `derive
source`, `round daily`, `publish accrued`, `decode evidence`, `scale cash`,
`combine cash`, `cash rounding`, `exact domain` and `normalize cash`. The
index is `Some(coupon ordinal)` for coupon work and None otherwise. Every
capacity error is proof-excluded by T9. Root `Error::Treasury` exposes the
exact error, followed by its `AmountError` cause.

## T8. Identity, provenance and guarantees

`TreasuryCashInstruction`, `TreasuryCashDecision` and `TreasuryCashReceipt`
are compact owned `Copy + Eq + Hash`. An instruction retains:

* recipe and source policy;
* C, D, I, F, M, first period, EOM and `(Q0, Q1)`;
* for coupons: the ordinal and its date; `(r1, s1)` only for Short/Long
  ordinal0, with no source ratio on regular coupons;
* for plan recipes: S and the settlement quasi pair; accrued/invoice recipes
  additionally retain the next actual coupon ordinal/date and accrued pieces,
  while the dirty-only recipe has neither coupon nor source-ratio fields;
* the derived exact source value;
* nominal, rounding, evidence and aggregation.

Unrelated quasi dates are not retained, and no instrument is cloned.
Equality is complete snapshot equality; equal cash does not erase differing
provenance. A receipt adds the final amount and guarantee. A Determined
decision includes that receipt. QuantumAmbiguous and ExactStageUnresolved
compare/hash only their listed bounds or stage and can equal diagnostics
from a different instruction; they are not standalone provenance records.
The caller retains the evaluated instruction for those outcomes.

Coupon and accrued receipts return `None` from `price_evidence_guarantee`,
because their provenance is the retained source recipe. Price recipes label
**only** the price ingredient:

* a declared interval gives `UniqueForDeclaredEnclosure`;
* a supplied binary64 gives `ForSuppliedBinary64`;
* a supplied decimal gives `ForSuppliedDecimals`.

No old `SettlementReceipt` is fabricated, and the label never claims that a
price equals or encloses the model's ideal price.

## T9. Width proof

All arithmetic is exact integer quotient/remainder work. Rounding compares the
remainder `r` with `d − r` and never doubles a remainder. Decimal powers are
combined into one exponent before any division. Final coefficients satisfy
`count·q <= num + q` because every denominator is at least the quantum
coefficient `q`. Range is checked only on the canonical final i128 value.
No intermediate is capped, and no i128 or float gate is used.
The exact caches and snapshot source values use main E's five-word Scalar;
the forms in main E's single table are the width contract: selected coupon
X/(`10^dscale*s_part`) and the published5 count. The8-word arithmetic path widens
those caches before fused cash/view work, without a public amount conversion.

**Source and view path: 8 limbs, 512 bits.** Write nominal
`N=n/10^Nscale`, quantum `q/10^Qscale`, and the selected source coupon per
$1,000 as main E's `X/(s_part*10^dscale)`. P_count is the published5 count,
so P1000=P_count/10^5. The following are exact integer ratios whose rounded
quotients are quantum counts (daily/publication rows are publication counts):

```text
daily:        5*Ccoef*10^max(9-Cscale,0) / (s*10^max(Cscale-9,0))
publication:  5*Ccoef*Σt_p*Π_{j≠p}s_j*10^max(5-Cscale,0)
                  / (Πs_p*10^max(Cscale-5,0))                           [ExactRatio]
              Σ K_p·t_p / 10^4                                          [NineDecimal]
coupon cash:  n*X*10^max(E,0) / (s_part*q*10^max(-E,0)),  E=Qscale-Nscale-3-dscale
accrued cash: n*P_count*10^max(E,0) / (q*10^max(-E,0)),   E=Qscale-Nscale-8
coupon view: X*10^max(E,0) / (s_part*q*10^max(-E,0)),     E=Qscale-1-dscale
AI view:      P_count*10^max(E,0) / (q*10^max(-E,0)),     E=Qscale-6
```

The raw coupon view uses the ExactRatio X/dscale/s_part form from main E
divided by10, regardless of the selected daily basis. Equivalently it
quantizes the exact raw cache per100. The X/10 form's quantization numerator/
denominator need at most261/265bits; the per100 raw-cache form needs at most
262/262bits. Both fit the stated265-bit view bound.

**Price path: 40 limbs, 2,560 bits.** Endpoints are decoded onto the shared
basis `R0 = 2^1074·5^38`. Every finite binary64 value and every decimal of
scale at most 38 is an integer multiple of `1/R0`, and so is `P1000`, since
`10^5` divides `R0`. Unlike gilt, no day factor enters the basis because cash
uses the published five-place value. Define the signed endpoint integer
B_price=R0*price_per100. The invoice ratio is
`n*abs(10*B_price+P_count*(R0/10^5))*10^max(E,0)/(q*R0*10^max(-E,0))`,
E=Qscale-Nscale-3. Capital/dirty use
`n*abs(B_price)*10^max(E,0)/(q*R0*10^max(-E,0))`, E=Qscale-Nscale-2.
Only endpoint arithmetic uses R0; nonpoint declared interiors remain the
continuous real sets of T5, not enumerated R0 lattice points.

The bound check computes conservative operand upper bounds exactly at every
scale combination. Monotonicity bounds smaller coefficients and day counts;
the robust t=s/two-full-piece and round-up allowances include a superset of
actual lifecycle states. It does not enumerate all financial instruments or
prove each reported maximum is attained by an admitted one. It covers:

* coefficients below `2^127` (signed decimal evidence up to `2^127`);
* every scale 0..38 for C, nominal, quantum and evidence;
* all finite binary64 values;
* both bases and both tie rules;
* one or two accrued pieces, with `1 <= t <= s <= 184`.

| Path | Operand upper bound (bits) | Width | Headroom |
|---|---:|---:|---:|
| Daily K / publication numerators / published count | 160 / 162 / 147 | 512 | — |
| Coupon cash numerator / denominator | 382 / **397** | 512 | 115 |
| Accrued cash numerator / denominator | 374 / 280 | 512 | 138 |
| Coupon / accrued views | 265 / 254 | 512 | 247 |
| Price endpoint on `R0` / invoice signed sum | 2187 / 2190 | 2560 | — |
| Capital, dirty, invoice numerator / denominator | 2433 / 1426 | 2560 | 127 |
| Component net coefficient | **2434** | 2560 | 126 |

These exactly computed upper bounds are not floating estimates. They hold
independently of `181 <= s` and of the tie rule. Ceil/count allowances may be
strictly larger than an actual nearest-rounded result.

## T10. Resources and work

Factories, views, evaluation and inspectors allocate no heap and perform no
date query, calendar lookup, callback, I/O, clock access, global state or
`unsafe`. Each evaluation has at most one source point, two price endpoints,
two component quantizers and one terminal sum. Universal-Exact checks are
constant structural tests. Division loops are bounded by 512 or 2,560 bit
steps.

The logical numeric storage budget is at most16 simultaneous40-limb values
(5,120bytes) and12 eight-limb values (768bytes) across one call chain. Count
explicit by-value callee copies while their caller values remain live,
including Wide division's quotient/remainder/alignment work. The
implementation review must census its actual temporaries against this budget.
This is not a compiler stack-frame guarantee. The plan's single bundle
reservation and the instrument's owned `Q` belong to the main contract and do
not extend to cash.

## T11. Evidence required before acceptance

Independent integer or `Fraction` oracles must be authored and frozen from the
adopted contract before Treasury Rust is written. All exact cash checks are
TOL-E0. They must include:

* **Source tables:** all 640 Table 2 cells under `NineDecimal` with both tie
  rules (no ties), and general-C daily ties.
* **Printed anchors:** every I.A/I.D anchor of T3 as full recipes, under both
  bases and both tie rules.
* **Basis discriminators:** the I.A.3 coupon at `1e-9` Exact, plus the 270
  (per rule) and 350 (either rule) basis-changing accrued cells, 584 raw and
  288 staged accrued ties.
* **Accrued lifecycle:** long-first two-piece accrued interest, non-reset at
  `Q1`, reset at `F`, zero at `S = D`, `D < I`, the I > Q[1] long first,
  and total versus component accrued differences.
* **Nominal and scale extremes:** non-$1,000 nominals, minimum and maximum
  nominal/quantum/C over all 39 scales, signed final i128 limits including
  `i128::MIN`.
* **Price evidence:** decimal, binary and declared evidence; adjacent floats;
  both zero signs; subnormal and MAX endpoints; point Inexact versus
  continuous unresolved; capital before accrued; net cancellation after
  component rounding; ambiguity; guarantee labels.
* **Errors and identity:** precedence, all stage and endpoint strings, root
  wrapper chains, Eq/Hash distinctions.
* **Width extremes:** valid inputs only, without fabricated private states.

## T12. Root integration decisions for the main draft

* The four variants are included once in the main draft's 24-variant
  `TreasuryError`; the public census is 14 Treasury types,126 total.
* The source identifier is exactly the string in T4; the four recipe
  identifiers are exactly the literal strings in its table, without an
  implicit prefix/suffix or parser.
* Plan construction retains the exact published count and pieces. Factories
  copy them; no publication is repeated at the factory or cash evaluation.
* Every local extension in T1 is required in public documentation. Main and
  cash drafts must be independently reviewed together before adoption.

## Reproduction of the width and anchor check

```sh
python3 -I /private/tmp/sabiquant-treasury-cash-width-check-v1/check.py \
  --xml /private/tmp/sabiquant-treasury-source-refresh-v1/part356-2026-10-16.xml \
  --report /private/tmp/sabiquant-treasury-cash-width-check-reproduction.json
```

The stdlib check (SHA-256
`61179fc3b36374dcb9e4828a0bfb06868fddf79cfc223e8fd8a013f861055f1b`) ran on
Python 3.9.6, macOS arm64, in about 20 seconds and exited zero. It wrote
`/private/tmp/sabiquant-treasury-cash-width-check-v1/report.json` (SHA-256
`cbcee6ee6a4238eeaafdcfa0f9d630b219fb21ce41aa07f8ab3833f477a76f39`). It
refuses other XML bytes and existing report paths. It is draft evidence, not
a frozen oracle or implementation test.
