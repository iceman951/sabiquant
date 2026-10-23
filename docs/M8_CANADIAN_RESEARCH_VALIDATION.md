# Canadian pre-contract research validation

Recorded 2026-10-23 Bangkok on local `feat/m8-canadian-readiness` from
`174ef32`. This is a source-research checkpoint. It adopts no Canadian API,
contract, financial requirement or general production oracle. No Canadian
Rust exists at this checkpoint; the implemented baseline remains the
[Treasury validation](M8_US_TREASURY_VALIDATION.md). No remote writes or
QuantLib implementation reads occurred.

The [source review](M8_CANADIAN_SOURCE_REVIEW.md) preserves the historical
versions, complete-page inspections, conflicting definitions and proposed
local reconciliations. The work retains the full fixed-coupon objective:
semiannual, quarterly and monthly payments, regular and odd endpoints,
compound and money-market quotation, price, bounded inversion, six risk
views and four exact-money operations. Unsupported interpretations must be
labelled local extensions rather than silently deleting difficult cases.
The complete [local design proposal](M8_CANADIAN_LOCAL_DESIGN.md),
[numeric/test companion](M8_CANADIAN_NUMERIC_AND_TEST_DESIGN.md) with72 future
semantic families and [independent numerical review](M8_CANADIAN_NUMERICAL_DESIGN_REVIEW.md)
are now recorded separately. They propose18 public types and33+5 domain
errors; none has been adopted or added to the implemented catalog.

## Reproducible source checks

The new stdlib-only [research checker](../validation/generator/canadian_source_research.py)
requires all four retained primary PDFs and verifies their complete SHA-256
bytes before mathematical work. It neither downloads sources nor reads Rust,
QuantLib, frozen fixtures or observed library output. It is a reviewed
reproduction and expansion of the independent source/cash audit v4, **not**
a second independent audit of that derivation.

```sh
python3 -I validation/generator/canadian_source_research.py \
  --source-directory /private/tmp/sabiquant-canadian-primary-review-v2 \
  --output /private/tmp/canadian-research-new.json
python3 -I validation/generator/canadian_source_research.py \
  --source-directory /private/tmp/sabiquant-canadian-primary-review-v2 \
  --check-report /private/tmp/canadian-research-new.json
```

Output is deterministic strict JSON and create-new only. All mathematical
gates use explicit exceptions and remain active under `python -O`. A byte
guard establishes retained source identity; it does not establish correct
formula transcription. Actual complete-page visual inspection supplies
that separate part of the source review.

| Passing research check | Actual extent and qualification |
|---|---|
| Primary identity | Four original IIAC1.1/1.2/1.3 and scoped BoC PDF hashes |
| Source discriminators | 14 named groups; this is not a Rust test count |
| Appendix 2 | All 21 principal/AI/component-total cells across seven nominals: 20 agree, one printed AI error |
| Irrational price enclosure | Exact 256-bit dyadic root bracket and integer power comparisons certify all seven unique nearest-cent principal and total outcomes |
| Decimal corroboration | 100- and 160-digit prices lie inside that exact rational enclosure; these Decimal calculations alone are not interval certificates |
| Gregorian research census | 148,800 direct clamped pairs per frequency, 446,400 total, years2000–2399 and 31 anchor days per month |
| Daily source comparisons | 734 semiannual, 366 quarterly and 122 monthly coordinates, total1,222; literal Canadian365 versus Act/Act, not an adopted monthly rule |
| Mathematical helper controls | 14 unique-nearest, tie-refusal, root-enclosure, strict-inequality-gap and signed-formatting controls |
| Tool integrity controls | 14 actual subprocess checks, including optimized-Python replay, overwrite/alias refusal, wrong source, modified report and CRLF rejection, forced failed gate and silently corrected source-cell rejection |

The calendar census finds lengths181–184 for f2,89–92 for f4 and28–31 for
f12. It is a finite census of direct clamped anchors, not a test of a future
Canadian schedule implementation or all possible input calendars. The
anchor-indexed census can repeat the same endpoint pair after clamping;
446,400 is the number of executed anchor/frequency cases, not a claim of
that many unique civil-date pairs. The daily matrix includes completed
endpoints specifically to expose the difference
between a literal partial-AI formula and the full-period coupon. Its monthly
literal Canadian365 candidate remains in conflict with the general monthly
Act/Act exception. No equality at `elapsed=365/f` occurs for integer days and
f2/4/12; a synthetic f1 helper checks refusal of that unspecified equality,
without proposing f1 as an admitted frequency.

