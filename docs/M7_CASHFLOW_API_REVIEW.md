# Local analytical cash-flow API review

This agent review inspects the [adopted local contract](M7_CASHFLOW_CONTRACT.md)
and [parent module](../src/cashflows.rs) read-only. The reviewer authored the
private exact-product kernel; its separate agent review and independent
Fraction oracle complement the self-review of that arithmetic. This document
does not grant owner, financial-convention, milestone, platform or publication
approval. Exact settlement, bond entitlement/ex-coupon rules and subsequent
analytics remain outside this tranche.

Reviewed parent source SHA-256:
`bd67f8fd631f9b02b4d14ead7d9ac4102592b347adb25c21737f63e29fb0ea8f`.
Kernel SHA-256:
`a781989372daef776b0a8b5ad0d286d47e1d883f46078c9847c3c7f5ac698673`.
Error wrapper SHA-256:
`c5b9f990277c4760db621b9987d89473fa09b99db2e45aa5f72dde7bb5dfba2e`.

## Values, units and retained context

The seven additions are PaymentDateInclusion, SimpleCashFlow, FixedCouponRate,
FixedRateCoupon, CashFlow, Leg and CashFlowError: **64 public struct/enum
declarations** overall. Inclusion is an explicit closed Include/Exclude choice
with Eq/Hash and no default. The flow enum and structured errors are
non-exhaustive. Opaque value fields have no mutable public access. Values
containing binary64 have definition PartialEq, rather than Eq/Hash; equal
amounts do not erase unequal notional, dates or convention definitions.

Amounts and notionals are finite signed amounts in one consistent caller unit;
rates are signed decimal SIMPLE annual rates. The discount quote is a separate
InterestRate with its own accrual and compounding. No currency, locale, clock,
parser, redemption, payment lag or market convention is inferred.

FixedCouponRate owns one immutable Arc<AccrualConvention>. Quote/coupon clones
share the complete context in O(1), while independent-value equality can scan
its content. This covers full ICMA periods, termination context and guarded
Business/252 policy/overrides/joints. Borrowed context identity is useful as a
storage regression, not a financial definition of equality. Leg cloning copies
its flow vector and shares each quote context; it is not an O(1) leg clone.

## Construction, factory and cached accrual

The manual coupon checks raw start<end, finite notional, the complete convention
query, nonnegative finite fraction, then the exact product. Zero notional/rate
does not waive coverage. Payment may be early or delayed, since its relationship
to raw accrual is explicit caller data. All exact zeros are canonical positive
zero; a distinct positive date interval can legitimately have zero day count.

Leg::new checks cap, supplied count, then the first original payment reversal.
It retains duplicate dates and does not sort or merge them. Leg::fixed_rate
checks count before coupon work, uses each raw contractual interval and its
adjusted END payment, and clones the supplied quote. Valid Schedule invariants
justify raw.len()-1 and equal-length adjusted indexing, and guarantee the
factory's chronological payment order. The quote convention is not inferred
from the Schedule. Retain the Schedule separately for generation/calendar
provenance; a Zero schedule still supplies one explicit nonperiodic interval.

Eligibility is payment>cutoff, plus equality only with Include. The retained
nondecreasing dates make the first-eligible partition predicate monotone.
Outstanding coupon accrual is zero when ceased or at/before raw start, a fresh
original-convention fraction on a strict partial interval, or the cached full
amount at/after raw end while still unpaid. An early payment can have clipped
earned interest below its amount due; delayed completed coupons can coexist
with new partial coupons. These definitions do not infer bond clean price.

## Work bounds and first-error order

Each NPV call charges and validates one equal-date valuation anchor, including
empty/all-excluded legs. Every eligible horizon is queried directly from that
anchor, in original order. Included zero amounts still query coverage/factors;
excluded past payments never form a reversed horizon. SIMPLE/hybrid factors
are not chained across adjacent payments. Existing M4 factor-required errors
are retained, including when later cancellation or amount scaling could rescue
a mathematically combined value.

