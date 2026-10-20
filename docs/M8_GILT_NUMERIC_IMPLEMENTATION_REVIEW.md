# Conventional gilt numerical implementation review

Review date: 2026-10-20, Asia/Bangkok. Scope: the initial conventional-gilt
numeric implementation against the frozen [contract](M8_GILT_CONTRACT.md),
especially FR-GILT-07..14/20..23. This is a source review and a small additive
public boundary suite, not implementation, financial or all-platform acceptance.
No production arithmetic, frozen fixture or generator was changed by this
reviewer. Root subsequently authorized the new test file described below.

No blocking or high-severity numerical defect was found in the reviewed
snapshot. The mathematical staging, retained risk arithmetic and inverse
payload use agree with the adopted contract. Independent corpus execution,
full regression and the remaining acceptance gates remain separate obligations.

| Reviewed artifact | SHA-256 |
|---|---|
| Contract | `03f0a7a96dd590bf3adb9fc2db26490866791efc6e2ed574ea6153dec595d292` |
| `src/bonds/gilt/numeric.rs` | `1c53413faa374a4048391b89333ab6032b54394816d5de83d412fc6eafbe148d` |
| `src/bonds/gilt.rs` | `ae35bd7a7d74203ebfbd8765026f65ff63216d8216f80e1c0af37dc73345c001` |
| `src/bonds/yield_risk.rs` | `85584994ffcff5e122663b26efcabf0334a2dc132591afd46475752464003d66` |
| `src/amounts/uint.rs` | `fa2b371da92f3c0aee949bd0c5b022f976f691b5dfeeff4b26c5b06234fcaaf9` |
| `src/math/solver.rs` | `182907c97c0ea617560289187b2a93ac08414ec6e7bdf4fc328abe2b6007a3c2` |

## Findings and disposition