## Source distinctions that tests must preserve

Valuation accrued and settlement accrued are different ingredients. With
C5, previous2005-08-01, settlement2005-12-01 and next2006-02-01, the exact
values are `305/184` and `122/73` per100. A183-day short coupon inside a184-day
quasi period separately gives valuation `915/368`, payable Canadian365
`363/146`, and plain fixed365 `183/73`. None may be substituted for another
because they happen to share a coupon rate and dates.

The literal late Canadian365 branch for a184-day semiannual period drops by
`1/146` per100 when elapsed182 becomes183 at C5. The report retains that
behavior and the completed181/182-day endpoint ambiguity. Smoothing, capping,
or unconditionally substituting fixed365 would be additional local choices.
The monthly conflict gives `15/31` versus `36/73` at C12, elapsed15, period31.
The p49 generalized MM formula counts one terminal coupon twice if its
printed NC is used literally; at y0, C12, f12, NC1 this gives102 rather than
the once-only101. The tool also tests NC2/3/12 to exclude a one-row coincidence.

Three printed defects remain explicit reference classifications, with the
original values preserved:

- `5*48/365` is `48/73`; its unique nearest8dp is0.65753425, whereas the
  source prints0.65734247.
- The183-day short-last example prints4.986301370 after adding an extra
  full2.50 coupon; the preceding short-last formula gives `363/146`.
- Appendix2's10,000 nominal AI prints83.298. Exact AI is `30400/365`, whose
  unique nearest cent is83.29; that row's total10,082.00 is consistent with
  83.29.

The independent source review visually confirms these printed values in all
three retained releases. A reference discrepancy is not resolved by silently
changing a source observation or freezing its erroneous answer as a future
production expectation.

## Appendix enclosure and monetary staging

For Appendix2, C8, y0.08000001, settlement2007-07-09 and maturity2023-06-01,
there are32 future semiannual coupons. Previous/next coupon dates give
elapsed38, period183 and remaining145 days. Define exact rational
`b=1+0.08000001/2`, then enclose `w=b^(-145/183)` by adjacent256-bit dyadics.
The checker compares their183rd powers with `b^-145` using integers, then
multiplies the exact annuity/redemption bracket and subtracts valuation
AI `152/183`. Every reported principal and one-shot total has the same
unique nearest cent at both enclosing endpoints. Settlement AI is the
separate exact `304/365` per100.

The resulting clean price begins
`99.9871345926088375011453170586975994`. At nominal1,000,000, separately rounded
principal and AI sum to1,008,200.12, while rounding their full-precision total
gives1,008,200.11. At nominal1,000,000,000, full-price principal rounds to
999,871,345.93; scaling the displayed six-place99.987135 instead gives
999,871,350.00. That comparison is a non-tie. The displayed-price100,000
counterfactual is exactly a half-cent and cannot identify an unspecified tie
rule. Scaling the shown per100 AI0.83 to nominal1,000 gives8.30 rather than
the source's8.33; presentation precision is not an intermediate publication
stage.

These checks corroborate the Appendix's full-price principal and separate
AI/component examples. They infer no universal midpoint, signed cash,
arbitrary-quantum or coupon-cash publication policy. In particular, choosing
an MM yield for a bond does not establish the three-place price rounding
policy described for money-market securities.

## Price basis and numerical review obligations

An exact square comparison of a four-payment C8 semiannual half-period
example separates fractional compound discounting from Treasury's rational
first-period factor. Reusing the Treasury kernel under another name would
change the mathematical model.

The research checker also supplies an ideal, dated counterexample to an
unqualified positive settlement-dirty gate: C=`i128::MAX` per100,
previous/accrual2025-09-01, S2025-09-02, M2026-03-01, one terminal coupon and
y=`2^540`. Exact600-bit root bounds prove positive model-dirty PV but negative
`model_dirty - valuation_AI + settlement_AI`. This is ideal mathematics, not
an observation of Canadian Rust. It motivates distinct model and settlement
price bases; it does not select their future binary64 rounding stages.

