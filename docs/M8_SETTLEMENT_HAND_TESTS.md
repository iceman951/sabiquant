# Local settlement hand-test evidence

| Field | Value |
|---|---|
| Date | 2026-10-17, Asia/Bangkok |
| Scope | Independent public hand/adversarial tests and shared root/API integration |
| Contract | [Frozen settlement contract](M8_SETTLEMENT_CONTRACT.md), FR-SETTLE-01..16 |
| Suite | [tests/settlement_hand.rs](../tests/settlement_hand.rs), **27 tests** |
| Actual focused result | Debug: 96 unit + 8 public-API + 27 hand tests; release: 95 + 8 + 27; all passed |
| Acceptance | Local engineering evidence only; no financial, named-market, owner, platform or release acceptance |

The independent hand author read the frozen contract, requirement rows and
existing public amount APIs before writing expected settlement results. The
settlement implementation and its private arithmetic were not read. The author
later integrated the public module, root error wrapper/catalog and public trait
assertions, without changing settlement arithmetic. The independently authored
production implementation passed every hand control on its first actual run.
No expected value was adjusted to match production output.

Settlement production remained unread through the final focused debug/release
and strict Clippy runs recorded below. A later, separately requested
[implementation review](M8_SETTLEMENT_IMPLEMENTATION_REVIEW.md) first reads
that production after this independent hand evidence was complete.

## Independent derivation

All comparisons use exact `DecimalAmount` equality, IEEE binary64 bits, typed
context or complete decision equality (**TOL-E0**). Expected values come from
signed integer arithmetic and explicit rational identities. Construction of an
expected decimal uses only the existing canonical decimal constructor/parser;
no settlement evaluator, amount quantizer, binary-to-decimal conversion or
generated settlement fixture supplies expected values.

Representative discriminators are:

- Nominal 1000 and dirty per100 101.25 yield 1012.5. Coupon per1000 12.5
  yields 12.5. Negative values reverse the contract's specified parties without
  changing nominal's positive-par requirement.
- At q=.05, .125 is 2.5 quantum counts; nearest ties-even yields .10 and
  ties-away yields .15. Signed directed modes operate on the signed quantity.
- Nominal 100, clean .5 and accrued .5 at q1 yield 1 under RoundTotal;
  component ties-even yields 0 and component ties-away yields 2. Component
  Exact fails at capital before any possible net rescue.
- Nominal 1,000,000, clean 100.0000005 and accrued per1000 .000005 yield
  1,000,000 under source ties-even and 1,000,000.02 under source ties-away.
  The corresponding unrounded per100 combination yields 1,000,000.01.
  A separate coupon control retains the unrounded .005 cash.
- Supplied binary64 .1 is exactly `3602879701896397 / 2^55`, above decimal
  .1. At q1e-18 it yields .100000000000000006; decimal .1 remains .1.
  Exact q.1 accepts only the decimal control. Positive/negative least
  subnormals round to canonical zero toward zero and to +/-1e-38 away.
- A source price `2^130`, outside public decimal range, with nominal 1e-38
  gives `2^130 / 10^40 = .1361129467683753853853498429727072845824`;
  cent ties-even yields .14 in both direct and staged recipes.
- With M=i128::MAX, `M^2 mod 100 = 29`. Capital `M^2/100` and its negative
  are individually far outside public range. Total ceiling gives 0, whereas
  separately rounding both signed components and exact-summing gives 1.
- Staged price `2^1020` and accrued per1000 `-10*2^1020` cancel using the
  recipe's actual 10+1 common-basis weights. Both source values are finite
  binary64 values outside public decimal range. The exact IEEE patterns and
  cancellation identity were separately checked with Python integer/Fraction
  arithmetic. Maximum finite binary64 values also cancel on the direct route.
- Discrete source counts 0,1,2 scaled by nominal 50 give cash inputs
  0,.5e-6,1e-6. At cash q1e-6 Exact, the two endpoints are multiples but the
  interior is undefined. Nominal 100 instead gives an Exact-defined discrete
  image. A continuous declared interval remains unresolved, including an
  interval between adjacent binary64 values.
- At positive i128::MAX+.5, ties-even/away exceed range. The negative tie
  `-(MAX+.5)` rounds to i128::MIN and is valid. Exact still reports Inexact
  before final admission. Both signed limits also have controls at every scale.
- At q1e-38, endpoint counts `10*C` and `10*(C+1)` for
  `C=17014118346046923173168730371588410573` normalize into public range;
  interior count `10*C+1` exceeds i128 and cannot normalize. The API returns
  ambiguous representable extremes without asserting interior representability.

## Complete small staged images

One of the 27 tests evaluates **432** distinct complete small image controls:
six non-Exact price modes, six non-Exact accrued modes, six non-Exact cash
modes and both aggregations. In source-quantum units the declared intervals
are [-1.5,2.5] and [-2.5,1.5]. Independent quarter-grid probes include each
integer/half-integer threshold, every tie and a point in each adjacent open
cell. Since these are all transition locations of the adopted modes, the
resulting integer sets exhaust these particular continuous source images.

