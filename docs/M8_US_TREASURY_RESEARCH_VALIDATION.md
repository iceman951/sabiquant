# Treasury pre-contract research validation

Recorded 2026-10-20 on local `feat/m8-us-treasury` from `fc3f629`.
This checkpoint validates source research and challenges a complete proposed
model. It implements no Treasury Rust, adopts no Treasury contract or
requirement, and freezes no general production oracle. No remote writes or
QuantLib implementation reads occurred.

The next design is the complete
[Treasury proposal](M8_US_TREASURY_LOCAL_DESIGN.md): source/date preparation,
price, yield, all six risk views and four exact-money recipes together.
Existing [gilt validation](M8_GILT_VALIDATION.md) remains the authoritative
implemented baseline. Research checks below are not additional Rust test
functions or a new production-coverage percentage.

## Source refresh and exact examples

The [source refresh](M8_US_TREASURY_SOURCE_REFRESH.md) records inspected
versions and pins. Root directly compared the archived XML units and
fixed-principal subsections and visually inspected the material PDF pages.
The 2026-10-15 Appendix B amendment changes TIPS material; I.A/I.D/II are
unchanged. Original dated and original issue dates still require separate
model roles. A second successful network fetch is not claimed: root's API
attempts returned HTTP 406.

The [source checker and report](M8_US_TREASURY_SOURCE_EXAMPLES.md) reproduce:

| Family | Passing comparison |
|---|---|
| Table 1 | 48 Gregorian day-count cells |
| Table 2 | All 640 daily-decimal cells, exact rational rounding and Decimal160/240 |
| I.A/I.D | 19 printed coupon/accrued/scaling anchors |
| II A–G | All seven prices and date/formula inputs |
| Staging sweep | 116,800 rate/half-year/elapsed-day cases, both tie rules |
| Derived component control | 32,940 cases; 8,320 component-versus-total AI differences retained |
| Printed intermediate trace | 79 cells classified; the unexplained G numerator remains explicit |

The full staging sweep contains 584 exact raw five-place ties and 270
raw-versus-table staging changes under each rule (350 under either rule).
These are observable differences, not permission to change source fixtures
or choose an unspecified regulatory tie rule.

Independent review found that the initial Decimal route checked dirty price
but did not corroborate all claimed accrued/clean quantities. The expanded
checker independently decodes accrued components and all three clean
recipes at 160/240 digits under both rules: 28 example/precision/rule
profiles. It checks the exact mixed absolute-relative criterion named in the
report. A separate integer-quotient implementation reproduces all 640 and
116,800 cases, plus explicit cash-flow prices and external Decimal controls.

Trace qualifications are also corrected: 34 exact-agreeing cells have no
chained candidate, 18 exclude a tested chained candidate, 24 fit both, two
fit chained only and one fits neither. There are no ideal-model ties, but
three chained candidates are ties. The old 52-cell combined count cannot
be described as 52 independently excluded chained alternatives.

Root reran the expanded checker successfully. Its result is byte-structurally
identical to the peer's v2 report after removing the actual UTC timestamp.
Historical v1 evidence is preserved. Overwrite and wrong-source refusals
were checked separately from passing source arithmetic.

## Outward powers and audit corrections

The [power research](M8_US_TREASURY_POWER_DESIGN_REVIEW.md) models positive
p=192/256 outward endpoints with extended exponents. Its per-operation exact
inequalities and positive interval induction establish containment for each
executed cell. Its conversion/operation weights count the expanded power
tree and give a linear-in-j width certificate; they do not incorrectly use
only the number of binary-exponentiation instructions.

The selected set contains 1,944 source cells and 3,120 edge factor cells.
Exact end-to-end checks cover 1,944 source and 3,024 edge cells. The 96 very
large edge cells have exact per-operation containment checks plus directed
Decimal400/600 corroboration, which is not a substitute for proof.
Observed zero unresolved cells supplies no universal correct-rounding claim.