Complete numerical design must separately establish the compound elementary
profile, exact quasi coordinates, direct source-times-factor observation,
MM affine-domain/ratio arithmetic, all six independently fallible risk views,
three price bases and yield-target shifts. A fractional-compound normalized
convexity can underflow while the raw second derivative remains finite.
Treasury's model-specific exclusions therefore cannot be imported without
a new proof. Getter failures must be audited on valid admitted financial
states rather than by forging private report fields.

## Independent numerical design challenges

A separate producer derived18 private mathematical groups before reading
the future complete Canadian API proposal. It read the genuine sources and
the earlier source/cash review, used exact Fraction/root arithmetic, and
corroborated selected compound quantities with independent second-order
Decimal jets at160/240 digits. Its72 candidate cases cover f2/4/12, eight
regular/odd/composite shapes and negative/zero/positive yields:504 ideal
quantity enclosures, including dirty and all six risks. These are candidate
mathematical geometries; source conflicts and generalizations remain
unadopted. They are not504 prescribed Rust observations.

Root re-executed the unchanged producer in a fresh private directory. All
payloads match after removing only the actual UTC timestamp. Root separately
rechecked73 nonrational integer-root certificates and cross-checked1,134
ideal interval endpoints or exact points through CPython's Fraction-to-float
conversion, against the producer's integer IEEE extractor. Another31 root
certificates are exact rational cases rechecked in the actual producer
replay. Together these are104 certificates; they are not104 extra dated
financial inputs or a universal elementary-function accuracy theorem.

| Ideal dated discriminator | Raw quantity RN bits | Normalized quantity RN bits |
|---|---|---|
| Compound C=`i128::MAX`, S2025-08-31, coupons2025-09-01/2026-03-01, y=`2^540` | Second derivative `03a78156925d44c9`, normal | Convexity `0000000000000000` |
| MM, same maximum C, S2025-06-01, coupons2025-09-01/2026-03-01, y=`2^540` | First derivative `845c99d7ae5bf3d0`, normal | Modified duration `0000000000000000` |
| Same MM dates/C, y=`2^370` | Second derivative `028c99d7ae5bf3d0`, normal | Convexity `0000000000000000` |

The compound proof has first PV above `2^122`, dirty below `2^128`, raw second
derivative above `2^-968` and ideal convexity below half MIN_SUB. The MM
proofs use exact rational affine formulas. A future contract must convert
nonzero-to-zero into its own typed getter failure; these ideal bit values
do not claim an existing public error path or select the staged observation.

The same research includes conditional raw-moment-underflow candidates for
f4/f12. They require a model that allows compound quotation with one future
coupon and admits a sufficiently extended discount/source-product rescue.
A factor-first binary64 admission policy can reject them earlier. Therefore
they remain design-dependent challenges, not unconditional required runtime
failures. Range exclusions for MM also require its own dated/coupled proof,
separate from conservative primitive capacity envelopes.

## Complementary closure after the initial proposal

The [API review](M8_CANADIAN_API_REVIEW.md) found three concrete specification
problems and independently checked their corrections: original coupon ordinal
versus compressed future position; a false amount-residual overflow claim; and
length/requested-slot limits being confused with caller Vec spare capacity or
allocator bytes. Its19,692 dated full-definition reconstructions,
108,864 accrued geometries,14,352 positive-finite residual pairs and actual
Rust Vec ownership witness pass. The18types/6metadata-enums/33+5errors and
CAD01–72 proposal census stays unchanged. Reviewed pre/post snapshots remain
pinned in that review; later public editorial/numeric integrations have their
own current bytes rather than inheriting those old snapshot hashes.

The later [whole source/cash/API integration review](M8_CANADIAN_CONTRACT_INTEGRATION_REVIEW.md)
closes the original missing-L identity and empty/C0 accrued-geometry findings,
checks the full source reconciliation and four cash-stage definitions, and
finds no additional financial/API gap. Its actual14 mathematical groups and
2,380 signed quantizer comparisons include wide component cancellation,
capital-first Exact failure, nonpoint Exact refusal, later stage collapse and
source-view failure with otherwise representable fused cash. These counts
are separate from the source checker's14 groups, not new Rust tests. Its
one integration defect was a circular adoption-order sentence, now corrected:
review/adopt the entire local contract, freeze all independent preproduction,
then begin Canadian production. Literal version/recipe/source-view mappings
are mechanical contract clarifications without a new market policy.