Nominal 150 makes either source-count step 1.5e-6. Cash q2e-6 therefore has
quantum-count ratio 3/4. The test enumerates every pair in the independently
derived finite source images and computes cash counts using a small signed
integer oracle. Its truncating quotient, remainder and count-parity rules are
separate from production. All intermediate oracle integers have magnitude
below 100, so its doubled-remainder comparison is bounded. The complete set's
minimum/maximum is compared to the public decision; a singleton would require
a determined receipt with retained declared-enclosure provenance.

This is a complete enumeration for the stated small source images, not a
proof for every finite input. Separate algebraic controls test Exact continuous,
point and discrete image rules, independently varying inputs, collapsed source
images and per-component stage ordering.

## Test inventory and requirement traceability

The following are the actual 27 Rust test functions. Multiple assertions or
enumerated profiles within a function are not counted as extra Rust tests.

| Test function | Main evidence | FR-SETTLE |
|---|---|---|
| `registries_round_trip_and_reject_case_trimming_and_aliases` | Complete aggregation/guarantee registries; exact unknown text/concept | 04,12 |
| `evidence_bounds_identity_and_signed_zero_are_canonical` | Four evidence kinds, bits, signed zeros, equal declared endpoints, canonical Eq/Hash | 02,03,12 |
| `nonfinite_endpoint_order_is_lower_then_upper` | Infinity/NaN sign/payload classes, finite-before-order, lower/upper labels and reversed intervals | 03,14 |
| `all_four_recipe_inspectors_retain_every_explicit_policy` | All units/family IDs, absence, aggregation, fixed source quanta and caller modes | 01,04,05,12 |
| `every_factory_rejects_zero_and_negative_nominal_without_evaluating_prices` | All four positive-par guards; nominal minimum/maximum zero-cash controls | 01,14 |
| `units_and_signed_party_directions_follow_the_four_formulas` | Positive, negative and zero trade/coupon amounts with distinct per100/per1000 roles | 01,05 |
| `all_seven_modes_apply_to_signed_arbitrary_quantum_counts` | Seven modes, both signs, non-power-of-ten quantum/count parity and point Exact errors | 04,05 |
| `all_thirty_nine_scales_have_exact_quantum_controls` | 39 scales x 7 modes x 2 signs = 546 exact-multiple controls | 04,13 |
| `aggregation_and_directed_rounding_have_distinct_exact_results` | Total/component double-rounding differences and signed cancellation | 06,07 |
| `fixed_source_stages_are_observable_before_cash_rounding` | Six/five-place source stages versus direct arithmetic; unrounded coupon; signed source direction | 05,06 |
| `supplied_binary64_is_the_exact_represented_real` | Exact decimal/binary .1 distinction, least subnormals, explicit guarantee | 02,05,12 |
| `huge_source_and_cash_components_can_be_rescued_only_by_the_final_net` | Source-range rescue, MAX-square wide component cancellation and staged 10+1 weights | 06,13 |
| `exact_source_price_then_source_accrued_then_cash_components` | Complete mandatory Exact order; no component net rescue | 07,10 |
| `no_future_range_or_exact_prevalidation_changes_required_stage_order` | Earlier Exact work precedes later errors; wide prior results receive no early admission | 07,10,14 |
| `continuous_exact_images_are_unresolved_but_declared_points_are_point_errors` | All-real/adjacent-float intervals versus equal declarations and stable point labels | 09,10 |
| `discrete_exact_images_detect_undefined_interior_counts_and_integral_steps` | Undefined interior despite Exact extrema; integral steps; collapsed-source point errors | 09 |
| `independent_images_cannot_invent_cancellation_or_net_rescue_exact_components` | Rectangular independence and separate component increment checks | 08,09 |
| `intermediate_ambiguity_collapses_only_at_the_required_final_stage` | Wide intermediate images collapse to unique cash; distinct extrema stay ambiguous | 08,11,12 |
| `complete_small_staged_images_match_all_signed_source_and_cash_modes` | Complete source-cell/endpoint set checks in 432 small recipes | 08,09,16 |
| `final_admission_preserves_pointness_before_cash_rounding` | Point/declared point/lower labels, source collapse and cash collapse; coupon range context | 10,14 |
| `cash_net_pointness_and_upper_admission_have_distinct_context` | Lower-before-upper admission, exact cash-net pointness and quantize versus sum causes | 10,11,14 |
| `quantum_ambiguity_does_not_certify_interior_decimal_representability` | Valid extreme amounts with a known interior precision hole | 11 |
| `maximum_finite_binary_sources_cancel_without_an_early_public_range_check` | Maximum finite signed sources, all modes/aggregations, ordinary final range failures | 06,13 |
| `signed_final_limits_and_half_quantum_admission_use_the_exact_result` | Both signed limits at all 39 scales, asymmetric ties and quantum MAX | 04,06,13 |
| `receipt_identity_retains_evidence_family_and_every_policy` | Complete definition identity, canonical equal definitions, receipt retention and guarantee precedence | 02,12 |
| `reachable_error_catalog_and_typed_root_sources_are_complete` | Five reachable error kinds; fields, root/domain/AmountError source chains | 14 |
| `all_types_have_required_traits_and_concurrent_reuse_is_deterministic` | Seven Copy/Eq/Hash values, error traits and eight immutable reuse threads | 15 |

