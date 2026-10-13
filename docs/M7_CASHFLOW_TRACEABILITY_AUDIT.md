# Independent local M7 cash-flow traceability audit

This review covers the adopted analytical cash-flow tranche under the continuing
local-only authority recorded in the [contract](M7_CASHFLOW_CONTRACT.md). It maps
[FR-CF-01 through FR-CF-12](REQUIREMENTS.md)
to concrete public operations, independently inspected consumers and actual
native checks. It found no concrete production defect in the reviewed source.
The [financial research](M7_CASHFLOW_RESEARCH.md) and separate
[API/source review](M7_CASHFLOW_API_REVIEW.md) explain the analytical choices and
private-invariant proofs; this audit does not change those choices.

The scope is simple signed payments, plain SIMPLE fixed coupons, immutable
ordered legs, direct flat-rate NPV and explicitly defined outstanding accrued
interest. Exact decimal settlement under NI-01, product entitlement/ex-coupon
rules, the seven remaining specialized schedule rules, later BPS/yield/risk,
native Linux/Windows execution and formal owner/milestone/release acceptance
remain open. Public QuantLib reference concepts do not establish compatibility.

## Requirement-to-operation evidence

`core` below means [cashflows_core.rs](../tests/cashflows_core.rs), `resources`
means [cashflows_resources.rs](../tests/cashflows_resources.rs), `analytics`
means [cashflows_oracles.rs](../tests/cashflows_oracles.rs), and `products`
means [cashflow_products.rs](../tests/cashflow_products.rs). All names identify
actual consumers rather than proposed tests.

