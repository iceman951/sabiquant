# Generic bond yield and risk design proposal

Status: **proposal only**, prepared on 2026-10-16 against local bond core
`5ea07cd`. This document adopts no contract, changes no production code, and
claims no tests for its proposed algorithms. It develops the remaining generic
yield/risk portion of [the bond API proposal](M8_BOND_API_DESIGN.md) and resolves
the cancellation/scaling obligations in [the design review](M8_DESIGN_REVIEW.md).
The adopted [bond core contract](M8_BOND_CORE_CONTRACT.md) remains unchanged.
The parent is separately auditing its evidence; this proposal does not replace
that work or certify full M8 or NI-01 settlement acceptance.

Inspection covered current `src/bonds.rs`, `src/bonds/quote.rs`,
`src/math/solver.rs`, `src/math/sum.rs`, `src/rates.rs`,
[M6 solver research/contract](M6_SOLVER_RESEARCH.md), and
[numerical policy](NUMERICAL_CORRECTNESS.md). Mathematical duration definitions
use the existing [historical ISMA source ledger](M8_ICMA_SOURCE_REVIEW.md);
that source's original inspection and precision qualifications remain intact.
No QuantLib implementation was inspected. All work and proposed validation
stay local; no push, publication or remote writes are involved.

## 1. Complete proposed surface

Extend the prepared generic direct-horizon plan rather than rebuilding dates
or instrument terms inside numerical callbacks. Names below are proposals:

```text
BondPricingPlan::yield_from_price(target: BondPrice,
    bracket: RootBracket, options: SolverOptions)
    -> Result<BondYieldReport, BondYieldError>
BondPricingPlan::risk(yield_rate: f64) -> Result<BondRisk, BondError>

BondYieldReport: original target/kind; converted target amount;
    dirty target amount; cached dirty amount; original bracket;
    SolveReport; residual_scale_exponent; sensitivity classification;
    fallible unscaled financial residual and repriced quote views
BondYieldSensitivity::{MathematicallyStrictlyDecreasing, Uncertified}
BondRisk: yield, settlement, face, dirty amount; independently fallible
    price_derivative(), price_second_derivative(), time_weighted_amount(),
    modified_duration(), macaulay_duration(), convexity()
```

New values have private fields, no `Default`, and are immutable `Send + Sync`.
Float-containing values use `PartialEq`, never `Eq` or `Hash`. `BondRisk` can
be `Clone` without being `Copy`: its private accumulators are larger than a
small scalar value. The sensitivity enum is non-exhaustive and contains no
claim that a root exists or that the rounded callback is strictly monotone.
Do not freeze an updated public type/error census until the actual contract
chooses the final declarations.

`RootBracket` and `SolverOptions` are the existing checked M6 types. No guess,
bracket expansion, default tolerance, default evaluation count, automatic
Newton selection or root-search restart is added. Brent is the complete first
bounded inversion method; analytic derivatives do not require that inversion
use Newton. This does not remove any requested risk measure.

The risk return value deliberately separates representability of each output.
A finite modified duration must remain obtainable when its unnormalized
derivative cannot be represented by binary64. Likewise a finite normalized
measure can survive an unnormalized nonzero value that rounds to zero.
Price-only evaluation continues to avoid all risk computation.

## 2. Yield coordinate, model and target semantics

All derivatives and inversion coordinates are **annual decimal yield** `y`.
A change of `0.01` means one percentage point, not one basis point. Amounts,
entitlement and prepared binary64 horizons `t_i` are held fixed while `y`
varies. The mathematical comparison model is

```text
P(y) = sum_i A_i d(y,t_i),
M(y) = sum_i t_i A_i d(y,t_i).
```

`A_i` are the bond's supplied/generated rounded binary64 amounts. `P` is the
real model; the existing computed dirty price `D_hat` reduces individually
rounded PV terms. Analytic derivatives approximate derivatives of `P`, not
derivatives of the discontinuous rounded program. Numerical risk views use
`D_hat` as their observed dirty denominator and account for its error when
compared with the real model. They do not silently define a new market model.