The concurrent test performs eight threads x sixteen repetitions x four
instructions = 512 threaded decision/error comparisons, plus checks before
and after. It includes determined supplied-binary and declared-enclosure
receipts, an ambiguous discrete image and an Exact component error.

## Integration and actual focused execution

[src/lib.rs](../src/lib.rs) exposes the module and describes its explicit
generic scope. [src/error.rs](../src/error.rs) adds transparent
`Error::Settlement` kind/Display/source/From integration. Its exhaustive
catalog now samples **131** domain variants, including all six settlement
variants, and its unique-kind, nested-source and failing-writer formatting
checks pass. The InternalCapacity catalog entry constructs an error value
only: it does not execute a capacity failure or forge a private financial
state. Public hand inputs reach the other five kinds.

[tests/public_api.rs](../tests/public_api.rs) contains **97 unique**
compile-time value/thread assertions, with all seven new non-error types
checked for Copy and Eq/Hash, plus settlement error traits and root conversion.
The declaration-to-assertion census and full policy/package checks remain part
of the tranche's complete validation, rather than being inferred from this
focused compilation alone.

Actual commands used Rust/Cargo 1.99 from
`/private/tmp/sabiquant-rustup/toolchains/1.99.0-aarch64-apple-darwin/bin`,
with a separate target directory for each simultaneous run:

```sh
cargo test --offline --locked --test settlement_hand --test public_api --lib
cargo test --offline --locked --release --test settlement_hand --test public_api --lib
cargo clippy --offline --locked --lib --test settlement_hand --test public_api -- -D warnings
rustfmt --edition 2024 --check --config skip_children=true src/lib.rs src/error.rs tests/public_api.rs tests/settlement_hand.rs
git diff --check -- src/lib.rs src/error.rs tests/public_api.rs
```

| Focused gate | Actual result |
|---|---|
| Debug | 96 unit + 8 public-API + 27 hand; no failure, ignored or filtered test |
| Release | 95 unit + 8 public-API + 27 hand; no failure, ignored or filtered test |
| Strict focused Clippy | Exit0, no warning |
| Owned Rust formatting and tracked diff whitespace | Exit0 |

The inherited debug-only interval-invariant test accounts for the one-test
debug/release difference. Native execution was macOS arm64. These commands
do not execute doctests, other integration binaries, Linux or Windows.
The first strict Clippy invocation found only the large Decision enum layout:
the contract requires an inline Copy receipt and allocation-free arithmetic.
Root added a local documented lint expectation; the final strict invocation
above passed without a command-line lint allowance.

Final focused logs are local scratch evidence:
`/private/tmp/sabiquant-settlement-hand-final-{debug,release,clippy}.log`.
Every associated process was terminal with exit0 when this report was written.
These focused logs precede the two subsequent implementation-review cleanups
(strict finite-before-zero-canonicalization sequencing and a defensive
secondary-decode stage label). Neither cleanup changed a mathematical result;
the root's complete post-review validation must verify the amended source.

## Pins and limits

| Artifact | SHA-256 |
|---|---|
| [Frozen contract](M8_SETTLEMENT_CONTRACT.md) | `47f1b08ff1ca646c502c90b1e7cb380fa4d74a4763b3aa9e32d729adc89faeba` |
| [Hand suite](../tests/settlement_hand.rs) | `163446e073a9b19ea3e6147876efd6b81d2b51bb81d8fe1c26b5067df2ea1070` |
| [Public API suite](../tests/public_api.rs) | `d6b78a9994aef6c9d5b8ad1495068712833015e7f662c03a5c38c3e6feaf8859` |
| [Root errors/catalog](../src/error.rs) | `f40430d465ffad358d0312a7d890e7f2f37b3f71914a820502fa279fa9d1a126` |
| [Module integration](../src/lib.rs) | `b5e97322a5115c065e6d5fa9eab101241954b3c48eb90bf5b678d83a6aaf2935` |

No universal coverage or numerical proof is inferred from these examples.
The hand suite does not force OOM, inject invalid private states, execute the
proof-excluded InternalCapacity arithmetic path, prove the 40-limb capacity
bound or establish named DMO/Treasury financial acceptance. The independent
settlement fixture/consumer, complete regression, fresh production-region
coverage, extracted std-only package checks, old-artifact byte audit and
platform compilation require their own actual evidence. Full M8 yield/risk,
named models and independent owner/platform/release gates remain open.

Receipt guarantees retain their contract meaning: supplied values are exact
inputs; a unique declared-enclosure result is conditional on the caller's
enclosure and recipe. Neither establishes that an ideal financial price lies
inside that enclosure. Work and artifacts remain local; nothing was pushed or
published by this hand-test task.