Charges are one plus actual ICMA overlapping periods or Business/252 absolute
calendar-span days; scalar/termination and equal-date queries cost one. The
match handles all current accrual variants explicitly. Cached/ceased/before-start
accrual contributes no query work. A work limit is local per call, with no
mutation of the quote/context or future-horizon prevalidation.

ICMA subtraction is safe even before a coverage failure. For lower<upper,
every positive period satisfying end<=lower also satisfies start<upper.
Thus the first partition's prefix is a subset of the last partition's prefix,
so last>=first, including windows wholly outside coverage. Equal dates take
their own zero-overlap path. Public unequal NPV horizons currently begin at an
already validated anchor; coupon partial queries lie inside a validated full
span. Artificial private invalid contexts are unnecessary for this proof.

Positive disjoint determination periods contain at least one civil day, so
their count is at most 3,652,058. A query charge is at most 3,652,059; at most
Leg::MAX_FLOWS queries plus one anchor need at most
**13,337,534,939,482** units, below 1.34e13 and far below u64::MAX. That proves
the unchecked cumulative addition is safe for valid public values. Before each
required query, the work error retains original index/phase/cap/used/next total.
Partition searches, calendar trees/overrides, allocation and flow traversal
remain separate costs. Repeated ICMA prefixes can be O(nM) when the caller
permits that budget; no blanket linear-time claim applies to flat NPV.

Growing flow/term vectors use fallible reservation, within a separate count
cap. NPV reserves eligible-term capacity after validating its anchor and before
its horizons. Standard Arc construction and ordinary Vec/Leg Clone retain
system-OOM behavior; the API does not promise to recover every allocation or
manufacture an allocation failure for coverage.

## Exact arithmetic and numerical limits

Each finite operand is an integer significand times a binary power. Three
at-most-53-bit significands multiply into at most **159 bits**, represented by
three u64 limbs with bounded u128 intermediates. Exponent sums are -3222..2913.
The retained unit is 2^max(top-52,-1074). Guard/sticky and retained parity perform
one nearest/ties-even rounding; carries directly cross the minimum-normal and
overflow midpoint boundaries. Exact zero is canonical +0. Nonzero rounded zero
or infinity is rejected by the existing signed numeric domain/error policy.
No floating intermediate product, scaling, logarithm or exponential is used.

The [Fraction generator](../validation/generator/cashflow_product_oracles.py)
and [excluded kernel consumer](../tests/cashflow_products.rs) validate original
binary64 factors, not ideal financial accrual. The harness compiles only the
actual dependency-free private kernel: no test-only public API and no duplicate
library/unit suite. Its **105,272 rows / 631,632 operand permutations** match at
**0 ULP** in debug/release: 95,393 finite nonzero, 296 exact zero, 5,202 nonzero
underflow and 4,381 overflow rows. Every finite exponent field, midpoint parity,
sticky bits, signed boundaries and intermediate-range rescue are covered.
Generator --check reproduces bytes. Fixture SHA-256:
`79f35f8fb75a208a7aa8c54a4cd4a460bb9a2c3391418c2dfbe39ad1ce0f6c3a`.

Coupon products are correctly rounded for the SUPPLIED rounded year fraction.
Per-flow PV uses the existing amount-times-returned-factor operation. exact_sum
then rounds the exact sum of these rounded terms; it is not an exact dot product
or currency settlement. Failed individual products/factors are not rescued by
later terms. Financial comparisons require propagated absolute error and gross
flow conditioning, especially under cancellation; existing day-count/rate
allowances and Rust elementary-function qualifications are not widened.

## Error catalog and reachable paths

The nine stable cashflow kinds bring the exhaustive nested root catalog to
**89 variants**. Display retains structured context and source chains preserve
ValueError, RateError and SumError through CashFlowError and root Error.