The [conditional compound profile](M8_CANADIAN_COMPOUND_PROFILE_REVIEW.md)
audits the completed native Rust1.99 macOS arm64 experiment:1,207 log rows,
7,374 exponent/factor rows,20 zero-exponent identities and7,354 actual nonzero
exp invocations per mode. Debug/release bits agree and158 consumed inputs
remain unchanged. Initial Decimal-zero harness failure is preserved; the
corrected run uses exact-zero identities and analytic tiny intersections.
The JSON floating display of a maximum error is not a directed upper bound;
in particular the shown ln1p1.0u must not be claimed as <=1u. Exact comparisons
establish the finite run's<2u gate. This remains conditional finite native
primitive evidence, not universal libm accuracy or foreign runtime.

Under explicit2u primitive hypotheses the reviewed shared-log bounds are2u
for f2/4 and6u for f12. Non-tiny exponent radius is
`(beta*abs(z)+2^-1075)/(1-beta)` with beta4u/8u; tiny radius retains both exact
RN distance and the analytic logarithm remainder. The
[bounded guard design](M8_CANADIAN_COMPOUND_GUARD_DESIGN.md) supplies the
fixed20-word exact endpoint/threshold comparison, closed-touch refusal,
tiny/far-range bypass proofs and full typed error mapping. A disjoint
certainly-out-of-range interval still calls exp once and validates the actual
result. Its7,374-record integer replay plus36 capacity,10 touch and72 tiny
controls pass. The63 proposed pre-exp refusals and resulting7,291 exp calls
are a simulated future protocol, distinct from the actual measured7,354 calls.

The [date-coupled MM range proof](M8_CANADIAN_MM_RANGE_REVIEW.md) closes the
full selected factor/PV/dirty and all six getter ranges, including arbitrary
admitted true-maturity delay. It proves no MM factor/PV/dirty zero/overflow,
no risk overflow, and normal finite raw moment. Nonzero subnormal factors,
PVs and dirty remain admitted. Raw first/second derivatives and normalized
Modified/Convexity underflows are still real. Its selected observed one-day
Macaulay can be slightly below ideal1/365; the correct observed bound is
`(1-u)^2/((1+u)*365)>1/730`. Exact clean/bridge/target transformations and
positive-finite amount residuals cannot overflow or round nonzero to zero;
exact cancellation remains valid. These are domain proofs, not invented
runtime failures or additional Rust coverage.

A separate complete dated two-flow witness certifies the selected MM dirty
observation at MAX with five strict exact nearest-cell checks. Its bits
`3fd5375375375375` are below the ideal image floor181/546 by
`59/4917930793088581632`. Future observed inversion must accept a matching
cached upper target with two function calls and zero derivative calls; no
ideal inverse exists for that target. This is a protocol expectation and
exact Python staged certificate, independently reproduced and checked at all five
stages. The future fixture pins valid solver options(1e-12,0,2); these checks
are not an actual Canadian Rust solve or a
conditioned ideal-root accuracy row. The immutable witness and complete
financial-definition metadata are linked by hash below.

The [numerical resource review](M8_CANADIAN_NUMERIC_RESOURCE_REVIEW.md)
proves an available by-reference route: MM factor2176bits, selected second
term6420bits, source PV214bits, compound terms192bits, retained/direct
normalization9502bits. It counts exact_sum's two34-word arrays inside the
32large-slot allowance, with3retained+3scratch risk arrays. The resulting
40,336byte logical wide-payload ceiling plus1,920guard bytes is42,256bytes.
No stack/layout/allocation measurement is inferred. Actual controls include
all39 input scales and fixed53-step quotient execution. The separate complete
cash-resource review is now closed as described below; actual future Rust
lifetimes remain an implementation gate.

