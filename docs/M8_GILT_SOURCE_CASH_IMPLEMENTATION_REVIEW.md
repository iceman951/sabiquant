# Conventional gilt source and cash implementation review

Status: scoped implementation review on 2026-10-20, Asia/Bangkok. **No confirmed
production defect was found in this review.** The reviewed implementation is
uncommitted work on `feat/m8-conventional-gilt`, after the frozen pre-production
oracle checkpoint `1bada34`. This report does not accept the full tranche,
financial model, owner API, foreign native platforms, release or full M8.

The authority is the complete [adopted contract](M8_GILT_CONTRACT.md), SHA-256
`03f0a7a96dd590bf3adb9fc2db26490866791efc6e2ed574ea6153dec595d292`.
This review covers source/date preparation and exact cash, including their
resource and provenance requirements. Price/risk/inversion arithmetic and
their separate ideal/primitive error qualifications are outside this scope.
No QuantLib implementation was read. No original source, existing test,
frozen generator or fixture was changed by this review.

## Reviewed bytes and method

| File | SHA-256 |
|---|---|
| `src/bonds/gilt.rs` | `ae35bd7a7d74203ebfbd8765026f65ff63216d8216f80e1c0af37dc73345c001` |
| `src/bonds/gilt/cash.rs` | `ec07451e86975910bddf7af171a69db7705714df72677399fac2a887702e3135` |
| Supporting `src/amounts/uint.rs` | `fa2b371da92f3c0aee949bd0c5b022f976f691b5dfeeff4b26c5b06234fcaaf9` |

The review read the full production paths of both gilt files against sections
S, A-C, E-F, H, J and C1-C8, and inspected the supporting checked fixed-width
arithmetic. Line references below identify these exact reviewed bytes.
The derived source/cash formulas were additionally exercised through public
APIs against Python `Fraction` calculations in a separate exact-copy crate.

The new retained [supplemental checker](../validation/generator/gilt_source_cash_supplemental_check.py)
was **authored after reading production**. It is independently calculated
supplemental evidence, not a blind or pre-production oracle. The already
frozen [source/cash corpus](M8_GILT_SOURCE_CASH_ORACLES.md) remains separate
and unchanged; this review does not claim to have executed its public consumer.

## Source/date findings

| Obligation | Reviewed location | Assessment |
|---|---|---|
| Deterministic definition failure order | `gilt.rs:378-500` | Cap/count/coupon/date/order/coverage/cycle/first-role/ex-window checks precede the single derived-vector reservation. Input vectors are retained without cloning. |
| Direct maturity-anchored semiannual dates | `gilt.rs:197-211`, `430-448` | Every expected date is derived from original maturity in one month operation. Preserve uses original maturity's month-end status. No recursively clamped recurrence or calendar enters. |
| Minimal coverage and all first roles | `gilt.rs:423-466` | `Q0 <= I < Q1` is enforced. Regular/Short/Long distinguish `I=Q0` from an interior issue; unsupported on-cycle Long is rejected. |
| Compatible inclusive ex windows | `gilt.rs:475-492` | `max(previous quasi, I)` equals the specified first Regular/Short/Long and later lower bounds. The upper bound is the actual dividend date. |
| Full-C first publication | `gilt.rs:161-190`, `501-514` | Decimal powers are combined before exact division. Comparison with `denominator-remainder` implements positive half-up ties without doubling a bounded remainder or admitting an i128 intermediate. Regular dividends retain exact `C/2`. |
| Seven AI branches | `gilt.rs:619-670` | Ordinary cum/ex, Short cum/ex, Long first interval cum, and Long second interval cum/ex match the raw-source formulas. The combined Long cum numerator is `r1*s2+r2*s1`; no published dividend is subtracted. |
| Skipped and actual boundary lifecycle | `gilt.rs:626-629`, `655-663`, `741` | At skipped Long Q1 the second Long formula has `r2=0`, retaining earned AI. At actual F, the strictly next index makes the lifecycle ordinary with `t=0`. Settlement at maturity is rejected. |
| Entitlement and stable lattice indices | `gilt.rs:738-805`, `880-891` | Lookup selects the strictly following Q. Entitlement checks Ceased first, then strict-after detachment. Detached/missing coupons are absent without compressed quasi indices. |
| Zero coupons and redemption | `gilt.rs:754-758`, `784-801` | Entitled actual zero coupons count and remain bundles. Final par100 is retained and grouped with an entitled dividend, or alone when detached. Source zero is not used to erase a factor observation. |
| Preparation resources | `gilt.rs:719-777` | Invalid cap and settlement domain precede count/cap, then exact cached AI, then one fallible bundle reservation. Requests count elements. Lookup/count/fill have the specified bounded complexity. |

