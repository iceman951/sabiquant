# Canadian compound bounded-guard design review

Unadopted review promoted2026-10-23 Bangkok from the immutable private proposal
SHA256 `fa33329c2a81e4d8a9dc1feecb8e863ea4fb147fdced1838a972ea8178d8cc31`;
evidence SHA256 `f5a98bdd7df6ed5a4c65ee635fe815923117e48a4724dc6be27b1954347e7e1b`.
Private artifacts remain under
`/private/tmp/sabiquant-canadian-compound-guard-design-v1/`. The text below is
that review, with one integration clarification: its threshold limbs are
little-endian, so the stated Horner reconstruction traverses **reversed limb
order** (most significant first), exactly five steps. Forward traversal would
construct a different integer. This clarification is part of the proposed
algorithm and must be covered by later primitive controls.

Read with the [conditional native profile](M8_CANADIAN_COMPOUND_PROFILE_REVIEW.md),
[complete local design](M8_CANADIAN_LOCAL_DESIGN.md) and
[numeric/test companion](M8_CANADIAN_NUMERIC_AND_TEST_DESIGN.md).
This records a bounded mechanism; no Canadian contract or production is adopted.

## Original independent proposal

Private complete-contract integration proposal, 2026-10-23 Bangkok. This is
neither adoption nor a Canadian Rust implementation. No repository, Git,
AGENTS or HANDOFF mutation was made. Local own-source definitions
`src/rates.rs`, `src/error.rs`, `src/numeric.rs` and `src/amounts/uint.rs` were
read; no QuantLib implementation was read. The prior independent profile review
and its exact threshold certificates remain separate immutable inputs.

## Selected mechanism and meaning

Use the proposed scaled-log policy, exact rational exponent observation and
conditional primitive profile unchanged. For non-tiny quotes write

```text
u = 2^-53; H = 2^-1075
k = 4 for f2/4, or 8 for f12
beta = k*u
R = (beta*abs(z)+H)/(1-beta)
```

Here z is the actually admitted binary64 logarithmic exponent, not the shared
scaled logarithm g or a rounded horizon. Under the newly reviewed primitive
hypotheses the ideal exponent lies in the CLOSED interval J=[z-R,z+R]. The
guard computes its endpoint comparisons exactly. It never rounds R inward,
never adopts the old gilt band, and never retries at a larger precision.

The exact thresholds are Z0=-1075ln2 and Zinf=ln(2^1024-2^970). Use all four
signed256-bit dyadic endpoints in the prior `thresholds.json`, whose common
denominator is2^256. Each certified box is one2^-256 unit wide. Retain both
endpoints; do not replace a box by its midpoint or a nearest floating log.

**Chosen guard action:** if J has a closed intersection with either certified
threshold box, return indexed `RateError::NumericalBoundary` before exp. Touching
either endpoint counts as intersection. If J is disjoint from both boxes,
proceed to exactly one exp observation (except the defined zero-exponent
identity), then validate the actual result as positive and finite.

This intentionally permits the call when J lies entirely below the zero
threshold or above the overflow threshold. That supplies an actual zero or
infinite observation for the ordinary `RateError::Value` path. Do not fabricate
an observed zero/infinity, or mislabel a certainly out-of-range interval as
uncertain, merely to avoid that single bounded elementary call. If the actual
elementary result behaves unexpectedly, the same actual-result check applies;
the positive-finite-only exp profile does not become a universal range theorem.

## Exact bounded integer algorithm

All `Wide` words here are64-bit limbs, matching the actual local helper. Choose
`GuardWide=Wide<20>`:1,280 bits/160 bytes. A sign is a separate Boolean and
zero magnitude always has the nonnegative sign. Signed comparisons reverse
magnitude order for two negative operands. No signed overflow arithmetic is
needed.

For finite nonzero |z|<2^24 use

```text
E = 2^1075
A = exact(abs(z))*E                  # nonnegative INTEGER
B = 2^53-k                         # positive u64
C = A*B
N = k*A+2^53                       # radius numerator
denominator = B*E                  # implicit; never materialized or divided

if z>0: L=C-N; U=C+N
if z<0: L=-(C+N); U=N-C
```