Resolve the target once, before any trial:

1. Retain the supplied finite signed `BondPrice`, including `Clean`/`Dirty`.
2. Convert its per-100 value to amount units with the adopted single-rounding
   `amount_from_price_per_100(target.value, face)`.
3. For dirty targets, that rounded amount is `T`. For clean targets, set
   `T = exact_sum([converted_clean_amount, cached_accrued_amount])` once.
4. Retain both the converted target and `T`. An unrepresentable conversion or
   target addition is a typed target error before any callback.

The callback solves computed dirty amount against this explicitly derived
dirty target. Per-100 and clean/dirty round trips need not be identities.
Do not add a second target tolerance to make a round trip look exact.

Factor/dirty evaluation needs a private workspace entry point that computes
dirty amount without eagerly calculating clean amount or normalized quotes.
Existing `BondPricingPlan::evaluate` can call it and then perform its unchanged
clean reduction. Dirty inversion and dirty-normalized risk should not fail
because an unused clean amount or per-100 quote overflows at an intermediate
trial. Final report quote/clean views remain independently fallible.

## 3. Overflow-safe residual with a fixed scale

A raw subtraction of two finite signed values can overflow although a valid
yield is bracketed. A complete fixed-scale remedy is available without a NaN
sentinel, a changing residual scale or a false zero near underflow.

Choose one exponent `s` from the converted dirty target, and retain it for
every callback in the solve:

```text
s = 0 when abs(T) < 2^970; otherwise s = 1.
R(y) = RN((D_hat(y) - T) * 2^(-s)).
```

Compute the exact difference of the two binary64 operands in a private signed
integer accumulator, adjust the output exponent, then round once. Do not first
admit the unscaled difference to binary64 or separately halve the operands.
Both can destroy information this contract retains.

Proof of finite range and preservation of sign/zero:

- The binary64 overflow midpoint is `MAX + 2^970`. For `s=0`, the exact
  magnitude is at most `MAX + abs(T)`, strictly below that midpoint. Its
  correctly rounded result is finite. A nonzero difference of binary64
  operands is an integer multiple of `MIN_SUB`, so cannot round to zero.
- For `s=1`, `abs(D_hat-T)/2 <= MAX`. If `abs(T) >= 2^970`, the smallest
  nonzero difference with another finite binary64 number is at least `2^917`
  (the predecessor spacing at `2^970`); farther or opposite-sign operands
  have larger differences. Halving therefore cannot create zero/underflow.
- Thus every accepted residual is finite, has the exact difference's sign,
  and is zero **iff the computed dirty amount equals `T`**. Canonical zero
  is positive. The scale depends only on `T`, never on a trial result.

This is a proposed exact binary-input residual profile, not yet implemented
or validated. The underlying `SolveReport::residual()` is the scaled residual.
Expose `residual_scale_exponent()` and a separately fallible amount-unit
residual view, computed from cached `D_hat` and `T`. An overflowing financial
residual does not invalidate an otherwise usable solver report. Original
unscaled residual interpolation paths/counts are not promised identical after
introducing this explicitly documented profile.

## 4. Shared fallible solver core and cached payload

Refactor one Brent algorithm into a **private** generic fallible core. Reuse
its existing interpolation, fallback, midpoint, endpoint ordering, bracket
test and budget rules. A practical internal boundary is

```text
try_brent(callback: FnMut(f64) -> Result<(f64, Payload), E>, bracket, options)
    -> Result<(SolveReport, Payload), SolveFailure<E>>

SolveFailure<E>::Solver(SolverError)
SolveFailure<E>::Callback { stage, point, cause:E, evaluations }
```

The callback payload for bonds is the cached dirty amount; the residual is
the first tuple item. Endpoint/state entries retain their corresponding
payloads. Returning the selected endpoint then needs no unbudgeted repricing
and no assumption that another elementary-function evaluation returns the same
bits. A last-result-only latch is insufficient: the best returned endpoint
can be an initial endpoint retained across many trials.