No date underflow, multiplication or indexing defect was identified on valid
constructed definitions. Bounds follow from the validated cycle and global
date cap: half-year days are positive and at most184; Long AI products fit
i32; `L=r+j*s` stays below `2^22`. The unused final `l += s` also remains far
inside u32. The fallback conversions in direct date/period construction are
unreachable under the preceding public cap, rather than input panics.

## Cash findings

| Obligation | Reviewed location | Assessment |
|---|---|---|
| Required public traits and compact ownership | `cash.rs:70-80`, `128-245` | Instructions/decisions/receipts are fixed-size owned Copy/Eq/Hash values. Snapshots contain source/context scalars, no instrument clone or borrowed plan. Recipe variants prevent partial mixed shapes. |
| Debug representation | `cash.rs:82-125` | Debug reports recipe, public context and policies. Published source values use a descriptive label instead of dumping the private Wide buffer. Receipt/decision Debug reuses this representation. |
| Selector and nominal priority | `cash.rs:307-341`, `354-419`, `907-912` | Dividend selection is checked before nominal. Other factories require positive nominal before snapshot work. Zero source cash does not waive nominal validation. |
| Context and provenance | `cash.rs:422-478`, `502-707` | Issue pair, settlement pair and next actual ordinal come from validated/prepared values. Accrued snapshots retain the next actual dividend even when detached. Dirty recipes have no source dividend/ex fields or source value. |
| Explicit source views | `cash.rs:286-295`, `657-673` | Source views quantize the exact retained source once, with no nominal scaling, at the correct source stage/point. Dirty returns `Ok(None)`. Evaluation never calls these diagnostic views. |
| Pure exact-source arithmetic | `cash.rs:993-1023`, `1067-1120` | One combined decimal exponent forms the source/nominal/quantum ratio in eight limbs. There is no intervening DecimalAmount, f64 or enclosure. Published micro values retain their full width. |
| Exact price basis | `cash.rs:1025-1057`, `1123-1135`, `1199-1273` | Decimal and represented binary endpoints decode exactly on `R0*dayD`; source AI divides out its `2*dayD` exactly. No extra powers-of-five multiplier or re-encoding of rounded components appears. |
| Required stage order | `cash.rs:711-780` | Pure/dirty/total recipes use their selected stage. Component capital quantization completes before accrued quantization; both remain wide until terminal exact signed sum. No intermediate i128 range test can prevent valid cancellation. |
| Universal Exact | `cash.rs:785-789`, `1286-1295` | A continuous nonpoint required image returns ExactStageUnresolved before endpoint arithmetic. A point nonmultiple returns CashArithmetic/Inexact at point. Equal declared bounds remain points while retaining declared provenance. |
| Signed rounding and ties | `cash.rs:1279-1316` | All seven modes use exact quotient/remainder. Tie parity is the quantum count's parity. Signed directed rounding is correct; zero is canonicalized by signed magnitude. |
| Endpoint/range order | `cash.rs:774-779`, `825-835`, `846-881`, `1321-1354` | Single/total pointness is measured before quantization; net pointness is measured after component quantization. Final lower admission precedes upper. Canonicalization precedes signed i128 admission, including MIN. Only cash-net range errors use `sum`. |
| Ingredient guarantee | `cash.rs:874-894`, `916-924` | Pure source receipts have None. Declared provenance takes priority over binary/decimal labels, including a declared point. The retained original instruction/evidence supplies scoped price-ingredient provenance. |
| Resources and isolation | All production paths in `cash.rs` | Factories, views and evaluation use fixed arrays/scalars and bounded loops. No Vec/Box/heap reservation, date arithmetic, callback, I/O, global or interior mutation was found. Debug formatting has the contract's separate ordinary formatting qualification. |

The eight-/40-limb implementation follows the actual staging of the C7 proof.
Pure source count construction combines all powers before multiplication,
and its rounded coefficient stays below the eight-limb bound. Price endpoints
and AI share the single `R0*dayD` denominator. Capital/accrued rounded
coefficients are stored directly at quantum scale, and their net is added
without a gratuitous common-basis multiplication. The reviewed maximum
intermediates remain below405 and2460 bits respectively. Supporting Wide
addition/subtraction retain their carry/borrow; multiplication's nonnegative
row terms cannot cancel a discarded high contribution; division aligns and
shifts the divisor without doubling a remainder.

