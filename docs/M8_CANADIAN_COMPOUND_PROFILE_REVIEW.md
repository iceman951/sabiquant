# Canadian compound primitive profile: independent closure

Repository pre-contract review, recorded 2026-10-23 Bangkok. Root promotes the immutable
private review with editorial formatting and this provenance preface; no contract, API,
requirement, oracle or Canadian production is adopted. Read the [complete local
proposal](M8_CANADIAN_LOCAL_DESIGN.md), [numeric/test
design](M8_CANADIAN_NUMERIC_AND_TEST_DESIGN.md), [source
review](M8_CANADIAN_SOURCE_REVIEW.md) and [research
validation](M8_CANADIAN_RESEARCH_VALIDATION.md) with their separate scopes. The original
producer's no-repository-write statements below describe its private work; root owns
this new repository record.

Original review SHA256
`bbc3499402140792835ea69f97e1bc63695e63e19323585a9075dc91c96a658d`; private evidence
`/private/tmp/sabiquant-canadian-compound-profile-review-v1/evidence.json` SHA256
`699c9e4c7ef6cc79501274dc99be4e2361c54ad7adad597679ea957283e7d12a`. The completed native
run's evidence SHA256 is
`1697b753a6ca6fcf0916226d048c91cf26eb3ec2a77fd0ae1f270f65cbe58dbc`. Root also counted20
exact-zero discount identities among7,374 discount rows: there are7,354 nonzero-argument
exp invocations per native mode. The6,309 positive finite discount-result budget checks
include those20 identities; they are not 6,309 distinct nonzero exp calls. The exact
per-row observations are unchanged. The [MM range
review](M8_CANADIAN_MM_RANGE_REVIEW.md) proves a different rational model.

Private pre-contract review, 2026-10-23 Bangkok. No Canadian contract, API, production
implementation, financial acceptance, release or foreign runtime is adopted here. This
reviewer read the complete 250-line completed runner, its 245-line failed predecessor,
generated Rust source, native command evidence, environment and raw files. No QuantLib
implementation was read. No repository, Git, AGENTS or HANDOFF file was changed by this
reviewer.

The existing run is **actually complete**, not a presumed-live process. Its
`run-v2/evidence.json` records start 2026-10-22T18:17:12.811777Z, finish
18:18:53.285651Z, final PASS and six status-0 native subprocesses. This review did not
restart it. It made a separate bounded reference/audit pass in `audit.py`; the actual
audit exited 0. `evidence.json` pins this report, audit output, runner, raw files,
environment and preserved initial failure by SHA-256.

## What actually passed

The measured environment was rustc1.99.0 b940084d7, LLVM23.1.1, aarch64-apple-darwin on
macOS26.6.2 build25G83. It built the same 980-byte Rust primitive program with
opt-level0 and opt-level3, then executed each binary with one log batch and one exp
batch. All six commands returned0. Both raw debug and release output pairs are byte
identical. These are finite native measurements, not Linux/Windows runtime results or a
universal elementary-function theorem.

| Observed branch | f2 | f4 | f12 | Total |
|---|---:|---:|---:|---:|
| ln |53|56|59|168|
| ln1p |261|259|260|780|
| tiny identity |88|84|81|253|
| signed-zero inputs |2|2|2|6|
| Total log rows |404|401|402|1207|

There are 7,374 exact-lambda exponent rows: 2,488 f2, 2,442 f4 and2,444 f12. The
selected exact coordinate candidates include 1/184,1/31,1/2,1,185/184,
11/10,36/31,17+1/181+1/184,19999+183/184 and20000. These are primitive-coordinate cases;
neither every coordinate/frequency combination nor all targeted midpoint cases are
asserted to have a valid dated Canadian instrument.

The input constructor includes represented-y floor neighbors, branch/tiny threshold
neighbors, signed MIN_SUB and normal/subnormal boundaries, MAX, binary-exponent grids,
familiar decimal inputs, deterministic random inputs and represented-y neighbors
obtained from exp-range target construction. The random seeds and all resulting input
bits are retained. The decimal target constructor only selects inputs; the subsequent
interval comparisons validate those actual represented inputs. It is not a certificate
that every constructed target lands on its intended boundary.

The runner found47 exact-base-versus-premature-base-rounding discriminators and 1,730
exact-lambda-versus-rounded-lambda exponent discriminators. Nine rational nearest/even
boundary/tie controls are explicit in the runner. The new audit independently checked
all948 non-tiny arguments and all7,374 exponent nearest/even conversions against
CPython's separate Fraction-to-float conversion, all1,207 branch/result equations,
all7,374 exact radii, and all raw TSV IDs and rows. It did not execute a native binary
again.

