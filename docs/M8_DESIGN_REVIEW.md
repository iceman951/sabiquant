# M8 design adversarial review

Read-only review dated 2026-10-16, from local baseline `4da3f43` on
`feat/m8-bond-readiness`. This review does not adopt a contract, approve an
ADR change, implement bond or amount APIs, close NI-01, or certify M8.
Continuing delegated authority permits the local design/implementation path;
there is no new user-permission requirement for preparing or carrying out
that authorized local work. Formal owner, financial, platform and release
acceptance remain distinct claims.

The reviewer authored [the bond API proposal](M8_BOND_API_DESIGN.md), so the
comments on that document are self-review. The reviewer independently inspected
parent-owned [DMO evidence](M8_GILT_SOURCE_REVIEW.md),
[historical ISMA evidence](M8_ICMA_SOURCE_REVIEW.md), another agent's
[exact amount proposal](M8_EXACT_AMOUNT_DESIGN.md),
[Treasury evidence](M8_US_TREASURY_SOURCE_REVIEW.md) and
[public reference exploration](M8_BOND_REFERENCE_DESIGN.md). No QuantLib
implementation was read. Only this new review document was written for the
review; the amount proposal's author made the fix identified below.

## 1. Findings that affect an executable contract

| Finding | Severity before implementation | Current disposition |
|---|---|---|
| DMO reuses `d1` for different financial roles in price and accrued formulas | Major: wrong negative ex accrual if one field is reused | Source pages visually confirmed; separate fields required in any named kernel |
| Amount-times-bounded-ratio does not cover every decimal price-per100 product | Major: rejects representable rounded final values | **Resolved in proposal** by fused two-decimal multiply_divide and an eight-limb bound |
| Positive-time signed flows can still have identically constant price | Important: zero-horizon scan is not complete degeneracy detection | Add explicit cancellation case and qualify any constant-price detection claim |
| Generic direct-horizon/hybrid factors do not establish named Treasury/DMO yield | Major semantic risk | Proposals distinguish model kernels; source-complete capability checks remain required |
| Exact rounding of supplied decimal/f64 data does not certify ideal settlement | Major claim risk | Exact amount proposal correctly preserves this distinction and the NI-01 phases |
| Yield residual and risk normalization can fail even when price is finite | Important numeric/domain risk | Freeze residual/risk error contracts rather than manufacture a finite answer |

These are pre-implementation design obligations, not discovered production
regressions. No M8 production code exists in the reviewed tranche.

## 2. Literal DMO algebra and the duplicated symbol

The existing complete renders of printed DMO pages 4, 5 and 17 were visually
inspected using the read-only PDF workflow, after reading the PDF skill. No
PDF was authored, edited or re-exported. Source acquisition/version/hash and
the parent's other inspected pages remain as recorded in the source review.
The price transcription in that review matches the rendered formulas.

Let `a=r/s`, `v=1/(1+y/f)`. For n>=1 the geometric part expands as

```text
(c/f) * (v^2 + v^3 + ... + v^n), containing exactly n-1 terms.
```

Therefore n=1 has **no geometric terms**. It must bypass the quotient even
when y=0, since its literal numerator/denominator would otherwise be 0/0.
The separately printed n=0 formula is `v^a*(d1+100)`; applying the n>=1
expression at n=0 would introduce an erroneous term.

Exact zero-yield limits are:

```text
n = 0: P = d1 + 100
n >=1: P = d1 + d2 + (n-1)*c/f + 100
```

A stable finite term plan can represent coupon exponents a, a+1, ..., a+n,
with redemption at a+n, preserving separate coupon/principal roles even when
exponents agree. It avoids a geometric cancellation near y=0. A direct sum
must still have a bounded count and operation order; an unbounded loop is not
an acceptable implementation. A log1p/expm1 closed-form alternative needs its
own zero/near-zero and summation error evidence.