Capacity failures are therefore proof-excluded for the public finite domain.
They were not forced by creating invalid private states. Allocation failure
was not induced. The absence of cash-time heap calls is an implementation
inspection result, not a claim based on a modified global allocator.

## Executed supplemental evidence

Reproduce locally, with the repository's Rust1.99 toolchain on PATH:

```sh
python3 -I validation/generator/gilt_source_cash_supplemental_check.py \
  --output /private/tmp/sabiquant-gilt-source-cash-supplemental-new-run
```

The output directory must be new and outside the repository. The checker
snapshots every original Rust/support file and package/lint metadata, verifies
the copied bytes, adds only its generated integration test to the temporary
crate and runs offline debug, release and strict Clippy. It hashes the runner,
contract, generated inputs/test, original/copy manifests, toolchain and command
outputs, and rejects original/copied source mutations during execution.

The recorded run is
`/private/tmp/sabiquant-gilt-source-cash-supplemental-review-v1/evidence.json`,
SHA-256 `602b413dca8149ed4191b48c3a32b7050760e2636eb30e1e626142052c6eab55`.
Native debug and release each executed **one test checking all3,986 records**;
both passed with zero failed/ignored tests. Strict Clippy for the copied
library plus generated test exited0. Every original/copy library byte remained
unchanged. This test is supplemental, outside the standard Cargo test census.

| Operation | Records |
|---|---:|
| Pure accrued cash | 1,404 |
| Quantized accrued source view | 1,053 |
| Pure dividend cash | 469 |
| Quantized dividend source view | 354 |
| Dirty trade | 235 |
| Total invoice | 235 |
| Component invoice | 236 |
| Total | 3,986 |

The systematic part covers all seven source AI branches, skipped/actual first
boundary controls, all39 coupon scales and all seven rounding modes. Extremes
include zero/minimum/full coupon coefficients, minimum/maximum nominal and
arbitrary quantum coefficients at opposite scale extremes. The price part
adds signed exact decimals and represented binary64, both binary zero signs,
subnormal/normal/MAX operands, declared point/nonpoint intervals, all four
evidence kinds, Exact decisions, typed signed range errors and ingredient
guarantees. This finite selection is not the full Cartesian input space.

Ten final algebraic controls make particularly sensitive distinctions:

* Rows3976-3977: with `s1=182,r1=31,C=182e-6`, Short and Long first
  publications are respectively16e-6 and107e-6 from exact half-micro ties.
* Rows3978-3979: regular ex AI at1999-06-06 with `C=109382` is exactly
  -300.5. Clean300.5 and nominal i128::MAX cancel to exact zero under
  RoundTotal/Exact. Components fail at `cash capital`, point Inexact,
  before a possible cancelling accrued component.
* Row3980: exact capital `3*i128::MAX` stays wide and does not cause an early
  range error; the nondecimal AI instead fails Exact at `cash accrued`.
* Rows3981-3983: declared price[201,202], nominal=quantum=i128::MAX and
  nearest-even all map to the unrepresentable amount `2*i128::MAX`.
  Dirty/total report `cash total/lower/quantize`; components report
  `cash net/point/sum`, because the rounded net image has collapsed.
* Rows3984-3985: `C=1e-38` gives regular source dividend5e-39. Its Exact
  1e-38 source view fails; nominal200 makes final dividend cash exactly1e-38.

The generated inputs have SHA-256
`7247830e3df38253409295e83e862881b8cf3276ed760c067bdfd6e5de04fbe5`;
the generated Rust test has SHA-256
`664cd847b9f384a3d2b01e1761cd736a021210e5b90cbd320694133f98ef6269`;
the checker at that run has SHA-256
`2ddd68d633b0d9fba11979777b43dd9b76e11ca38a3ab5848b6fe09b7f253c7c`.

## Remaining acceptance work

There are no confirmed blocker/major/minor production findings to hand back
from this scope. That conclusion is limited to the reviewed bytes and the
finite evidence above; it is not a proof of defect absence.

Complete tranche acceptance still requires the frozen10,448-row public
source/cash consumer, public trait/error/source-chain and identity evidence,
the separate numerical/ideal/inverse ledgers, full unchanged generic regression,
coverage classification, package verification and required tooling checks.
This checker does not replace the400-year cycle oracle, a maintained UK
calendar, issued-instrument/record-date evidence, legal cash policy selection,
financial review or native Linux/Windows measurements. Source/cash correctness
alone does not accept a partial price/risk/yield implementation.
