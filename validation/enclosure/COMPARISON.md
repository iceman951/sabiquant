# E2 financial consumer comparison rules

Frozen before the first financial producer implementation. This is B4 for the
existing four-model scope, not a new oracle or reference family. The FS1 price
and source fixtures project the unchanged reference inputs under `references/`;
the single B2 boundary fixture and existing H/HC/HYBRID controls remain separate
evidence layers. `comparison.json` records the machine-readable policy.

## Identity and complete consumption

Every price row retains its original id, source id, model, basis, exact trial
yield bits and certificate id. The consumer must construct that exact public
source input, call the applicable model's producer and inspect its actual origin,
basis, per100 meaning, canonical successful quote, requested precision/budget and
progress. No source/query row is silently dropped, duplicated or substituted.
Each model implementation consumes its complete model partition; all four
partitions together consume all 173,448 price rows. Deferred models are reported
explicitly until their own E3 commit. Source/public-constructor checks do not
establish private cached-flow identity by themselves.

Standard FS1 parsing uses the unchanged shared reader. Require exact columns,
unique complete row ids, declared row/model/reference counts and pinned input,
generator and certificate identities. Binary words are sixteen lowercase hex
digits. The `reference_*_k` fields retain signed **hexadecimal** exponents; they
must never be interpreted as decimal. Missing endpoints are explicit sentinels.
Public tests must reject any row they cannot interpret rather than skip it.

## Exact containment

Containment is TOL-E0: no epsilon, overlap or agreement with an old observed price.
For an exact rational target, the converted reference stores the tight finite
binary64 floor and ceiling. Any finite binary64 producer interval contains the
target iff its lower is no greater than that floor and its upper is no smaller
than that ceiling. The numerator/denominator and preferred fitting point are
retained for exact-point comparisons and provenance.

For an independently certified reference interval, require the producer interval
to contain the complete certified interval, not merely intersect it. Since
producer endpoints are binary64, comparison against the outward binary64 image
of the certified reference is equivalent to the endpoint inequalities. Original
normalized sign/m/K endpoints and precision remain in the fixture. A point
inside a nonpoint reference interval is not an independent proof of equality.
If nesting cannot be established, the assertion is unproven/failing; do not
change frozen bounds or accept by overlap. An independently justified direct
endpoint proof may resolve an actual failure, with its diagnosis recorded.

ExactDecimalPoint must equal the independently fitting canonical coefficient and
scale; ExactBinary64Point must equal the proved exact bits. Prefer Decimal when
both fit. A general directed calculation is not required to discover every
rational identity. Once the producer claims ExactRationalIdentity or a complete
final IntervalCollapse, its budgeted fitting preference and NoFittingPoint result
must obey the contract. All evidence is DECLARED; a point never upgrades receipt
guarantees. Strict outward bounds and the permitted proof facts follow B3.

## Ordinary usefulness and explicit test allowances

The old ordinary price corpus uses its existing modest trial quotes. At p64 and
max_work=1,000,000,000,000, its complete targets are required to return finite
evidence satisfying exact containment. This allowance is a test input, not an API
default. Other four allowed precisions are crossed with all Generic configurations
and representative named shapes/bases in actual implementation tests; their
results need containment, not bit-identical bounds or a hidden refinement ladder.

An ordinary low-magnitude subset also requires useful width:

`width <= 2^-36 * max(1, abs(reference_outer_lower), abs(reference_outer_upper))`.

The subset is selected from the unchanged source rows: all exact selected amounts
and applicable AI shifts have magnitude at most 1,024; Generic face is between
1 and 1,024 inclusive. The complete original source data determines this
selection, including negative coefficients. This criterion is separate from
containment and deliberately does not demand one specific floating-point pair.
Compute the dyadic width exactly or use a proved conservative upper bound; a
rounded subtraction alone is not a containment premise. No midpoint or cash
recommendation follows. E4 separately compares actual originating evidence with
the existing exact cash recipes, modes and quanta.

## Boundaries and honest unresolved outcomes

The one B2 fixture covers face extrema, exact signed cancellation/range rescue,
native-affine overflow with a valid exact domain, signed tiny results and true
unavailable finite endpoints. Its strict exponential certificates classify range
without evaluating exp(MAX). A completed true out-of-range target cannot produce
finite clipped bounds. For tiny targets the implementation must preserve outward
zero/MIN_SUB direction whenever it returns finite evidence. Resource stops obey
B3; a partial expression never becomes an enclosure. A finite result is always
checked for containment even on a stress case. Unresolved is not counted as a
passing finite-price assertion where the selected test explicitly requires one.

The H43 equations and HC38 conditional cash consequences retain their existing
scope. HYBRID144 retains its exact prepared horizons and branch propositions;
zero-yield price equality alone does not test branch selection. Actual producer
selector tests must exercise the same branch decision used by the new code.
No old native result, private helper count or storage marker substitutes for an
actual new financial producer, cash consumer or E5 resource result.