Independent audit proved the coupled binary64 exponent difference in
[-51,9] from the same admitted y/r/s; the older [-52,9] capacity premise is
conservative. It separately confirmed all 730 r/s pairs over 275 biased
yields (200,750 cells). The Treasury derivative ratio capacity has its own
aligned-sum/temporary proof; gilt's different coefficient proof is not
substituted.

Review also found and corrected research-tool defects before adoption:

- the nearest-midpoint diagnostic omitted the closer midpoint below a binade;
- values strictly above overflow could be mislabeled exact boundary ties;
- the source PASS gate omitted some reported observations;
- a failed gate printed FAIL while returning success;
- a nondegenerate interval merely touching a tie-even midpoint was described
  too broadly as necessarily unresolved;
- nonfinite log diagnostics required explicit string encoding for strict
  JSON reports, rather than bare Infinity tokens.

The updated checker includes exact helper controls for zero/subnormal/normal/
overflow ties, binade boundaries, strict straddles, touching and exact points.
Separate guard probes force failed research gates; these are artificial tool
controls, never fabricated financial states. Historical runs remain retained
with their original limitations.

## Complete model challenges

A separate date/source audit enumerated 3,719,442 adjacent clamped semiannual
pairs over years 1–9999. Every interval is 181–184 days; fixed-day clamping
covers Ignore, and anchor-day 31 covers the EOM-preserving case. It also
reconstructed all seven examples' D/I/F/M/S, Q, raw coupons, dirty price and
raw AI independently. The seven examples do not discriminate short/long
periods with D<I; the proposed D-based generalization is explicitly local.

The candidate observation selects extended53 RNE of a p256 **lower endpoint**,
then directly rounds the exact source amount times that factor to IEEE PV.
It is a deterministic approximation with a stated error bound, not a claim
of correctly rounding every ideal factor. The final contract still needs
the actual bounded algorithm and an independent observation oracle.

The selected direct IEEE PV gate rejects nonzero-to-zero and infinity before
exact price reduction. This prevents a hidden twenty-million-bit extended-PV
accumulator. Risk uses the same accepted finite PVs, complete exact derivative
ratios rounded once to extended53, and direct normalization; raw getter
range errors do not gate other views.

The source review's finite ideal final-period image requires care. Root and
the date/source reviewer independently reproduced a fully valid example:
C=5, r=1, s=182, j=0, y=next_up(-2). Its observed price exceeds the ideal
supremum `18655/181` by `47/6368371348078592`. An ideal-image precheck would
reject a price that evaluation returns. The draft removes that precheck;
actual finite-bracket callbacks govern inversion. Mathematical sensitivity,
observed-price plateaus and residual/conditioning accuracy remain distinct.

The retained observation review proves, with `t=2^-53` and
`v<=60001*2^-255`, the selected factor ratio in
`[(1-t)*(1-v), 1+t]`. For every accepted finite PV its universal error bound
is `(2*t+t^2)*ideal_PV + 2^-1075`. A valid zero-coupon model has accepted
price MIN_SUB and relative price error almost 100%; it therefore cannot be
covered by an unconditional tiny relative tolerance. The absolute bound
and a directly rounded subnormal witness are retained instead.

It also supplies Treasury-specific resource analysis: factor exponents fit
i32 even at j=19999; normalized p256 products require 512 bits, while the
modeled quotient extraction needs a **513-bit** temporary (nine words).
Input construction separately needs 34 words. Accepted-IEEE-PV risk terms
and their positive sums fit the proposed 243-word, 2^-8192 basis with large
headroom, and the coupled derivative numerator/denominator fit five words.
These are mathematical obligations for the forthcoming actual algorithm,
not evidence that Treasury Rust has already passed those paths. The retained
checks cover 260 factor cells and 1,300 PV candidates (1,112 accepted and
188 correctly rejected), with 1,112 corresponding risk cells, plus separately
labeled range/subnormal/rounding witnesses;
j-cap claims here are symbolic bounds.