| Requirement | Production operation and concrete evidence | Qualification |
|---|---|---|
| FR-CF-01 | `SimpleCashFlow::new`, `FixedCouponRate::new` and `FixedRateCoupon::new`: core `signed_simple_amounts_have_explicit_dates_and_canonical_zero`, `coupon_quotes_keep_all_context_and_accept_signed_simple_rates` and `complete_coupon_terms_and_amount_signs_survive_early_or_late_payments` check signed finite extremes, both input zero signs, rejected NaN/infinities and retained terms. `immutable_quotes_coupons_and_legs_reuse_across_threads_with_fresh_budgets` checks independently equal definitions, shared clones and 8×16×32 concurrent valuations with fresh budgets. | Finite binary64 amounts in one supplied consistent unit; no currency, clock, settlement rounding or inferred financial default. Values containing binary64 have definition `PartialEq`, not `Eq`/`Hash`. |
| FR-CF-02 | `FixedRateCoupon::new`: core `coupon_construction_orders_raw_notional_and_context_checks` pins period validation, notional validation and complete accrual/coverage order before zero scalars. `ordered_zero_accrual_does_not_form_an_overflowing_intermediate_product` uses distinct 30E/360 Jan 30–31 dates; `distinct_weekend_dates_can_have_zero_accrual_and_still_require_query_work` uses Business/252 Saturday–Sunday. Analytics coupon rows independently check civil fractions and product outcomes. | A positive raw date interval can have zero year fraction. Early/delayed supplied payment dates are permitted data; neither adjusted payment dates nor elapsed-day proportions replace the raw convention query. |
| FR-CF-03 | The actual private integer product is compiled by `products::exact_original_three_factor_products_match_fraction_bits_and_status`. Core `exact_public_coupon_products_recover_range_and_round_midpoints` reaches production coupon composition, including `MAX×2×0.5=MAX`, `MIN_SUB×0.5×2=MIN_SUB`, normal/subnormal ties and signs. `coupon_range_failures_retain_sign_quantity_and_typed_source` pins nonzero zero-underflow and infinity overflow diagnostics. Analytics maps every permitted rounded fraction to exact Fraction product bits/status. | One correctly rounded product of the **supplied binary64** factors. A rounded year fraction is not an exact financial fraction. The private harness alone is not public coupon coverage. |
| FR-CF-04 | `FixedCouponRate` retains private immutable `Arc<AccrualConvention>`; inspectors return `&AccrualConvention`. Core quote/definition tests distinguish equal and different complete contexts; `generated_multi_period_icma_context_is_retained_for_partial_coupons` retains four generated references across the equal-explicit-stub schedule. Resources `large_schedule_leg_shares_the_complete_icma_definition` constructs an actual 200,000-reference leg and verifies shared complete storage. Thread reuse checks immutable ownership. | `Arc` cloning avoids context copies, but equality of independently allocated definitions can traverse their contents. First `Arc` construction and ordinary `Clone` retain standard system-OOM behavior; fallible vector reserve does not make them fallible. Pointer comparison in a storage-regression test creates no public pointer-identity contract. |
| FR-CF-05 | `Leg::new` checks caller cap/count before original adjacent payment order. Core `leg_count_order_and_duplicate_payment_metadata_are_explicit` pins empty/cap-zero, over-limit/reversal precedence, first reversal index and retained duplicates. Resources `maximum_scalar_leg_retains_every_duplicate_and_exact_count_budget` constructs all 3,652,059 permitted flows and checks exact work cap and cap−1. | No sorting, coalescing, implicit principal or cap metadata in value equality. `Leg::new` takes ownership of the caller's existing vector; its prior allocation is outside this constructor. |
| FR-CF-06 | `Leg::fixed_rate` uses each `Schedule::contractual_dates()` interval and corresponding `adjusted_dates()[i+1]`. Core `schedule_leg_uses_raw_accrual_and_adjusted_payment_for_all_49_convention_pairs` independently checks raw Actual/360 fractions and shifted payments across 7×7 convention pairs. `schedule_directions_stubs_eom_explicit_and_zero_need_no_coupon_inference` covers Forward/Backward, short/long automatic stubs, EOM choices, explicit equal outer roles and Zero. The multi-period ICMA case checks supplied context retained by factory coupons. | The factory preserves coupon quote/accrual terms, but does not embed the full source Schedule; callers retain that Schedule for its separate generation/calendar provenance. It guesses no frequency, lag or redemption. |
| FR-CF-07 | `PaymentDateInclusion`, `FixedRateCoupon::accrued_amount`, `CashFlow::accrued_amount` and `Leg::accrued_amount`: core `accrued_coupon_cutoffs_clip_raw_dates_and_payment_eligibility` checks before/start/interior/end/payment/after-payment dates, Include/Exclude and early/delayed payment. `partial_convention_queries_are_not_elapsed_day_scaling` independently checks Bond Basis and termination-aware 30E/360 ISDA partial queries. `delayed_completed_and_new_partial_coupons_are_both_outstanding` combines cached and partial coupons. Analytics contains exact partial-convention/product checks and multi-flow accrued reductions. | Simple flows contribute positive zero. Completed eligible coupons return cached full amounts; ceased/before-start coupons return zero. This is the adopted generic outstanding accrual rule, not a bond settlement or ex-coupon entitlement convention. |
| FR-CF-08 | `Leg::net_present_value` validates the equal-date anchor before processing eligible horizons. Core `empty_and_filtered_legs_validate_the_anchor_and_original_flow_indices` covers empty/all-filtered context validation and original indices. `flat_npv_uses_direct_original_horizons_and_retains_duplicate_payments` distinguishes direct Simple discounting from adjacent chaining. `every_discount_configuration_retains_exact_zero_rate_identity_and_query_charges` checks all 32 compounding configurations. Resources `every_hybrid_frequency_crosses_its_switch_without_adjacent_factor_chaining` uses real annual ICMA references to place the early/later payments on opposite sides of both hybrid switches for all ten periodic frequencies (20 cases); NPV matches the direct returned-factor sum bitwise and the chained alternative differs by more than 16 epsilon. Core TARGET/source and individual-PV failure cases retain zero-amount validation and inherited M4 domain failures. | An individually unavailable discount factor or PV remains an error, even when a factorless mathematical product or later cancellation would be finite. Existing `InterestRate` semantics/tolerances are retained. |
| FR-CF-09 | Each `Leg` call creates a fresh local `AccrualWork`; it charges immediately before the needed original query. Core ICMA/TARGET/filtered/partial-underflow cases and resources `icma_charges_each_original_horizon_and_preserves_first_failure`, `business252_budget_counts_calendar_span_including_zero_amount`, `overlapping_manual_accruals_share_a_per_call_budget_but_cached_paths_cost_zero` and `empty_legs_require_only_the_discount_anchor_query` pin cumulative exact caps, cap−1, charge-before-coverage, charge-before-product, zero/excluded/cached paths and original indices. Analytics work-error rows independently record maximum, used and required cumulative charge. | Every `u64` value, including zero, is a valid budget. ICMA costs 1+overlaps, Business/252 costs 1+absolute calendar span, scalar/termination contexts cost 1; equal queries cost 1. Structural/cached accrued paths cost zero. There is no future prevalidation or optimized batch implementation. |
| FR-CF-10 | Source `reserved` uses `try_reserve_exact` for factory and reduction vectors, bounded by retained flow count. NPV/accrued collect finite rounded terms then call `exact_sum`. Core `reduction_recovers_prefix_overflow_but_not_failed_individual_present_values` reaches prefix recovery, final NPV/accrued Sum failures and earlier individual PV failure. Resources `equal_horizon_signed_cancellation_preserves_the_small_payment_exactly` checks all 32 configurations ×3 signed rates ×6 permutations: the residual equals the returned discount factor bitwise in all 576 cases. Analytics contains exact zero-rate sum/status and propagated financial comparisons. | This is exact reduction of fixed rounded contribution bits, not an exact dot product. It cannot recover product rounding, four individually underflowing terms, or individually overflowing opposite-sign PVs. Allocation error diagnostics are catalog-tested; no real-OOM test is claimed. |
| FR-CF-11 | `CashFlowError` preserves nine stable kinds, original index/phase and typed Value/Rate/Sum causes; root `Error::CashFlow` exposes the domain source. Core signed coupon/partial underflow, TARGET nested source, order/cap/work and final reduction cases reach public failures. Resources `schedule_factory_failures_preserve_the_original_interval_and_cause` reaches the factory's Value/Rate index adapters: a finite first coupon then overflowing second coupon preserves index 1/`coupon amount`; nonfinite notional preserves index 0/`coupon notional`; an uncovered second interval despite zero scalars preserves index 1/`coupon accrual` and typed root→CashFlow→Rate→ICMA sources. [The centralized error catalog](../src/error.rs) checks all 89 nested variants for unique kinds, transparent source chains and formatting-writer failure propagation; the nine new variants are included. Analytics pins aggregate error kind, cause kind, index/phase and exact work fields. | Catalog construction is not a claim that every defensive arm is reachable from valid public values. Partial coverage/rate and negative/nonfinite fraction wrappers are proof-excluded for today's successfully constructed immutable conventions. Real partial-product underflow is reachable and tested in both standalone and original-index Leg calls. |
| FR-CF-12 | `products` checks exact Fraction bit/status anchors; analytics independently computes civil rational accruals, supplied-factor product maps and two-precision Decimal direct NPV with absolute propagated budgets. Core exact identities/range ties and resources' 576 bitwise cancellation cases strengthen cases where gross-conditioned tolerances would conceal a lost residual. | Calibrated financial propagation checks do not certify universal libm accuracy, universal few-ULP NPV error or QuantLib parity. Existing accrual/rate profiles are unchanged; error budgets must remain finite and informative. |