All158 before/after input guards are still equal to their current bytes. They include
source/test Rust files, Cargo metadata, the retained private v2 numeric proposal,
first-pass review/derivation/arithmetic inputs, all four primary PDFs and the measured
rustc executable. The later public proposal is not falsely substituted for the actually
guarded private v2 file.

## Conditional primitive hypotheses

Write u=2^-53, H=2^-1075, G=f*ln(1+y/f), and assume finite represented y>-f. The
non-tiny route uses the proposed branch exactly:

* if y<=-f/2, b_hat=RN((f+y)/f), l=ln(b_hat);
* otherwise x_hat=RN(y/f), l=ln1p(x_hat);
* g=RN(f*l).

The conditional primitive assumptions are

`|l-ln(b_hat)| <= 2u*|ln(b_hat)|`, or respectively `|l-ln(1+x_hat)| <=
2u*|ln(1+x_hat)|`.

For an **observed positive finite** exp result e at represented z, assume

`|e-exp(z)| <= 2u*exp(z)+H`.

This last statement alone says nothing about an observed zero, infinity or NaN; those
outputs require their explicit runtime validation/range rules. There is no claim that
Rust documents these hypotheses universally, that finite tests prove them for every
input, or that another target shares them. The ordinary floating RN assumptions include
no flush-to-zero and no reassociated operations.

The maximum recorded primitive relative-error enclosure estimates, in unitsu, are
approximately0.932900121396328116934 for ln and 1.000000000000000037007434154 for ln1p.
The scaled-log maximum is approximately 1.754159644577822302487. All exact interval
assertions are below their applicable 2u/6u budgets. The JSON `max_upper` fields are
binary64 renderings of exact Fraction computations; their names do not make that
rendering directed upward. In particular `ln1p max_upper:1.0` must not be reported as a
proof of <=1u. The ln1p witness L00689 exceeds1u slightly and remains safely below2u.

All6,309 positive finite exp outputs have actual true budget flags: 5,897 use the normal
relative-error metric and412 use the subnormal absolute metric. The normal maximum
witness E004228 is approximately 0.999999999999999944488848769u. Zero and infinity are
separate outcomes and are not counted as6,309 positive finite comparisons. The runner
records4,104 cases where an independent narrow ideal-factor interval certifies the same
RNE bits as the observation and2,205 where that conjunction is false. The latter flag
can mean a different observation or an interval that does not certify common bits; it is
not an automatic profile violation or proof of incorrect rounding.

## New scaled-log derivation

For f2/4, non-tiny y/f and the final multiplication byf are exact binary scalings. In
the near-floor branch f+y is exact by Sterbenz and division byf is exact. All non-tiny
relevant results are normal and finite. Thus

`|g-G| <= 2u*|G|`.

For f12, f+y is still exact in the near-floor branch. Its division by12 has relative
error <=u. The exact base b lies in(0,1/2], so |ln(b)|>=ln2>1/2. The resulting logarithm
perturbation is at most

`|ln(b_hat)-ln(b)| <= u/(1-u) <= [2u/(1-u)]*|ln(b)|`.

In the ln1p branch, x=y/12>-1/2 and |x_hat-x|<=u|x|. For x>=0 use ln(1+x)>=x/(1+x) and
the mean-value denominator 1+x-u|x|, obtaining relative perturbation <=u/(1-u). For
-1/2<x<0 use
|ln(1+x)|>=|x| and 1+x-u|x|>(1-u)/2, obtaining <=2u/(1-u).
Consequently both f12 branches admit alpha=2u/(1-u). Apply the conditional primitive2u
and one final multiplication roundingu:

```text
|g-G|/|G| <= (1+alpha)*(1+2u)*(1+u)-1
               = (5u+5u^2+2u^3)/(1-u) < 6u.
```

The inequality is exact for u=2^-53, not fitted to the observed1.75416u maximum. The
maximum valid ln magnitude is small enough that g stays finite under these hypotheses.
This derivation neither imports gilt's old profile nor silently treats division by12 as
exact.

## Exponent radius, including tiny and zero cases

Let exact lambda>0, E=-lambda*ln(1+y/f), q=-g*lambda/f, z=RN(q). One rational-to-IEEE
rounding gives |z-q|<=u|q|+H. If kappa=2u for f2/4 or 6u for f12, then

`|z-E| <= [(1+u)*(1+kappa)-1]*|E|+H`.

The bracket equals3u+2u^2<4u or7u+6u^2<8u. With beta=4u or8u respectively, solve the
inequality using |E|<=|z|+|z-E|:

`R=(beta*|z|+H)/(1-beta)` and `E in [z-R,z+R]`.

This is the exact formula used in all non-tiny stored rows. It is conditional on the
newly stated primitive hypotheses and exact rational exponent policy; it is not an
unconditional bound for a floating `lambda`, chained floating products, arbitrary
platform libm or an unspecified compiler transformation.

