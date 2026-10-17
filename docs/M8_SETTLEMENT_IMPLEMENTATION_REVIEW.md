# Local settlement implementation review

| Field | Value |
|---|---|
| Date | 2026-10-17, Asia/Bangkok |
| Authority | [Frozen local contract](M8_SETTLEMENT_CONTRACT.md), FR-SETTLE-01..16 |
| Reviewed production | [settlement.rs](../src/settlement.rs), [arithmetic.rs](../src/settlement/arithmetic.rs), reused [uint.rs](../src/amounts/uint.rs) |
| Result | No remaining reachable arithmetic, image/domain, pointness, failure-order or provenance defect found |
| Changes resulting from review | Two small contract-context/order cleanups, applied by root and inspected again |
| Acceptance | Local engineering review; complete validation and independent financial/platform/owner gates remain separate |

The reviewer first authored and executed the [27 public hand tests](M8_SETTLEMENT_HAND_TESTS.md)
without reading settlement production. Expected results and the complete
432-case small image enumeration were fixed before the first production read.
After focused debug/release and strict Clippy passed, root explicitly requested
this separate source review. No hand expectation was adapted after source
inspection. The reviewer also authored module/root-error/public-trait integration;
that integration assessment below is self-review, whereas the settlement
arithmetic was independently authored by root. The reviewer did not change
settlement production during this review.

## Findings resolved before the final source snapshot

1. **Defensive stage context.** The clean RoundTotal route initially decoded
   secondary evidence using `cash accrued`, although its required rounding
   stage is `cash total`. Such decoding cannot exhaust forty limbs on valid
   inputs, so no reachable public failure was observed. Root now selects the
   secondary decode context from aggregation: `cash total` for RoundTotal and
   `cash accrued` for component rounding. The amended branch was inspected.
2. **Literal constructor sequencing.** Interval construction initially used a
   helper that canonicalized lower zero immediately after its finite check,
   before validating upper. Upper was never skipped and observable error
   precedence was correct, but contract section3 requires both finite checks
   before canonicalization. Root split `check_finite` from `canonical_bits`:
   interval construction now checks lower, checks upper, canonicalizes both,
   then compares finite values. The single supplied-value constructor retains
   its combined helper. The amended constructor/helpers were inspected.

The earlier strict Clippy layout finding is recorded in the hand report.
The inline receipt retains a complete Copy instruction without allocation;
its local documented `large_enum_variant` expectation is consistent with the
adopted contract. No boxing or reduced provenance was introduced.

## Complete contract assessment

| Contract area | Source assessment | Evidence/limits |
|---|---|---|
| Exact positive par, explicit unit/sign recipes | Every factory checks positive nominal. Dirty/clean use per100, coupon per1000, staged uses 10+1 weights over1000. Scaling never applies an unsigned party sign after rounding. | Hand sign/unit/nominal and stage discriminators passed. Named-market validity remains open. |
| Four immutable evidence kinds | Private Decimal/Binary variants retain bounds and declared flag. Binary fields are finite canonical bits. Derived Eq/Hash retain kind and provenance; no NaN is stored. | Constructor order inspected after cleanup; hand zero/subnormal/NaN/order controls passed before cleanup. |
| Eight public types and exact registries | Exactly the adopted eight values/errors; no public Recipe, Default, arithmetic operators, inferred conversion, contextual parser or Ord is added. Registries have exact names/order/no aliases. | Seven value Copy/Eq/Hash and error traits compile;97 unique public value assertions. Full declaration census belongs to final validation. |
| Four complete recipes and inspection | Private Recipe has only complete factory-created shapes. Original evidence, nominal, aggregation and all source/cash policies remain inspectable. Fixed source quanta are1e-6 and1e-5; coupon has no source rounding. | All inspector and direct/staged/aggregation hand controls passed. |
| Ordered wide execution | Staged price completes before accrued decode/round. Components round capital before accrued; only their exact net is admitted. Total recipes combine before one cash round. No source/component DecimalAmount admission occurs. | Source/cash range rescue and earlier Exact failures were checked blind. |
| Point/continuous/discrete Exact domains | Decode marks unequal raw endpoints continuous. Source rounding resets to consecutive source-count steps. Nonpoint continuous Exact is unresolved; discrete Exact checks lower base and both retained steps. Point nonmultiples return typed Inexact. | Exact extrema-with-bad-interior, integral-step, collapsed-point and independent-variable hand controls passed. |
| Numeric input pointness | Quantizers retain `image.is_point()` before output collapse. Source collapse creates a later point. Cash-net pointness compares exact combined rounded endpoints. | Point/lower/upper and cash-net range contexts match the frozen order. |
| Complete interval consequence | Positive weights and monotone signed quantizers preserve attainable extrema. Independent source steps remain separate. Intermediate differences are not returned early. Final equal amounts determine; different representable extrema are ambiguous. | Complete432 small-image checks passed. Precision holes are documented; no interior representability promise is made. |
| Final signed normalization | Wide magnitude loses decimal trailing zeros before range checks. Zero is canonical. Negative magnitude2^127 maps to MIN; positive2^127 fails; other magnitudes use checked i128 conversion. No scale can grow past the valid cash-quantum scale. | Both signed limits at all39 scales and asymmetric ties passed. Only Inexact/OutOfRange amount causes are generated. |
| Full receipt/provenance | Receipt stores amount, entire instruction and guarantee. Original declared evidence wins over binary evidence, then decimal. Collapsed declared points retain conditional-enclosure provenance. | Blind equality/hash/receipt/mixed-kind controls passed. No ideal financial-price accuracy is inferred. |
| Errors and root integration | Exactly six domain kinds. Quantizers/admission use operation quantize, except component-net admission sum. Stable stages/endpoints and exact typed sources are retained by root. | All131 catalog samples, transparent sources and failing writers passed in focused unit tests. InternalCapacity is not claimed reachable. |
| Resources/state | Forty stack limbs, bounded count arrays, finite recipe enum, no callback or expression graph. No heap allocation in arithmetic/evidence handling, unsafe, I/O, clock or global/interior state was found. | IdentifierError's existing unknown-text String exception remains explicit; no forced OOM/private-state forgery. |

