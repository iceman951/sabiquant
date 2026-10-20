# Complete Treasury test and oracle design

Adopted as the test plan on2026-10-20 with the complete local
[main contract](M8_US_TREASURY_CONTRACT.md) and
[cash contract](M8_US_TREASURY_CASH_CONTRACT.md), following the
[integration review](M8_US_TREASURY_CONTRACT_REVIEW.md). This is a plan, not a
generated fixture, executed test result or coverage claim. Every family
remains an obligation until its actual evidence is inspected. No Treasury
production exists yet and no QuantLib implementation was read.

The 76 semantic families are **purposes**, not a promise that 76 Rust
functions or a large row count alone proves coverage. One family may require
several independent domains and tests; one test may cover multiple families
only when its assertions establish each obligation. Authoritative FR-UST IDs
are those in the main contract. Broader M8/financial/native-platform gates remain
open independently.

## Independent artifacts and comparison rules

With the contract adopted, author four new independent corpora before Treasury
Rust: source/date/cash, prescribed numerical primitives, ideal all-six-risk,
and target-driven inverse. Use separately derived integer/Fraction and
dual-precision Decimal routes, with actual environment/source/self hashes,
create-new outputs and an attested shared fixture reader. Use representative
domain partitions and exact algebraic constructions rather than blindly
duplicating all implementation branches.

Exact source dates/cash/decisions and prescribed binary observations use
TOL-E0. Ideal values use the final `TOL-UST-RATIONAL` outward absolute and
conditioned bounds, including final subnormal rounding. Inverse accuracy
must include observed residual, model error and slope/conditioning. Do not
relax a tolerance to absorb a mismatch. Record uninformative rows explicitly
with no financial-accuracy assertion; their exact staging can still be tested.

Literature displays remain separate from binary64 oracles. Source A–G do not
uniquely choose daily basis/ties; I.A/I.D unrounded products discriminate the
table stage. Printed intermediate anomalies and source typos remain preserved.
Current research counts are historical checks, not counts of future fixtures.

At least one hand author must derive independent discriminators before
reading Treasury production. General oracle authors must not import the
production implementation, use its output as expectations or rewrite old
gilt/generic fixtures. Later implementation-read supplemental controls must
be labeled accordingly.

## Semantic families