## Source and resource proof checks

The source review verified constructor validation before zero shortcuts; raw
accrual versus payment separation; private complete context retention; monotone
payment filtering; original-index enumeration after filtering; fresh local work
ledgers; charge-before-query failure order; direct original valuation horizons;
finite individual terms before exact reduction; and typed source forwarding.
The budget cost match explicitly covers all four current `AccrualConvention`
variants, so a future variant requires a deliberate compile-time change.

For an unequal ICMA query, the two ordered `partition_point` boundaries identify
the retained periods with `end>lower` and `start<upper`. Ordered positive
contiguous references make the first prefix a subset of the second, so
`last−first` is nonnegative and is precisely the overlap count. An out-of-coverage
query still pays for its actual retained overlaps before the original coverage
error. Equal queries take the explicit zero-extra path and still validate their
context. Business/252 charges civil span, independent of how many business days
its calendar returns.

There are at most 3,652,059 flow queries, each costing at most 3,652,059 units,
plus one NPV anchor. Thus cumulative required work is at most
`1 + 3,652,059² = 13,337,534,939,482`, below `1.34e13`; the unchecked-looking
`u64` addition follows an established date/reference/count bound. The count cap
includes duplicate payment dates, rather than relying on chronology to limit
their number. The actual maximum-flow and 200,000-reference tests support these
different count/storage limits without claiming every combined maximum was
allocated at once.

Time is bounded by flow iteration and caller-permitted charged interval work,
plus ICMA partition searches and calendar-definition costs. Repeated direct
ICMA prefixes can still cost O(nM) when the caller permits that work. No ordered
batch/prefix-sum optimization or calendar-graph-depth bound is implied.