L and U use canonical signed magnitudes when a subtraction crosses zero. They
are the exact numerators of z-R and z+R on common denominator B*E. Decode z
directly into its integer significand m<2^53 and exponent e>=-1074, then
`A=m<<(e+1075)`. The shift is nonnegative; for |z|<2^24 it is at most1046.
For subnormal z the unit exponent is-1074 and the shift is1. All exponent
arithmetic is small checked i32/u32 metadata, independent of allocator size.

For a signed threshold endpoint T/2^256, compare L or U directly with

`K(T)=T*B*2^819`.

This is exact because2^256 divides E. **Cancel that binary denominator first.**
A naive cross-product that shifts L by256 needlessly enlarges the capacity.
The denominator B*E is common and positive; neither division nor an approximate
reciprocal appears in any comparison.

The two closed-intersection predicates are

```text
L <= K(Z0_upper)   && U >= K(Z0_lower)
L <= K(Zinf_upper) && U >= K(Zinf_lower)
```

Evaluate one threshold endpoint at a time and discard its temporary after the
comparison. The four endpoint numerators have at most266 bits and fit five
u64 limbs each. Their literal little-endian limb values are in
`controls.json.threshold_limbs`. Reconstruct a magnitude with exactly five
Horner steps `t=(t<<64)+limb`, including leading zeros if useful. Multiply by
the small B using ordinary bounded multiplication and shift by819. Each step
uses the same fixed20-limb capacity. There is no text parsing, allocation,
variable-precision retry or exponent-sized vector.

Universal capacity bounds on this branch, including deliberately loose carry
headroom, are

| Quantity | Bound |
|---|---|
| A | <2^1099 |
| C=A*B | <2^1152 |
| k*A | <2^1102 |
| N=k*A+2^53 | <2^1103 |
| every signed endpoint magnitude | <2^1153 |
| abs(T) | <2^266 |
| abs(T)*B | <2^319 |
| abs(K(T)) | <2^1138 |

Thus even19u64 words would cover the loose1,153-bit endpoint bound;20 gives
additional64-bit headroom and a simple symmetric helper type. This proof does
not require all mathematical extrema to be jointly dated-reachable. The
observed controls reach1,152-bit endpoint magnitudes and1,138-bit threshold
comparands, but the universal proof is the reason for the selected capacity.

The prepared source coordinate lambda=Nlambda/Dlambda has Nlambda<2^30 and
Dlambda<2^16. It is used only by the preceding exact exponent RN. It does not
enter the guard arithmetic a second time. Under the reviewed primitive
hypotheses the source limit lambda<20000 and ordinary valid finite y imply
|z|<2^24. Do not turn that conditional elementary fact into an unchecked memory
assumption: apply the following fixed defensive path as well.

For any admitted finite |z|>=2^24, bypass the wide construction and proceed to
the ordinary exp/result-validation step. Indeed beta<=2^-50 and

`R/abs(z)=(beta+H/abs(z))/(1-beta)<1/2`.

Therefore every point in J has the sign of z and magnitude greater than2^23,
far beyond both threshold boxes (which are contained in[-1024,1024]). J cannot
intersect either box. This path is exact and constant-sized; it handles a
finite out-of-profile elementary observation without a false internal-capacity
error. A nonfinite z is rejected by its own value validation before this guard.
No new public financial domain restriction is introduced.

## Tiny and zero paths retain their real error

Shared branch selection is from the original quote: 0<|y|<2^-50, not a guess
from g. With lambda<20000<2^15 and f>=2, q=-y*lambda/f satisfies |q|<2^-36.
The exact log-series bound gives

```text
abs(ideal E) <= lambda*abs(y)/(f*(1-abs(y)/f)) <2^-35
abs(z) <= (1+u)*abs(q)+H <2^-35
Rtiny = abs(z-q)+lambda*y^2/[2*f^2*(1-abs(y)/f)]
      < 2^-89+H+2^-88/(1-2^-51) <2^-86.
```