For 0<|y|<2^-50 the selected identity is g=y. The convergent log series gives

`|G-y| <= y^2/[2*f*(1-|y|/f)]`.

Set q=-y*lambda/f and use its actual exact RN distance, not a zero-relative-error
shortcut:

`R=|z-q|+lambda*y^2/[2*f^2*(1-|y|/f)]`.

This handles MIN_SUB/f and underflowed exponents without first losing the represented y.
For either signed-zero input, G=E=z=0 exactly and R=0. The defined z==0 discount
identity1 does not require an exp call.

An optional accepted-positive-factor error consequence is

`|e-exp(E)| <= [(1+2u)*exp(R)-1]*exp(E)+H`.

It follows from the conditional exp budget and |z-E|<=R. Its subnormal term must remain
additive; a uniform relative-only bound is invalid. This is a candidate enclosure
relationship, not a frozen Canadian oracle schema.

## Exact midpoint classification and implementation limits

The exact RNE factor zero and overflow exponent thresholds are

```text
Z0   = ln(H) = -1075*ln(2)
Zinf = ln(2^1024-2^970)
     = 1024*ln(2)+ln(1-2^-54).
```

RNE at Z0 rounds the exact factor to zero; at Zinf it rounds to infinity. With
independently certified bounds Z0lo<=Z0<=Z0hi and Zinflo<=Zinf<=Zinfhi, the completed
runner's conservative classification is reproduced by

```text
safe-interior:          z-R > Z0hi and z+R < Zinflo
certain-ideal-zero:    z+R <= Z0lo
certain-ideal-overflow:z-R >= Zinfhi
boundary-ambiguous:    everything else.
```

The review independently derives the threshold enclosures from exact rational series:
ln2=2*sum_{k>=0}(1/3)^(2k+1)/(2k+1), using300 terms and its positive geometric tail;
ln(1-2^-54) uses8 negative-series terms and a geometric tail. `thresholds.json`
publishes outward256-bit dyadic endpoint pairs, each one unit wide. Those fixed
constants and the exact rational threshold bounds separately reproduce all7,374 stored
classifications. Thus this closure of the threshold classification does not rely on the
runner's Decimal ln reference.

| Classification | Observed positive finite | Observed zero | Observed infinity |
|---|---:|---:|---:|
| safe-interior |6284|0|0|
| certain-ideal-zero |0|684|0|
| certain-ideal-overflow |0|0|343|
| boundary-ambiguous |25|30|8|

There are no NaNs. All63 ambiguous rows remain explicitly ambiguous; positive finite
observations among them do not justify accepting their ideal range. No universal
primitive no-range-failure theorem is inferred from this table. The particular R
supplies a sizeable margin near |z|~=700, but a positive-finite-only exp hypothesis does
not itself specify all other output classes. Future production must preserve its
explicit observed-result check even after adopting an ideal midpoint guard.

The runner only classifies and asserts outcomes; it is **not** a production guard
implementation or resource proof. Future Rust must compute an outward R with reviewed
bounded integer/rational storage and compare it to certified threshold constants.
Rounding R inward, testing only z, or substituting a gilt8u band in f2/4 while
overlooking f12 would require new proof and controls. Typed boundary behavior, original
coupon ordinals, shared-log failures and actual allocation/lifetime limits remain part
of complete contract review.

## Preserved initial failure and remaining adoption gates

The initial process failed at L00000, lambda1/184, with `radius miss` before any exp
batch. It computed a symmetric artificial Decimal enclosure around ln(1)=0 while its
actual analytic zero radius was0. That is a reference-harness failure, not a
Canadian/Rust profile counterexample. The only substantive v2 changes are the exact-zero
interval and intersection of tiny Decimal intervals with the proved analytic remainder,
plus an explanatory metadata label. This does not inflate Decimal precision to conceal a
discrepancy; both old and new runs use400/600 digits. The initial runner/log/raw data
are retained and pinned.

No blocking mathematical error was found in the conditional2u/6u log profile, 4u/8u
exponent radii, tiny remainder or exact ideal midpoint classification. The concrete
reporting correction is to treat stored binary64 metric displays as approximations and
retain the greater-than1u ln1p witness. The conditional profile is ready to integrate
into a **proposal**, with its assumptions and limits preserved. It does not close all
Canadian preproduction or authorize production on its own.

Next fixes for root: replace the proposal's unproved10u/16u target with this derived,
explicitly conditional per-frequency profile; specify the outward bounded guard and
exact constants; keep zero/inf/NaN runtime validation; freeze the complete
source/model/cash/API/resource contract and independent full preproduction corpora
before Canadian Rust. Keep primitive, dated-instrument, native/foreign and
empirical/universal evidence separate. The full Canadian
frequencies/shapes/price/yield/all-six-risk/all-four-cash scope remains open.
