# Treasury cash and source implementation review

Reviewed2026-10-21, local only. **No unresolved source/cash semantic or logical
storage blocker remains** after the parent source-scratch and publication-form
corrections below. This covers the complete four cash recipes, their source
views and compact identity, and the parent source/date preparation needed by
them. Numeric price/yield/risk review and final whole-tree verification remain
separate obligations.

The reviewer authored the new cash implementation and then performed this
read-only implementation/resource review; this is a self-review of that file,
complemented by the separately authored frozen hand suite, pre-production
corpora, independent oracle audit and public fixture consumer. It is not a
claim of independent cash-code authorship. The reviewer did not change any
production file during this review. Root made the three parent corrections.

The complete pre-production four-corpus gate finished before Treasury Rust,
with `complete_scope:true`; evidence SHA-256
`0b91013dfee5a065515126591d13663c69f53e997e936df7a779a26bb6d6420e`.
Frozen generators, corpora and original hand expectations were not changed.
The adopted [main](M8_US_TREASURY_CONTRACT.md) and
[cash](M8_US_TREASURY_CASH_CONTRACT.md) contracts remain normative.

## Reviewed source and semantics

| Input | Reviewed SHA-256 |
|---|---|
| `src/bonds/treasury.rs` | `4edbd3d4c5478546d4d18ec9ee2ad78707f40945d0859b1d84c853ddab38976e` |
| `src/bonds/treasury/cash.rs` | `47039a9d25b40fe6ac038cd7169faf9195520a88e6866acd615837123f40011d` |
| Shared `src/amounts/uint.rs` | `fa2b371da92f3c0aee949bd0c5b022f976f691b5dfeeff4b26c5b06234fcaaf9` |
| Shared `src/bonds/yield_risk.rs` | `85584994ffcff5e122663b26efcabf0334a2dc132591afd46475752464003d66` |
| Source/cash consumer | `97ecddea0b0196e897c909c15f51e6c6e9a3d4dd7491b456599f363b6b1dac37` |
| Consumer exact helper | `bdfa0e99243739bf46641a77beb4dbced3842d2f8fb19512b65eef05e8a23257` |

The parent pin was refreshed after root added a981-byte `///` usage-example
block on UsTreasury. Removing only that block reconstructs the earlier
reviewed parent SHA `7105a9da13b385f859fb79cca9d860da4ce1ae5852f716fba728f93b05bd589b`
exactly; every other byte, including all mathematical/control-flow code, is
identical. The reconstructed old file is not asserted to be a historical
pre-existing snapshot: its agreement with the independently recorded prior
hash establishes the exact comment-only delta. Evidence is
`/private/tmp/sabiquant-treasury-cash-review-doc-delta-v1.json`.

The cash path reads the prepared source formula/publication count directly.
Coupon cash is `R(N*X/(1000*10^dscale*s_part))`; accrued cash is
`R(N*published_count/10^8)`. Its helper fuses all decimal powers before
division, while requested source views omit nominal and use the per100
divisor. The raw coupon view independently quantizes the exact raw rational.
No cash ingredient passes through a public DecimalAmount or binary64 cache.
Regular coupons remain C/2 and source first coupons retain their selected
daily basis without a coupon-level publication stage.

Price endpoints are exact integers on `R0=2^1074*5^38`. Decimal decoding
multiplies by `R0/10^scale`; binary decoding retains the complete finite
significand/exponent, including subnormals and either zero sign. Total
invoices form `10*B_price+published_count*(R0/10^5)` and use the per1000
divisor regardless of whether accrued is zero. Dirty/capital stages use the
per100 divisor. All seven modes compare exact remainders; nearest modes
compare `r` with `d-r`, without doubling a remainder.

Component invoices finish the capital quantizer, including its universal
Exact check, before quantizing the nonnegative published accrued point.
Both components stay wide and are exact-summed at `cash net`. There is no
intermediate public range check and no second net quantizer. Canonical
trailing-zero removal precedes final signed-i128 admission, including the
asymmetric i128::MIN magnitude. Lower admission precedes upper admission.

Nonpoint Exact is a structural unresolved result over the continuous real
image, even when both endpoints happen to divide the quantum. Total/dirty
pointness is measured before their cash quantizer. The component net image
is measured after component quantization: a capital collapse can produce a
point net. This distinction also selects the final point/lower/upper error
label. Earlier Exact failures cannot be waived by later cancellation.

Factories check coupon ordinal before positive nominal for coupon cash;
other factories check positive nominal first. They copy cached d0/s0,
publication pieces/count, existing dates and containing pairs. They perform
no date arithmetic, calendar query, source publication, allocation or callback.
Dirty source views return None without running a quantizer.