Hence all of [z-Rtiny,z+Rtiny] lies in(-2^-34,2^-34), far inside both certified
range boundaries. The tiny path may bypass the guard **without materializing
the roughly2,200-bit exact tiny-radius denominator**. Its true nonzero analytic
remainder and exact RN distance remain in reference tolerances. Do not set
Rtiny=0, claim g equals the ideal logarithm, or skip those terms in independent
price/risk comparisons.

For either signed-zero input, shared g and ideal E are exactly0. For any
resulting z==0 the prescribed discount identity is1 with no exp call. This
includes tiny exponents that rounded to zero. Such a tiny identity is still an
approximation to the ideal nonzero exponent, with the same analytic error
retained. For tiny z!=0 call exp once and validate the actual result. There is
no threshold retry and no materialized tiny-radius workspace.

## Liveness and operation order

The guard has no heap allocation. Its staged implementation may budget at most
12 simultaneously live logical GuardWide payload slots (1,920 bytes):

* five persistent magnitudes A,C,N,L,U;
* one current threshold magnitude/product K;
* six conservative slots for helper receiver/argument copies, result array,
  return payload and consumed-result temporaries.

Use explicit scopes between threshold endpoints and between construction,
comparison and the subsequent PV/risk work. Do not keep four K values alive or
build tuple/vector collections of large intermediates. The input z, f, k, B,
signs, shifts, ordinals and comparison flags are scalar metadata. The four
threshold literals occupy160 bytes total as constant source data. The helper
functions are bounded by20 limbs and their fixed nested multiplication loops;
there is no division inside the guard.

This is a logical payload/lifetime allowance, **not** a compiler-stack, ABI,
spill or allocator measurement. Root must merge it into the complete numerical
resource census. Even pessimistically adding all1,920 guard bytes to the
currently proposed40,336-byte MM/risk payload allowance gives42,256 bytes,
before separately accounting metadata/constants. That comparison is an
integration headroom check, not a new whole-program stack claim. A future Rust
review must confirm that actual helper expressions obey this lifetime plan.

Required evaluation sequencing remains:

1. Validate quote finiteness, then its exact compound domain y>-f.
2. Reserve the required PV workspace fallibly before any shared elementary
   observation, including zero/tiny quotes. The guard creates no second reserve.
3. Observe one shared g per evaluate/risk invocation or actually invoked solver
   callback; shared-log errors have index=None.
4. Traverse future bundles in original actual-coupon order. Derive and validate
   z once, take its zero/tiny/ordinary guard path, then observe at most one exp.
5. Validate the factor before the source PV even when the source coupon is0.
   Observe source-times-factor once, then the selected risk terms if requested.

Yield inversion first resolves its typed target and positive model-dirty gate,
then residual scale and one reusable PV reservation before lower-first Brent
callbacks. No per-trial guard/PV allocation or hidden final callback is added.
A failed callback retains its actual stage, point and evaluation count. The
guard does not move an invoked callback failure into preparation, or compress
the original coupon ordinal into future-bundle position.

## Exact typed mapping to actual local error definitions

Actual `RateError` has `Value(ValueError)` and
`NumericalBoundary{quantity:&'static str,logarithm:f64}`. Actual `ValueError`
distinguishes nonfinite inputs, finite out-of-domain inputs, nonfinite computed
results and finite computed results outside their domain. Its numeric helper
checks finiteness before domain. Preserve that distinction through the
prospective `CanadianError` wrapper.

