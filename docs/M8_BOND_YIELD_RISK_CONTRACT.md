# Local generic bond yield/risk contract

| Field | Value |
|---|---|
| Status | **Adopted for local implementation; frozen** |
| Baseline | Local `65ff67e`, after the M8 phase C settlement boundary and contextual bond oracle tranche |
| Date | 2026-10-17, Asia/Bangkok |
| Design inputs | [Complete proposal](M8_BOND_YIELD_RISK_DESIGN.md), [independent mathematical review](M8_BOND_YIELD_RISK_DESIGN_REVIEW.md), [contract draft](M8_BOND_YIELD_RISK_CONTRACT_DRAFT.md), [bond core contract](M8_BOND_CORE_CONTRACT.md), [M6 solver contract](M6_SOLVER_RESEARCH.md) |
| Decision scope | Complete generic prepared-plan yield-to-maturity inversion and all six risk views together; no owner, market, milestone, platform or publication acceptance |

Adopted on 2026-10-17 under the owner's continuing authority for autonomous
local engineering. Root reviewed the complete proposal, the independent
pre-adoption mathematical review (`docs/M8_BOND_YIELD_RISK_DESIGN_REVIEW.md`),
and the complete contract draft (`docs/M8_BOND_YIELD_RISK_CONTRACT_DRAFT.md`),
freezing every signature, result field, error category and stage into one
authoritative local contract. It introduces no guess, default tolerance,
default budget, automatic bracket expansion, partial risk implementation or
named-market convention. Authoritative `FR-YR-01..16` rows are adopted in the
requirements register; all earlier contracts, generators and fixtures remain
frozen.

The generic model retains buyer-entitled prepared binary64 amounts and direct
settlement-to-payment binary64 times from the existing plan. Its yield is an
**annual decimal** coordinate: `.01` is one percentage point. No time, amount,
entitlement, accrued adjustment or hybrid branch changes while yield varies.
DMO quasi-period powers, Treasury fractional-first prefixes, Canadian/named
models, legal calendars, analytical enclosures, financial acceptance and full
M8 acceptance remain separate required work. The [settlement boundary](M8_SETTLEMENT_CONTRACT.md)
has its own units, policies and conditional guarantees; this contract supplies
no implicit conversion into a settlement certificate.

## 1. Complete public surface

All additions belong in `bonds`. Four new public types are adopted:
`BondYieldReport`, `BondYieldSensitivity`, `BondRisk` and `BondYieldError`.
Existing `BondPricingPlan` gains both operations below. No new public callback,
solver, wide-number, factor-class, risk-measure or workspace type is introduced.

```text
BondPricingPlan::yield_from_price(&self, target:BondPrice,
    bracket:RootBracket, options:SolverOptions)
    -> Result<BondYieldReport,BondYieldError>
BondPricingPlan::risk(&self, yield_rate:f64) -> Result<BondRisk,BondError>

BondYieldSensitivity::{MathematicallyStrictlyDecreasing,Uncertified}

BondYieldReport::target(self) -> BondPrice
BondYieldReport::converted_target_amount(self) -> f64
BondYieldReport::dirty_target_amount(self) -> f64
BondYieldReport::dirty_amount(self) -> f64
BondYieldReport::accrued_amount(self) -> f64
BondYieldReport::face(self) -> f64
BondYieldReport::settlement(self) -> BondSettlement
BondYieldReport::compounding(self) -> Compounding
BondYieldReport::yield_rate(self) -> f64
BondYieldReport::initial_bracket(self) -> RootBracket
BondYieldReport::options(self) -> SolverOptions
BondYieldReport::solve_report(self) -> SolveReport
BondYieldReport::residual_scale_exponent(self) -> u8
BondYieldReport::sensitivity(self) -> BondYieldSensitivity
BondYieldReport::amount_residual(self) -> Result<f64,BondError>
BondYieldReport::clean_amount(self) -> Result<f64,BondError>
BondYieldReport::dirty_price(self) -> Result<BondPrice,BondError>
BondYieldReport::clean_price(self) -> Result<BondPrice,BondError>

BondRisk::yield_rate(&self) -> f64
BondRisk::settlement(&self) -> BondSettlement
BondRisk::face(&self) -> f64
BondRisk::compounding(&self) -> Compounding
BondRisk::dirty_amount(&self) -> f64
BondRisk::price_derivative(&self) -> Result<f64,BondError>
BondRisk::price_second_derivative(&self) -> Result<f64,BondError>
BondRisk::time_weighted_amount(&self) -> Result<f64,BondError>
BondRisk::modified_duration(&self) -> Result<f64,BondError>
BondRisk::macaulay_duration(&self) -> Result<f64,BondError>
BondRisk::convexity(&self) -> Result<f64,BondError>
BondYieldError::kind(&self) -> &'static str
```