| Family | Requirements | Discriminator and required evidence |
|---|---|---|
| UST-01 | 01 | Exact retained source version; unchanged fixed-coupon source bodies and explicit II.E typo interpretation |
| UST-02 | 02 | Fourteen-type census, ALL order, normal/error traits; no defaults/parsers/implicit float-money conversion |
| UST-03 | 02,12 | Instrument/report/instruction identity retains policies even where all current numeric outputs coincide |
| UST-04 | 03,23 | Date cap before count; too-many before too-few/financial validation; negative C and D/I/M precedence |
| UST-05 | 03,05 | D=I and D<I regular; short/long D<I local interpretation; long I>Q1 still accrues from D |
| UST-06 | 04 | Direct maturity anchoring versus recursive February clamp; supplied Q mismatch retains original index |
| UST-07 | 01,04 | All48 Table1 day cells; month-end Preserve versus Ignore, including Apr30/Oct31 and leap February; correct Treasury Q with Ignore is rejected |
| UST-08 | 04,24 | Date MIN/MAX, full supported span and conservative cap; no oracle dates beyond year9999 |
| UST-09 | 05 | Every accepted Regular/Short/Long shape; malformed F, D=Q0/F=Q2 and I>=F errors |
| UST-10 | 05,08 | Missing long Q1 is not a payment and does not renumber j or original error indices |
| UST-11 | 08,23 | Bundle cap and S<I/S=M/S>M order; S=I accepted; actual coupon on S excluded |
| UST-12 | 08 | Ceased/BuyerEntitled optional inspectors and invalid indices; zero coupon remains actual; no ExCouponDetached state |
| UST-13 | 06 | All640 daily table cells; no daily9 ties on the1/8 grid; exact-ratio alternative retained |
| UST-14 | 06 | General-C daily9 exact ties at each s181..184; both tie rules discriminate before multiplication |
| UST-15 | 06,09 | Full116800 staging grid, raw/staged5dp ties, 270/350 basis-changing partitions; independent integer rounding |
| UST-16 | 07 | Full39 C scales, zero, canonical equivalence, i128 positive maximum;161-bit source caches and144-bit two-piece AI exercised at scales0/38; no public decimal gate |
| UST-17 | 07,18 | Raw versus source short/long coupon, unrounded nominal scaling, full regular coupon not daily9*s |
| UST-18 | 07 | Scale39 regular half coupon and nonterminating stub; coupon instruction view equals instrument source view, including quantum/mode errors |
| UST-19 | 09 | AI at D, skipped long Q1, F and later actual coupons; first piece persists only until actual payment |
| UST-20 | 06,09 | Total-first AI5 versus component-first; tiny C rounds daily9 to zero while raw AI is positive |
| UST-21 | 10 | Exact b/q construction from represented y; MIN_SUB, smallest normal, near-2, MAX, ±0 and alpha extremes |
| UST-22 | 10 | No early rounded y/2 or alpha; exact Big seed cancellation, divide/round normalization and carry |
| UST-23 | 10 | p256 low/high containment and width weights; MIN_SUB error amplifies with j, not log j |
| UST-24 | 10 | L2R power tree differs from cumulative/reassociated tree; freeze selected observations at carry-sensitive inputs |
| UST-25 | 10 | j0 skips q*1, j1, powers-of-two j, mixed-bit j and conservative cap; exact identities when b/q are powers of two |
| UST-26 | 10,24 | 512-bit product and513-bit quotient temporary; nine-word carry/normalization; exponent extremes without huge shifts |
| UST-27 | 10 | Extended53 RNE of lower endpoint, including carry; observed factor can exceed ideal and is not an ideal lower bound |
| UST-28 | 10 | Endpoint touch versus strict midpoint straddle and exact point; no uniform ideal-RNE claim inferred from observed samples |
| UST-29 | 10,23 | Nonfinite y, y=-2/below; canonical ±0 report yields; validation before allocation or seed work |
| UST-30 | 10,24 | Independent exact per-operation proof controls and huge-exponent corroboration; Decimal cross-check is not containment proof |
| UST-31 | 11 | Exact source coefficient times accepted factor with one direct IEEE rounding; no source coefficient float gate |
| UST-32 | 11 | Dated direct-versus-double-rounding discriminator and exact PV tie; numeric RNE stays even under both source tie policies; separate primitive witnesses |
| UST-33 | 11 | Finite subnormal PV accepted, nonzero-to-zero rejected, infinity rejected with original source index |
| UST-34 | 11 | Tiny factor rescued by source scaling; factor itself has no IEEE zero/infinity gate |
| UST-35 | 11,23 | First failed nonzero PV wins even if later reduction would appear usable; zero source still observes its bounded factor |
| UST-36 | 11 | Accepted individual PVs but overflowing exact dirty sum; Sum phase occurs after complete source traversal |
| UST-37 | 11,12 | All A–G dirty displays, native per100 units, policy-independent dirty, no six-place display in the kernel |
| UST-38 | 09,12 | Both cached AIs and policy on every report; clean uses published AI; sourceF raw-clean discriminator |
| UST-39 | 13 | Exact fractional-simple derivative formulas; stub Macaulay1/4 versus b*Modified41/162; alpha1 ordinary identity |
| UST-40 | 13,14 | Independently derived symbolic/dual-Decimal derivatives, positive gross terms and time moment; finite differences as corroboration only |
| UST-41 | 14 | Same-y delta_hat/q_hat coupling; exponent extrema, three-term alignment and one complete second-term RNE |
| UST-42 | 14,24 | 311-bit scalar and extended exponent carry; exact retained sums, no early raw IEEE rounding |
| UST-43 | 15 | Raw first-derivative negative infinity and negative-zero underflow from valid plans |
| UST-44 | 15 | Raw second-derivative positive infinity and positive-zero underflow from valid plans |
| UST-45 | 15 | Raw moment overflow with finite Macaulay; finite subnormal moment accepted; underflow excluded by principal proof |
| UST-46 | 14,15 | Raw derivative failure does not preempt Modified/Macaulay or nonzero subnormal Convexity |
| UST-47 | 15 | Convexity subnormal absolute error, positive-underflow error and no overflow; underflow also forces rawD2 underflow |
| UST-48 | 14,15 | Single-positive-PV cancellation, including MIN_SUB dirty with accurate normalized getters; no uniform price-relative claim |
| UST-49 | 13,15 | Macaulay/Modified and zero-dirty/opposite-sign exclusions proved; no forged reports to hit unreachable guards |
| UST-50 | 13,14 | Ideal all-six comparisons conditioned on actual magnitude/denominator; record every uninformative quantity separately |
| UST-51 | 16 | Dirty and clean native targets; published rather than raw AI; negative clean resolving positive and nonpositive resolved rejection |
| UST-52 | 17 | Lower-zero one-call success with invalid unqueried upper; no eager endpoint validation or auto bracket expansion |
| UST-53 | 17,23 | Lower/upper/interior callback failures with exact point, stage and counts including failure; budget checked before invocation |
| UST-54 | 17 | Budget1, exhausted iteration, no-sign-change, coordinate/precision limits; inherited M6 shared semantics explicitly mapped |
| UST-55 | 17 | Residual threshold2^970, near-equal/large/subnormal differences, scale fixed per target and no separately halved operands |
| UST-56 | 12,17 | Cached selected payload and no final repricing; report getters/cash factories consume zero new callbacks |
| UST-57 | 17 | n0 finite ideal image with alpha<1, observed-supremum overshoot, valid target roundtrip; no ideal-upper precheck |
| UST-58 | 17,25 | Target-driven independent inverses, exact one-payment anchors and plateau/conditioning qualifications; no uncounted shortcut |
| UST-59 | 18 | Source coupon cash under both bases/ties, every first-period shape and source daily/coupon publication distinction |
| UST-60 | 18 | Non-$1000 nominal explicit local scaling, zero coupon with invalid nominal, index before nominal error |
| UST-61 | 19 | Total AI5 before nominal; valid C=.000368, one elapsed day of184, N499600 gives.004996→cent0, not extra-scaled5dp→.01; published view Exact1e-6 versus nonterminating raw AI |
| UST-62 | 20 | Clean invoice RoundTotal versus RoundComponentsThenSum, including signed prices and nonnegative published AI |
| UST-63 | 21 | Dirty trade one quantizer, source_value None, no accrued shift or phantom coupon metadata |
| UST-64 | 22 | Decimal/binary/declared evidence; adjacent floats denote continuous interiors; equal endpoints preserve declared provenance |
| UST-65 | 22,23 | Point Exact Inexact versus nonpoint ExactStageUnresolved even when both endpoints are multiples |
| UST-66 | 20,22 | Capital Exact before accrued; wide rounded components outside i128 can cancel to representable final net |
| UST-67 | 22 | Point/lower/upper labels use stage input image; collapsed output does not erase source pointness; net collapse differs |
| UST-68 | 18–23 | All39 nominal/quantum/source scales, all7 cash modes, arbitrary positive quanta and both signed final limits |
| UST-69 | 22,24 | MAX/subnormal evidence, full-C/source widths, ambiguity extrema and correct price-ingredient guarantee labels |
| UST-70 | 02,18–23 | Inspector/Some/None matrix; instruction/receipt Eq/Hash retains provenance; non-determined decisions compare only diagnostic fields |
| UST-71 | 02,12,24 | Borrowed plan lifetime and owned reports after dropping plan/instrument; scoped-thread immutable reuse |
| UST-72 | 23 | Root error wrappers, source chains; complete phase/operation/index ledger checked against docs, without forcing excluded capacity failures |
| UST-73 | 24 | Fallible reserves and explicit caps, bounded arithmetic/temporary census; allocation failures classified without OOM injection |
| UST-74 | 25 | Mutants compile and fail intended witnesses; valid controls before/after; expectation/source byte guards |
| UST-75 | 26 | All existing Rust/fixtures preserved; full native debug/release, strict lint/fmt/docs and public/error catalog |
| UST-76 | 26 | Fresh production-region coverage, line-mapped exclusions, actual extracted package and compile-only foreign targets labeled accurately |

