# Complete Treasury contract integration review

Recorded 2026-10-20 from local `7bfd94f`, before any Treasury Rust or general
production fixture. This records independent challenges, corrections and
root reproduction for the complete price/yield/all-six-risk/four-cash
contract. It is local engineering evidence, not owner financial, legal,
calendar, native foreign-runtime or full M8 acceptance. No QuantLib C++ was
read and no remote write occurred.

The reviewed [main draft](M8_US_TREASURY_CONTRACT_DRAFT.md) and
[cash draft](M8_US_TREASURY_CASH_CONTRACT_DRAFT.md) are preserved as snapshots.
The authoritative local adoption is the paired
[main contract](M8_US_TREASURY_CONTRACT.md) and
[cash contract](M8_US_TREASURY_CASH_CONTRACT.md). The
[test design](M8_US_TREASURY_TEST_DESIGN.md) retains76 semantic families
against26 requirements. Those are obligations, not executed Rust tests.

## Independent review scope and closure

Claude peer sabiquant-a2 reviewed original date/source interpretations and
then the complete joint API/cash drafts. Claude peer sabiquant-d5 reviewed
the deterministic observation/proofs and then the actual joint draft bytes,
including quantitative comparison allowances. Reports pin their exact
input snapshots and distinguish untouched earlier reports from new deltas.

| Challenge | Resolution in the retained contract |
|---|---|
| Derived source caches exceed128bits | All derived exact caches/counts use five-word Scalar; source X can need161bits, raw two-piece AI144bits, published count147bits; no public amount gate |
| Constructor/plan capacity ledger incomplete | Explicit first-cache, daily9, total AI5 and IEEE-cache phase/operation/index rows; Actual coupon ordinal differs from numeric absolute Q index |
| Coupon instruction view stage/identity unspecified | Source coupon/point/quantize; instruction source view equals instrument selected-source view including errors; AI view uses exact published value |
| I>=F error/precedence unclear | FirstCoupon before role inference, after Q/cycle validation |
| Coupon form notation collided across documents | One main-E table defines X/dscale/s_part and Ccoef/Cscale; cash aliases its first-day variables explicitly and defines every scale in fused ratios |
| Source raw-view width phrasing | X/10 quantization261/265bits, versus per100 raw-cache262/262; both inside265bit bound |
| Cash logical temporary budget too tight | Sixteen40-word slots count explicit by-value callee copies, plus12eight-word slots; actual implementation census remains required |
| Cash phase/source provenance ambiguity | Published AI binary cache has distinct phase; every recipe's inspector matrix and view identity is required |
| Diagnostic decisions versus identity-bearing receipts | Instruction/Determined receipt retain full snapshot; unresolved/ambiguous decisions compare only listed diagnostic fields |
| Source ties/nominals not fully specified | One explicit source tie rule governs daily9 and total AI5; no extra five-place stage after nominal; final cash uses independent caller rounding |
| Whole-domain PV width overestimated by uncoupled shifts | Correlated magnitude-first proof gives at most206bits, five words; denominator subnormal alignment is bounded by numerator bits+1 |
| Risk final subnormals and single-term cancellation | Exact absolute/conditioned allowances, independent getters, explicit h and a sharper single-positive-PV cancellation profile |
| Some legacy sweep rows were not admitted instruments | Preserved as primitive supersets only; financial sanity evidence replaced with date-validated, legal DecimalAmount-domain inputs |
| n0 target-image precheck would reject observed prices | Removed; ideal bound is not observed range; actual bracket callbacks govern solving |

Both final targeted reviews report no remaining adoption blocker in their
stated complementary scopes. The v3 API/cash review's two nonblocking notes
are integrated: the raw-view width sentence now names both forms accurately,
and UST-61 names the actual fractional-nominal discriminator. These do not
change an API, recipe, width class, error or numerical algorithm.

## Quantitative evidence actually reproduced by root

1. **Cash bounds and anchors.** The unchanged standard-library checker
   reproduced every field of the peer report except actual UTC time.
   Conservative full-scale operand bounds are397bits for source/views and
   2434bits for price recipes, inside512/2560bits. These are calculated
   bounds over an explicit superset, not claims that each maximum is an
   attained economic cash flow.
2. **Dated numerical models.** Root reran the independent joint checker:
   5688 admitted date/DecimalAmount definitions,4302 accepted by the PV
   observation,1386 correctly rejected. Getter range exclusions had zero
   violations;2590 single-positive-PV normalized cases satisfied their exact
   cancellation bounds. It also checked876582 semiannual spacings over all
   146097 maturity days of a Gregorian400-year cycle and both EOM policies.
   Output matches the peer's report except actual UTC time.