The factory's `raw.len()−1` and adjusted-end indexing depend on validated
Schedule invariants: at least two ordered retained raw boundaries and matching
strictly ordered adjusted boundaries. Payment reversal from this factory is
therefore proof-excluded, while reversal in a manual Leg remains public and
tested. Factory Value/Rate index adaptation is separately reachable: the valid
Jan 31→Feb 28→Mar 31 EOM schedule accepts the 28-day first `MAX×12×YF` coupon
then rejects the overflowing 31-day second one, and a complete quote covering
only the first interval rejects the second even with zero notional/rate. The
new public resource test reaches both adapters without private corruption,
returns no partial leg and preserves each original interval/source.
Partial coverage/rate and invalid fraction defenses remain useful if
new conventions are added; today's valid immutable full coupon context covers
its ordered partial subinterval. Neither private state corruption nor forced
allocation exhaustion was used to manufacture coverage.

## Public type, error and package guards

Independent read-only enumeration found 64 unique public struct/enum definitions
across `src`, matched by all 64 centralized value/copy trait assertions in
[public_api.rs](../tests/public_api.rs). The seven cash-flow types are
`PaymentDateInclusion`, `SimpleCashFlow`, `FixedCouponRate`, `FixedRateCoupon`,
`CashFlow`, `Leg` and `CashFlowError`. Copy/value assertions require the intended
Clone/Debug/PartialEq/Send/Sync/`'static` traits; separate error/key assertions
check StdError and the closed inclusion policy's Eq/Hash use. This enumeration
is a review check, not a newly installed source-exhaustiveness script.

The 89-variant root catalog includes all nine CashFlowError kinds and tests
stable uniqueness, transparent root/domain/nested source forwarding and
fallible formatting. The [policy tests](../tests/policy.rs) reject every
dependency-table form and ensure the lockfile contains only SabiQuant. Cargo
forbids unsafe code and requires documented/debuggable public values; the
[lint configuration](../clippy.toml) guards clock, I/O, output and mutable
global-state facilities. The reviewed cash-flow source adds no dependency,
unsafe block, I/O, hidden evaluation date or mutable shared financial state.
Private immutable `Arc` ownership is consistent with these guards.

The separate API reviewer executed an actual offline local archive/extraction
check, captured in `/private/tmp/sabiquant-m7-cashflow-package-evidence.json`
(SHA-256 `e6cb01aedccfbe02008452fb70f0802e6517849ed8c37cf27e1f5bb407652e69`).
I inspected that evidence's source pin, byte-comparison count and successful
terminal summary: 35 allowlisted archive files, 33 original files byte-equal,
78 extracted unit tests and 27 doc tests, strict extracted Clippy and
warning-denied public/private docs, zero extracted warnings. Archive SHA-256 is
`7defdb6b1f1aa9af55fc11b9422bf6603adadc056146f8d036c380d5d5342b9a`.
The 148,133-byte compressed / 642,760-byte content archive excludes integration
tests, fixtures, generators and research docs, as the manifest intends. Cargo's
49 notices about omitted integration targets are qualified separately. This
is actual local package evidence, with no publication or native cross-platform
execution claim; it was executed by the API reviewer, not this trace reviewer.

## Independent numerical consumer review

The arithmetic fixture contains 105,272 input rows: 95,393 finite nonzero,
296 canonical zero, 5,202 nonzero underflow and 4,381 overflow. Its actual-kernel
consumer checks all six operand permutations, 631,632 calls, at 0 ULP for finite
bits and exact statuses. The rows cover all 2,047 finite exponent fields with
bounded significand shapes and seeded finite bits. Five self-contained
production wrapper unit cases add range/sign/quantity context and midpoint
coverage; public core tests independently connect the kernel to rounded
accrual and coupon validation.

The analytics generator uses Python civil `datetime`/calendar coordinates,
Fraction and Decimal at 120/200 digits. It does not import Rust financial
outputs or QuantLib. The Rust consumer independently computes civil ordinals,
weekday/calendar counts and reduced rational accruals without production
`Date::days_since`, `Date::weekday` or day-count answers as expected values.
Its bounded integer rational conversion explicitly proves numerator and
denominator are within 2^53 before a single binary64 division.

Scalar, termination-aware and Business/252 accrual anchors are exact at their
declared rounded rational values; ISDA/ICMA retain the existing four-ULP
accrual allowance. The fixture maps each allowed rounded fraction bit pattern
to its independently exact triple-product result. The consumer verifies
actual accrual tolerance and exact metadata, then selects the matching map
entry; it does not assume the composite financial fraction is itself 0 ULP.
Overflow/underflow coupon rows inspect the production Value cause, signed
infinity/zero, quantity and open sign domain. Successful partial accrued rows
also select the actual rounded partial-fraction product bits.