| ID / severity | Concrete finding, witness and disposition |
|---|---|
| GNR-01 / medium evidence gap, public controls added | The original hand suite exercised the zero-factor midpoint, but did not independently witness both sides of the overflow guard or the signed tiny-branch transitions. `guard` is at [numeric.rs:282](../src/bonds/gilt/numeric.rs#L282), its caller at [306](../src/bonds/gilt/numeric.rs#L306). A new [public suite](../tests/bonds_gilt_elementary_boundaries.rs) uses a valid 2,000-period zero-coupon instrument, target-selected H/Z yields and outer-side neighbors. All three tests pass in native debug/release. This closes the concrete public-path gap; exact internal band-edge ownership is a narrower primitive check still recommended below. |
| GNR-02 / medium acceptance dependency | The one-round source PV, extended risk rounding and direct normalization need the independent 75,849-row private-primitive adapter, not only comparisons with ideal prices. [The frozen corpus](M8_GILT_NUMERIC_ORACLES.md) supplies the correct E0 expectations. Its execution was assigned separately while this review ran; this document does not claim its completion. The 10,448-row source/cash and 374+51-row ideal/inverse consumers are also separate evidence. Root must use their actual final results. |
| GNR-03 / qualification retained | Native runtime tests cannot establish an unconditional transcendental accuracy or range-classification theorem. [numeric.rs:311](../src/bonds/gilt/numeric.rs#L311) calls native ln/ln_1p and [330](../src/bonds/gilt/numeric.rs#L330) calls exp outside the explicit uncertainty band. Source comments and public API documentation retain this qualification. No defect or policy change is requested. |

No finding authorizes changing the frozen contract or weakening its oracles.
An uninformative ideal-comparison row remains uninformative even when runtime
returns an acceptable typed range failure. It must be counted separately.

## Arithmetic and error-order audit

| Obligation | Reviewed implementation and conclusion |
|---|---|
| Exact source coefficients | [gilt.rs:116](../src/bonds/gilt.rs#L116) constructs the signed exact source rational without a public decimal or binary observation. [97](../src/bonds/gilt.rs#L97) groups par100 into the same denominator before pricing. The numerical module does not reconstruct coefficients from published float views. |
| One IEEE source PV round | [numeric.rs:339](../src/bonds/gilt/numeric.rs#L339) multiplies the exact source numerator by the accepted factor's significand, retaining its power separately, and directly invokes the one-IEEE converter. It never first rounds A to f64 or rounds an extended53 result before IEEE admission. Zero source returns +0 only after `observe` has validated the factor at [363](../src/bonds/gilt/numeric.rs#L363). Nonzero-to-zero and infinity preserve Value/absolute-index/source-PV context. |
| Direct rational IEEE conversion | [yield_risk.rs:1398](../src/bonds/yield_risk.rs#L1398) obtains the exact binade by aligned integer comparison, makes mathematical early zero/infinity decisions, clamps the output unit to MIN_SUB, and uses quotient/remainder with complement/parity for one RN. It handles normal carry, subnormal crossover and overflow midpoint without an intervening extended53 round. This helper's arithmetic body was not changed for gilt. |
| Scaled logarithm and exponent | [numeric.rs:306](../src/bonds/gilt/numeric.rs#L306) preserves explicit zero and tiny identities, the y<=-1 ln branch and the other ln_1p branch. [263](../src/bonds/gilt/numeric.rs#L263) rounds represented -g*L/(2*s) directly, without rounding lambda or a g*L product. A rounded zero exponent is canonicalized to +0 and yields factor1. |
| Outward uncertainty guard | [numeric.rs:282](../src/bonds/gilt/numeric.rs#L282) uses the specified constants, exact power-of-two scale, two outward next_up additions and downward distance bound. H is tested before Z. Tiny/zero bypasses follow the contract's range proof. Outside the guard exactly one exp call occurs and the existing typed validator preserves its actual result. |
| Extended term rounding | [numeric.rs:374](../src/bonds/gilt/numeric.rs#L374) performs one normalized53 rounding with exact remainder/complement parity and renormalizes the 53-bit carry. [477](../src/bonds/gilt/numeric.rs#L477) forms first, second and moment multipliers with exact L, L+s and s; delta is the admitted RN(2+y), not the ideal real sum. No raw IEEE admission occurs at this stage. |
| Fixed-sign retained sums | [numeric.rs:447](../src/bonds/gilt/numeric.rs#L447) shifts each rounded extended term onto the exact 2^-8192 basis and adds to a 243-word unsigned magnitude. All first contributions are negative and other contributions positive, so separate sign-pair subtraction is unnecessary and is expressly permitted by N6. No financial cancellation model is fabricated. |
| Independent getter admission | [numeric.rs:537](../src/bonds/gilt/numeric.rs#L537) admits a raw magnitude once and preserves signed infinity/zero errors through `finish_risk`. [546](../src/bonds/gilt/numeric.rs#L546) divides the retained magnitude directly by represented dirty before its own IEEE admission; it never calls a raw getter. Thus raw overflow/underflow cannot erase a valid normalized result. The defensive exact-zero branches are not claimed reachable in a successfully admitted positive-par plan. |
| Price/risk order and zero coupons | [numeric.rs:1038](../src/bonds/gilt/numeric.rs#L1038) and [1056](../src/bonds/gilt/numeric.rs#L1056) validate quote before reservation. Traversal is original quasi-index factor -> source PV -> requested extended terms. Dirty exact reduction follows the traversal, before a report escapes. Source-zero actual coupons retain their factor observation. Raw risk admission is deferred to each getter. |
| Target resolution and preparation | [numeric.rs:1104](../src/bonds/gilt/numeric.rs#L1104) shifts a clean target once with cached AI, canonicalizes zero, fixes scale at the exact 2^970 bit threshold, rejects nonpositive dirty targets with zero counts, then reserves one reusable PV vector. It performs no face conversion or sensitivity sort. |
| Exact residual and payload | [numeric.rs:581](../src/bonds/gilt/numeric.rs#L581) subtracts represented operands exactly in the MIN_SUB basis; [611](../src/bonds/gilt/numeric.rs#L611) adjusts the output exponent before the single RN. [1136](../src/bonds/gilt/numeric.rs#L1136) passes (residual,dirty) together through the frozen payload Brent routine. The selected cached dirty is returned with `solve_report.point()`; no final repricing or last-callback latch appears. |
| Counted lower-first callbacks | [solver.rs:877](../src/math/solver.rs#L877) evaluates lower and returns immediately on computed zero. Upper is queried only afterward. [971](../src/math/solver.rs#L971) checks cap before invocation, increments counts including failure, canonicalizes the attempted coordinate and retains the immediate typed cause. Endpoint/history payloads stay paired with their points. Gilt introduces no alternate solver path. |
| Cached views | [numeric.rs:629](../src/bonds/gilt/numeric.rs#L629) independently admits the unscaled cached difference; [645](../src/bonds/gilt/numeric.rs#L645) uses one exact dirty-minus-AI reduction. Result getters do no source, calendar, factor, allocation or solver work. |

## Width, storage and lifecycle review

The selected widths match N5/N6/N8: five-word scalar, 34-word residual and
243-word risk magnitude. Source PV and exponent arithmetic use the same
checked direct-IEEE helper, whose aligned shifts occur only after early
mathematical zero/infinity classification. The adopted 202-bit source-PV,
76-bit exponent and 176-bit risk-scalar working bounds fit the 320-bit scalar.
The source rational's numerator/denominator are not cast down to public i128.

The following is a census of explicit logical numeric storage, including
ordinary Copy arguments where their original value is still needed. A value
whose last use transfers it to the next calculation need not be counted again
as a simultaneously live original. No assumption about actual optimizer stack
slot reuse follows from this accounting.

| Path | Census and budget |
|---|---|
| Risk construction | Three retained magnitudes plus one shifted term. Nested `Wide::checked_add` has receiver/argument copies, output words and up to two owning limb iterators. Even counting a return/destination separately, the path is below12 risk-width slots. There are no simultaneous positive/negative pairs. |
| Risk getter / converter / division | A deliberately conservative count of20 includes the three fields of the existing risk object, getter argument, converter numerator/denominator and aligned operands, division arguments, aligned divisor/remainder/quotient, nested subtraction words/Copy arguments/iterators and a two-value return transfer. The initial-binade comparison and the later quotient phase are sequential, not nested. This is below the24-slot adopted budget (46,656 bytes). |
| Residual difference | Two decoded magnitudes; at a signed add/subtract, receiver/argument, result words and owning limb iteration fit within eight residual-width slots. Sign selection and exact-zero branches create no extra magnitude collection. |
| Residual IEEE admission | With last-use transfers, the converter preserves its aligned denominator for the post-division complement. At the deepest division subtraction it also has the loop's aligned divisor and quotient, a remainder receiver, a copied aligned-divisor argument and output words: six live magnitudes. Receiver limb iterators replace their consumed receiver arrays; reserving two transfer slots gives eight (2,176 bytes). The source magnitude and pre-alignment operands have no later use. |
| Scalars | Source PV may retain two source-rational fields while entering the converter; the converter/division needs at most eight live scalar magnitudes by the same last-use reasoning. Coefficient construction, binade alignment and risk multipliers are sequential. Fewer than16 scalar slots (640 bytes) suffice. Extended terms contain only a u64 significand and i32 exponent. |

The worst conservative aggregate uses the contract's complete separate
budgets, 46,656+2,176+640=49,472 bytes, below50KiB before small metadata. The
hot paths do not simultaneously need all three budgets. This is not a bound
on compiler-generated stack frames, caller thread stacks, standard Clone or
Debug formatting. GiltRisk's custom Debug omits the retained arrays.

`Wide` methods are fixed-width and nonrecursive. Checked subtraction and
left shift preserve all carries/borrows; division at
[uint.rs:189](../src/amounts/uint.rs#L189) has at most64*N iterations and never
doubles a remainder. The numeric module performs no I/O, global mutation,
calendar lookup, recursion or unsafe code. `reserve` requests the retained
bundle count in f64 slots once; callback reuse clears and refills that already
reserved vector. No callback can push more elements than the initial request.
This is a structural no-per-callback-allocation review, not an allocator/OOM
stress experiment.

## New public boundary evidence

[The additive suite](../tests/bonds_gilt_elementary_boundaries.rs) contains three
tests. It constructs the complete regular 2000-01-01..3000-01-01 cycle with
C=0 and settlement at issue. Every entitled zero dividend remains a bundle;
the final grouped source coefficient is100. Thus final lambda=2000 and the
other factors are separated from the final boundary by at least0.35 in z.

The two target-selected quotes are:

| Boundary | Independently selected formula | Yield bits |
|---|---|---|
| H | RN(2*(exp(-ln(2^1024-2^970)/2000)-1)) | `bfe31eb9e43a71b5` |
| Z | RN(2*(exp(1075*ln(2)/2000)-1)) | `3fece4a42b68a4cb` |

Native evaluation and risk both return the exact final-index Rate /
NumericalBoundary context at these quotes. Moving128 yield ULPs to either
side produces four distinct outcomes: factor PositiveInfinity, accepted
finite factor followed by source-PV PositiveInfinity, factor +0 with the
strict positive domain, and accepted MIN_SUB factor with dirty100 MIN_SUB.
This checks that the guard neither guesses at its midpoint nor replaces
ordinary far-range outcomes with a blanket numerical-boundary result.

The other test covers both signs of next_down(2^-52), equality and
next_up(2^-52). Prices from both evaluate and risk must belong directly to
closed outward intervals around independently computed ideal prices. The
absolute2^-43 budget is derived from the contract's conditional elementary
profile and the one source-PV rounding; it is not an arbitrary ULP allowance.
The quotes and interval bits agree independently at160/240 Decimal digits.

Reference artifact:
`/private/tmp/sabiquant-gilt-elementary-boundaries-reference-v1.json`, SHA-256
`25359babedab833aab70cc87cffc34ecb33f536e726119fed4514750fa3d2e10`.
The test source hash is
`b64df7db1acb9c05f956c123ad11507779ba8afe3a4f9cc72da64e52144eb718`.
Focused native debug and release each passed3 tests; focused strict Clippy
and rustfmt passed. Logs use the prefix
`/private/tmp/sabiquant-gilt-elementary-boundaries-` with
`debug.log`, `release.log`, `clippy.log` and `fmt.log` suffixes. A first
Clippy run rejected constant-false failure assertions; replacing them with
explicitly expected test-only panic branches changed no mathematical witness
or production code.

## Remaining checks and qualification

Exact guard primitive controls should call the production `guard` with the
threshold bit patterns and adjacent values at both sides of each calculated
band edge, checking rejection at equality and admission immediately outside.
The independently checked10,913-exponent drafting evidence supports the
contract algorithm, whereas the new public suite witnesses this runtime path.
Neither alone claims every internal guard-cell boundary was exercised.

The private primitive corpus must independently establish exact ties,
subnormal double-round discriminators, extended carry, full risk-basis
certificates, direct normalization after raw range failure and residual
sign/scale staging. Primitive admitted factors and PVs are mathematical
inputs, not assertions that a public yield/libm observation realizes them.
Public controls must continue to establish genuine source-domain range/error
reachability. No fake zero/negative dirty state, capacity failure or OOM test
is requested to close proof-excluded branches.

Full native regression, package extraction, warning-denied documentation,
coverage classification, exact old-artifact preservation and final source
hash review remain root responsibilities. Linux/Windows compilation does
not constitute native foreign-platform execution. Conditional primitive
accuracy, ideal-model comparisons, owner/financial/calendar/platform/release
approval and broader M8 scope remain separate gates.