3. **Ideal allowances.** Root reran the exact-Fraction allowance checker:
   3795 valid definitions,3161 accepted, zero allowance violations across
   dirty/clean and all admitted raw/normalized getters. It checks exact
   inequalities, not first-order constants. Only one source policy was used
   for its clean check; the general policy/source checks are separate.
4. **Concrete nominal stage discriminator.** D=I=2025-07-15,F=M=2026-01-15,
   S=2025-07-16,C=.000368 has published AI1e-5 per$1000 under all four source
   policies. Nominal499600 gives exact cash input.004996. Final nearest/away
   cents gives0; an extra five-place stage after nominal gives.01. Root and
   the source reviewer independently reproduced it.

The numeric sweeps corroborate analytical proofs; they do not prove the
whole domain statistically. No result here is a Rust integration pass.
Existing library/test bytes remain the verified gilt baseline until the
separate Treasury implementation phase.

## Evidence qualifications retained

The older getter_ranges.py used C=2^127 and independently sampled incompatible
date coordinates. Its3000-row sweep is a primitive superset and is not
financial-domain evidence. The older direct-PV width addendum similarly
paired two consecutive184-day periods in one extreme; its4000-row sweep is
primitive evidence only. Neither file/report was rewritten. The dated joint
checker replaces those uses. Their analytic proofs and independently dated
real witnesses are separately assessed.

The earlier numeric review wrote A0=c+100 for every n0 case. That shortcut
only describes regular/later coupons. The actual contract uses raw coupon
plus100, including short first and long first settled after the skipped Q1.
The general excluded ideal upper image A0/(1-alpha) is unchanged. Its exact
counterexample depends on the selected factor/PV observation; it is not
misrepresented as a property of ideal correctly rounded price at those same
inputs.

The sharper multi-term relative-error lemma and first-order getter constants
in early research are optional and are not used as contract guarantees.
The contract instead adopts exact absolute/product forms with conditioning,
including final subnormal rounding and single-positive-PV cancellation.
The binary exponentiation sequence stays fixed; cumulative powers remain a
possible future profile change, requiring new observation evidence.

## Pinned reports and reproductions

| Artifact | SHA-256 |
|---|---|
| Final reviewed main draft | `c850551b1512b039c5876ffdefdb9d8c783458098316b6fafcf3877f9e1c5753` |
| API/cash review v1 `/private/tmp/sabiquant-treasury-joint-api-cash-review-v1.md` | `9a4740553a8b5648e31409cd7413efcce2df47084f68971ffc92c58f7974561e` |
| API/cash review v2 `/private/tmp/sabiquant-treasury-joint-api-cash-review-v2.md` | `d341b0820b2a0c4e2d61d72c8de4188167dd22b4e09124da19546a0b2e5f35fc` |
| API/cash final review `/private/tmp/sabiquant-treasury-joint-api-cash-review-v3.md` | `f53576d0849225d77a476f45bb233e99e7c966f10e43ec8d0a4381e896785010` |
| Numeric review v1 `/private/tmp/sabiquant-treasury-joint-numeric-review-v1.md` | `c19b93f5070fa7e22219f23e8fdc3f1b8589cef0b1c41c2c8a661bbe23473144` |
| Numeric review v2 `/private/tmp/sabiquant-treasury-joint-numeric-review-v2.md` | `d89eb357cd0889e48a9bfd977abd6dfc161658d2647c3d99bc20bbba972b2135` |
| Numeric final review `/private/tmp/sabiquant-treasury-joint-numeric-review-v3.md` | `fe832dcfa056122c55ddb444b503d67765724e11daf6851387774d2da0b652ee` |
| Cash root reproduction `/private/tmp/sabiquant-treasury-contract-review-v1/cash-width-reproduction.json` | `be3eba3fc2d407576a939f2f141665c682e96583856e1a646e82cd51ff2832a9` |
| Dated numeric root reproduction `/private/tmp/sabiquant-treasury-contract-review-v1/joint-numeric-reproduction.json` | `ec6412f932b45cd5f814c67bcbad0bad72d6a2345248022dc5023bd225300af2` |
| Allowance root reproduction `/private/tmp/sabiquant-treasury-contract-review-v1/allowance-reproduction.json` | `7024f272079fc7ef33009291cb17472e67da2929edbbeadaea04310d1c7e4862` |
| Root nominal discriminator `/private/tmp/sabiquant-treasury-contract-review-v1/fractional-nominal-discriminator.json` | `b06105b50da3fe7f592a2c07b8d15589b732ec077d124517e66e34d5f48741df` |

The final adopted files make only recorded adoption/reference/status edits
to the reviewed draft semantics. The byte/structural audit checks the final
pair,26requirement and76family counts, declared input pins and preserved
baseline. It is separate from future production verification.