| `cashflow.` kind | Public path / qualification |
|---|---|
| invalid_accrual_period | Equal/reversed manual raw dates; impossible from valid Schedule intervals |
| invalid_flow_limit | Caller cap exceeds explicit maximum |
| flow_limit_exceeded | Supplied/factory count exceeds caller cap before coupon work |
| payment_order | First reversal in manual Leg input; factory reversal is excluded by valid adjusted Schedule invariants |
| allocation_failed | Fallible vector reserve; catalog/inspection, without forcing system OOM |
| accrual_work_limit_exceeded | Anchor or next original indexed partial/horizon query exceeds per-call work cap |
| value | Numeric inputs and full product range; nonzero partial-product underflow is reachable |
| rate | Full accrual coverage, valuation anchor, original horizon/factor/PV failure; partial-query causes are currently proof-excluded by immutable covered full span |
| sum | Exact reduction of finite rounded terms overflows; temporary prefix overflow can instead cancel successfully |

Current convention implementations give nonnegative finite fractions on valid
ordered covered intervals. Consequently invalid/nonfinite fraction causes and
partial Rate coverage failures remain propagation paths, not claims of a
currently reachable public failure. Cached/before-start/ceased paths compute no
new fraction/product. Partial product underflow remains reachable despite a
representable full amount. These qualifications concern current conventions,
not an unconditional guarantee for future accrual variants.

## Local execution and remaining gates

Independent Rust 1.99.0/macOS ARM64 checks pass five private wrapper tests and
the single full-kernel oracle test in both debug/release, and strict all-target
Clippy. The [resource suite](../tests/cashflows_resources.rs) independently
passes six tests in both profiles, including all 3,652,059 duplicate-date simple
flows and a 200,000-period factory whose coupons share the complete ICMA value.
The root 89-variant uniqueness test passes independently. The
[public core suite](../tests/cashflows_core.rs) independently passes all
**23 tests in debug/release**, including all 49 payment-convention pairs, direct
NPV, signed real partial-product underflow with original-index/work precedence,
cached reduction overflow, TARGET zero/guard paths and concurrent reuse.
Its frozen source SHA-256 is
`7a0ea04f656c042c5c53b000614d6a0d37bd6fd400816e9946bc199aaba0e06f`.
Centralized
[public API tests](../tests/public_api.rs) now cover every one of the 64 declared
types, including new Copy/value/error/Send/Sync/static traits, inclusion Hash
and CashFlowError conversion. The initially missing seven declarations were
reported and added without changing packaged source. All eight centralized
API tests independently pass in debug/release. Extracted-package results follow.

No Linux/Windows native execution, formal market convention acceptance or
publication is inferred. Parent aggregate/coverage evidence, primary platform
CI, maintenance and owner/security/release gates remain separate.

## Independent extracted package

The actual `cargo package --allow-dirty --no-verify --offline --locked` archive
was extracted into a fresh temporary directory and built/tested there, with a
separate fresh target and warnings denied. Cargo's automatic verification was
deliberately disabled for this packaging command; actual extracted execution
supplies the validation evidence instead.

The exact allowlist contains **35 files**. All **33 original source/manifests/
support/README/changelog/license/config files** match workspace bytes; normalized
Cargo.toml and generated VCS metadata are the two remaining files, and
Cargo.toml.orig matches the original manifest. No integration harness, fixture,
generator, research document or native/reference artifact ships. Archive VCS
base is `5ef7af9f16618ec327707618c7ed1dee2256b999`, dirty=true. The originals,
parent/kernel/wrapper and archive hashes were rechecked unchanged afterward.

Actual extracted Rust 1.99.0/macOS ARM64 checks pass **78 unit tests** and
**27 doctests** (24 ordinary, three compile-fail), strict all-target Clippy,
and warning-denied public/private documentation: zero warnings/errors/ignored
tests in those checks. Cargo packaging emitted 49 expected notices for excluded
integration-test targets. The archive is 148,133 bytes compressed and 642,760
file bytes. Archive SHA-256:
`7defdb6b1f1aa9af55fc11b9422bf6603adadc056146f8d036c380d5d5342b9a`.

Full per-file byte identities, command output, toolchain/host, count summaries
and before/after source snapshots are retained at
`/private/tmp/sabiquant-m7-cashflow-package-evidence.json`, SHA-256
`e6cb01aedccfbe02008452fb70f0802e6517849ed8c37cf27e1f5bb407652e69`.
This independent package check made no edits to packaged workspace inputs or
Git. It does not attest native Linux/Windows execution or approve publication.