The existing public `brent(FnMut(f64)->f64)` is an infallible lift using
`Infallible` and `Payload=()`. Every existing public result, count, endpoint
short circuit and failure must remain unchanged. No public generic solver
API is required. If shared initialization/evaluation helpers also affect
bisection/Newton, require their entire current regression suites, not merely
one Brent square-root test.

Before each callback, check the shared cap; increment the function count when
actually invoking it, including an invocation returning `Err(E)`. Preserve
the typed callback cause, attempted stage, point and actual counts. A finite
`Ok` value follows ordinary solver logic; a nonfinite `Ok` remains ordinary
`SolverError::NonFiniteEvaluation`. Never return NaN/infinity to encode `E`,
discard an error after an apparent root, or use `catch_unwind` as a normal
financial error channel.

Preserve lower-endpoint-first behavior, one-call lower exact-zero success,
upper-endpoint zero, opposite-sign tests without multiplication, explicit
full-width/overflow-half tolerance, adjacent-float exhaustion, and no
residual-only stopping criterion. **Only actually evaluated coordinates are
validated.** A lower exact-zero return does not prove the unqueried upper
endpoint has valid factors. The earlier API sketch's simultaneous promise
of both-endpoint validation and this one-call rule must be resolved in favor
of this explicit rule, or replaced by a separately adopted eager-validation
contract with different counts. This proposal chooses the existing M6 rule.

## 5. Sensitivity, constants and multiple roots

For nonnegative buyer-entitled amounts with at least one positive amount at
strictly positive prepared `t`, all supported fixed-time branches give a
strictly decreasing **mathematical** dirty price in their common domain.
Record `MathematicallyStrictlyDecreasing`; this is a conditional uniqueness
property, not a proof of existence, platform accuracy, or monotonicity of
every rounded observation. Signed amounts remain supported with `Uncertified`.

Detect useful constant cases without altering original pricing semantics:

- Every nonzero term has `t=0`, including distinct civil dates collapsed by
  30/360; empty and fully ceased/detached plans are included.
- Within each identical positive-time factor class, pair every nonzero amount
  with an exactly opposite amount of the same magnitude. A factor class is
  the selected branch, its frequency where relevant, and exact prepared-time
  bits. Ignore amount zero for this certificate, after keeping its validation
  obligations in the real pricing traversal.
- If no positive-time amount remains unpaired, only time-zero constants
  remain in the mathematical model. Sum those constants exactly in a private
  signed accumulator. This is a sufficient mathematical certificate, not a
  symbolic proof covering all possible analytic identities or a certificate
  that every computed callback observation has identical bits.

Build a separate fallibly reserved vector of factor/magnitude/sign keys and
sort deterministically in `O(n log n)`, or prove an equivalent bounded method.
Do not sort, merge, group or skip the actual pricing flow vector. In general
`sum(A_i)=0` within a factor class does **not** establish
`sum(RN(A_i*d))=0`. Opposite equal pairs cancel in the real model and in
computed arithmetic when both use the same factor bits. The existing pricing
traversal evaluates each factor independently; Rust does not promise identical
elementary-function bits on separate calls, even within one execution. Thus
pairing does not establish a universally constant computed callback. Basic
Simple arithmetic and explicit zero-rate identity paths have stronger exact
guarantees; do not extend them to arbitrary elementary evaluations.

For a mathematically certified constant plan, perform one normally counted
lower-endpoint pricing evaluation, retaining the original factor/product
checks at every index. If it fails, preserve that callback error. Otherwise
compare the exact mathematical time-zero sum `C` with the exact binary value
of `T`, without first rounding `C`. Return `NoYieldSensitivity` when they
are equal, or `ConstantPriceMismatch` when they differ. These errors describe
the mathematical model's lack of an identifiable yield, even if floating
evaluation noise would produce scattered computed matches. Retain the
observed lower dirty amount separately; it is not used as proof that the
computed function is constant. A mismatch error can expose an optional rounded
`C` diagnostic (absent if unrepresentable), plus the finite target/observation.
The first case means every mathematically admissible yield has that target
price, rather than a unique zero yield. Neither case asserts all unqueried
bracket coordinates admit valid individual factors.

