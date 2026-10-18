# Conventional gilt model proposal

Status: **read-only engineering proposal, not adopted**. This document defines
one complete candidate for conventional fixed-rate gilt pricing, first
dividends, accrued interest, entitlement, yield, all six risk measures and
settlement interfaces. No runtime, existing contract, fixture or requirement
is changed. The full [M8 obligations](M8_LOCAL_READINESS.md), including other
named markets, financial acceptance and analytical enclosures, remain in scope.

## 1. Primary evidence and scope

The primary publication is the [DMO fourth edition, 18 December 2024](https://www.dmo.gov.uk/media/334d05fo/yldeqns_v4.pdf).
Local source `/private/tmp/sabiquant-m8-dmo-gilt-v4.pdf`, 366,414 bytes, SHA-256
`bd752ac495ec340c0d34e7d6d1e6fe0af2e1d25819364935312f53a9f5037745`,
contains 29 PDF pages. The PDF skill was applied read-only. PDFKit rendered
and the author visually inspected the **complete** PDF pages 2-6, 15-21 and
29: contents, introduction, conventional price/inverse equations, standard
and irregular dividends, all conventional AI equations and their continued
definitions, and notes. Printed page numbers are one less than PDF numbers.
Scratch renders are `/private/tmp/sabiquant-m8-gilt-model-v4-p*.png`.

Source facts, condensed: the maturity defines an unadjusted semiannual quasi
cycle; buyer-entitled cash flows are discounted on it. A cycle-date settlement
uses a full next period. Conventional short/long first dividends receive
six-decimal publication rounding. Standard, short and long AI use distinct
equations, with cum treatment on the ex-date and ex treatment strictly after.
Cash settlement rounds on traded nominal without intermediate price rounding.
The note describes seven business days for the ex-date. These are facts from
printed pp.2-5,14-20,28; the engineering representations below are proposals.

The [earlier source review](M8_GILT_SOURCE_REVIEW.md),
[bond proposal](M8_BOND_API_DESIGN.md), [reference design](M8_BOND_REFERENCE_DESIGN.md),
frozen [bond core](M8_BOND_CORE_CONTRACT.md),
[generic yield/risk](M8_BOND_YIELD_RISK_CONTRACT.md) and
[settlement](M8_SETTLEMENT_CONTRACT.md) contracts were read. Rust bond/rate/
solver production and QuantLib implementation were not read to derive this
proposal. Root independently checks the publisher's historical conventional
worked examples; they are corroboration, not an implementation template.

The candidate covers conventional fixed coupons and par redemption. Index
linking, strips, floating/callable/amortizing instruments and other sovereign
conventions are not silently included. Their existing roadmap obligations
remain. A zero coupon rate can be a supported mathematical limit of these
equations without claiming that this factory describes a named stripped gilt.

## 2. Inputs and date roles

Use a named `ConventionalGilt` definition, separate from generic Bond's accrual
and discount model. Inputs are original issue date I, maturity M, exact decimal
annual coupon **C per100 nominal**, actual first dividend date F, a complete
unadjusted quasi-date vector, a complete explicit actual-dividend ex-date
vector, and an allocation/count cap. C=6 denotes a six-percent annual coupon;
the yield coordinate is annual decimal, so y=.05 denotes five percent.
Par redemption100 is an explicit invariant of this named model.

Require C>=0 and I<M. Retain original issue terms across reopenings; a later
auction date must not become the first-dividend accrual origin. The quasi vector
covers the full period containing I and every date through M, is strictly
ascending, and ends at M. Every adjacent interval is one six-month cycle.
Actual coupon dates are F and every subsequent quasi date through M. A long
first period has one intervening quasi date without a dividend; that missing
payment remains a real discount-cycle boundary.

Propose three validated first-period roles:

- Regular: issue is on the cycle and F is the following quasi date.
- Short: issue is strictly inside a period and F is its next quasi date.
- Long: F is the second quasi date after issue, with exactly one skipped
  intervening quasi date. The formula also has a well-defined full-year limit
  when issue lies on a cycle date; confirm that eligibility before adoption.

No arbitrary multi-period first stub is inferred from the two-period formula.
If a source-complete future use requires one, it needs its own evidence and
contract rather than being approximated as this Long role.

No cash-flow or quasi date is business-day bumped. If an adapter accepts a
Schedule, require raw/pay equality and validate the complete cycle. A generic
adjusted schedule does not become a gilt cycle by retaining its frequency.
Maturity-anchor month/day preservation, missing February days and end-of-month
roll behavior need an explicit cycle rule or publisher-supplied vector. The
present publication does not resolve every synthetic year1..9999 anchor. Until
that rule is separately supported, explicit validated cycle dates are the
authoritative inputs; do not claim automatic market schedule generation.

Each calculation supplies settlement S. Propose the active trading domain
I<=S<M. At S on a cycle date, that day's payment is already outside buyer PV:
select the strictly following quasi date. The generic Include-at-payment
option is therefore not an interchangeable named-gilt policy. Settlement at/
after redemption has no n>=0 conventional price equation here; return a typed
trading-domain error, preserving generic Bond's separate paid-plan behavior.

Explicit ex-dates fix the source's After boundary. Propose structural
compatibility checks: cutoff<=actual dividend; a short first cutoff>=I; a long
first cutoff>=its skipped quasi date; later cutoffs lie in their preceding
quasi period. These are engineering domain checks for the piecewise equations,
not assertions that the DMO printed those inequalities. Without them, a short
issue after its ex-date contradicts zero initial accrued interest, or a long
first-period ex-date precedes the branch that supplies ex AI. Such incompatible
inputs need a typed source-domain error until auction/entitlement evidence
resolves them.

Explicit cutoffs establish caller-supplied definitions, not UK calendar
compliance. A future seven-business-day factory must own verified UK holiday,
exception, endpoint, lag and source-version policies. WeekendsOnly, TARGET or
an unverified arbitrary Calendar cannot substantiate a UK market-calendar
claim. There is no implicit today, settlement lag, auction-right or registry
transfer determination.

## 3. Published dividends and exact accrued ingredients

Let B=C/2. Keep C as an exact decimal rational. Standard dividends are exactly
B under the printed standard formula. For a short first period let r1 be days
I to F, s1 the containing full quasi period; publish
`Dfirst=Round6(B*r1/s1)`. For Long let Q1 be the skipped quasi date,
r1=days(I,Q1), s1 its full containing period; publish
`Dfirst=Round6(B*(1+r1/s1))`. Round6 is positive decimal nearest with ties up;
for nonnegative values it agrees with NearestTiesAwayFromZero. This is a
**source publication stage**, not binary rounding or final cash settlement.
An exactly half-micro dividend must be decided by integer remainder/parity
independently of its nearest binary64 approximation.

Retain the raw ratio and the published result separately. Never derive source
AI by scaling or subtracting an already rounded first dividend. For standard
periods, use the full next published dividend D, not its buyer-entitled value.
Let t be elapsed days since the previous quasi date, s the full current period,
and E the next actual dividend's ex-date. The exact source AI is:

| Role/settlement | S<=E | S>E |
|---|---|---|
| Standard | `(t/s)*D` | `(t/s-1)*D` |
| Short first, I<=S<F | `B*days(I,S)/s1` | `B*(days(I,S)-r1)/s1` |
| Long first, I<=S<Q1 | `B*days(I,S)/s1` | structurally inadmissible input |
| Long first, Q1<=S<F | `B*(r1/s1+days(Q1,S)/s2)` | `B*(days(Q1,S)/s2-1)` |

At a skipped Q1, Long AI remains B*r1/s1; it does not reset to zero with the
discount-cycle coordinate. At the actual first payment F, switch to the next
ordinary dividend period and AI resets to zero. At an ordinary cycle date,
t=0. Source first-period status and current discount-period status are
independent pieces of retained provenance.

For C=6,I=1999-05-07,Q1=1999-06-07,F=1999-12-07, the long raw dividend is
639/182 while its six-place publication is3.510989. Source AI after the ex-date
is B*(r2/s2-1), not cum AI minus3.510989. The difference is
`1/91000000`, about1.0989011e-8 per100. A generic earned-minus-rounded-coupon
adjustment would implement another model. Exact clean-price continuity across
that publication difference must not be invented as an invariant.

Cash-flow inspection should expose original C, role, dates, raw first ratio,
publication quantum/mode and published dividend. Retain the **full** existing
DecimalAmount input domain for nonnegative C. In particular C=1e-38 gives
standard B=5e-39, which cannot be stored in DecimalAmount. A large six-place
first-dividend coefficient can require147 bits even though its value is finite;
combining dividend and par can also overflow an i128 common coefficient.
Store these source components in bounded private rational/wide form. A
separately fallible decimal inspector may report derived representability
failure; it must not clip, round standard B to six places or invalidate an
otherwise usable plan. No arbitrary coupon cap or arbitrary-precision public
money type is proposed.

## 4. Quasi powers and stable pricing

Let Q be the strictly next quasi date after S, r=days(S,Q), and s the current
full quasi period. Thus 0<r<=s and a=r/s; equality at a cycle date gives a=1.
Let n be the number of full periods Q to M. Index j=0..n, and retain the exact
dimensionless quasi exponent

```text
lambda_j = (r+j*s)/s
b = 1+y/2,   y > -2
P(y) = sum_j A_j * b^(-lambda_j)
```

A_j is the buyer-entitled published dividend at that quasi date, plus100 when
j=n. Missing long-first and detached dividends contribute no buyer cash flow;
principal is never detached. Ordinary later dividends equal B. This finite
sum algebraically reproduces the printed n>=1 expression and the n=0 result.
At n=0 the inverse mathematical control is
`y=2*(exp(log((D+100)/P)/a)-1)`, with D the entitled last dividend.

Do not use the literal geometric quotient at y=0, or subtract nearly equal
powers to sum coupons. Propose direct bounded summation of accepted PVs,
with one exact reduction and ordinary factor/product errors, and a y=0
identity path. A stable geometric evaluation is an optional optimization only
after proving the same numerical staging, counts and error profile; a formula
identity alone does not establish bit parity.

Propose one pricing bundle per actual quasi payment: combine the exact source
dividend and par principal at maturity in bounded rational form, multiply that
exact coefficient by the accepted binary64 discount, and round the PV **once**.
There is no earlier binary64 coefficient admission. Retain dividend/principal
separately in the instrument for distributions. This is an explicit proposed
numerical policy; it must not inherit generic Bond's separate final coupon/
redemption PV staging accidentally. Exact coefficient1e-6 times
`next_down(1)` gives PV bits `3eb0c6f7a0b5ed8d`; rounding the coefficient first
gives `3eb0c6f7a0b5ed8c`. Freeze that discriminator rather than asserting parity.

For the full C input range, exact bundle coefficients need a numerator below
2^148 and denominator below2^128; multiplying a discount's53-bit significand
therefore needs below2^201. Relative normalized alignment fits202 bits inside
320. Keep the discount's binary exponent separate and classify the final IEEE
binade before absolute shifts: blindly aligning a tiny product to MIN_SUB can
exceed320 bits despite the relative bound. Compute one exact IEEE rounding,
including subnormal ties; rounding to53 extended bits first and then admitting
a subnormal can double-round. Existing f64-only factor-list interfaces are not
automatically suitable for the wider source numerator.

The ideal named price uses exact source coefficients and exact rational
lambda. Numeric evaluation approximates log/exp and rounds PVs once, retaining
that factor/product error and the exact lambda provenance.
`t=RN(lambda/2)` passed to generic Compounded:2 is not definitionally the same
model. Even exact mathematical ICMA equivalence does not erase M3's supplied
YF rounding profile. No change to generic InterestRate or BondPricingPlan is
proposed.

For each power, derive and record the logarithm `-lambda*log1p(y/2)` with a
range/conditioning policy. Near -2, either delta=RN(2+y) divided by2 or direct
`1+y/2` retains the positive boundary distance: halving is exact there and
Sterbenz applies. Do not claim a counterexample for that fixed-frequency2
identity. The cancellation concern is instead `log(1+y/2)` near y=0;
log1p preserves a representable small half-quote, while y=MIN_SUB still halves
to zero and needs the existing accuracy qualification. Any lambda approximation receives an explicit error term
`abs(log(b))*E_lambda`; rounded log/product and reciprocal errors remain
separate. Range-uncertain factors return typed NumericalBoundary rather than
being guessed finite. Existing empirical M4 tolerances cannot be promoted to
a formal analytical enclosure.

## 5. Complete analytical risk and bounded representation

Differentiate the fixed source ideal above in annual decimal yield, not the
rounded program and not a time-dependent branch. For each exact lambda:

```text
d1 contribution = -PV * lambda/(2+y)
d2 contribution =  PV * lambda*(lambda+1)/(2+y)^2
moment contribution = PV * lambda/2
Modified = -P'/P;  Convexity = P''/P;  Macaulay = moment/P
```

There is no convexity factor1/2. The periodic identity
`Macaulay=(1+y/2)*Modified` is a qualified real-model comparison, not a shortcut
that discards separate moment accumulation. All six raw/normalized views are
required together, including positive/negative zero/range policies even if
ordinary positive-coupon cases have predictable signs.

Propose the same accepted PV observation for price and all risk metadata.
With L=r+j*s, numerical multiplier terms use **exact small integers** L,s and
the represented delta=RN(2+y):

```text
-Vhat*L/(s*delta)
 Vhat*L*(L+s)/(s*s*delta*delta)
 Vhat*L/(2*s)
```

Round each rational contribution once to53 normalized bits with extended
exponent, accumulate positive/negative magnitudes exactly, subtract once, and
admit each raw/normalized getter independently. Normalization divides retained
sums directly by observed dirty price. Raw failure must not preclude finite
normalized values. Zero dirty remains a typed normalized error, not division
by a numeric sentinel.

For valid C>=0, positive par and S<M, accepted nonzero principal PV makes dirty
strictly positive. Zero-dirty and signed-cancellation states are consequently
not reachable public gilt controls; their generic-kernel policy may remain a
defensive check. Do not forge a negative coupon or invalid internal state just
to cover that branch. Generic signed Bond evidence remains separate.

This is not a derivative using exact real2+y: for y=MIN_SUB that exact sum can
need about1076 coefficient bits. The rounded-delta numerical policy therefore
needs a separate multiplier error E_delta relative to the ideal. Near -2,
Sterbenz gives exact represented subtraction; farther away its relative error
is small, but still belongs in the gross propagation proof.

An independent conservative scalar proof uses n+1<=MAX_FLOWS<2^22 and
0<r<=s<2^22, giving L,L+s<2^44. P'' then needs numerator below2^141 and
denominator below2^150 after binary significand extraction; normalized53-bit
alignment fits204 bits, hence five u64 words. Actual six-month civil periods
have much tighter bounds (s<=184 and fewer than20000 cycle points). Existing
five-word helpers can be reused only after reviewing their operand interfaces
and the new integer-factor proof; the old two-f64-denominator argument alone
is insufficient.

For the actual semiannual/date domain, y>-2 implies delta>=2^-52. With accepted
finite Vhat, the second-derivative contribution is conservatively between
2^-3131 and2^1158 in magnitude. Moment and first derivative fit more tightly.
After normalized53-bit rounding, `2^-8192` basis and existing243-word sums
comfortably cover these terms and the bounded count. A final contract must
audit every shift/carry/division path and retain typed impossible-capacity
guards, not merely repeat these broad bounds.

Financial comparisons propagate factor observation, final coefficient-times-
discount rounding, lambda/delta multiplier error, term rounding and exact reduction on gross
terms. Use the frozen generic contract's conditioning formula for ratios when
dirty uncertainty excludes zero; otherwise expose an uninformative normalized
bound rather than widening a net relative epsilon. Exact source coupon/cash
checks, ideal real-model checks and staged numeric checks are separate ledgers.

## 6. Yield inversion, API and sequencing

Candidate minimal public roles are ConventionalGilt, GiltPricingPlan,
GiltValuation, GiltYieldReport, GiltRisk and GiltError. A first-period role can
be an inspectable enum if needed; avoid an untyped market-name string that
silently changes generic Bond behavior. Existing BondPrice/Kind, explicit
RootBracket/SolverOptions and exact Amount/rounding types can be reused.
Every new type is immutable, private-field, documented and Send+Sync; there is
no default clock/calendar/bracket/accuracy or implicit binary-money conversion.
Public names/counts remain proposals requiring a complete contract before code.

Concrete candidate signatures are:

```text
ConventionalGilt::new(original_issue, maturity, coupon_per_100:DecimalAmount,
    first_dividend, quasi_dates:Vec<Date>, ex_dates:Vec<Date>, max_dates)
    -> Result<Self,GiltError>
ConventionalGilt::dividend_per_100(index) -> Result<DecimalAmount,GiltError>
ConventionalGilt::dividend_cash(index, held_nominal:DecimalAmount,
    rounding:SettlementRounding) -> Result<DecimalAmount,GiltError>
GiltPricingPlan::new(&ConventionalGilt, settlement:Date, max_bundles)
    -> Result<Self,GiltError>
GiltPricingPlan::evaluate(yield_rate:f64) -> Result<GiltValuation,GiltError>
GiltPricingPlan::yield_from_price(BondPrice,RootBracket,SolverOptions)
    -> Result<GiltYieldReport,GiltError>
GiltPricingPlan::risk(yield_rate:f64) -> Result<GiltRisk,GiltError>
GiltPricingPlan::accrued_cash(traded_nominal:DecimalAmount,
    rounding:SettlementRounding) -> Result<DecimalAmount,GiltError>
```

Instrument inspectors retain dates, exact C, all source ratios/roles and cutoff
provenance. Plan/result inspectors retain model/version, settlement, original
target/policies, AI, per100 dirty/clean and cached solver payload/counts. GiltRisk
provides price_derivative, price_second_derivative, time_weighted_amount,
modified_duration, macaulay_duration and convexity, each with the defined
independent admission policy. A trade-proceeds adapter uses existing explicit
SettlementEvidence/Instruction policy where appropriate; it never upgrades an
analytical approximation into an ideal-price guarantee implicitly.

The instrument retains source terms; plan preparation resolves current period,
exact lambda coordinates, entitlement, source AI and bounded pricing bundles
once. Repeated price, risk and yield do zero date/calendar work. Count/cap/
shape checks precede allocation; reserve fallibly. Original indices and source
roles survive sparse skipped/detached dates. Quote validation precedes risk
workspace allocation; each original buyer bundle validates its factor before
product and extended risk work. Missing/detached payments are absent, not
synthetic zero flows used to bypass a failing factor.

Propose per100-native analytic results, making units clear without an arbitrary
pricing face. Cache one correctly rounded binary source AI view for analytic
clean/dirty conversion; retain exact rational AI ingredients separately for
cash. Dirty evaluation and risk do not eagerly fail on an unused clean view.
Yield takes explicit Clean/Dirty BondPrice, resolves the clean shift once and
uses fixed target-derived scale0/1 with exact residual subtraction. It reuses
the private fallible payload-carrying bounded Brent, exact lower-first counts,
typed failure stages and cached dirty endpoint payload. No final repricing or
special uncounted algebraic inversion is introduced. The n=0 formula is an
independent reference, not permission to forge a SolveReport.

For C>=0, par100 and S<M, all retained lambda are positive and the real dirty
price is strictly decreasing on y>-2. That proves conditional uniqueness, not
that a supplied bracket works, finite factors are representable, the rounded
function is strictly monotone or a reported root has exact mathematical zero.
Resolve the new API's nonpositive dirty-target policy explicitly: either a
proved no-admissible-target error before callbacks, or the generic solver's
normal explicit-bracket behavior. Do not change generic signed-target rules.

Proposed error families preserve typed Value/Amount/Calendar/Sum/Solver causes,
source-role/date/index/phase, actual callback point/counts and stable kinds.
They must distinguish invalid source terms, insufficient cycle coverage,
unsupported adjustment/entitlement domain, publication precision/range,
factor NumericalBoundary, PV range, dirty reduction, raw/normalized view range,
target conversion/shift, allocation/cap and defensive capacity. Root error
integration and a complete public census are adoption work, not implicit here.

## 7. Settlement without erasing publication stages

Use exact positive traded/held par nominal N, independent of analytic face.
Published dividend distribution uses N*D/100 with exactly the chosen final
cash quantizer; preserve issuer-to-holder roles. AI cash uses the **exact source
rational** directly on N, not a rounded per100 AI. For standard/short/long
roles it can be represented as `N*C*k/(100*d)` with small signed k and positive
d (including s1*s2 when needed). Its final minor-unit quantizer can be decided
by exact integer quotient/remainder.

Existing SettlementEvidence cannot denote arbitrary nonterminating rational
AI as an exact decimal point. A rounded decimal approximation must not be
passed as exact source AI. Propose a bounded private rational cash adapter,
or a separately justified declared enclosure with an honest guarantee. With
N/C scales<=38, source coefficients require at most147 bits, and standard AI's
two-day-ratio multiplier fewer than18 bits. Including quantum scale/coefficient,
conservative scalar numerator/denominator bounds stay below410 bits, inside
eight u64 words. The final contract must audit those bounds for each source
role, shift, signed tie and canonical i128 result path. Never multiply
two DecimalAmounts and prematurely reject an intermediate that a division
would rescue.

For yield-to-cash settlement, multiply the unrounded analytical price by N/100
before final penny quantization. A binary64 price gives the established
ForSuppliedBinary64 meaning, not certified rounding of the ideal gilt price.
A genuine ideal-price receipt needs a justified enclosure whose entire image
rounds to one quantum; empirical M4 comparison tolerances alone cannot supply
that certificate. Close-price quotes supplied as exact decimals are a distinct
input mode, with their own exact arithmetic meaning.

Keep signed cash mode and aggregation explicit. Positive six-place publication
ties are resolved by the source. Negative AI half-penny ties and final invoice
RoundTotal versus component rounding are not completely resolved by the
reviewed wording. Offer the existing explicit policy vocabulary without calling
an arbitrary choice DMO-certified. These gaps remain required source work,
alongside UK business-calendar maintenance and final financial acceptance.

## 8. Adoption checklist and remaining decisions

Before code, root must review a complete frozen contract covering all sections
together. Required choices are exact cycle generation/coverage and February
rules; first-role/auction domain; supplied versus verified ex-date provenance;
source-publication and fallible-view representation; final exact bundle PV staging;
rational-lambda factor/error policy; RN-delta risk policy and width proofs;
every public signature/error/order/count; signed cash/aggregation semantics;
and the analytical-enclosure boundary. No partial adoption of price while
omitting yield, duration or convexity is proposed.

Required evidence includes publisher standard/short/long examples with raw
precision and displayed-rounding distinctions; exact six-decimal ties; every
AI branch and boundary; original/reopened issue roles; skipped-quasi versus
actual-payment equality; unadjusted-cycle rejection controls; n=0 and n=1;
y=0, adjacent yields and near -2; exact-quasi versus rounded annual-time model
discriminators; all six ideal/staged risk values; independent closed-form and
multi-payment inverse bounds; full callback traces/caps/payloads; signed range
and coefficient/accumulator extremes; exact nominal/cash publication-stage
discriminators; and unchanged generic M3/M4/M6/M7/bond/settlement evidence.
The companion [test design](M8_GILT_TEST_DESIGN.md) and
[numerical review](M8_GILT_NUMERICAL_DESIGN_REVIEW.md) supply independent
challenges. This proposal is ready for that review; it is not financial,
platform, full-M8 or publication acceptance.