For an illustrative standard c=5, f=2, d1=d2=5/2 and a=1/2, independent
Fraction arithmetic gives the following model anchors. These are original
algebraic checks, **not printed source examples**. Derivatives are with
respect to decimal nominal annual y.

| n | P(0) | P'(0) | P''(0) |
|---:|---:|---:|---:|
| 0 | 205/2 | -205/8 | 615/32 |
| 1 | 105 | -155/2 | 1545/16 |
| 2 | 215/2 | -1045/8 | 7265/32 |
| 17 | 145 | -2155/2 | 150545/16 |

A scratch Python Decimal comparison at 80 and 120 digits also compared the
expanded terms with the appropriate n=0/n=1/geometric forms for n in
{0,1,2,17} and y in {.05,-1.9,1e-20,-1e-20}. Every comparison was below
1e-55 relative disagreement at each precision. This checks this derivation;
it is not a library battery, a universal libm bound or an interval certificate.
The y=0 checks used exact rational sums rather than evaluating 0/0.

The critical symbol-role distinction is on the rendered pages themselves:

- Pricing p4 `d1` is the next buyer cash flow and may vanish in the ex period.
- Accrued p17 `d1` is the published next dividend. It remains the full dividend
  used in `(t/s-1)*d1` after the cutoff.

A named kernel must retain separate `entitled_next_cashflow` and
`published_next_dividend` concepts. Substituting a zero pricing d1 into the
accrued expression would wrongly turn negative ex accrual into zero. The
parent has been informed. The source review alone should not be treated as
proof that one shared program field is appropriate.

The p17 equality rule is strict: on the cutoff remains the first case;
strictly after uses the negative-accrual case. The public QuantLib probes
already remove the coupon at ex equality. These are deliberate boundary
classes, not floating-point discrepancies. Keep explicit After/OnOrAfter
choices and classify the reference difference. The generic proposal has no
boundary default. A maintained UK holiday calendar and legal market-date
factory are still unimplemented; accepting supplied cutoff dates does not
claim seven-business-day UK date generation.

## 3. Treasury, gilt and generic time coordinates

The Treasury source review correctly limits its rules to the inspected auction
contexts and distinguishes an inferred n=0 case from a printed final-period
example. Its known source inconsistency and illustrative intermediate rounding
remain visible. It does not supply a universal secondary-market entitlement
contract.

A DMO conventional factor has the form `(1+y/f)^(-L)`, with specified
quasi-period exponent L. A Treasury regular reopening combines a fractional
first **simple** factor and subsequent integer-period powers. In general,

```text
(1+a*y/f)^(-1) * (1+y/f)^(-k)
    != (1+y/f)^(-(a+k)).
```

Existing SimpleThenCompounded switches the whole factor by its supplied time;
it does not automatically mean a simple fractional prefix multiplied by
periodic factors for later payments. Selecting that enum cannot substitute for
the full Treasury formula. Similarly, direct YF(settlement, adjusted payment)
may differ from a market's raw quasi-cycle coordinate. The exploratory public
reference reports an adjusted payment outside a schedule-aware yield day-count
hull; using another context to hide that error would change the inputs.

A complete named contract needs formula-specific factor terms, exact source
version, date-role construction, entitlement/accrual mapping, irregular-period
capabilities, yield units, finite domain and rounding provenance. Formula-only
low-level inputs may be useful, but must not be presented as a fully generated
market bond with dates the caller never supplied or the library never verified.
The generic direct-horizon model remains useful on its explicitly stated terms.

Historical ISMA definitions support Macaulay weighting, modified duration
`-P'/P` and convexity `P''/P`; the Taylor expansion's half belongs outside the
convexity definition. The source review appropriately classifies the rounded
intermediate duration example. It must not become a widened ideal-oracle
tolerance. A hybrid's branch is fixed when yield varies at fixed times; its
time-switch issue is a different differentiation question.

## 4. Constant price, roots and risk denominators