Struct fields are private. New enums and contextual variants are
non-exhaustive. All four types are immutable, `Debug`, `Clone`, `PartialEq`,
`Send+Sync+'static`. `BondYieldReport` is `Copy`; `BondYieldSensitivity` is
additionally `Copy+Eq+Hash`. `BondRisk` is not `Copy` and owns its retained
fixed-capacity signed sums. Float-containing report/risk/error types have no
`Eq` or `Hash`. There is no `Default`, `Ord`/`PartialOrd`, implicit numeric
conversion or arithmetic operator. Sensitivity has no parser, `Display` or
identifier registry: it is an outcome classification, not a user-selected
policy. `BondYieldError` implements `std::error::Error` and `Display`; wording
is not stable. Risk `Debug` shows financial metadata without dumping private
accumulator arrays. `Clone` retains ordinary stack/resource behavior; no
fallible-allocation claim is attached to `Clone`.

Reports are owned: no reference to a mutable workspace or large cloned ICMA
context is retained. Compounding is copied; the original plan remains the
authority for its source dates/convention/horizon preparation. This tranche
adds no convenience `Bond` wrapper: callers explicitly prepare once, then use
both complete operations. Existing `Bond::evaluate` and `plan.evaluate` retain
their accepted behavior.

The public catalog is **101**, from current 97 plus four. Section 8 adds six
`BondYieldError` variants and two `BondError` variants, so the domain catalog
is **139**, from 131 plus eight. Root adds `Error::BondYield` with transparent
`kind`/`Display`/`source` and `From<BondYieldError>`; root wrappers are not
counted again as domain variants.

## 2. Target and computed callback

Write `P(y)=sum A_i*d(y,t_i)` for the mathematical price using the exact real
values of prepared binary64 amounts/times. Existing computed dirty price
`Dhat(y)` is the exact reduction of independently rounded accepted binary64 PV
terms, followed by its final binary64 rounding. This distinction remains
visible in numerical claims; analytic derivatives below are derivatives of `P`,
not of the discontinuous floating program.

The inversion target is resolved once, in this order, before allocation or
trial work:

1. Retain original target/kind. `BondPrice` is already finite/canonical.
2. Convert its per-100 value with existing single-rounding
   `amount_from_price_per_100(value,face)`; retain the finite converted amount.
3. Dirty target: `T` is that amount. Clean target: `T` is one existing `exact_sum`
   reduction of `[converted_amount,cached_accrued_amount]`. Retain `T`.
4. Select the residual exponent once from `T` as specified below.

Target conversion/shift errors precede workspace, classification and every
yield quote/factor error. No price/amount round trip is assumed exact. Negative
and zero targets remain valid. Accrued amount is never recomputed per trial.

A shared private dirty-only evaluator validates the quote, traverses actual
flows in original order, computes each existing accepted PV, and performs the
existing dirty reduction. It does not eagerly form clean amount or a per-100
quote. The old `evaluate` method calls that helper and its unchanged clean
reduction; dirty inversion/risk do not fail on an unused clean/quote view.

## 3. Exact fixed-scale residual

Let `MIN_SUB=2^-1074` and `MAX=2^1024-2^971`. Select `s=0` when `abs(T)<2^970`,
otherwise `s=1`, including equality and both signs. Every callback returns

```text
R(y) = RN_even((Dhat(y)-T)*2^-s).
```

Decode both finite binary64 operands onto the exact `MIN_SUB` integer basis,
subtract as signed magnitudes, retain cancellation, then adjust the **output
binary exponent** by `-s` and round once. Never admit the unscaled difference
first; never halve the two operands separately. Canonical zero is positive.