## Width and elementary operation review

The basis constructed as `10^38 << 1036` is exactly
`D0=2^1074*5^38`. Decimal decoding divides this basis by `10^scale`, which is
exact for every valid scale0..38, then multiplies by the unsigned coefficient
magnitude. This includes i128::MIN without signed negation.

Finite normal binary64 bits decode as a53-bit significand times
`5^38 * 2^(field-1)`; subnormals use their fraction times `5^38`. These are
exact D0 coordinates, including the normal/subnormal boundary. Maximum
field2046 gives shift2045. The native constant `5_u128.pow(38)` is below2^89
and therefore cannot overflow u128.

Independent Python integer checks, rather than floating logarithms, verified:

```text
D0 = 2^1074 * 5^38 = 10^38 * 2^1036       bit_length 1163
Fmax = (2^53-1) * 2^971
C = Fmax * D0                            bit_length 2187
11*C                                    bit_length 2190
11*C*(2^127-1)*10^38                     bit_length 2443
D0*10^38*1000*(2^127-1)                  bit_length 1426
```

Fmax is itself an integer and an exact multiple of both fixed source quanta.
Monotonic source rounding therefore cannot raise any valid finite source
above Fmax in magnitude. The checks also confirmed all conservative frozen
inequalities: D0<2^1163, source coordinate<2^2188, weighted coordinate<2^2192,
unreduced cash numerator<2^2446, cash denominator<2^1427, source division
numerator<2^2314 and denominator<2^1290. These independently check the
width calculations; they are not an implementation test of all integers.

For the source/cash ratio, the code forms only the bounded numerator factor
and denominator, then divides exact unsigned magnitudes. Its quantum-count
product remains below the conservative2^2447 bound. Components are added in
their shared cash-quantum coefficient scale, rather than reconstructed on D0.
An independent conservative bound is
`(Fmax*MAX + MAX)*10^38 < 2^1280` for either rounded component's coefficient;
their exact sum is below2^1281. Thus the component-addition carry also fits
comfortably within forty limbs. This bound ignores the at-least100 divisor
and is deliberately loose.

The reused kernel's numeric algorithms are unchanged; the tranche widens only
crate-private visibility and documents the forty-limb instantiation. Source
inspection confirms:

- addition retains final carry; subtraction retains borrow;
- row multiplication rejects all lost high products/carries, with each
  native row temporary bounded by u128::MAX;
- left shifts reject every lost nonzero bit;
- aligned shift/subtract division has at most2560 positions, never doubles
  an unchecked remainder, and retains exact quotient/remainder;
- small division retains remainder; to_u128 rejects nonzero higher limbs.

Rounding compares remainder to denominator-minus-remainder. Parity is the
integer quantum count's parity, and direction uses the original signed value.
Nonzero explicitly rounded zero is valid and canonical. No intermediate
binary64 arithmetic, epsilon, shortest-decimal interpretation, GCD truncation,
range-clipping or unchecked wide carry is used.

## Image and work review details