All paid/ex-detached plans and all nonzero flows with t=0 are useful constant
price cases. Distinct future dates can have t=0 under 30/360, so date comparison
alone is insufficient. They are not the only cases.

Counterexample: one annual coupon has amount -100 and redemption is +100,
with the same positive-time payment and discount kernel. The buyer dirty value
is identically zero for every valid yield, as are both analytic derivatives.
A zero-horizon-only scan will miss this. A constant target of zero admits every
yield; another target admits none. A generic bracket solver may nevertheless
return FunctionZero at its first valid endpoint. That is valid as a computed
root observation and supplies no unique-yield claim.

Before freezing, choose explicitly between structural constant-price detection
and a qualified generic root result. At minimum detect/document the trivial
same-kernel cancelling case. Grouping identical kernel terms can help establish
mathematical degeneracy, but must not silently change the original per-flow
rounding/validation semantics; exact sums of individually rounded products are
not generally interchangeable with a product of grouped amounts. A sufficient
certificate must not be advertised as a complete symbolic equivalence solver.

The nonnegative-flow, positive-time uniqueness certificate in the bond proposal
is appropriately conditional. Signed coupons must remain supported, with
uncertified global uniqueness and independently constructed multiple-root tests.
A single yield round trip is inadequate evidence because price and inverse
can share the same wrong kernel.

Dirty price zero makes normalized modified duration, convexity and time-weighted
duration undefined even when raw derivatives are finite. Negative dirty price
must not be normalized with abs(P) without changing the definition. Empty,
fully paid and exactly cancelled plans need explicit tests of these distinctions.
Price-only queries should not eagerly fail because an unused risk derivative
would overflow. Every yield callback failure must preserve the actual typed
pricing cause/stage/index rather than return NaN or a spurious successful root.

## 5. Exact amount widths and the resolved fused-operation gap

The scale0..38, canonical i128 model is internally coherent as a proposed
finite decimal value set. Separate sign/magnitude correctly handles MIN;
normalization before final range checks admits representable cancellation
cases. Quantum-count parity, negative rounding directions and exact supplied
binary64 interpretation are explicitly defined. The following independent
integer calculations confirm the proposal's magnitude bounds:

| Operation magnitude | Maximum bit count |
|---|---:|
| abs(i128 numerator)*10^38 | 254 |
| u128 denominator*positive i128 quantum | 255 |
| two coefficient magnitudes*10^38 | 381 |
| u128 divisor*positive i128 quantum*10^76 | 508 |
| largest finite binary64 significand*2^971*10^38 | 1151 |
| positive i128 quantum*2^1074 | 1201 |

The original amount-times-ratio signature used a u128 denominator and could
not encode every decimal price divided by 100. A concrete counterexample is
notional MAX and price `(10^37+1)/10^37`. The exact intermediate decimal product
has a 250-bit canonical coefficient, so it cannot be a DecimalAmount. The ratio
price/100 has reduced denominator 10^39, exceeding u128. Nevertheless the
single final quantum-one nearest/ties-even result is representable:

```text
1701411834604692317316873037158841057
```

The amount author responded by adding fused
`multiply_divide(other_decimal, nonzero_u128_divisor, quantum, mode)`, with
A=abs(C1*C2)*10^s and B=divisor*Q*10^(a+b). The revised proposal retains
381/508-bit bounds and recommends one eight-u64-limb (512-bit) phase-A core.
The counterexample, API, precedence, bounds and tests are now explicit. The
bounded amount-times-ratio operation remains useful but no longer claims full
two-decimal price coverage. This finding is resolved at design level, pending
implementation and independent exact integer/Fraction tests.

Eight limbs also cover a one-bit shifted division remainder under the 508-bit
bound; the implementation must maintain that invariant rather than discard an
outgoing carry. Nineteen limbs (1216 bits) cover the separate full-finite
binary64-to-quantum path's 1201-bit denominator and remainder state. These are
fixed capacities with input-bound proofs, not a public arbitrary integer API.
The five-limb exact-slice-reduction argument on 64-bit targets remains valid:
per-term aligned magnitude <2^254 and count <2^64 give accumulator <2^318.
No claim requires constructing an impossibly large slice to test that proof.