For `s=0` the magnitude is below overflow midpoint `MAX+2^970`. A nonzero binary
operand difference is an integer multiple of `MIN_SUB`, so cannot round to zero.
For `s=1` the magnitude is at most `MAX`; the smallest nonzero difference beside a
target of magnitude at least `2^970` is at least `2^917`, so halving cannot erase it.
Thus valid residuals are finite, preserve sign, and equal zero iff `Dhat==T`.
No epsilon/near-zero criterion or trial-dependent scale is used.

A signed difference is below `2^1025`, hence its `MIN_SUB` coefficient is below
`2^2099` (at most 2099 bits). **34 u64 limbs = 2176 bits** suffice for each unsigned
scratch magnitude, including sum/carry before cancellation. The exponent
adjustment is stored separately: basis coefficient need not be truncated or
shifted right when `s=1` is chosen. The same width supports section 5 constants.
Capacity/rounding guards return typed errors, not panics or numeric sentinels.

`SolveReport.residual` is `R`, not an amount-unit residual. The report retains `s`,
cached `Dhat` and `T`. Its independently fallible `amount_residual` computes
`RN_even(Dhat-T)` directly from those cached operands, with `s=0` and ordinary
signed range errors. Overflow here does not invalidate a successful solve.

## 4. Shared fallible Brent, payload and exact budget

Reuse one private Brent algorithm with a callback returning
`Result<(finite_residual,dirty_payload),BondError>`. Private generic failure
separates `SolverError` from callback `{stage,point,cause,evaluations}`. The
existing public `brent(FnMut(f64)->f64)` is its `Infallible`/unit-payload lift;
no public generic fallible callback surface is added.

Each endpoint/history entry owns its associated payload. Coordinate/value
replacement, endpoint permutation and best/history updates move all three
together. A successful `BondYieldReport` takes `Dhat` from the selected cached
point. There is **no final uncounted repricing** and no last-result-only latch.
Repeated transcendental calls are not assumed to reproduce identical bits.

Preserve every M6 public count, result, error, safeguard and exact callback
trace. This includes lower-first evaluation, one-call lower-zero success,
upper-zero success, opposite signs without multiplication, retained
noncollapsed bracket, lower endpoint on equal absolute residual, fallback
midpoints, two-step interpolation history, and precise full-width/overflow-half
tolerance operation order. No residual-only stopping or hidden epsilon floor
is added. `PrecisionExhausted` remains distinct from tolerance success.

The shared cap is checked before each actual callback. Increment function
count on invocation, including a callback returning `Err`; derivative count is
always `0` in this API. Discarded arithmetic proposals have no callback cost.
Callback failure retains attempted M6 `EvaluationStage`, canonical finite point,
actual counts and exact `BondError`. No failed trial is silently discarded.
No NaN/infinity, `catch_unwind` or global latch encodes a financial failure.

Only queried coordinates undergo quote/factor validation. A lower exact-zero
success does not validate upper. A one-call cap can succeed there; otherwise
it stops before upper. `BracketTolerance` returns a cached endpoint and all
M6 qualifications: computed zero/small bracket prove neither continuity,
mathematical existence, accuracy nor uniqueness. Solver `NotBracketed` is not
a no-root certificate for signed/tangential models. All M6 suites are required
after refactoring; bisection/Newton behavior also remains unchanged if any
shared helper is affected.

## 5. Classification and mathematically constant plans

After target resolution, reserve one PV workspace of the bounded entitled
flow count, then one sensitivity-key vector of that count, in that order.
Both reservations use `try_reserve`; no per-trial vector allocation occurs.
Classify using immutable prepared data; never sort/group/skip actual pricing
flows. Keys contain active fixed-time branch, periodic count where relevant,
canonical positive horizon bits, magnitude bits and sign. Sort deterministically
in `O(n log n)`. Amount-zero terms need no key, but still retain all real pricing
factor/product checks.

The mathematical-constant certificate succeeds iff every nonzero positive-time
amount can be paired with an opposite **equal magnitude** in the identical
factor class; all remaining nonzero terms have time `0`. This is sufficient and
deliberately incomplete: an unpaired class sum of zero, a local zero slope or
a floating match does not gain the certificate. Fixed-time branch identity
is selected through the shared M4 helper, including its exact rounded-time
hybrid threshold, not a duplicated almost-equal comparison.