The [complete cash-resource review](M8_CANADIAN_CASH_RESOURCE_REVIEW.md)
closes four recipes, seven signed modes, both aggregations and continuous
Exact semantics with an explicit arithmetic route. Pure quantum ratios fit
414/409bits and coupled rounded coefficients390bits. The branch-specific
binary-price decoder combines one cached source denominator once, giving
2501/1483bit ratios within40words; an uncancelled Treasury decoder transplant
would have a2589bit bound and is not the reviewed route. Cash keeps a WIDE
quotient, distinct from normalized53 numeric division. Available by-reference
schedules count nested source scratch and four source-pair copies:960bytes
pure and8640bytes price, inside their1024/10240 limits. Instruction/receipt
field budgets remain proposed sizes, not measured layouts.

Its first run has39,312 source-scale,14,112 price-width,70 signed-midpoint and
28 evaluated-stage controls; an additional precision-hole record explains29
stage payload records. The boundary run adds252 explicitly executed source
views and1,008 pure cash controls, plus four discriminators and six bounded
bit divisions. Canonical-source controls cover1,404 cases; stored nonzero
source pairs are chosen gcd-reduced, zero0/1, with constructor/preparation work
and no gcd during cash evaluation. A later53,235-quantizer grid visits all39
nominal/quantum scales as39x39 pairs with rotated coupon scale. This is not
all39^3 triples, unique financial input coverage or Canadian Rust tests.
Root re-executed the unchanged first two scripts: all mathematical payloads
match, excluding only actual UTC/current input-snapshot metadata where present.

The [independent complete resource/protocol challenge](M8_CANADIAN_RESOURCE_INTEGRATION_REVIEW.md)
finds no remaining blocking design issue. Its own27 relative-width, seven
subnormal, eight signed-Horner,77 signed-mode,14 magnitude, six quotient and
14 cancellation controls and1,404 canonical-record cross-checks are separate
from producer reproduction. It preserves an initial checker prose-match
failure, corrects lambda>=1/184 and the first Euclidean-operand wording, and
confirms actual layout/copy audits follow implementation. It also checks the
[proposed preproduction protocol](M8_CANADIAN_PREPRODUCTION_PROTOCOL.md):
canonical LF is an artifact-byte rule above the reader's LF/CRLF acceptance;
each quantity/ordinary regime needs predeclared informative checks; inverse
reference equations distinguish exact-source external bases from cached
resolved targets. Actual native ell_hat/exp observations use certified ideals
WITH their primitive allowance. Exact seeded integer stages stay distinct
from dated public inputs, and public single-result/guard expectations require
stability across allowed primitive variation. No macOS-only elementary bit
snapshot is silently made a portable contract.

The unchanged production baseline was checked by exact byte continuity:
266native,267coverage,54package-payload and158primitive-profile consumed
inputs match, with no src/tests/fixtures/old-generator change across281
tracked paths. It reuses earlier guarded Treasury evidence without claiming
a new native run or inflating the914debug/913release baseline.

## Retained execution evidence