## 6. Analytical per100 conversion need not wait for NI-01

The proposed bond normalization is a different, binary-input operation.
For finite nonzero X and strictly positive finite face, write finite binary64
values as `M*2^e`, with at most 53 significand bits. Then

```text
100*X/face:
    numerator = 25*M_X
    denominator = M_face
    exponent = e_X - e_face + 2

face*price/100:
    numerator = M_face*M_price
    denominator = 25
    exponent = e_face + e_price - 2
```

The largest numerator needs at most 106 bits. Keep the binary exponent as a
separate checked small integer; do not materialize an enormous power of two
before deciding the result's exponent/range. Derive the final target binary64
significand and exact quotient/remainder tie from a separately proved scaled
rational rounder. Near zero/overflow, classify with exact exponent/comparison
logic; do not first round a quotient or product that a later factor rescues.

This supplies a feasible bounded-integer route for correctly rounded binary
price normalization without requiring the 19-limb decimal quantizer, currencies
or settlement quantum. It remains a design outline: the final helper needs a
complete width/range/rounding proof and a Fraction fixture across finite
exponent/significand/tie classes. It is not claimed implemented or tested here.
Reusing an exact amount kernel later is possible only when it preserves this
specific input and rounding contract.

## 7. Concrete local adoption path

The following stages preserve the full objective. They are proposed engineering
choices; they neither redefine M8 around an easier subset nor demand repeated
user permission for work already locally authorized.

1. **Phase-A exact supplied decimal foundation.** Record a narrow local
   ADR-0006 reconsideration: keep binary64 analytics unchanged and add the
   canonical finite decimal/quantum surface, seven explicit modes, exact scalar
   operations/reduction, bounded rational operations and fused two-decimal
   multiply/divide. Freeze parser/input limits, source errors, validation order,
   eight-limb invariants and independent test obligations before implementation.
   This is a useful NI-01 dependency, not cash settlement or M8 completion.
2. **Phase-B explicit binary/decimal boundary.** Adopt all four distinctly
   named exact/rounded conversions together; verify the 19-limb whole-finite
   quantizer and outgoing correct rounding. Exact rounding of observed bits
   does not certify an ideal financial value.
3. **Bond instrument and analytical date contract.** Freeze zero/fixed terms,
   raw/adjusted provenance, supplied settlement, payment inclusion, both ex-date
   inequalities, generic earned-minus-detached accrued adjustment, direct
   prepared price and robust binary per100 conversion, with explicit work caps.
   Preserve the market-specific gaps rather than silently copy reference defaults.
4. **Full analytics and selected sourced models.** Add bounded typed-failure
   yield inversion, degeneracy/uniqueness qualification, analytic risk, and each
   source-complete named market factor/date capability. Close literature and
   behavioral reference obligations with NEW attested fixtures and classified
   divergences. Continue beyond a generic kernel until the actual selected M8
   requirements and remaining gates are addressed.
5. **Financial settlement adapter and certification.** Record units, nominal,
   quantum/mode and per-payment/net rounding stage under a complete financial
   contract. Distinguish supplied decimal input, observed binary input and
   certified ideal-value paths. Return an explicit indeterminate outcome when
   a claimed ideal-value error interval crosses a rounding decision boundary.
   This is phase C, with its own evidence; neither A nor B closes it.

Every stage keeps local signed-off commits, independently generated oracles,
strict debug/release/lint/docs/resource/error/package checks and accurate
readiness updates. Existing frozen fixtures/generators remain unchanged.
No push, draft PR or remote write is part of this path. The parent remains
responsible for recording current authoritative progress in the local handoff
before stopping, including this review and any adopted follow-on contract.