A concrete reachable signed fixed-bond test has annual coupons `-1`, `-1`
and final redemption `2`, with annual prepared times `1`, `2`. Under annual
periodic discounting, `v=1/(1+y)` and `P=v^2-v`:

- Target `-3/16` has two admissible yields, `1/3` and `3`, found in separate
  explicit sign brackets; neither gets a global uniqueness claim.
- Target `-1/4` has a tangential root at `y=1`; a same-sign bracket may return
  `NotBracketed`, which is not proof that no root exists.
- At `y=0`, dirty price is zero but `P'=-1`, `P''=4`, `M=1`. Raw derivatives
  are meaningful while normalized measures are undefined.
- One coupon `-100` and redemption `100` at the same positive horizon gives
  identically zero dirty price. A zero-horizon-only scan misses this case.

Computed callback zeros and bracket termination retain M6's qualifications.
An uncertainty in price divided by a small/zero slope can dominate root error;
a yield round trip is not an independent correctness oracle.

## 6. Analytic risk equations and domains

Use dirty value for every conventional normalization:

```text
modified_duration = -P'/P
convexity         =  P''/P
macaulay_duration =  M/P
```

Convexity has no factor `1/2`; that belongs in a Taylor price-change term.
Negative prices retain their sign in the denominator. A zero observed dirty
price returns a typed error from each normalized getter, even if raw
derivatives are finite. A raw zero derivative is valid and canonical positive
zero; do not infer unique yield or constant price from one local observation.

For each fixed `t`:

| Active branch | `d(y,t)` | `d'/d` | `d''/d` |
|---|---|---|---|
| Simple | `(1+y*t)^(-1)` | `-t/(1+y*t)` | `2*t^2/(1+y*t)^2` |
| Continuous | `exp(-y*t)` | `-t` | `t^2` |
| Periodic `m` | `(1+y/m)^(-m*t)` | `-t/(1+y/m)` | `t*(t+1/m)/(1+y/m)^2` |

Validate the yield through the existing M4 quote boundary first, including
`y > -m` for hybrids even if all selected branches are Simple or all times
are zero. Validate factors and nonzero PV products in original flow order.
All existing `NumericalBoundary` guards remain effective. Do not rescue an
invalid M4 factor or per-flow PV by later cancellation.

At `t=0`, derivative and time-weight terms are exactly zero after normal
yield/factor validation. At `y=0` and positive `t`, price may take an identity
path but derivatives generally are nonzero. No zero-yield risk shortcut is
permitted.

Hybrid branch selection is the exact existing rounded-time comparison for
each prepared flow, reused privately from `rates.rs`; do not duplicate an
almost-equivalent comparison. The branch is fixed as `y` varies. Its yield
derivatives are smooth within that branch's domain; this says nothing about
time/date differentiation through a switch. Test `t=RN(1/m)` and both adjacent
times with near-boundary negative yields. Named Treasury fractional-prefix
factors are not this whole-horizon switch.

For pure periodic common `m`, the mathematical identity is
`Macaulay=(1+y/m)*Modified`; for pure Continuous it is `Macaulay=Modified`.
These are qualified numerical comparisons after independent rounding. Neither
identity is universal for Simple or mixed hybrid branches.

## 7. Extended-range risk terms and normalization

Do not materialize `PV*t`, `PV*t*t`, inverse-base squares, raw derivative sums
or raw moment sums as binary64 prerequisites for normalized views. Their
range can be much greater than the final ratio's range.

One complete bounded implementation route is a private scaled-rational term
kernel plus signed fixed-point accumulators:

1. Evaluate each factor once through the existing shared M4 kernel, obtaining
   its discount and the rounded per-flow PV `V_hat`. Preserve exactly the
   current product/error policy. Reuse that same factor observation for risk;
   do not make a second transcendental call to recover price metadata.