For a certified constant, accumulate original time-0 amounts exactly on the
`MIN_SUB` basis in separate positive/negative 34-limb arrays and subtract once.
Each magnitude is below `2^1024`, and `Bond::MAX_FLOWS=3,652,059<2^22`, so each
unsigned accumulator coefficient is below `2^2120` (at most 2120 bits), safely
inside 2176. No early binary64 admission tests the mathematical constant `C`.

Then perform **one normally budgeted lower-endpoint dirty evaluation**, with
all original quote/factor/product/reduction checks and full workspace traversal.
A failure is `Callback` with `LowerEndpoint` and function count `1`, derivative `0`.
Only after success compare exact `C` to exact binary64 `T`:

- equal: `NoYieldSensitivity`;
- unequal: `ConstantPriceMismatch`.

Both errors carry initial bracket, `T`, observed lower `Dhat`, `constant_amount`
diagnostic and `counts(function=1,derivative=0)`. The diagnostic is one
`RN_even(C)`, canonical positive zero; `Some` iff this rounding is finite and
does not erase a nonzero value. Otherwise `None`. No signed-saturation or `MAX`
clipping is permitted. `C` is a `MIN_SUB` integer, so only overflow can make this
diagnostic absent in this certificate. For a match it is necessarily `Some(T)`.
The exact comparison does not use that diagnostic or observed `Dhat`.

These errors describe mathematical identifiability, not constant computed
callback bits or admissibility of unqueried upper. Opposite factor pairs
cancel in the real model; independent elementary observations retain their
existing numerical qualification.

For nonconstant plans, classify `MathematicallyStrictlyDecreasing` only when
**all original entitled amounts are nonnegative** and at least one positive
amount has strictly positive time. Other signed plans are `Uncertified`, even
when a stronger symbolic proof might exist. The former means conditional
mathematical uniqueness on the common fixed-branch domain, not existence,
rounded monotonicity, accuracy or a slope lower bound. Classification does
not remove any required trial factor/product check.

## 6. Complete risk model, observation and numerical representation

Risk validates yield through existing M4 quote validation **before** its one
fallible PV-workspace reservation (`risk present value terms`). Periodic/hybrid
quotes require `y>-m` even for empty/time-0/active-Simple plans. Then traverse
every original entitled flow once, in order. A private shared factor/PV result
provides the active branch, accepted discount, accepted rounded PV `Vhat` and
the same fused Simple factor `q` when relevant. There is no second elementary
call to recover metadata and no large accrual-context clone or date query.

Preserve existing factor and nonzero PV representability, `NumericalBoundary`
guards and indexed failures. Zero amount still validates its factor. No later
signed cancellation rescues an invalid individual factor/PV. Risk terms are
formed only after that flow's price checks succeed. Accumulate all three
measures and dirty PVs; admit dirty reduction with existing `exact_sum` after
the traversal. A dirty reduction error precedes returning `BondRisk`. No raw
risk getter's representability is tested during construction.

For fixed prepared time `t` and annual decimal `y`:

| Branch | `d'/d` | `d''/d` |
|---|---|---|
| Simple | `-t/(1+y*t)` | `2*t^2/(1+y*t)^2` |
| Continuous | `-t` | `t^2` |
| Periodic `m` | `-t/(1+y/m)` | `t*(t+1/m)/(1+y/m)^2` |

At time `0` the three contributions are exactly `0` **after** ordinary quote/factor
validation. Yield `0` does not zero derivatives at positive times. Branch
selection stays fixed as `y` varies; yield derivatives do not differentiate a
date/time switch. `Macaulay=(1+y/m)*Modified` only for a common pure periodic
model; `Macaulay=Modified` for pure Continuous, with numerical comparison errors
retained. No such identity shortcut is used for Simple or mixed hybrids.

The numerical contributions use these explicit finite binary64 operands:
`q=RN_fma(1+y*t)` from the accepted Simple factor; `delta=RN(m+y)`;
`w=RN(t+RN(1/m))`. Positive `delta` is exact near the periodic boundary under
Sterbenz and finite over the validated domain; `w` is positive finite at `t>0`.
Use exact rational products of their represented binary values:

```text
moment           Vhat*t
Simple P'       -Vhat*t/q
Simple P''       2*Vhat*t*t/(q*q)
Continuous P'   -Vhat*t
Continuous P''   Vhat*t*t
Periodic P'     -Vhat*t*m/delta
Periodic P''     Vhat*t*w*m*m/(delta*delta)
```

Every nonzero term is rounded **once to 53 normalized binary significant bits,
nearest/ties-even, with an unrestricted retained binary exponent**. This is
not IEEE binary64 admission. Decode nonzero finite operands into unsigned
significands and exponents, form at most five numerator/two denominator
significands exactly in a private **five-u64-limb = 320-bit** scalar kernel,
locate the ratio binade exactly, align to 53 retained bits and keep quotient,
remainder and carry. A rounded significand `2^53` renormalizes to `2^52` and raises
its exponent by `1`. Sign is the exact rational sign; a true zero term is `+0`.
No nonzero term underflows or overflows this extended representation.

`Numerator<2^265`, `denominator<2^106`; exact binade correction and normalized
53-bit quotient alignment need at most 266 significant bits, below 320.
Remainder comparison uses `d-r`, not unchecked doubling. Powers such as the
Simple factor `2` adjust the retained exponent without adding a sixth numerator.
All failures of the proved private capacities are typed `InternalCapacity`.

Independent broad finite-factor bounds place an unrounded term strictly
between `2^-7418` and `2^7268`. Its 53-bit rounded value is below `2^7269` and has no
retained bit below `2^-7470`. Store it exactly on the common basis `2^-8192`.
The normalized significand/exponent representation therefore has nonzero
53-bit significand in `[2^52,2^53)` and exponent in `[-7470,7216]`, where value is
`significand*2^exponent`. Actual prepared financial times can be much smaller;
the proof deliberately does not rely on economically typical inputs.

For each of `P'`, `P''` and `moment` maintain independent positive/negative fixed
arrays of `WORDS=(15461+usize::BITS).div_ceil(64)` u64 words. On current 32/64-bit targets
`WORDS=243`: six scratch arrays total 11,664 bytes before small metadata. Each
term's basis coefficient is below `2^15461`; a slice has fewer than `2^usize::BITS`
terms, so accumulators fit. Subtract once after traversal and retain three
canonical signed sums in `BondRisk`. No per-term `f64` range admission or
order-dependent cancellation is introduced.

Raw getters round the retained signed `sum*2^-8192` once to IEEE binary64.
Normalized getters round the retained signed sum divided directly by the
finite observed dirty `Dhat`, never by a rounded raw getter:

```text
modified_duration = -extended_P_prime / Dhat
macaulay_duration =  extended_moment  / Dhat
convexity         =  extended_P_second / Dhat
```

These getter divisions use a fixed `WORDS`-wide unsigned helper. For a nonzero
dirty significand of at most 53 bits with exponent `[-1074,971]`, output alignment
at a binary64 unit exponent `[-1074,971]` needs denominator shifts at most 10134
bits, thus at most 10187 denominator bits. The retained numerator is below
the accumulator width. Quotient/remainder, nearest-even carry and final
normal/subnormal/overflow classification stay within `WORDS`; no arbitrary
precision retry or raw getter prerequisite is used.

If `Dhat` is either zero sign (construction canonicalizes to `+0`), each normalized
getter first returns `ZeroDirtyPrice` even when its numerator is `0`. Raw exact
zero/cancellation and a defined normalized zero return `+0`. Negative dirty
prices retain their denominator sign. Convexity has no factor `1/2`; that belongs
only to the Taylor expansion coefficient.

## 7. All independent getter failures and phases

Every six risk getter is separately usable; one failing view never mutates or
invalidates another. `BondRisk` may validly retain raw derivatives outside
binary64 while normalized values remain finite, or vice versa. For example,
Continuous `PV=MAX,t=2,y=0` has raw `-2*MAX` and `4*MAX` but `Modified=2,Convexity=4,
Macaulay=2`. `PV=MIN_SUB,t=.5,y=0` has nonzero raw values that round to zero, while
the corresponding normalized values `.5,.25,.5` remain finite.

