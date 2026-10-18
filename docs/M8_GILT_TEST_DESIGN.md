# Conventional gilt test design

This is an independent **test proposal**, dated 2026-10-18. It adopts no new
API, error, numerical policy or requirement. The named conventional gilt model
must have a complete reviewed contract before implementation expectations are
frozen. `GILT-Txx` identifiers below are planning labels, not authoritative
requirement identifiers. Existing generic bond, amount and settlement contracts
remain unchanged.

The author read the primary publications and frozen contracts, but did not read
gilt production, saved fixtures, current tests or QuantLib implementation for
this tranche. Source examples, independent derived controls and proposed
operational tests are distinguished below. This document records no executed
gilt implementation test or market acceptance.

## Inspected evidence and precision

| Publication | Local PDF SHA-256 | Relevant complete pages visually inspected |
|---|---|---|
| [DMO formulae, fourth edition, 18 December 2024](https://www.dmo.gov.uk/media/334d05fo/yldeqns_v4.pdf) | `bd752ac495ec340c0d34e7d6d1e6fe0af2e1d25819364935312f53a9f5037745` | Printed 2,4/5,15/16,17/18/19/20,28; PDF pages 3,5/6,16/17,18/19/20/21,29 |
| [DMO worked examples, conventional sections of the historical example PDF](https://www.dmo.gov.uk/media/ftjpyv1z/yldconv.pdf) | `8f491b8567c6f3a75ee2a7cbc02ccc4275810257df5533ed1fd349992465e018` | PDF pages 1,2,5 |

The read-only PDF workflow was applied. Local PDFKit renders were inspected;
text extraction located pages but did not establish table-column
correspondence or formula layout.
The historical file's date is contextual: its URL/file contains worked
examples of the conventions introduced in 1998, with 1999 settlement dates.
Do not invent a publication day from those inputs. Index-linked example pages
were not used as conventional gilt evidence.

Reference contracts read for compatibility:

| Contract | Frozen SHA-256 |
|---|---|
| [Bond core](M8_BOND_CORE_CONTRACT.md) | `009f5da71d5a3b1af0b8d258754350f8810dff327a20d2c77efee17ce9fdd69c` |
| [Generic yield and all six risk views](M8_BOND_YIELD_RISK_CONTRACT.md) | `62e3bffe1f0f95b5742c81f7431539dc135d2fcf2d92b1dc31b6e1de9958605d` |
| [Settlement](M8_SETTLEMENT_CONTRACT.md) | `47f1b08ff1ca646c502c90b1e7cb380fa4d74a4763b3aa9e32d729adc89faeba` |

The independently executed [primary-example checker report](M8_GILT_SOURCE_EXAMPLES.md)
records the source-cell census and actual high-precision checks. It is source
validation, not an executed Rust gilt test.

The earlier [source review](M8_GILT_SOURCE_REVIEW.md) and
[exploratory reference design](M8_BOND_REFERENCE_DESIGN.md) give context, but
their proposals and public-reference observations are not adopted expectations.

Printed prices and AI are decimal display observations, not full-precision
oracles. Associate each retained cell with its displayed precision and source
location. A nearest-rounded decimal display gives a rounding cell, not a
binary64 ULP budget. Preserve whether a tie endpoint is included under the
source's declared positive rounding rule. Do not substitute rounded printed
`v` values for the supplied yield when reproducing a displayed price.

In particular, page 5's selected ex-AI result is printed to six decimal places,
but the daily ex-AI tail is printed to five. For example, its final daily
negative entry has fewer decimal places than the separate highlighted case.
The checker must retain individual displayed strings/precision and explicitly
classify discrepancies; silently assigning six decimal places to every cell
would assert precision the table does not show.

## Independent mathematical controls

For review, use the finite-sum representation

```text
b = 1 + y/2
a = r/s
lambda_j = (r + j*s)/s
P(y) = sum_j A_j * b^(-lambda_j), y > -2
```

Here `A_j` is the buyer-entitled published per 100 cash flow at the corresponding
quasi date, including redemption at the final date. This is a mathematical
test model. Its eventual floating evaluation/staging remains a contract choice.
The representation exposes a missing first payment, a long first dividend at
the next-but-one date and the zero-yield limit without dividing by `1-v`.
At a cycle-date settlement use the following quasi interval with `r=s`;
exclude that day's coupon when the named contract requires that convention.
These date and cash-flow distinctions are visible in the
[conventional formula and worked examples](https://www.dmo.gov.uk/media/ftjpyv1z/yldconv.pdf).

For fixed exact `lambda_j`, independent derivatives are

```text
P'  = -sum_j V_j * lambda_j / (2+y)
P'' =  sum_j V_j * lambda_j*(lambda_j+1) / (2+y)^2
M   =  sum_j V_j * lambda_j/2
Modified = -P'/P; Macaulay = M/P; Convexity = P''/P
```

Use dirty price for normalization. These are derivatives of the mathematical
model, not derivatives of a rounded program; first-period AI is independent of
yield. The prospective contract must resolve the exact staged operands and
error propagation for rational exponents, derivatives and normalization. Do
not reuse the generic `t=RN(lambda/2)` model merely because most displayed
prices agree.

The following controls are independently derived, not printed DMO results:

| Control | Exact mathematical result | What it distinguishes |
|---|---|---|
| One remaining `A=102.5`, `a=1/2`, `y=0` | `P=102.5`, `P'=-25.625`, `P''=19.21875`, `M=25.625`; Modified=Macaulay=`1/4`, Convexity=`3/16` | Yield 0 still has nonzero risk; convexity has no Taylor factor 1/2 |
| Same payment, `y=6`, hence `b=4` | `P=51.25`, `P'=-3.203125`, `P''=.6005859375`, `M=12.8125`; Modified=`1/16`, Macaulay=`1/4`, Convexity=`3/256` | A nonzero-yield exact radical/rational anchor |
| Same payment, `y=-1.5`, hence `b=1/4` | `P=205`, `P'=-205`, `P''=615`, `M=51.25`; Modified=`1`, Macaulay=`1/4`, Convexity=`3` | A valid negative yield and all risk signs/units |
| `A_0=3`, `A_1=103`, `a=1/2`, `y=0` | `P=106`, `P'=-78`, `P''=97.125`, `M=78`; Modified=Macaulay=`39/53`, Convexity=`777/848` | Independent multi-payment sums and weights |
| Issue 1999-05-07, next quasi 1999-06-07, prior quasi 1998-12-07, `c=6` | Short raw dividend=`93/182`; long raw dividend=`639/182`; six-decimal published amounts `.510989` and `3.510989` | Short versus long first payment and publication stage |
| Same short geometry, exact `c=.000546` per 100 | Raw first dividend `.0000465`; positive half-up result at six decimal places `.000047` | Ties-even would give `.000046` |
| Same long geometry, exact `c=.000182` per 100 | Raw first dividend `.0001065`; positive half-up result at six decimal places `.000107` | Ties-even would give `.000106` |
| `r=8`, `s=181`, later payment `j=1` | `RN(189/181)` has bits `3ff0b509e68a9b95`; `RN(RN(8/181)+1)` has bits `3ff0b509e68a9b94` | Exact quasi coordinates differ from a rounded fractional part followed by an integer addition |
| One remaining `A=102.5`, `a=1/2`, target price respectively `102.5`, `51.25`, `205` | Algebraic inverse `y=2*((A/P)^2-1)` gives respectively `0`, `6`, `-1.5` | Exact target-driven root controls, separately derived from the price equation |

The exact-rational anchors above were calculated with Python `Fraction`, with
100-digit Decimal output only for inspection. No production or reference
library result was used. Whether a floating implementation must hit an exact
anchor bit-for-bit or within a justified numerical enclosure is a prospective
contract decision; a mathematically exact result is not by itself a libm
correct-rounding theorem.

For the historical long first dividend, the source uses `r1=31`, `s1=182`,
`s2=183`, `c=6`. AI at 1999-09-07 is `22417/11102`; ex-AI at 1999-12-06 is
`-1/61`. Published dividend `3.510989` differs from raw `639/182`. Subtracting
the published rounded dividend from raw cum-AI differs from the source's ex
formula by `1/91000000`. At par 100,000,000 the published coupon gives cash
3,510,989.00; rounding the unrounded raw coupon at the final cent instead gives
3,510,989.01. These are useful staging discriminators. Source payment and AI
formula roles must remain separate; exact clean-price continuity must not be
invented as a test invariant.

At the skipped long-first quasi date 1999-06-07, the AI is still `93/182`,
not zero. Before that date, the next quasi cash flow is zero and the following
one is the long dividend; on that date the new next quasi cash flow is the
long dividend. The actual first payment 1999-12-07 resets AI. Test the date,
cash-flow roles and accrued formula together rather than applying a standard
coupon-date shortcut at every quasi date.

## Named-model conditioning checks

Keep three numerical objects separate: the ideal quasi-period price at exact
source cash flows/rational coordinates, the adopted staged price, and the
displayed or settlement-rounded amount. A root of the staged price at a
rounded target is not automatically the ideal root at an unrounded source
quote.

For nonnegative supported cash flows with at least one positive future flow,
the exact model is strictly decreasing and convex on `y>-2`. On a finite
bracket `[l,u]`, an exact slope floor is `m=abs(P'(u))>0`. If a validated uniform
absolute price error is `E` and any separately intended target uncertainty is
`E_T`, ideal root displacement is bounded by `(E+E_T)/m` when the compared roots
and connecting interval remain in that domain. The M6 full-width termination
tolerance contributes separately. A small residual or a dual-precision match
alone proves no such root bound. With only an empirical elementary profile,
the resulting numerical comparison remains conditional on that profile.
For a returned cached point with nonzero exact unscaled amount residual `h`,
the direct residual bound instead includes `abs(h)` in its numerator. The
solver's fixed-scale residual must first be interpreted in its retained units;
its raw magnitude cannot silently be used as a price error.

For the one-payment control, bound roots directly by independently inverting
`T+E` and `T-E` where both targets are positive; do not replace this with an
unjustified yield-ULP allowance. For multiple payments, use a separately
evaluated monotone interval/bisection reference or a validated slope bound.
Include a large positive yield/short horizon case with a small slope and a
near-minus-two case with steep derivatives: an identical price tolerance
must not imply an identical yield tolerance.

Rational-coordinate error needs its own allowance. If an implementation uses
`lambda_hat=lambda+e`, its discount ratio is `exp(-e*ln(b))`. Thus a coordinate
error bound `abs(e)<=E_lambda` gives a relative bound
`exp(E_lambda*abs(ln(b)))-1` before other elementary/staging errors. The
one-ULP coordinate discriminator above should be tested at ordinary yields
and at large `abs(ln(b))`; it must not disappear inside an arbitrary global
price epsilon. A future kernel retaining the exact rational coordinate can
remove that particular input error, but still needs its own elementary proof
or explicitly measured comparison profile.

For every risk numerator, propagate accepted PV uncertainty and multiplier
uncertainty on each gross term before signed reduction. For example, with
`Nhat`, ideal `N`, observed dirty `Dhat` and ideal `P`, bounds
`abs(Nhat-N)<=E_N`, `abs(Dhat-P)<=E_P` imply

```text
abs(Nhat/Dhat - N/P) <= (E_N + abs(N/P)*E_P)/abs(Dhat)
```

when both denominators are nonzero; final getter rounding is additional.
The numerical normalization must continue to use retained extended sums,
even if an independently queried raw getter is outside binary64. Positive
source cash flows may exclude generic signed cancellation/zero-dirty states;
prove those exclusions instead of fabricating a named-plan witness.

## Coverage matrix

Choose small controls that distinguish the stated behaviors. Expand a row
only when its combination exercises a different formula, boundary, admission
or context. A large Cartesian corpus with indistinguishable outcomes is not
a substitute for these discriminators.

| ID | Family and inputs | Required assertion or independent control |
|---|---|---|
| GILT-T01 | All 8 conventional price scenarios on example PDF page 1 | Retain complete dates, `r/s`, `n`, cash flows, yield and displayed price; each correct rounding cell must be checked |
| GILT-T02 | All 8 conventional inverse scenarios on PDF page 2 | Independent root enclosures for targets 135/108; check displayed decimal yield separately, without a planted-yield round trip |
| GILT-T03 | Long first dividend and 3 highlighted AI cases on PDF page 5 | Published first dividend and each displayed AI cell, with their own precision and branch |
| GILT-T04 | Entire 214-date daily AI trajectory 1999-05-08..1999-12-07 | Source date membership, time-variable reset, quasi transition, ex transition and payment-date reset; no selected-date-only claim |
| GILT-T05 | Standard first period, short first period, long first period | Exact full/raw/published first dividend and number/position of actual payments on the common quasi lattice |
| GILT-T06 | Short first issue at start/end/one day inside supported interval | Issue AI zero where its admissible cum-dividend assumptions hold; payment duration and rounding boundaries remain explicit |
| GILT-T07 | Long first period before, on and after the intermediate quasi date | No phantom cash flow on the intermediate date; correct first/second AI branches; re-anchor `a`, `n`, `d1`, `d2` together |
| GILT-T08 | Long first period in its second quasi interval, cum and ex | Published first dividend remains retained while buyer entitlement changes; use the raw-ratio AI discriminator above |
| GILT-T09 | Leap and nonleap quasi periods, day lengths 181/182/183/184 | Integer civil day counts and correct denominator at each endpoint; no fixed 182-day assumption |
| GILT-T10 | January/July 31, February 28/29, August 31 and 30-day month anchors | Apply only the explicitly adopted EOM/lattice policy or supplied quasi-date vector; direct maturity anchoring must not drift after a clamp |
| GILT-T11 | Explicit cutoff minus one day, equality, plus one day | Equality remains cum; strict-after changes buyer entitlement and AI branch. A generic `OnOrAfter` ex policy is a deliberate mismatch |
| GILT-T12 | Standard, short-first and long-second-period ex windows | Full next published dividend and buyer-entitled next dividend are independently inspected; AI is not forced to zero when `d1=0` |
| GILT-T13 | Settlement on actual coupon date versus intermediate no-payment quasi date | Correct payment exclusion/roll; an intermediate long-period quasi date is not an actual coupon receipt or AI reset |
| GILT-T14 | Settlement one day before/equal/after final payment | Explicit supported tradability/terminal policy. Do not invent a post-maturity zero-price result from a generic empty plan |
| GILT-T15 | Raw weekend/holiday payment date, all dates retained | Source quasi discount cycle is unbumped; no clock/calendar lookup or hidden Following adjustment |
| GILT-T16 | An adjusted coupon payment or redemption shifted from its raw quasi date | Named-model rejection must retain exact source/date/flow context; do not silently price as an unadjusted gilt |
| GILT-T17 | Same economic definition under a calendar/convention that leaves every date unchanged | Contract decides whether validation is by actual date equality or by metadata; verify the chosen rule, not a guessed restriction |
| GILT-T18 | Irregular last period, missing/duplicate/off-lattice middle payment, unsupported coupon count, wrong redemption | Each unsupported definition fails the adopted named validation before numerical work; no partial named coverage hidden behind a generic fallback |
| GILT-T19 | One final payment, both cum and ex | Fractional power `A*b^-a`, algebraic inverse and all six risk measures; no final-period switch to a Simple denominator |
| GILT-T20 | `n=1`, `n=2`, and a longer schedule | Empty/multiple geometric tail indexing, redemption index and `d2` position; compare direct finite sum with independently evaluated source formula away from 0 |
| GILT-T21 | `y=+0`, `-0`, smallest nonzero signed neighbor and moderate yields | Zero-price formula is a stable finite sum; no `0/0`, lost coupon, spurious constant classification or derivative zeroing |
| GILT-T22 | `next_up(-2)`, moderate negative yield, exactly -2, next below -2, NaN and both infinities | Complete named quote domain and quote-before-factor ordering, including a zero coupon amount and time 0 policy where admissible |
| GILT-T23 | Final-period one-payment model discriminator at `a=1/2`, `y=.05`, coupon 2.5 | Quasi power differs materially from a fractional Simple denominator; classify model difference before tolerance comparison |
| GILT-T24 | Exact rational `r/s` with denominator 183 and multiple later payments | Preserve `(r+j*s)/s`; compare against independently rounded generic ICMA-time model and explicitly bound or classify the difference |
| GILT-T25 | Large positive yield and a very short remaining fraction | Finite dirty price with raw/normalized risk underflow as genuinely reachable; never flush an extended term to zero before its getter |
| GILT-T26 | Just above -2 with longest supported horizon and ordinary positive redemption | Individual factor/PV overflow/domain failure remains indexed; no invalid-PV rescue or infinity admitted as a price |
| GILT-T27 | All six risk views at zero, positive and negative yields | Independent derivative/moment anchors; dirty normalization, annual decimal units, no convexity factor 1/2; each getter separately fallible |
| GILT-T28 | Several payments with differing rational powers | Sum derivatives independently; check Macaulay=`b*Modified` only with propagated allowance, not as the only risk oracle |
| GILT-T29 | Central differences with `h`, `h/2`, `h/4` at high precision | Demonstrate convergence against symbolic first/second derivatives before binary price resolution dominates; freeze entitlement and dates during perturbation |
| GILT-T30 | Fixed source/risk state queried in different getter orders | Results and typed errors unchanged; failed raw view does not invalidate normalized views; no mutation or recomputation of source dates |
| GILT-T31 | Dirty and clean target quotes with nonzero positive/negative AI | Single exact adopted quote conversion and one cached AI shift; returned target/kind and amount-unit target retained independently |
| GILT-T32 | Face 100 and a nonidentity face such as 300, where the named API supports face | One-round quote/amount normalization and its independent range failure; do not use a per 100 round trip as an exact inverse premise |
| GILT-T33 | One-payment inverse at supplied target, multi-payment high-precision root | Independent roots from closed forms/monotone interval bracketing; include negative yields and first-dividend cases, not only original-yield round trips |
| GILT-T34 | Root at lower endpoint, at upper endpoint and in the interior; cap 1/cap 2/budget edge | Existing M6 exact callback counts, stage, initial bracket, options and cached payload; no final uncounted repricing |
| GILT-T35 | Invalid lower/upper/trial quote or PV; invalid upper unqueried after lower zero | Error precedence/stage/count follows the full adopted solver contract; do not prevalidate a trial the solver never calls |
| GILT-T36 | Same-sign endpoint residuals, nonpositive target for a strictly positive supported gilt | Report no bracket with finite scaled residuals/counts; positive-cash-flow monotonicity does not prove target existence |
| GILT-T37 | Constant/empty/paid/time 0 source shapes | Test the named constructor's declared support or rejection. Do not forge a zero-dirty/constant report to cover generic error variants |
| GILT-T38 | Valid dirty solve with an unused clean/per 100 view outside range, if publicly reachable | Independent report-view failure, exact typed source and unchanged successful solve. Otherwise record a domain proof, not a fabricated state |
| GILT-T39 | Explicit nominal, exact dirty decimal evidence and final cent policy | `dirty_per_100` performs one final traded-amount quantization; raw price display is not a source-price quantization stage |
| GILT-T40 | Positive and negative AI with clean/accrued split | Exercise both explicit aggregation choices; whole invoice and separately rounded components may differ. Report that the source does not settle every signed/aggregation choice |
| GILT-T41 | A source six-decimal first dividend and large held nominal | Preserve publication rounding before coupon distribution; the 100m-par long-coupon discriminator separates it from rounding only final raw cash |
| GILT-T42 | Exact decimal 1.005 versus supplied binary64 nearest to 1.005, par 100, cent rounding | Each evidence kind proves only its own exact input; neither receipt proves the ideal transcendental price was supplied |
| GILT-T43 | Declared price/AI interval below, across and exactly on a cash threshold | Determined/ambiguous decision and guarantee follow the frozen settlement contract; no measured rate tolerance is silently promoted to an enclosure |
| GILT-T44 | Point interval versus supplied value; decimal versus binary provenance | Equal cash retains distinct evidence/receipt identity and guarantees; Eq/Hash/Send/Sync follow the existing contract |
| GILT-T45 | Arbitrary final quantum and all seven explicit signed modes; Exact at each mandatory stage | Preserve sign, positive par, stage strings, endpoints, Inexact/Unresolved order and independent source/cash range admission |
| GILT-T46 | Repeated price/risk/inversion/settlement use of one prepared definition | Zero new date-query work per numerical reuse where promised; no growing cache, per-trial allocation or implicit clock/calendar/default |
| GILT-T47 | Small supported source limits, exact boundary and one below needed count | Resource/coverage failures retain original requested/used/required fields and original flow index; no OOM testing or physically maximal allocation |
| GILT-T48 | Source/root errors, metadata, traits and concurrent immutable reuse | Every introduced reachable public field, typed cause and root conversion; deterministic debug/release and thread results |
| GILT-T49 | Exact annual decimal coupon with an odd coefficient at scale 38 | Regular `c/2` may require scale 39; retain its exact rational or return the declared domain error. Do not silently six-decimal-round a standard conventional coupon |
| GILT-T50 | A positive short-first coupon whose published six-decimal amount becomes zero | Retain its original position/metadata and required quote/factor checks; distinguish a genuinely zero published flow from a missing long-period quasi payment |
| GILT-T51 | Same price-error allowance at steep and shallow portions of the named price curve | Independent inverse envelopes/slope conditioning, with M6 termination allowance added separately; a small callback residual alone is insufficient |
| GILT-T52 | Exact versus rounded quasi coordinate at ordinary and large `abs(ln(b))` | Propagate rational-exponent error through its own discount ratio; do not borrow a generic rounded-time tolerance or a source display tolerance |

Rows GILT-T16..18 and GILT-T22 require the **future named contract's** exact
error categories and sequencing. They do not authorize changing frozen generic
errors. Existing FR-YR and FR-SETTLE obligations remain applicable to their
existing operations; a new named plan must separately specify what it shares.

## Settlement controls and limits

Use the actual frozen settlement API, with every quantum, mode, aggregation,
unit and nominal supplied. A direct DMO yield-to-trade example belongs to
`SettlementInstruction::dirty_per_100` after the evidence assumption is made
explicit. `staged_price100_accrued1000` deliberately introduces fixed source
quanta and different accrued units; its existence does not make that staging
the DMO conventional yield-to-price recipe.

Useful exact operational discriminators, not claims about an actual security:

| Inputs | Expected difference |
|---|---|
| Par 100, clean 100.005, accrued .005 per 100, cent positive half-up | RoundTotal gives 100.01; RoundComponentsThenSum gives 100.02 |
| Par 100, dirty exact decimal 1.005 versus supplied binary64 nearest 1.005 | Positive half-up gives 1.01 for exact decimal; the supplied binary value lies below the tie and gives 1.00 |
| Par 100, clean .5 and accrued .5, whole-unit Exact cash | RoundTotal is 1; separately required component Exact fails at capital. Net cancellation or summation cannot waive an earlier Exact stage |
| Held par 100,000,000, published long coupon 3.510989 per 100 | Convert units explicitly to 35.10989 per 1000 before `coupon_per_1000`; correct cash 3,510,989.00 |

Use opposite party signs to verify directed rounding on the signed expression.
Do not encode direction by negative par. Negative cash/AI tests exercise the
explicit local policy; the DMO's positive rounding explanation alone does not
establish signed tie behavior or the legal convention for every net invoice.
Coupon source rounding, invoice rounding and display rounding are three
different stages and need separate controls.

No automatic named-price-to-guaranteed-settlement conversion should be inferred.
A receipt for supplied binary64 certifies that represented value only. A
`UniqueForDeclaredEnclosure` receipt is conditional on the caller's enclosure.
The measured M4 E3 profile and agreement at two Decimal precisions are not, on
their own, a rigorous enclosure of an unknown ideal price.

## Evidence construction and mutation challenge

Maintain at least three separately identified evidence tiers: source display
cells, exact/rational staged controls, and source-real high-precision numerical
references. Pin source PDFs, publication pages, complete generator source,
contract/requirement rows, schema, body and record census. Use new additive
artifacts; preserve all earlier bytes. Every input/context/status/enclosure/
source field in a fixture must be consumed or explicitly verified unused.

For each numerical oracle, fix the input meaning before looking at production.
Use exact civil day differences, exact decimal publication rounding and exact
rational quasi powers. Compute independent high-precision price, both
derivatives, moment and roots at separately justified precisions; require
agreement of all serialized fields and propagate reference uncertainty. Bound
elementary and staging error on gross contributions and through signed dirty
normalization. A nearly cancelled result gets no arbitrary net-ULP allowance.
Do not select an expected staged map using a separate reevaluation of the
production PV. A repeated same-host target/reuse check is operational evidence,
not an independent root-accuracy oracle.

Mutation controls should kill at least: ex equality changed to inclusive
detachment; wrong `r=0` cycle-date convention; dropped long-period empty quasi
slot; unrounded first dividend used for payment; rounded first dividend used
inside the raw AI formula; Simple final-period discount; literal geometric
`0/0` at yield 0; premature rounding of rational `lambda`; wrong annual factor
in either derivative; convexity half factor; clean-price normalization; dirty
target/clean target confusion; last-trial payload latch; premature per 100 or
component invoice rounding; and ignored adjusted payment. Mutations stay in
a reproducible temporary copy and never justify fabricated financial states.

For finite-width helpers, first derive capacities from the **new rational
operands** and supported date/count domain. In particular, expanding
`lambda*(lambda+1)/(2+y)^2` introduces the exact factors
`n*(n+s)/(s*s*delta*delta)`. The old five-numerator/two-denominator argument
cannot simply be cited without showing how the extra integer denominator is
represented. Exercise valid finite operands at carries, quotient ties,
renormalization and range boundaries. Classify proof-excluded capacity guards
and allocation failure separately; no forged exponent/report, allocator/OOM
stress or unreachable getter error should be added for coverage percentage.

## Decisions needed before expectation freeze

The test matrix identifies the following contract choices rather than filling
them with defaults:

1. Exact admissible source definition, coupon sign/range, redemption/face,
   full first-dividend calculation and published amount representation; whether
   the adapter constructs these amounts or verifies explicitly supplied ones.
2. Maturity-cycle/EOM treatment, especially February maturity versus August
   month-end; first/last supported lattice dates and all adjusted-payment
   rejection rules. Explicit supplied quasi dates can avoid an inference.
3. Exact admissible ex-date windows and first-issue versus secondary-trade
   scope. The long-first first-quasi AI branch has no ex subcase; an arbitrary
   early cutoff must not silently select an unsupported formula. An issue
   after an arbitrary short-first cutoff also needs a declared domain.
4. On-payment/final-payment/after-maturity behavior and whether a caller's
   Include policy is rejected, constrained or part of a distinct generic mode.
5. Retained rational-power numerical model, exact derivative operands, all
   independently fallible views, capacities, source-validation/error order,
   work ledgers and precise root/report payload semantics.
6. The path from numerical results to supplied or declared settlement evidence;
   signed ties, final aggregation and currency/calendar/auction acceptance
   remain explicit external or future named-contract obligations.

After those choices are reviewed, the proposed tests should be bound to exact
adopted requirement IDs and full public signatures before implementation.
Source examples and local mathematical tests do not close named-market,
legal-calendar, financial, owner, native-platform or release acceptance gates.