Instruction identity retains source policy, canonical C, D/I/F/M, first role,
EOM, dated pair, relevant settlement/coupon metadata, source value, nominal,
rounding, evidence and aggregation. It omits unrelated Q values and caps.
Determined receipts retain the complete instruction. Diagnostic decisions
compare only their declared bounds or stage. Guarantees label only the
supplied price ingredient; source-only receipts have None. Custom Debug
prints public provenance without formatting wide source buffers.

Parent validation implements the ordered cap/length/date/first-role ledger,
direct maturity anchoring, D-based short/long source amounts, long I>Q1,
skipped-Q1 non-reset, actual-coupon reset and strict-future entitlement.
Bounds on dates, ratios and caps make internal indexing safe after validation.
No input-domain panic, unsafe block, hidden state or library I/O was found.

## Concrete review findings and corrections

1. **Five-word IEEE extraction scratch.** The original parent passed its
   five-word cache directly to the generic IEEE extractor. Counting the held
   plan caches, by-value arguments, scaled pair, division alignment/remainder/
   quotient and nested subtraction gave a conservative19 five-word values,
   above the16-temporary budget. Root now losslessly widens only extraction
   inputs to nine words; persistent source caches remain five words. This
   charges14 nine-word scratch values against the separate32-value budget,
   with at most8 five-word temporaries in that path.
2. **Raw creation lifetime.** Numerator and denominator creation locals for
   accrued raw AI originally shared the surrounding publication scope. Root
   enclosed them in an explicit block returning the raw rational. Their
   lifetimes now end before nested daily/publication division. The16-Scalar
   census below does not assume an optimizer discards unused Copy locals.
3. **ExactRatio publication form.** `raw_numerator*10^6/raw_denominator`
   was mathematically correct but retained an uncancelled factor2 and could
   construct a163-bit numerator, exceeding T9's162-bit publication-operand
   bound. Root now uses `5*Ccoef*weight*10^max(5-Cscale,0)` over
   `period*10^max(Cscale-5,0)`, exactly T9's combined-power form. The raw cache
   and every financial expectation are unchanged.

The width witness is a valid long first coupon: Q dates2024-08-15,
2025-02-15,2025-08-15; D=I2024-08-16; F=M2025-08-15; S2025-08-14;
C=i128::MAX at scale0. Its pieces183/184 and180/181 give weight66,243 and
period33,304. The old/new numerators have163/162 bits and denominators17/16
bits; their exact rational values agree. The independent Fraction witness is
`/private/tmp/sabiquant-treasury-source-publication-width-diagnostic-v1.json`,
SHA-256 `e7a79e3e0086665de1540a83d73183631af9c15b543bca8a75337b9e615bd4ed`.

## Logical temporary census

This counts numeric payloads across an active call chain, including explicit
by-value callee copies while caller values remain held. It includes a lower
rounded endpoint retained while computing the upper endpoint, component
buffers during net admission, and helper result/argument materialization.
It does not infer compiler stack layout, inlining, ABI copies or optimized
last-use storage reuse. Existing borrowed instrument caches, caller Q and
the already-owned bundle vector are input payloads, not newly created scalar
temporaries; copied fields and construction locals are counted.

Shared `Wide::div_rem` needs at most eight values at its deepest subtraction:
self and divisor2; aligned divisor, remainder and quotient3; nested
`checked_sub` arguments2 and result-word array1. Shift/zero/bit checks fit
below that peak. `checked_mul` has two by-value arguments and one result
array; small multiply/shift/division likewise include their input copy and
result array. These inner arrays are charged; they are not omitted merely
because Wide wraps an array.

| Active path | Count / allowance | Explanation |
|---|---:|---|
| Source/view quantum division, eight words | 11 / 12 | Caller numerator/denominator2, round-helper numerator copy1, complete nested division8 |
| Price upper-endpoint division, forty words | 14 / 16 | Borrowed image endpoints2, denominator1, completed lower1, local numerator1, round-helper numerator copy1, complete nested division8 |
| Price decode/invoice-unit conversion | <=16 / 16 | Both image copies when passed by value, source coefficient, retained lower, signed shift/add arguments and nested arithmetic arrays |
| Component net admission, forty words | <=16 / 16 | Capital pair, both accrued bindings, lower/upper, rounded argument/parameter copies, admission shadow/argument, small-division work and return allowance |
| Constructor NineDecimal first source, five words | 16 / 16 | Local raw-first pair2, source full amount1, daily base/numerator/denominator3, nearest arguments2, complete nested division8 |
| Second long-accrual daily stage, five words | 16 / 16 | Raw pair2, first daily piece1, daily base/numerator/denominator3, nearest arguments2, complete nested division8 |
| Final accrued publication, five words | 14 / 16 | Raw pair2, publication pair2, nearest arguments2, complete nested division8; branch-only creation locals have ended |
| Source IEEE extraction, five words | 8 / 16 | Held plan accrued cache3 and published pair2, copied source pair2, active resize input copy1 |
| Source IEEE extraction, nine words | 14 / 32 | Widened pair2, extractor arguments2, scaled pair2, complete nested division8 |