NPV/accrued financial comparisons propagate absolute amount, horizon, discount,
single-product and final-reduction errors from independent ideal formulas.
Budgets are evaluated at both Decimal precisions rather than by overflow-prone
binary64 gross sums. Nominal propagated rows require finite nonnegative
allowances; an overflowing gross telemetry field on an exact zero-rate sum row
does not license an infinite comparison allowance. Fixed supplied rounded-term
sum anchors remain bit-exact. General gross-conditioned bounds cannot alone
prove that a tiny cancelled residual is preserved; the independent public
576-case equal-horizon grid supplies a separate bitwise identity.

The 13 declared contexts include five scalar conventions, termination-aware
30E/360 ISDA, monthly/annual/daily ICMA, Null/weekend/custom Business/252 and
strict TARGET. The independent TARGET fixture model uses only the explicitly
marked 2026 closure set and queried 2026 days; it is not a full TARGET historical
coverage/projection oracle. Core tests independently cover inherited strict
2025/2029 coverage failures, work-before-guard order and explicit projection.
Fixture counts represent rows/calls, with repeated grids and product candidates;
they do not represent that many distinct market products.

Independent read-only parsing of the frozen 14,804,224-byte analytics fixture
confirmed its declared counts and provenance:

| Dimension | Frozen row counts |
|---|---|
| Operations | coupon 10,481; accrued 5,880; NPV 30,319; leg-accrued 464; total 47,144 |
| Comparison profiles | E0 supplied product 16,361; E0 rounded-term sum 8,429; E5 absolute propagation 22,354 |
| Aggregate outcomes | 30,665 values; 79 work-limit errors; 38 Rate errors; one final Sum error |
| Supplied-product row classifications | 15,065 unambiguous values; 895 unambiguous underflows; 397 overflows; four underflow-or-value boundary rows |
| NPV composition | 4,992 mixed-coupon rows; 25,327 simple-only rows; all 32 named compounding configurations |

The 38 Rate rows preserve three ICMA coverage, five numerical-boundary and
30 Value cause kinds; the final Sum row preserves `sum.arithmetic`. Work rows
pin original index/phase, caller maximum, cumulative used and next required
charge. Every one of the 30,665 aggregate value rows has a finite nonnegative
allowance. Only three exact-sum rows have infinite gross telemetry; their
comparison allowance is zero. E5 contains 22,236 numerical values and the 118
aggregate error rows, so profile counts are not all successful-value counts.

Four min-subnormal ICMA coupon rows straddle the final product rounding boundary
within the already permitted accrual profile. Their actual rounded alpha
selects the exact independent product bit/status. The consumer separately pins
the stable map classifications, permits only 895–899 actual underflows and
15,065–15,069 actual product values with fixed combined count 15,964, and checks
each actual signed product/error against its selected alpha. This is documented
upstream fraction uncertainty, not a relaxed product-rounding rule. No universal
cross-platform range-status count is asserted for those four rows.

The generator owner reports 47,121 distinct input records and 23 repeated
inputs when IDs and generated result/map fields are excluded. Counts are pinned
in both fixture metadata and consumer assertions. Generator `--check` rebuilds
and compares every data byte and all declared provenance/count/context metadata, excluding
only generation timestamp and host-platform metadata from the comparison.
The imported independent rate helper is itself pinned by SHA-256; the Rust
consumer has no external runtime/reference dependency. The generator owner
completed frozen `--check` successfully; the root completed the final consumer
in debug/release and strict all-target Clippy successfully. These are actual
owner-executed checks, distinguished from this reviewer's focused native runs.

On this native macOS arm64 snapshot, the consumer observed 899 product
underflows and 15,065 product values; all 397 overflows and all pinned aggregate
outcomes matched. It compared 22,236 E5 numerical values. Maximum observed
normalized absolute error was `0.9999999999999107` at
`CF-ANALYTICS-041397`. That maximum is a material conditioning case, not a
uniform financial accuracy claim. The flow pays 1 on 2024-05-01 from a
2024-01-01 valuation using 30E/360, SimpleThenCompounded/EveryFourthMonth and
`nextUp(-3)` as the rate. Exact financial time is 1/3; supplied binary64 time is
`0x1.5555555555555p-2`, differing by approximately `−1.850371707708594e−17`.