| Event | Proposed outer context | Immediate existing cause |
|---|---|---|
| Nonfinite quote | Value, None, `yield quotation` | NotFinite(parameter `rate`, actual NonFinite class) |
| Finite y<=-f | Value, None, `yield quotation` | OutOfDomain(parameter `rate`, y, Interval greater_than(-f)) |
| Nonfinite shared g | Rate, None, `discount factor` | RateError::Value(ResultNotFinite quantity `canadian scaled logarithm`, actual class) |
| Nonfinite rationally observed z | Rate, Some(original actual ordinal), `discount factor` | RateError::Value(ResultNotFinite quantity `canadian logarithmic exponent`, actual class) |
| Closed threshold-box intersection | Rate, Some(original actual ordinal), `discount factor` | RateError::NumericalBoundary{quantity:`discount factor`,logarithm:z} |
| Observed exp0, negative zero or finite negative result | Rate, Some(original actual ordinal), `discount factor` | RateError::Value(ResultOutOfDomain quantity `discount factor`, actual value, Interval greater_than(0)) |
| Observed expNaN or signed infinity | Rate, Some(original actual ordinal), `discount factor` | RateError::Value(ResultNotFinite quantity `discount factor`, actual class) |
| Guard checked integer operation unexpectedly exceeds its bound | InternalCapacity, Some(original actual ordinal), `discount factor` | operation `guard discount range` |

`guard discount range` is an explicit proposed addition to the capacity
operation vocabulary; the old list has no truthful guard-operation label.
These capacity failures are excluded by the integer proof on the selected
branch, not invented financial edge cases. Do not relabel them as RateError or
silently resize/retry. No new RateError variant is needed. The existing root
Error wrappers and callback wrappers must preserve their immediate causes.

Source/PV/risk failures retain their separate quantities/phases after this
factor stage. For example a bundle at compressed position0 may belong to
original actual ordinal17; its guard/exp error index is17. Shared g still uses
None. The full API contract, rather than this primitive test harness, must
bind that dated discriminator to actual public operations.

## Actual bounded controls and limits

`controls.py` was executed once with actual exit0. It reuses the already
completed7,374 exponent records without any native rerun, and compares the
proposed integer endpoints against exact Fraction z±R. All exact closed-box
decisions match the reference. Its simulated future control flow is

| Proposed path | Existing records |
|---|---:|
| signed-zero input identity |12|
| tiny exponent rounded-zero identity |8|
| tiny nonzero exp path |882|
| ordinary non-tiny exp path |6409|
| NumericalBoundary before exp |63|

Thus an eventual guard applying these rules to these primitive inputs would
have20 zero identities and7,291 exp calls. The earlier actual measurement had
20 identities and7,354 exp calls because it **classified** ambiguous rows; it
did not implement the new reject-before-exp guard. These counts are not
presented as actual future Canadian runtime results.

Additional finite controls are36 represented-z cases spanning signed zero,
MIN_SUB, normal minimum, signs, ordinary values, both2^24 branch neighbors,
and signed MAX;10 closed-touch/strict-disjoint predicate cases; and72 tiny
quote/frequency/lambda cases including MIN_SUB and threshold neighbors. Touch
predicate cases are abstract intervals, not claims that an irrational exact
financial boundary is attained by a dated quote. All158 original input guards
and the explicitly read local source files remain unchanged. No Rust test,
coverage or package count is increased by these Python proposal controls.

The mechanism is ready to integrate into the complete proposed Canadian
contract, with the new capacity operation and closed-touch behavior explicit.
Remaining gates are adoption of the entire source/model/API/cash/resource
contract, independent preproduction generation/freeze and later production
implementation/consumption/runtime/mutation verification. This bounded guard
does not narrow frequencies, shapes, price bases, risk getters or cash paths.

## Root replay and limb-order discriminator

Root re-executed an unchanged copy of `controls.py` in
`/private/tmp/sabiquant-canadian-root-guard-replay-v1/`. All7,374 rows and the
entire control payload match the independent output after excluding only its
actual UTC timestamp. The replay also reconstructs all four signed threshold
constants using five reverse-order Horner steps, compares both frequency
bands (eight exact comparands), and confirms forward little-endian traversal
produces a different integer in all four cases. These are integer construction
controls, not dated boundary reachability or Rust semantic mutants.
Evidence SHA256 `33399e79a6eec23c372515afdc80ad91a0867f5a3c509a18191493e5c0d378ce`.
No native run was repeated.
