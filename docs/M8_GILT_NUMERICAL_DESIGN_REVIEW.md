# M8 conventional-gilt numerical design review

This is an independent local engineering review dated 2026-10-18 against
`c70bf5f`, not an adopted contract, implementation, named-market acceptance or
financial-accuracy certificate. Only this new document is owned by this review.
The existing generic yield/risk contract and frozen numerical evidence remain
unchanged. No QuantLib implementation was consulted.

## Source scope and notation

The primary source is the [DMO fourth-edition formula paper, 18 December
2024](https://www.dmo.gov.uk/media/334d05fo/yldeqns_v4.pdf), locally pinned by
SHA-256 `bd752ac495ec340c0d34e7d6d1e6fe0af2e1d25819364935312f53a9f5037745`.
This review visually inspected complete PDF pages 5, 6, 16–21, in addition to
the inspection recorded in [the source review](M8_GILT_SOURCE_REVIEW.md).
The following source summary is deliberately limited to conventional gilts.

The source uses dirty price per 100, annual decimal yield, semiannual
quasi-coupon dates, and `a=r/s`, with `r=s` on a quasi date. The maturity cycle
includes dates without payments and retains unadjusted nonbusiness dates.
Its next and next-but-one entitled dividends can differ from regular coupons.
[Pricing definitions, printed pp.4–5](https://www.dmo.gov.uk/media/334d05fo/yldeqns_v4.pdf#page=5)

Short and long first dividends receive six-decimal publication rounding;
standard conventional dividends are `c/2`. Standard AI uses the full published
next dividend, but short/long first-period AI uses the original coupon and
day ratios. The ex side is strictly after the cutoff. Settlement rounding
acts on traded nominal; the price itself is not first rounded to six decimals.
[Printed pp.14–20 and introduction](https://www.dmo.gov.uk/media/334d05fo/yldeqns_v4.pdf#page=15)

All mathematics and numerical proposals below are independent deductions.
Write `q=1+y/2`, `delta=2+y`, `L_j=r+j*s`, and `lambda_j=L_j/s`. The proposed
domain is pre-redemption settlement, `0<r<=s`, and `y>-2`; lifecycle and input
validation require an explicit new contract. There is no implied UK calendar,
currency, settlement quantum, rounding mode or aggregation default.

## Stable price and the empty next quasi date

Replace the geometric quotient by its finite-sum identity. For `n>=1`, let
`D_0=d1`, `D_1=d2`, and `D_j=c/2` for `2<=j<=n`; add redemption 100 at index
`n`. For `n=0`, combine `d1+100` at index zero. With each resulting exact cash
bundle denoted `A_j`, the ideal source model is

```text
P(y) = sum(j=0..n) A_j * q^(-lambda_j).
```

The regular tail is empty when `n=1`; it must not manufacture an additional
regular coupon. At `y=0`, every discount is exactly one and `P(0)=sum A_j`.
This removes the removable `0/0` singularity in the printed geometric quotient.
For positive source dividends, an exact or compensated reduction of admitted
terms avoids geometric cancellation; the existing exact reduction gives a
particularly explicit reproducibility policy after individual PV admission.

Use `log1p(y/2)` rather than `log(1+y/2)` around zero. For example, at
`y=2^-53`, binary64 `1+y/2` is exactly `1`, although `log1p(y/2)` is positive.
The computational order still needs to be frozen: at `y=MIN_SUB`, an ordinary
binary64 `y/2` becomes zero. No finite staged observation then becomes an
ideal-price certificate merely by using `log1p`. A prospective implementation
can use `exp(-lambda_j*logq)` with a separately specified exact-ratio product,
or an explicitly qualified rounded exponent. Repeated multiplication by `v`
is an alternative with different accumulated error and range behavior; it
must not silently replace independent per-index observations.

At `next_up(-2)`, the exact base is `2^-53`. This is a valid quote whose early
terms can overflow their individual discount even when a tiny dividend could
rescue the product mathematically. Whether to preserve the generic rule
“factor admission before amount multiplication” must be explicit. Preserving
it is the narrowest reuse: no zero amount, small amount, or later reduction
rescues an invalid factor/PV. The floor itself is rejected before workspace
allocation and factor traversal. No elementary-function call is necessary
for the explicit zero-yield shortcut, but all other required validations and
the chosen PV grouping policy still apply.

An empty next quasi date is essential model data. At issue, take
`r=31`, `s=182`, `n=1`, `d1=0`, published `d2=3.510989`, and exact decimal
yield `.05`. Redemption and the first actual dividend occur at index one.
Independent Decimal calculations at 160 and 240 digits agree, when serialized
to 80 fractional digits, on

```text
correct: 100.562485828466661025439446813095124900517029555938...
shifted: 103.076547974178327551075432983422503023029955294836...
```

The second value wrongly moves the first actual payment to index zero after
discarding the empty quasi date. Its difference, approximately `2.5140621457`,
is a model error. A payment-date-only plan cannot recover the missing index
without retaining the maturity-based quasi lattice.

## Exact quasi coordinates and generic ICMA reuse

Preserve `L_j` and `s` as integers. In general

```text
RN((r+j*s)/s) != RN(RN(r/s)+j).
```

For `r=8`, `s=181`, `j=1`, correctly rounded `189/181` has binary64 bits
`3ff0b509e68a9b95`; rounding `8/181` first and then adding one gives
`3ff0b509e68a9b94`. An annual time assembled from a rounded partial ICMA
fraction and exact half-year pieces can inherit this second rounding. Feeding
that prepared time into generic periodic pricing does not establish equality
with the exact quasi-rational source model.

Even `RN(lambda_j)` is a proposed numerical approximation, not an exact
representation of the source ratio. If retained for the elementary factor,
its error must be carried explicitly. By the mean-value theorem,
`abs(exp(-lambda_hat*h)-exp(-lambda*h))` is bounded by
`abs(lambda_hat-lambda)*abs(h)*exp(max(-lambda_hat*h,-lambda*h))` for a fixed
real `h`; the computed logarithm and exponential contribute separate errors.
Use an outward bound and handle range boundaries explicitly. A measured
platform profile for the old binary-time expression does not prove this
different expression uniformly accurate over every date, quote or target.

The calendar coordinate is also semantically distinct from civil
Actual/365 elapsed time. Reusing schedule dates, bounded allocation helpers,
the exact PV reduction, residual scaling and the shared solver is reasonable.
Reusing the generic rounded-time model and its unchanged numerical claims is
not sufficient. This named model must not alter generic ICMA results.

## Analytic risk and an explicit numerical observation policy

With exact real source coefficient and coordinate, put
`W_j=A_j*q^(-lambda_j)`. Differentiating with respect to annual decimal yield
gives

```text
P'  = -sum W_j*lambda_j/delta
P'' =  sum W_j*lambda_j*(lambda_j+1)/delta^2
M   =  sum W_j*lambda_j/2
Modified = -P'/P       Macaulay = M/P       Convexity = P''/P.
```

Convexity has no factor one-half. `M` is the quasi-coordinate time-weighted
amount, not an assertion about civil elapsed years. The ideal common-periodic
identity is `Macaulay=q*Modified`. It is an oracle check with propagated
numerical error, not a shortcut that should replace independently retained
measures. The derivatives are those of the ideal model, not derivatives of
the discontinuous floating-point observation program.

A narrow numerical policy can retain an accepted rounded per-bundle PV seed
`Vhat_j`, use `delta_hat=RN(2+y)`, and form exact rational multipliers

```text
P' term  = -Vhat_j*L_j/(s*delta_hat)
P'' term =  Vhat_j*L_j*(L_j+s)/(s*s*delta_hat*delta_hat)
M term   =  Vhat_j*L_j/(2*s).
```

Round each nonzero contribution once to 53 normalized bits with an extended
exponent, then accumulate exactly as in the
[frozen generic risk contract](M8_BOND_YIELD_RISK_CONTRACT.md). Keep `L_j+s`
exact; do not round `lambda_j` or `lambda_j+1` to create risk operands. This
policy is not adopted here. It needs a new accuracy analysis for coefficient,
factor, PV, delta and extended-term errors. In particular, source-ideal
derivatives use exact `2+y`, while this local numerical recipe uses its RN
image.

That distinction matters for capacity as well as accuracy. Close to `-2`,
Sterbenz subtraction makes `2+y` exact. Over the entire finite quote domain,
it can have a much longer exact significand: at `y=MIN_SUB`, reduced rational
`2+y` has a 1076-bit numerator. A 53-bit-delta scalar proof cannot be used for
an exact-delta implementation. Either retain the explicitly qualified RN
policy, or prove and implement a wider exact-delta operation.

## Conservative capacity proof for the RN-delta proposal

The following proof requires an explicit bound `n+1<=M`, where
`M=3,652,059<2^22` is the existing flow cap, and day counts
`0<r<=s<2^22` (the entire supported 0001–9999 date span is smaller than this).
It does not rely on economically typical coupon lengths or yields. A more
restricted maturity lattice can tighten these bounds, but the contract must
state which cap counts quasi dates, including dates without a payment.

For `0<=j<=n`, both `L_j` and `L_j+s` are below `2^44`, since `M+1<2^22`.
After separating binary exponents, accepted PV and delta significands each
have at most 53 bits. Therefore:

| Exact scalar quantity | Strict upper bound |
|---|---:|
| `sig(Vhat)*L_j*(L_j+s)` | `2^141` |
| `s*s*sig(delta_hat)^2` | `2^150` |
| `sig(Vhat)*L_j` | `2^97` |
| `s*sig(delta_hat)` | `2^75` |

For positive integers `N<2^141`, `D<2^150`, let
`k=floor(log2(N/D))`. Alignment for a 53-bit normalized quotient shifts the
numerator by `52-k` when nonnegative; the shifted numerator is below
`D*2^53<2^203`. When the shift is negative, the shifted denominator is smaller
than the original numerator, so remains below `2^141`. One extra correction
bit gives a conservative 204-bit working bound. Exact quotient/remainder,
ties-even carry and remainder comparison using `D-remainder` fit 320 bits.
The old five-binary-numerator/two-binary-denominator operand-count argument is
not this proof; integer `s` factors must be included explicitly.

With finite `y>-2`, `2^-52<=delta_hat<2^1024`. For nonzero accepted PV,
`2^-1074<=abs(Vhat)<2^1024`, and `2^-22<lambda_j<2^22`. These yield conservative
unrounded contribution bounds:

For the second derivative, use the stronger retained count bound
`lambda_j<=M`: the actual `M*(M+1)<2^44`. The coarse inequality
`lambda_j<2^22` alone would only establish a `2^45` product bound.

| Measure | Lower magnitude | Upper magnitude |
|---|---:|---:|
| `P'` | `>2^-2120` | `<2^1098` |
| `P''` | `>2^-3144` | `<2^1172` |
| `M` | `>2^-1097` | `<2^1045` |

Allowing one bit for 53-bit rounding, no retained bit lies below `2^-3196`;
each rounded magnitude is below `2^1173`. Fewer than `2^22` terms put each
unsigned accumulator below `2^1195`, or below `2^9387` on the existing
`2^-8192` basis. The generic 243-u64-word arrays have 15,552 bits and are ample.
The same finite-binary64 dirty denominator leaves their direct normalized
getter alignment proof applicable. No early raw-derivative admission is
needed; raw overflow/underflow and normalized overflow/underflow remain
independent getter outcomes. Dirty zero must precede a zero numerator in a
normalized getter.

Integer scratch checks exercised the conservative extrema and confirmed
141/150-bit scalar products. A fixed-seed independent integer check of 10,000
positive ratios within those bounds reached 203 working bits and no larger
alignment; the analytic bound above is the capacity argument. These prove
capacity under the stated invariants,
not elementary-function accuracy or financial settlement correctness. Extending
the API to free integers, signed arbitrary bundles, larger schedules, or exact
unrounded delta requires a new proof before reuse.

## Published amounts, binary coupons and source asymmetry

Keep the exact decimal coupon per 100 separate from a binary annual rate.
The latter does not encode the same input. Compute short/long publication
rounding with integer quotient/remainder from the exact supplied coupon and
day ratios. Formatting an already rounded binary coupon to six decimals is
not an independent implementation of that rule.

Full existing decimal inputs expose two representation traps. If `c=1e-38`,
regular `c/2=5e-39` needs scale 39, beyond `DecimalAmount`. Also a large valid
coupon's six-decimal first dividend can require a coefficient of about 147
bits, although its magnitude and eventual binary64 conversion are finite.
Adding redemption to a dividend at a common decimal scale can exceed signed
i128 too. Do not silently narrow coupons, round regular dividends to six
decimals, or treat these as market arithmetic overflows. Retain a bounded
private rational cash coefficient, or explicitly restrict the proposed input
domain with justification before adoption. These input-policy decisions
precede a production-width proof for first-dividend publication and bundle
conversion.

There is a bounded std-only route without narrowing the coupon domain. If
the decimal coupon is `Ccoef/10^a`, a regular coupon-plus-redemption bundle
has numerator `Ccoef+200*10^a` and denominator `2*10^a`; conservative bounds
are 135 and 128 bits respectively. Computing a long first publication in
micro-units uses `Ccoef*(s+r)*10^6/(2*s*10^a)`. Under the broad day bound its
numerator is below `2^170`, and the rounded micro-unit coefficient is below
`2^147`. Adding par in this basis conservatively stays below
148 bits. Multiplying any such source coefficient by an accepted 53-bit
discount significand therefore has numerator below `2^201` and denominator
below `2^128`. Relative normalized quotient alignment fits a conservative
202 bits, within a five-word scalar. This does not authorize unchecked large
absolute shifts for IEEE subnormal admission: keep the discount's binary
exponent separate, classify range, and prove every alignment path. Existing
helpers whose interfaces only accept binary64 factors need adaptation and
review even when the underlying integer storage is sufficient.

For the independently proposed rational AI cash adapter, the actual validated
six-month cycle supplies `s1,s2<=184`. Writing long cum AI as
`Ccoef*(r1*s2+r2*s1)/(2*10^a*s1*s2)`, its small integer numerator is below
`2^17` and denominator day product below `2^16`. Multiplying by decimal
nominal, dividing by 100, and quantizing at any existing decimal quantum gives
conservative integer numerator/denominator bounds below 405 bits, within
512 bits, including both decimal scales up to 38 and quantum scale up to 38.
Published-dividend cash fits this bound too. This proof depends on the actual
semiannual-period validation. Replacing it by only the broad `s<2^22` date
bound raises conservative extrema above 410 bits (scratch extrema reached
425/431), although still within 512. Do not mix these two proof assumptions.
Signed ties, exactness, final i128 admission and operation/error context remain
separate contract obligations; no public rational-money framework is needed.

A new contract must also choose the PV observation order. The cleanest source
grouping is to combine same-quasi-date dividend and redemption exactly before
one PV observation, while retaining separate cash-distribution metadata. It
differs from generic per-coupon/per-principal PV seeds. Whether the coefficient
is rounded first or multiplied exactly by the accepted discount and rounded
once is another real choice. Exact integer ties-even scratch controls found:

| Inputs and observation | Result bits |
|---|---|
| exact decimal `0.000001 * next_down(1)` rounded once | `3eb0c6f7a0b5ed8d` |
| binary `RN(0.000001) * next_down(1)` | `3eb0c6f7a0b5ed8c` |
| exact `(3.510989+100)*next_down(1)` rounded once | `4059e0b40b34e768` |
| two separately rounded exact-cash PV seeds, then exact sum and RN | `4059e0b40b34e767` |

Thus algebraic equality does not decide the numerical contract. The source
alone does not select a unique binary64 program. Either policy needs explicit
coefficient-error propagation; using RN coefficients is a qualified
approximation of exact source dividends.

The long-first discriminator `c=6`, issue ratio `31/182` gives unrounded
dividend `639/182`, published as `3.510989`; their exact difference is
`1/91,000,000`. Independently, the long second-period cum AI at `r2=92`,
`s2=183` is `22417/11102`, while ex AI at `r2=182` is `-1/61`. Replacing raw
source AI by cum AI minus the rounded published dividend changes it. Neither
an exact earned-minus-published identity nor clean-price continuity is an
acceptance invariant. Keep published dividend, buyer entitlement and source
AI as distinct data.

## Error order, inversion and evidence obligations

Adopt and test an explicit order: constructor structure/date/amount/cap checks;
quote validation; bounded workspace allocation; quasi-index-order factor,
bundle PV and extended-risk terms; exact dirty reduction; report construction;
independent getter admission. Missing or detached payments still affect the
quasi index; whether their zero bundles cause factor observations must be
specified rather than inherited accidentally. Keep error indices defined as
quasi indices or retained-bundle indices, with an inspector permitting an
unambiguous interpretation. Source publication arithmetic and AI preparation
errors should occur at construction, not as a disguised callback failure.

The shared lower-first bounded solver, cached dirty payload, scaled exact
residual and counted callback failures are reusable without a new numerical
solver framework. Preserve lower-zero success without eagerly validating an
unqueried upper quote. A nonnegative conventional cash model with redemption
and positive quasi horizons is ideally strictly decreasing; that establishes
conditional uniqueness on `y>-2`, not existence, computed monotonicity or a
uniform inverse error bound. Signed generalized bundles, if added, need their
own classification; there is no need to expose them for the initial model.

The `n=0` inverse is useful as an independent oracle. An `expm1`/log-ratio
form is better conditioned near par than a power minus one, but it must not
introduce an uncounted special solver path with different reports. For any
inverse fixture, derive the root of the actual resolved target independently;
do not accept the yield that planted a rounded price as the root. Uniform
price-error bounds are necessary before extending an ideal root enclosure
to computed callback zeros. Solver tolerance is a coordinate stopping rule,
not a market quote-accuracy certificate.

Independent tests should cover these distinct obligations:

- Exact Fraction coordinate controls for every realizable semiannual period
  length, all settlement offsets, large indices, `r=s`, and the one-ULP example
  above; separately validate lambda-plus-one and annualization.
- Finite-sum versus geometric identity away from zero; exact zero limit;
  adjacent signed zero yields and small positive/negative yields; `n=0`,
  `n=1`, ordinary tails, long-first empty quasi and detached first coupon.
- Analytic derivative/moment oracles formulated independently, plus central
  differences only as a supplementary sanity check. Include denominator 183,
  near-`-2`, very large yield, raw-only and normalized-only range failures,
  and all exact extended-term rounding ties/carries.
- Exact decimal publication ties around the sixth place; all 39 existing
  scales and coefficient limits; the scale-39 regular half-coupon; grouped
  redemption and coefficient-first versus exact-product seed controls.
- Full published first dividend versus entitled dividend versus AI, every
  cutoff equality/neighbor and both long-first quasi periods. Check source
  rounding and signed cash policy separately, without assumed clean continuity.
- Original-index typed errors, factor failure before product rescue, zero
  bundles, dirty admission before report, independent getters, exact callback
  budgets and payload reuse. Boundary tests must use valid public prepared
  states, not forged risk reports.
- Fraction/integer authority for rational stages and at least two sufficiently
  high Decimal precisions for transcendental anchors. Matching two precisions
  is reproducibility evidence; interval or analytic bounds are a separate
  obligation for certified mathematical comparisons and cash decisions.

The seven explicit integer/bit controls and the 160/240-digit long-first
anchor calculations above were actually executed during this review. They
are scratch design evidence, not a saved generator, frozen fixture, Rust
test result, implementation verification or market acceptance claim.

## Decisions required before freezing a contract

The minimal necessary new surface is one explicit conventional-gilt prepared
model with inspected quasi coordinates, exact coupon/publication provenance,
entitlement and source AI, plus existing-style price/risk/yield reports and
typed errors. A general yield-model plug-in framework, configurable frequency,
new solver, arbitrary-precision dependency or implicit money defaults are not
needed to implement the reviewed semiannual case.

Before implementation, freeze the lifecycle domain, quasi-date cap/anchoring,
coupon input domain and exact cash representation, zero-bundle observations,
PV grouping/coefficient rounding, factor/exponent policy, RN-delta risk policy,
accuracy qualification, and precise error precedence. Keep the source
cutoff equality explicit. The exact rational AI-to-cash adapter also needs
an adopted bounded operation and explicit rounding/aggregation choices;
AI containing a denominator such as 183 is not exactly representable by
silently creating decimal or binary point evidence. The
[settlement contract](M8_SETTLEMENT_CONTRACT.md) remains authoritative for
its existing recipes; this review does not extend it.

Named UK calendar maintenance, issued-security coverage, historical cutoffs,
index-linked gilts, strips, other countries' conventions, NI-01 ideal-price
settlement guarantees and formal M8 acceptance remain separate gates. In
particular, exact rounding of supplied binary64 bits remains a statement
about those bits, not proof that the ideal gilt price lies on the same side
of a cash quantum boundary.