Each raw unequal enclosure contains continuous real variation. A non-Exact
source round instead creates a full consecutive range of quantum counts,
with one positive step when it remains nonpoint. Nearest/direct-mode tie
ownership does not create a missing intermediate count: each intermediate
cell has a reachable value, and closed original endpoints attain the extreme
counts. Combining two sources retains at most two independent steps. Positive
nominal/scaling and recipe weights cannot eliminate a variable step by an
invented correlation.

For Exact, divisibility of a signed base is equivalent to divisibility of its
unsigned magnitude. Divisibility of that base and each independent positive
one-count increment is necessary and sufficient for every reachable discrete
value. Continuous nonpoint inputs return unresolved before endpoint tests;
point inputs instead report Arithmetic(Inexact) at point. Cash component
quantizers execute separately, so a later sum never rescues an undefined stage.

The implementation repeats the lower-base division when subsequently rounding
the lower endpoint of a discrete Exact image. This is extra bounded work, not
a new numerical or domain decision. The contract's two endpoint pipelines and
at most two variable-step tests describe the semantic work; they should not
be reported as an exact count of low-level division calls. Every loop remains
bounded by fixed40-limb/2560-bit, at-most38-scale and finite recipe/stage limits.
There is no precision retry, enumeration of a large interval or input-sized
temporary allocation.

## Reviewed source snapshot and remaining gates

| Artifact | SHA-256 |
|---|---|
| [Frozen contract](M8_SETTLEMENT_CONTRACT.md) | `47f1b08ff1ca646c502c90b1e7cb380fa4d74a4763b3aa9e32d729adc89faeba` |
| [Public settlement implementation](../src/settlement.rs) | `eaf0306e738d90c44154de46e2fd834050400ea1cb63a17548436a78d7358363` |
| [Settlement arithmetic](../src/settlement/arithmetic.rs) | `eb8d66b195e0d6a142f63c388bd0bcdd7f80b11e23b2390dd037ae87d8a82dd1` |
| [Reused fixed-width kernel](../src/amounts/uint.rs) | `1e3cb59207ea9e13e135d00a192971370279bf43cacc352730be43bfe24c54ec` |
| [Amount module visibility integration](../src/amounts.rs) | `c3031d3fb24ad1ca579c3ba0115d100f7bead99172cddea9879ebb71e0f8a9bf` |

The focused hand report records the actual pre-review execution. The two
review cleanups above were source-inspected afterward; root must run the
complete post-review battery, independent fixture consumption, fresh coverage,
package extraction, platform compilation and old-artifact byte audit on this
amended snapshot. This review claims no such unexecuted results, exhaustive
finite-input enumeration or universal coverage. Proof-excluded capacity paths
remain defensive, and source review is not a machine-checked arithmetic proof.

Full M8 yield inversion/risk, source-complete named price/yield models,
justified financial enclosures and independent owner/platform/release
acceptance remain open. This review did not push, publish, alter frozen
contracts/fixtures, force OOM or forge invalid private state.

## Final independent consumer/documentation follow-up

The reviewer subsequently read the complete final
[settlement consumer](../tests/settlement_oracles.rs), SHA-256
`f1e1b7231476f786c4ea2f1bc3416953927c7f115112d4449517b7e992da2bb9`.
It visits every frozen record, checks all17 columns through active inputs,
exact outputs or required absence, rejects duplicate/empty contextual fields,
and rejects unknown recipes/decisions/errors rather than substituting a default.
Receipt inspection retains source policies and original evidence. Arithmetic
checks compare every AmountError field and exact root/domain/amount source
types; all ten record censuses are recomputed, with contextual censuses drawn
from actual typed outcomes. InternalCapacity remains explicitly unclassified
for oracle inputs rather than silently accepted.

A separate read-only Python audit verified current generator/contract/
requirement SHA pins, whole/body/record SHA and FNV pins, all154,241 sequential
records,154,063 distinct serialized inputs and five structural censuses.
The complete-fixture FNV additionally pins metadata/cross-check/width claims;
those generator-only claims are not presented as recomputed from Rust rows.
This consumer assertion was added while the root's full validation was active,
so its final runtime reconciliation remains the root's responsibility.

The public declaration set exactly matched97 trait/value assertions. A path
audit of15 changed/new Markdown files checked380 relative links and found only
the expected pending `M8_SETTLEMENT_VALIDATION.md`. Current living-doc bond
names matched `Bond::zero_coupon`/`Bond::fixed_rate`; the former invented
ZeroCouponBond/FixedRateBond types were absent. One stale architecture sentence
still described financial adapters/bond models as not adopted; this was sent
to root for correction. No source/test or frozen artifact changed during this
follow-up review.