| Artifact | SHA-256 |
|---|---|
| Current research checker | `6190d21003f60ed9c5f468b2164077a8ce8cbc0d0f328d344e820e19f8fe2a63` |
| Root report `/private/tmp/sabiquant-canadian-root-research-v3.json` | `750175af0d977ba40cd790860c91d7d35d79a63f14a24eea3856475e66c62feb` |
| Actual14-control evidence `/private/tmp/sabiquant-canadian-research-guards-v3/evidence.json` | `ed47c10464a942cddbc28bf54ded34248f6013827b0a4385e4c84ddc44e85905` |
| Guard runner `/private/tmp/sabiquant-canadian-research-guard-runner-v3.py` | `796af85aaf0ddd50f83f6ce407da965d70262e91d46864fea138c2d78dc46a08` |
| Reviewer's earlier CRLF counterexample `/private/tmp/sabiquant-canadian-source-review-check-v1/crlf-evidence.json` | `ef73be261933fbf4fafc25d58d72cad4b75560f7c4e62a70b8d6fe1ba6a8e2ac` |
| Independent source/cash review v4 | `71b9fef82fc1bb61613e4b47797eda8b77a92232ed8c37a517be55e03eb43cfa` |
| Its evidence `/private/tmp/sabiquant-canadian-sourcecash-challenge-v4/evidence.json` | `a377cb699d32aa0f54c06401260be44270d9c5eaae8f5302bffdc8c5c11e67f1` |
| Its derivation v4 | `42caf52a16b38a318b1762969b7a6220b5718d02c8ea7687ce12c150d4a60b4a` |
| Its arithmetic v4 | `ac3f3ff12bd94b349ecf1e7a159c3a595cc9475f96b2006dcda9be25d80a892b` |
| Coordinator primary evidence | `5f468b3510877bfaa23a0c63f8b5c1983d323cb2af6e4b3cf539b5b214636812` |
| Independent numeric producer v4 `/private/tmp/sabiquant-canadian-numeric-challenge-v4/derive.py` | `e28a558b56ed7b1f4b73d258918d9f2631aff943a365fdcffa1036d29ecfa79b` |
| Its18-group/104-certificate arithmetic v4 | `6a552ca275461cb550b48b7acf3bbc794d15970892ac99dfd0add249ada80bfc` |
| Root actual numeric replay `/private/tmp/sabiquant-canadian-root-numeric-reproduction-v1/arithmetic.json` | `32b6bb1b9e2b170254ca90ed0fbdb259d8a6747798686d5ecbfa32fcfdc5ab55` |
| Root numeric comparison/cross-check `/private/tmp/sabiquant-canadian-root-numeric-reproduction-v1/comparison.json` | `7ea12ee9272ebf51cf4cb04d0f9a5e17fb41eac75045db29d0e807bc5de04d42` |
| Independent final byte-guard confirmation | `73ffb4f5523810f2ca8ab96a606ee5544f2d2e85e089e81b9064583f928802d9` |
| Original complete proposal v3 | `8da905ca5178844005f67a4288744678f98e85e433bb34aa8f84593923a335e9` |
| Original numeric/test companion v3 | `ea2306ac4789d177745bd22dd2afa48195396d4eb982103298c0b1d16c9a85eb` |
| Proposal provenance `/private/tmp/sabiquant-canadian-complete-proposal-v3/provenance.json` | `27a9763209c8a204c00f829e79bce327118a4169f2c450a03141d38df0984b9b` |
| Original independent numerical review v4 | `986f68cafe82ecefd1f96f9f6fd7c98ebc11666d43cd8bc093070f227f294901` |
| Original independent numerical evidence v4 | `b6add8043ba8517239b114a8fbf5cdeb537befa7f6556cc52b464efea9842985` |
| Independent API evidence `/private/tmp/sabiquant-canadian-api-challenge-v2/evidence.json` | `78793f96125f5f2f24b51bd030f23f7d9ff848144b87bf8e61ce6b0fed6cc4ce` |
| API post-fix closure `/private/tmp/sabiquant-canadian-api-challenge-v2/closure/review.md` | `4c609dd057c5e6cc0c08cef109d720d34a692578678aedf979d82cddc6dc276e` |
| Complete source/cash/API integration `/private/tmp/sabiquant-canadian-contract-integration-review-v1/evidence.json` | `89f6182c5c7636e308bea912c1071393af753eca48294ae81d7a75dc4640589c` |
| Actual compound native profile `/private/tmp/sabiquant-canadian-compound-profile-v1/run-v2/evidence.json` | `1697b753a6ca6fcf0916226d048c91cf26eb3ec2a77fd0ae1f270f65cbe58dbc` |
| Independent profile audit `/private/tmp/sabiquant-canadian-compound-profile-review-v1/evidence.json` | `699c9e4c7ef6cc79501274dc99be4e2361c54ad7adad597679ea957283e7d12a` |
| Certified threshold boxes `/private/tmp/sabiquant-canadian-compound-profile-review-v1/thresholds.json` | `0cabba25a333caa8efa4b6dc287c0bd50d29f2d02608748c7ded841817ae395e` |
| Bounded guard evidence `/private/tmp/sabiquant-canadian-compound-guard-design-v1/evidence.json` | `f5a98bdd7df6ed5a4c65ee635fe815923117e48a4724dc6be27b1954347e7e1b` |
| Root unchanged guard replay and limb-order controls `/private/tmp/sabiquant-canadian-root-guard-replay-v1/evidence.json` | `33399e79a6eec23c372515afdc80ad91a0867f5a3c509a18191493e5c0d378ce` |
| Independent MM range evidence `/private/tmp/sabiquant-canadian-mm-range-review-v1/evidence.json` | `6dfe26700c24e392a393c1abb168c80d0a8cc80b7f3a34cd6c60b80e4476c1e1` |
| Root observed MM image `/private/tmp/sabiquant-canadian-mm-observed-image-v1/evidence.json` | `c306118cb5645e22b5340ca28810814a8b844c36c370b11da52a59880d65bac2` |
| Its complete definition `/private/tmp/sabiquant-canadian-mm-observed-image-v1/complete-definition.json` | `bd6dcd3fe0c2a224331c2c881d1fa8822df8414db03c0de10e5fa6afd955b05f` |
| Independent image/endpoint protocol challenge `/private/tmp/sabiquant-canadian-mm-image-review-v1/final-evidence.json` | `fd28688a2c8ee65ea6bff4059cb8d3b5b5cd7ef3f4344c56a1601acd01ec0f98` |
| Numerical resource evidence `/private/tmp/sabiquant-canadian-numeric-resource-review-v1/evidence.json` | `1a5979a2cf2e932e79b511f3f1a44cc768e1f1fa4bedea89b8c4bf7b0fdb41db` |
| Complete cash final index `/private/tmp/sabiquant-canadian-cash-resource-review-v1/final-index.json` | `a7e7a2cb4414d6217d822077837a9821c2850d2a865649ba46f03e6bd3906206` |
| Cash primary evidence `/private/tmp/sabiquant-canadian-cash-resource-review-v1/evidence.json` | `b9274e40ef118c0f5e9b9cdf8b289feda9f2457d9e1cfe118dc9d9f1cde091f9` |
| Root unchanged cash replay `/private/tmp/sabiquant-canadian-root-cash-resource-replay-v1/evidence.json` | `222232bc16b343656051fc731fb6c99b95b3611c2ad2566b8b9bcf246c70c519` |
| Independent complete resource/protocol challenge `/private/tmp/sabiquant-canadian-resource-integration-challenge-v1/evidence.json` | `d1b1da30d724c45fe9e3ca686ea211e94b442d78248b3d3bae39bbfbf9c49752` |
| Exact baseline continuity `/private/tmp/sabiquant-canadian-baseline-continuity-v1/evidence.json` | `2e01628802e813a0c85800839b9a93b1ab9ce804ba0c2ec31b24c60845b5062e` |