## Evidence ledger and mutation acceptance

For every family record artifact path/hash, expected domain, asserted
quantity/decision, comparison policy, actual profile/build and limitations.
Distinguish exact primitive, ideal accuracy, source display, shared-kernel
coverage, mathematical exclusion and allocation-only guards. Do not infer
family completion from a neighboring assertion or a passing aggregate count.

Mutants should target genuine alternative models: D versus I; recursive Q;
payment-on-S inclusion; missing long-first renumbering; raw versus selected
cash coupons; daily9/tie/total5 changes; raw AI clean shift; early y/2;
wrong power sequence; extended-PV double rounding; separately rounded
convexity summands; b*Modified Macaulay; eager raw getter; ideal-upper target
precheck; uncounted endpoint/repricing work; omitted AI source publication;
component/final range-order changes; and discrete-only declared intervals.
Each killed mutant needs a successful build, intended semantic witness and
restored positive control. Parse/compile failure is not a financial kill.
Pre-production controls are labeled separately from post-read supplements.

Coverage is an implementation-stage audit, not a design percentage. Review
uncovered production regions individually and add valid public/internal
domain cases where reachable. Keep capacity proofs, absent financial states,
and allocation failures explicit; never construct impossible reports to
inflate the percentage. Repeat broad checks only after final bytes or new
failures justify them.