The cash-only wide allowances are5,120+768=5,888 bytes of logical payload,
with bounded small metadata and compact source snapshots. Parent cache work
uses the separately stated main-contract scalar/endpoint allowances. These
paths do not reserve a price/yield/risk workspace. Actual compiler stack
frames, allocator success and platform stack limits are not certified here.

All source snapshots remain320-bit values; source/view/cash arithmetic uses
512 bits; price and component sums use2,560 bits. The normative fused forms
fit the adopted source/view/price operand bounds. The397-bit denominator
control is a valid admitted coefficient/scale extreme, not a fabricated
private financial state. The review makes no claim that every conservative
bound is economically attainable.

## Actual checks and observable limits

The reviewer ran the full public source/cash consumer:3 tests passed,
30,572 composite rows,93.14 seconds. Log:
`/private/tmp/sabiquant-treasury-cash-review-source-consumer-v1.log`.
Its binary predates the three resource/form corrections; root owns the final
whole-tree rerun on the corrected bytes. After the final publication-form
correction, the reviewer reran the three private cash boundary tests and all
38 separately frozen source/cash hand tests: both passed, logs
`…-cash-review-final-units-v1.log` and `…-cash-review-final-hands-v1.log`.
These are3+38 Rust functions, not30,613 functions. Earlier development
checks also passed the3 cash units and38 hand functions in debug/release,
with focused warning-denied Clippy; they are historical, not a substitute
for root's final all-target checks.

The consumer binds every fixture output, but five rational fields are
observed through the public API. Raw/source first coupon and instruction
source values use Exact/nearest-even projections at unit and1e-38 quanta;
raw AI uses exact IEEE cache bits and civil pieces, while published AI also
has source-view projections. The private rational text is not directly read
back from the library. Inexact/out-of-range projection results can coincide
for distinct private rationals, so this is public-observable coverage, not
856,016 direct exact private-rational comparisons. The independent Python
oracle review's856,016 exact comparisons are a separate pre-production
result. Fine private rational equality and defensive capacity exclusions
remain arithmetic-proof/internal-test obligations.

## Semantic mutation targets

These are concrete review targets, **not a claim that mutants were executed**.
Run them only in isolated copies with the ordinary frozen expectations.

| Mutation | Existing discriminating hand or corpus control |
|---|---|
| Omit invoice price-times10 when AI0, or select divisor from addend truthiness | All7,560 ZERO-AI-PRICE rows; invoice zero-accrual unit test |
| Rebuild a regular coupon as daily9 times days | `regular_full_coupon_does_not_reconstruct_from_daily_nine` |
| Use raw rather than selected source coupon in a view/cash recipe | `short_daily_tie_views_and_instruction_views_are_identical_for_every_quantizer`; source-policy grid |
| Publish each long accrued piece before summing | `long_accrued_publication_is_total_first_under_all_four_policies` |
| Apply another AI5 publication after nominal scaling | `fractional_nominal_discriminates_an_extra_accrued_quantizer` |
| Treat nonpoint Exact as two endpoint divisibility tests | `nonpoint_exact_is_unresolved_even_when_endpoints_are_quantum_multiples` |
| Admit rounded capital before net cancellation | `wide_components_cancel_without_an_intermediate_decimal_or_range_gate` |
| Check accrued Exact before capital Exact | `component_capital_exact_failure_precedes_an_otherwise_zero_net`; CAPITAL-BEFORE-ACCRUED rows |
| Use rounded-output equality for total-stage error pointness | `component_net_pointness_differs_from_nonpoint_total_after_rounding_collapse` |
| Check upper public range before lower | `interval_final_range_admits_lower_before_upper` |
| Round negative nearest-away ties toward positive infinity | `every_cash_mode_handles_signed_quarter_ties_and_point_exact_failures` |
| Reject i128::MIN using a symmetric positive limit | `exact_dirty_cash_admits_both_i128_limits_and_rejects_wider_final_amounts` |
| Use issue rather than dated date or reset at skipped Q1 | `long_issue_after_skipped_date_preserves_dated_metadata_and_actual_reset` |
| Omit policy/evidence provenance from instruction identity or guarantees | `determined_source_identity_retains_policy_and_original_issue_date`; `four_equal_price_evidence_kinds_keep_distinct_instructions_and_receipts` |

Final native all-suite execution, semantic mutant results, coverage, package,
logical review of numeric price/yield/risk and foreign-target qualifications
belong in the final validation record. This review does not close financial,
auction/calendar, analytical-enclosure, native foreign-platform or all-M8
acceptance gates.