Root v1/v2 reports and guards remain retained. v2 added an explicit
calendar-pair census gate. Independent review then found that text-mode
reading normalized CRLF and accepted different report bytes despite the
reproduction claim. Its actual counterexample and earlier13-control passes
remain retained; those passes had not tested CRLF. v3 compares bytes and
writes explicit UTF-8 in binary exclusive-create mode, and the new CRLF
refusal passes. Mathematical payloads are unchanged. The final14 controls
are tool-integrity/reproduction checks, not Canadian runtime tests, production
semantic mutants or additional coverage regions. Both final tool input and
the original report remained byte-identical throughout their guard run.

The repository proposals are editorially promoted copies of the private
v3/v4 snapshots. They update local links/formatting and explicitly name
derived scale39/40 values. The numeric companion corrects one strict
inequality to `<=` for C0 in its MM source-total bound; the subsequent
`<2^142` bound is unchanged. Private originals and their hashes above remain
immutable. A compact cash snapshot originally lacked the last coupon's
accrual start; independent review demonstrated equal first cash from two
different long-last schedules. v3 retains that date, proves full-definition
reconstruction and records516 finite abstract controls. Empty accrued
interval `(S,S)` and nonempty C0 geometry are also explicit. Those controls
remain distinct from the subsequent conditional compound profile, MM staged range
proof and numeric resource closure above.

## Remaining adoption gate

The complementary source/API/cash and full numeric/guard/cash-resource/protocol
reviews are now complete design inputs. Integrate and independently review the
entire candidate contract and freeze its exact component bytes before local
adoption. Actual implementation lifetimes and foreign native runtime remain
uncertified. Then freeze
independent source/date/cash, prescribed numerical-primitive, ideal price/
six-risk and target-driven inverse evidence, plus blind hand controls, before
any Canadian production. Keep every old contract, generator and fixture
unchanged. Native foreign runtime, financial/owner acceptance, calendar/legal
interpretation, rigorous ideal-price producers, release and wider M8 gates
remain open.

This checkpoint changes research documentation/tooling only. It requires no
new Rust coverage percentage or native/package rerun for unchanged production.