2. For Simple retain `q=RN(1+y*t)` from the fused factor path. For Periodic
   form `delta=RN(m+y)` and `w=RN(t+RN(1/m))`. The first is positive under
   the validated periodic quote domain, exact in the important cancellation
   neighborhood by Sterbenz's condition. It is finite for finite `y` and
   current integer `1<=m<=365`. The second is positive finite for finite
   nonnegative `t`: adding at most `1` cannot overflow even at `MAX`.
3. Form these rational expressions from their finite binary64 operands,
   retaining exponents separately and rounding only the normalized term
   significand to 53 bits, nearest/ties-even:

```text
moment:      V_hat*t
Simple P':  -V_hat*t/q
Simple P'':  2*V_hat*t*t/(q*q)
Cont. P':   -V_hat*t
Cont. P'':   V_hat*t*t
Periodic P': -V_hat*t*m/delta
Periodic P'': V_hat*t*w*m*m/(delta*delta)
```

These are approximations to the analytic contributions, with error from the
factor/PV seed, `q`, `delta`, `w` and one term rounding explicitly propagated.
The correctly rounded scaled rational is not claimed an exact elementary
derivative. An optional extra precision would need its own fixed contract;
the proposal's 53-bit normalized term already permits gross-conditioned
validation and prevents false intermediate range rejection.

At most five numerator significands and two denominator significands occur:
numerator `<2^265`, denominator `<2^106`. A five-`u64`-limb scalar kernel is
sufficient. Determining the ratio binade and aligning a 53-bit retained
quotient requires at most 266 significant integer bits; quotient/remainder
rounding must retain every carry/borrow/remainder. No unchecked remainder
doubling or shift is justified merely by the output's size.

Every nonzero input magnitude lies in `[2^-1074,2^1024)`. Consequently an
unrounded term using five numerator/two denominator factors lies strictly
between `2^-7418` and `2^7268`; powers of two such as the Simple factor `2`
can be included conservatively in these bounds. A 53-bit scaled rounding
therefore needs no bit below approximately `2^-7470` and remains below
`2^7269`. A common integer basis `2^-8192` safely contains every rounded
scaled term. There is no term underflow to zero in this extended format.

Keep separate positive/negative accumulators for each of `P'`, `P''`, `M`;
subtract once after all original-order terms are admitted. A conservative
capacity is `(15461 + usize::BITS).div_ceil(64)` words per unsigned
accumulator: the high endpoint `2^7269` relative to basis `2^-8192` needs
15461 bits, and a slice has fewer than `2^usize::BITS` terms. On 64-bit
platforms this is 243 words per unsigned accumulator, about 12 KiB for all
six arrays. The bound intentionally covers general finite operands rather
than relying on economically typical times or testing impossible financial
private states. Prove exact constants again when freezing the kernel.

Raw getters round the signed accumulator once to binary64. A nonzero value
rounding to zero or infinity returns the normal typed range error. Exact
cancellation returns positive zero. Normalized getters instead divide the
unrounded signed accumulator by the finite nonzero dirty amount and round
once to binary64, using its binary significand/exponent. They never call the
raw getter first. This also preserves a useful ratio after raw underflow.

Reachable public edge anchors include a zero-coupon bond with `PV=MAX`,
`t=2`, `y=0`: Continuous has raw `P'=-2*MAX` and `P''=4*MAX`, but finite
Modified `2`, Convexity `4`, Macaulay `2`. With `PV=MIN_SUB`, `t=1/2`,
`y=0`, Continuous raw derivatives round to zero, but Modified `1/2`,
Convexity `1/4`, Macaulay `1/2` remain finite. At a zero-yield identity path,
opposite equal-horizon amounts can also cancel derivatives outside binary64
range before a raw getter rounds them. Arbitrary separate elementary
observations retain their gross-conditioned accuracy qualification. A per-term
binary64 admission policy would incorrectly
reject these proposed risk operations; existing price-term policy is unchanged.

## 8. Resource, failure and return ordering