| Getter | `BondError::Value` phase and `ValueError` quantity |
|---|---|
| `price_derivative` | `price derivative` |
| `price_second_derivative` | `price second derivative` |
| `time_weighted_amount` | `time weighted amount` |
| `modified_duration` | `modified duration` |
| `macaulay_duration` | `macaulay duration` |
| `convexity` | `convexity` |
| `BondYieldReport::amount_residual` | `yield amount residual` |

All such `Value` errors have `index=None`. A nonzero result rounding to signed
infinity gives `ValueError::ResultNotFinite{quantity,value:PositiveInfinity or
NegativeInfinity}`. A nonzero result rounding to signed zero gives
`ValueError::ResultOutOfDomain{quantity,value:+0 or -0,domain}`; positive
underflow has `Interval::greater_than(0)`, negative has `Interval::less_than(0)`.
Preserve that zero sign in the error field. Exact zero is success `+0`, not an
underflow error. Ordinary nearest-even IEEE overflow midpoint and subnormal
ties apply. These are finite rounded extended-value policies, not a claim of
correct rounding of ideal transcendental derivatives.

Report `clean_amount` uses existing `exact_sum([cached_Dhat,-cached_AI])`, with
`BondError::Sum` phase `clean price reduction`; it does not reprice.
`dirty_price` calls existing `BondPrice::from_amount(Dhat,face,Dirty)`.
`clean_price` calls `clean_amount` first, then `from_amount(...,Clean)`.
Their normalization failure retains the existing `price per 100` `BondError`
phase/`ValueError` quantity; no second custom quote-normalization policy exists.
All four report views are independently fallible and make zero callbacks.

## 8. Fully resolved errors and sources

Add two contextual non-exhaustive variants to existing `BondError`:

| Variant / kind | Fields and source |
|---|---|
| `ZeroDirtyPrice` / `bond.zero_dirty_price` | `measure:&'static str, yield_rate:f64`; `None` |
| `InternalCapacity` / `bond.internal_capacity` | `index:Option<usize>, phase:&'static str, operation:&'static str`; `None` |

`ZeroDirtyPrice` measure is one of `modified duration`, `macaulay duration`,
`convexity`; `yield_rate` is the retained canonical finite quote. `InternalCapacity`
is a defensive invariant failure, never an ordinary mathematical range error.
For term operations `index` is the original flow index, with `phase` one of
`price derivative`, `price second derivative`, `time weighted amount`.
For getters `index=None` and `phase` is their table label. Yield residual uses
`index=None`, `phase` `yield residual`; classification uses `index=None`, `phase`
`yield constant classification`. Its `operation` is one of `decode binary`,
`normalize rational`, `round scaled term`, `accumulate risk`, `subtract risk sums`,
`round risk value`, `normalize risk value`, `accumulate constant`, `compare constant`,
`round constant diagnostic`, `residual difference`, `round residual` as applicable.
Do not forge invalid private financial states merely to execute these guards.

`BondYieldError` has exactly six variants (contextual variants non-exhaustive):

| Variant / kind | Complete fields / typed source |
|---|---|
| `Target` / `bond_yield.target` | `target:BondPrice, phase:&'static str, cause:BondError`; `Some(cause)` |
| `Preparation` / `bond_yield.preparation` | `phase:&'static str, cause:BondError`; `Some(cause)` |
| `Solver` / `bond_yield.solver` | tuple `SolverError`; `Some(cause)` |
| `Callback` / `bond_yield.callback` | `stage:EvaluationStage, point:f64, evaluations:EvaluationCounts, cause:BondError`; `Some(cause)` |
| `NoYieldSensitivity` / `bond_yield.no_yield_sensitivity` | `bracket:RootBracket, dirty_target_amount:f64, observed_lower_dirty_amount:f64, constant_amount:Option<f64>, evaluations:EvaluationCounts`; `None` |
| `ConstantPriceMismatch` / `bond_yield.constant_price_mismatch` | same complete fields as `NoYieldSensitivity`; `None` |

`Target` phases are `target conversion` and `target clean shift`. The first
retains the exact existing quote-to-amount `BondError` (normally `Value`, phase
`bond amount`). The second retains `BondError::Sum{phase:"yield target shift",
cause:the existing exact_sum error}`. No source string or generic sentinel
replaces those typed causes.