## Pinned execution evidence

| Artifact | SHA-256 |
|---|---|
| Expanded source checker | `04f268d313b82d1c77421903adc855ebb4ada5f3c25beeaf6db22dd4ef718e8d` |
| Root source run `/private/tmp/sabiquant-treasury-root-review-v1/source-examples.json` | `28d48aebd7bb97c6a13da3110e7ddd037ea1b271a06e68e0dd562e0a26ad7a25` |
| Root source reproduction metadata `/private/tmp/sabiquant-treasury-root-review-v1/source-reproduction.json` | `e2017c8ce1ee2cc2d5a0cff9981d640d60bb88db79855f6d3a7f989dd49c5a7a` |
| Independent source audit `/private/tmp/sabiquant-treasury-source-audit-independent-20261020-01.json` | `b645ace57ad937f465b3efbd33d57bd02ffa78a47b7790670caf751a8dc634de` |
| Source refusal/replay checks `/private/tmp/sabiquant-treasury-source-v2-audit-verification.json` | `93db6f76240b6c1efefcb654570990ad93afcb779f2928af2f16cec22ae6a35c` |
| Current power checker | `39ef382e51a0517de090796681ae32d31ad77a205b21d64045519b34269b5d6c` |
| Root strict-JSON power run `/private/tmp/sabiquant-treasury-root-review-v1/power-design-v3.json` | `2fcfbb8332f331d30f80ff14a567cec5ac61067182e8f4b80afa9b0f2afc81bd` |
| Root strict-JSON/numeric comparison `/private/tmp/sabiquant-treasury-root-review-v1/power-v3-encoding-verification.json` | `4b425679c2bb437c1cb276117bfc4e90a0c2beb0b04a9354f4e81f65ec1e2d6b` |
| Root power guards `/private/tmp/sabiquant-treasury-root-review-v1/independent-controls.json` | `9a7cd73154cf553d6d6318f3793efeeed9351f06fbed57037100fd9a7653298e` |
| Full date/model audit `/private/tmp/sabiquant-treasury-design-source-audit-v1.json` | `0ce5f1483d985449ad329a003ca71959eedd138cd71542dba826e0dc76d23c40` |
| Historical observation review `/private/tmp/sabiquant-treasury-observation-review-v1.md` | `64486e399de75fa7cee7cc6b53599037e1ffa831ab02c6b54a2b4d8559584d20` |
| Observation checks `/private/tmp/sabiquant-treasury-observation-review-v3.json` | `0c2da8454837fc8b2e31a0085bce0bfbffbd70dc939fb011bf6f1a3fffa5a89e` |
| Root upper-image counterexample `/private/tmp/sabiquant-treasury-root-review-v1/ideal-supremum-counterexample.json` | `e79835079629033aa085fcaad986a85987a62610f05de69fc20d5cbed2eba4f9` |

The observation review/checks pin historical proposal bytes and the v2 power
checker. The final power run is numerically identical to v2 after strict
diagnostic encoding. The current proposal removes the challenged upper-image
precheck and qualifies short/long D<I interpretation. The retained reports
do not claim review of those subsequently edited bytes; the corrections and
independent root counterexample make their use traceable.

## Remaining implementation gate

Complete and review the full signatures, traits, error/stage ledger,
constructor order, resource/temporary proofs and numerical comparison policy.
Then freeze independent source/cash, primitive observation, ideal all-risk
and target-driven inverse expectations before production. The current
research scripts are not those general fixtures and must not be presented
as implemented Treasury behavior.

No library or existing test changes require a new Rust regression or coverage
run at this checkpoint. The final byte audit records the untouched baseline
and the one intentional readiness-document update at
`/private/tmp/sabiquant-treasury-root-review-v1/final-review.json`. Native foreign runtime,
legal/announcement/calendar checks, owner financial acceptance and broader
M8 work remain separate.