Plan preparation retains its existing shared date-work ledger. Inversion and
risk use only prepared times and cached accrued amount: no date/calendar/ICMA
queries, large convention clones, clock reads or global/interior mutable state
are introduced per trial. For a convenience future `Bond` wrapper, prepare
the plan first, preserving context/resource failures before yield failures.

For yield inversion: target conversion/addition; fallible reservation of one
PV workspace and sensitivity keys; classification; then counted callback
evaluation in M6 order. Reserve capacity once up to the already bounded flow
count and clear/reuse the workspace. No per-trial vector allocation. Sorting
classification is `O(n log n)` once; callbacks are `O(n)` and there are at
most `options.max_evaluations()` actual invocations. Even `MAX_FLOWS` times
a `u32` callback cap is below `2^54`, although a caller can choose expensive
valid limits. No unbounded normalization or data-dependent retry loop is used.

Risk traverses once in original flow order, reusing a private factor/PV state
and fixed-capacity accumulators. If a private factoring change affects M4/M7,
retain their complete regression behavior and error precedence. Allocate any
required PV workspace fallibly before evaluation; fixed accumulator storage
is independent of flow count. Clone and debug of the public risk value retain
the project's ordinary stack/resource qualification; do not print wide
private implementation arrays into ordinary financial presentation.

Use a separate non-exhaustive `BondYieldError` to avoid recursively boxing
`BondError` merely to retain a callback cause. Proposed variants are target
conversion failure, solver failure, typed callback failure with stage/point/
counts, `NoYieldSensitivity`, and `ConstantPriceMismatch`. A callback variant
contains a `BondError` directly; `BondError` does not contain `BondYieldError`.
The root `Error` can add a distinct wrapper and preserve every `source()`.
No new heap allocation is needed simply to construct a nested error.

Risk can reuse `BondError` plus an explicit normalized-zero-price variant;
range getters preserve sign, operation/measure and typed `ValueError`.
Internal capacity guards are supported by width proofs, not forged private
financial input tests. No partial successful result escapes a callback or
kernel failure. A `BondRisk` whose explicitly fallible raw view overflows is
still a valid value with potentially usable other views.

## 9. Independent validation and proposed acceptance rows

Freeze a new contract and new authoritative requirements before production.
The following are proposed obligations, not existing `FR-BOND` rows:

| Proposed row | Required independent evidence |
|---|---|
| P-YR-01 | Explicit clean/dirty target conversion and one-time AI shift; negative/zero targets; rounding-sensitive per-100 and AI cases; no inferred guess or dates |
| P-YR-02 | Exact Fraction residual scaling: target `2^970` and both neighbors, both signs, all exponent fields, MAX/opposite MAX, signed MIN_SUB, equal/adjacent inputs; no false zero/sign or overflow |
| P-YR-03 | Fallible core parity for every current M6 test; endpoint/trial errors, exact cap counts, unchanged safeguards and finite bracket arguments; payload belongs to returned cached point |
| P-YR-04 | Closed-form one-payment inversions; independent high-precision multi-payment roots; all 32 compounding configurations and relevant mixed hybrid branches; exact root and bracket reasons remain distinct |
| P-YR-05 | Empty/paid/time-zero plans, same-factor opposite cancellation, nonpairable cancellation limits, constant mismatch/match, signed two-root and tangential-root examples; no unqualified uniqueness claim |
| P-YR-06 | Analytic `P'`, `P''`, moment, Modified/Macaulay/Convexity, all signs and zero-price cases; zero-yield and time-zero controls; correct annual decimal units and no convexity half |
| P-YR-07 | Full-range scaled rational/sum/divide integer oracles, midpoint/parity/carry boundaries, positive/negative sums, raw overflow/underflow rescued normalized outputs, and genuine final range errors |
| P-YR-08 | Every scalar day count and contextual family, ICMA coverage/budget, 30E/360 ISDA termination, both US EOM choices, Business252 plain/TARGET/joint/override policies; preparation error order and zero date work in reuse |
| P-YR-09 | Both hybrid thresholds and adjacent representable times for all ten frequencies, negative periodic-base conditioning, active Simple domain failures, `NumericalBoundary`, individually invalid PVs before cancellation |
| P-YR-10 | Native debug/release, strict Clippy, public/private rustdoc, traits/error census, concurrency, fresh production coverage, actual extracted-package verification, frozen artifact audit and accurate platform limits |