`Preparation` phases are `yield present value terms`, `yield sensitivity keys`
and `yield constant classification`. The two reservation failures retain
`BondError::AllocationFailed` with that phase and requested entitled-flow
capacity; the classification phase retains a defensive `InternalCapacity` if
its proven scratch bounds fail. `Callback` causes retain ordinary quote/factor
`Rate`, dirty reduction `Sum` or defensive residual `InternalCapacity` exactly;
their existing original indices/phase/source chains are not flattened.

Both constant errors have function count `1`, derivative `0` after a successful
ordinary lower dirty evaluation; callback failure takes precedence. Their
input bracket is retained, not described as a verified numerical sign bracket.
Solver errors preserve their actual M6 counts/context; only `Callback` adds
typed financial cause. Source is always the immediate typed cause above,
then whatever existing source chain that cause carries. `Display` wording is
unstable; `kind`/fields/phase/measure/operation strings are stable.

## 9. Complete sequencing and work limits

Inversion: target conversion -> optional target clean shift -> fixed residual
scale -> PV workspace reservation -> classification-key reservation -> exact
classification/constant accumulation -> lower-first counted pricing -> constant
error or ordinary bounded Brent -> cached successful report. Only the normal
callback performs yield validation, so reservations/classification precede a
bad lower yield. Later and unqueried coordinates are not eagerly validated.

Risk: yield quote validation -> PV workspace reservation -> original-index
factor/PV/extended-term traversal -> dirty reduction -> exact accumulator
subtraction -> immutable `BondRisk`. Getter normalization checks dirty `0` before
any numerator special case. Getter range admission occurs only when requested.
No partial risk or solver report escapes a failed operation.

No repeated date/calendar/ICMA work is done by inversion or risk. Preparation
keeps its existing shared accrual-work ledger; these operations use only the
prepared times, amounts, face, settlement and cached AI. No per-trial allocation,
global cache, hidden mutable state, clock, environment read, I/O, unsafe or
new dependency is introduced. Classification is `O(n log n)`, a callback/risk
traversal `O(n)`, and actual callbacks `<=options.max_evaluations()`.
`MAX_FLOWS*u32::MAX<2^54`, so the finite maximum is explicit even when expensive.
Scalar loops are bounded by their fixed bit widths; error paths retain all
carries/borrows/remainders. `Clone`/`Debug` and optional identifier infrastructure
retain their existing ordinary resource qualifications; no forced OOM is used.

## 10. Adopted requirements and independent evidence

These `FR-YR-01..16` rows are authoritative for this complete local tranche and
are recorded in the requirements register:

| ID | Complete obligation | Required independent evidence |
|---|---|---|
| FR-YR-01 | Explicit prepared-plan annual decimal yield, fixed amounts/times/branch and complete API/metadata | All four new types, every accessor, both operations and all six risk views; no omitted measure/default/date inference |
| FR-YR-02 | One-time clean/dirty target conversion/AI shift, independent report quote/residual views | Signed/zero/extreme/rounding-sensitive targets, per100 round trips, conversion/shift errors and unused-clean overflow isolation |
| FR-YR-03 | Exact fixed-scale residual, no false zero/sign/overflow | Exact integer/Fraction oracle at threshold `2^970` and neighbors, every exponent field, signed `MIN_SUB`/`MAX`/opposite `MAX`, equal/adjacent values and output exponent handling |
| FR-YR-04 | Shared fallible Brent preserves all M6 behavior and payload identity | All existing solver suites/fixtures/traces, cap 1/2/boundaries, endpoint/trial errors, early zeros, midpoint/tolerance/precision safeguards, returned payload from a retained earlier endpoint |
| FR-YR-05 | Complete bounded generic inversion across all 32 compounding configurations | Closed-form one-payment controls and independent high-precision multi-payment roots; explicit signs/brackets; near-boundary negative yields and hybrid mixed branches |
| FR-YR-06 | Sufficient constant certification and qualified sensitivity | Empty/paid/time-0 plans, exact opposite same-factor pairs, unpaired zero-sum limits, exact `C` versus `T` with optional diagnostic, one-call errors/counts and unqueried invalid upper |
| FR-YR-07 | Signed multi-root/tangential/zero-price behavior | Annual `P=v^2-v`: targets `-3/16` roots `1/3` and `3`; target `-1/4` tangent at `1`; `y=0` raw `P'=-1,P''=4,M=1` and normalized errors; no root/uniqueness overclaim |
| FR-YR-08 | All analytic equations, signed dirty normalization and correct units | Independent real-model `P',P'',moment,Modified/Macaulay/Convexity`; all signs, `t=0`/`y=0`, no convexity half, qualified periodic/Continuous identities |
| FR-YR-09 | Exactly specified extended 53-bit term rounding and exact signed sums | Integer rational oracle for 320-bit products/division, every rounding boundary/parity/carry, exponents, signed zero, valid width extremes and cancellation |
| FR-YR-10 | Raw and normalized views are independently admitted | `MAX/t=2` and `MIN_SUB/t=.5` Continuous anchors; raw overflow/underflow with finite normalized results; normalized-only failures, negative/zero dirty and exact cancellation |
| FR-YR-11 | Prepared-context semantics and reuse work | Every scalar/contextual DCC, ICMA regular/stub/budget/coverage, ISDA termination, US EOM flags, plain/TARGET/joint/override Business252; zero query work per numerical reuse |
| FR-YR-12 | Existing quote/factor/PV domains and failure order remain | All frequencies and hybrid threshold/adjacent-time controls, `y>-m` even empty/Simple-selected hybrid, active Simple boundary/`NumericalBoundary`/invalid product before cancellation |
| FR-YR-13 | Fixed capacities and explicit bounded resources | Independent 34/5/WORDS width derivations, full valid carry/division/normalization extremes, no per-trial allocation/context clone, honest proof-excluded guards and callback accounting |
| FR-YR-14 | Six yield plus two bond error additions, exact typed context/root causes | All reachable fields/stages/counts/range signs/Intervals, 139 catalog, 101 public types, source/format writer tests and qualified excluded capacity/allocation paths |
| FR-YR-15 | Immutable deterministic reuse and regression preservation | Traits/threads, native debug/release, strict lint/fmt/public+private docs/policy, exact old M4/M6/M7/bond results and byte-identical old contracts/generators/fixtures |
| FR-YR-16 | Independent numerical evidence and honest complete local gate | Exact staged helper oracles, two independently justified precision real-model references, conditioned absolute errors, root-location qualification, fresh production coverage, actual extracted std-only package, native versus cross-target compilation scope |

Exact binary interpretation/residual/extended-term/sum/getter rounding profiles
use TOL-E0. Financial comparisons separately evaluate ideal formulas from
exact binary64 inputs using independently justified high precision at two
precisions; merely reproducing rounded M4 formulas in Python binary64 or
checking signs/inverse round trips is insufficient. Generators must provide
their precision/domain/conditioning derivations before expectations freeze.

For PV seed error `E_V` and multiplier error `E_g`, propagate a contribution
allowance `abs(g_hat)*E_V+(abs(Vhat)+E_V)*E_g+term_rounding_error` on gross
terms. For numerator error `E_N`, observed dirty `Dhat` and price error `E_P`, when
`abs(Dhat)>E_P` a usable normalization allowance is

```text
E_N/abs(Dhat)
 + (abs(Nhat)+E_N)*E_P/(abs(Dhat)*(abs(Dhat)-E_P))
 + final_ratio_rounding_error.
```

Calculate bounds with scaling; an overflowing/informative-failure bound is
reported as such, never used to skip every difficult numeric assertion.
When `abs(Dhat)<=E_P` retain gross-conditioned absolute price/numerator checks,
report lack of a useful normalized certificate, and still include independent
exact signed/cancellation/range anchors. Empirical M4 allowances are comparison
criteria, not a rigorous platform libm theorem or settlement enclosure.

Finite differences are supplementary: fixed dates/times and yield-domain,
documented `h/h2/h4` sequence, high-precision truncation convergence and explicit
binary repricing noise amplification. No claimed convergence below machine
resolution or through a time-branch switch. Printed ISMA rounded-intermediate
examples keep their source-precision classification. Root coordinate accuracy
combines M6 tolerance with independent callback/conditioning evidence; flat or
tangential models receive no invented slope guarantee.

The full generic tranche, all measures and all error/result/resource semantics
must be implemented and validated together. Completion of this contract,
generic inversion/risk, or local green checks does not complete named
market conventions, analytical financial enclosure producers, settlement
financial acceptance, full M8, or owner/platform/release gates.