This reviewer independently checked the case with Fraction: the ideal Simple
compound factor is approximately `1.4802973661668753e−16`, while the factor for
the rounded time is exactly representable as `2.0354088784794536e−16`. Their
correctly rounded reciprocal values are `6,755,399,441,055,744` and
`4,913,017,775,313,269`; difference `1,842,381,665,742,475` fits the unchanged
propagated allowance `1,842,381,665,742,639.5`. The allowance is about 27.27% of
gross, reflecting time condition number `|rα/C| = 6,755,399,441,055,743` near
the Simple pole. It does not indicate a libm failure or certify a small relative
financial error. Both Decimal precisions agree; no allowance was widened.

Independent fixture scanning found only four E5 value rows with
allowance/gross greater than `1e−6`: `041397`, `041451`, `041478` at about
27.27% (events/year 3, 6, 12) and `041586` at about 5.277% (Daily). The pass
for these rows validates propagation of date-time conditioning. Existing
factor-required domain/guard and scalar/composite accrual tolerances remain
unchanged. An informative audit must preserve this exception alongside the
exact product/reduction claims and the separate exact cancellation identities.

## Scoped actual verification and source snapshot

This reviewer independently ran the frozen 23 core tests in debug and release
and strict focused Clippy, with no failures/warnings. It also ran the actual
kernel fixture consumer (one test), public API suite (eight tests), policy suite
(three tests) and all root error tests (11 tests) in debug. The new 576-case
equal-horizon resource test, 20-case hybrid-switch/direct-horizon resource test
and new factory Value/Rate index-adapter test passed independently in debug and
release. The resource owner completed all nine final resource tests in both
profiles, strict all-target Clippy and formatting checks, with no failures or
warnings. The
separate API reviewer also independently executed the prior six-resource
snapshot. These added tests change neither source nor those prior cases.
Full-project aggregate counts/coverage belong to the final
[validation report](M7_CASHFLOW_VALIDATION.md), once its exact snapshot checks
have completed; they are not inferred from these focused runs.

| Reviewed file | SHA-256 |
|---|---|
| `src/cashflows.rs` | `bd67f8fd631f9b02b4d14ead7d9ac4102592b347adb25c21737f63e29fb0ea8f` |
| `src/cashflows/product.rs` | `c5b9f990277c4760db621b9987d89473fa09b99db2e45aa5f72dde7bb5dfba2e` |
| `src/cashflows/product/binary.rs` | `a781989372daef776b0a8b5ad0d286d47e1d883f46078c9847c3c7f5ac698673` |
| `tests/cashflows_core.rs` | `7a0ea04f656c042c5c53b000614d6a0d37bd6fd400816e9946bc199aaba0e06f` |
| `tests/cashflows_resources.rs` | `2d396a0d5fe77e04fd0991cb23aaba14ae50fd15caadfc94f0bc12b4f9132dac` |
| `tests/public_api.rs` | `e000feace0b80710a3cb25933a6bc4f3f611220cac651dda2404e8ee23b7beab` |
| `src/error.rs` | `3c73f7a06f77a334adfd6ec7fb2abe20e943b9e4f2f9a8be66a50a1ff818348f` |
| `docs/REQUIREMENTS.md` | `37eaaf4f2d00ddd899fe0871ed1b7ae3ea1bb23d68900a11ecf8d5441aa1efa9` |
| `tests/cashflows_oracles.rs` | `2f984d434f8ff8fc4286a7c026b7f547dc98791f6433c36b6d65c66e2092946f` |
| `validation/generator/cashflow_oracles.py` | `ee8bd7059f6ca1aead42c67121e3ad30c2a357365c2e153b254757a8d89788b8` |
| `validation/fixtures/cashflows/analytics-oracle.fixture` | `2c61c8fd00dd0feb0f57070b672ae4e2cf89e58a716a9a3680621c1cdc8405a9` |
| `validation/generator/rate_oracles.py` helper | `a7217309db77fc0c908de9571e0c172ac9e5df597b39ebc53ba984887d7dd573` |
| Analytics fixture data body (without header) | `9d89143b15c77b29a8b15cd8f5af5b7863b1398bbe81720379c4530a7b08e6f3` |
| `validation/generator/cashflow_product_oracles.py` | `d20ccf195e276f1a5540da95a5122afa19721c9157ecbe985ef67c2804f67d2e` |
| `validation/fixtures/cashflows/product-oracle.fixture` | `79f35f8fb75a208a7aa8c54a4cd4a460bb9a2c3391418c2dfbe39ad1ce0f6c3a` |

This audit owns only this new document. It introduces no production/test/
generator changes, external writes, publication or additional approval gate.