The financial oracle must derive elementary prices and derivatives directly
with independent high-precision Decimal evaluation at two justified precisions,
using the supplied binary64 inputs as exact rational data. It must **compare
numerical values**, not merely signs or agreement between production price
and its own inverse. Exact rational Simple/zero-yield/integer-power cases
supply independent anchors. Derivative expectations cannot be generated by
copying the same rounded M4 branch formula into Python binary64.

Maintain separate output profiles for exact input interpretation, residual
rounding, scaled rational rounding, elementary seed error and real-model risk
error. Let `E_V` bound a PV seed's error and `E_g` its analytic multiplier's
error. A usable absolute contribution bound is

```text
abs(g_hat)*E_V + (abs(V_hat)+E_V)*E_g
    + error from the final scaled-term rounding.
```

Sum these bounds on gross terms, then add final reduction/normalization
rounding. For numerator approximation `N_hat` with bound `E_N`, and observed
dirty denominator `D_hat` with price bound `E_P`, when `abs(D_hat)>E_P`,
one conservative ratio allowance is

```text
E_N/abs(D_hat)
  + (abs(N_hat)+E_N)*E_P /
      (abs(D_hat)*(abs(D_hat)-E_P))
  + final ratio-rounding allowance.
```

Evaluate bounds with scaling as well; a bound that overflows is recorded as
uninformative, not used to skip every difficult numerical assertion. At
`abs(D_hat)<=E_P`, report that no useful normalized relative certificate is
established and retain gross-conditioned absolute numerator/price checks.
Never claim a blanket net-price or risk ULP bound near signed cancellation.
The committed public counterexample in `tests/bonds_core_review.rs` gives a
25% difference between staged price and ideal net PV despite individually
ordinary rounded terms. New evidence must include that class rather than
widen tolerances after discovering it.

Finite differences are supplementary checks of these analytic formulas.
Use fixed-yield, fixed-horizon central differences with a documented step
sequence, staying within the same quote/factor domain. High-precision
differences should exhibit truncation convergence; binary64 repricing adds
gross rounding noise amplified by `1/h` or `1/h^2`. Do not assert continued
convergence as `h` approaches machine resolution, or vary dates through a
hybrid switch and call that a yield derivative. Printed ISMA examples with
rounded intermediate duration remain a separate source-precision class.

Root-location evidence combines the explicit M6 coordinate tolerance and
callback/oracle conditioning. A local slope lower bound can translate a
price-error bound to a root-error allowance; signed flat/tangential cases
do not have that guarantee. Do not replace root tests with residual-only
success, or treat `NotBracketed` as a no-root certificate.

## 10. Remaining M8 obligations and adoption order

This is the complete proposed **generic** inversion/risk tranche. It does
not supply UK DMO quasi-period powers, Treasury fractional-first Simple
prefixes, Canadian conventions, automatic legal settlement/ex-date calendars,
or NI-01 financial denomination/quantum/rounding-stage certification. The
[gilt](M8_GILT_SOURCE_REVIEW.md), [Treasury](M8_US_TREASURY_SOURCE_REVIEW.md),
[historical ISMA](M8_ICMA_SOURCE_REVIEW.md), and
[reference exploration](M8_BOND_REFERENCE_DESIGN.md) keep those distinct.
Existing source-version/capability gaps and named-market examples remain
required work toward the full M8 objective.

Next: independently review this proposal's residual/capacity proofs and public
surface; freeze the complete local contract and requirement rows; author new
independent oracles/blind hand cases; implement the shared fallible core,
payload/workspace, inversion and scaled risk together; measure actual errors
and fresh evidence; then make a local signed-off commit. Do not report this
proposal, a price/yield round trip, or a generic-only milestone as complete
financial or full M8 acceptance.
