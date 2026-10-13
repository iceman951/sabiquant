# Local M7 analytical cash-flow validation

This tranche follows saved `5ef7af9` under continuing local-only authority.
The [adopted contract](M7_CASHFLOW_CONTRACT.md) advances analytical F-CF-01
and flat NPV/outstanding accrual in F-CF-02. [Research](M7_CASHFLOW_RESEARCH.md)
separates primary financial evidence from delegated generic analytical choices.
No financial-product/settlement, formal milestone or release acceptance is claimed.
NI-01 exact decimal settlement, seven specialized rules, later BPS/yield/risk,
native Linux/Windows execution and existing maintenance/owner gates remain open.

## Arithmetic and numerical scope

The private kernel retains at most 159 exact significand bits and rounds once.
An excluded integration harness compiles that ACTUAL dependency-free kernel
source via path; it exposes no test-only public API and duplicates no library
self-tests. Production-linked public coupon tests exercise the domain wrapper
and actual accrual composition. The independent Python Fraction fixture has
105,272 rows: 95,393 finite nonzero, 296 canonical zero, 5,202 nonzero
underflow and 4,381 overflow. All 631,632 operand permutation calls match
expected bits/status at 0 ULP in debug/release. Five self-contained wrapper
unit tests cover sign/range/error context and exact ties; package tests need no
repository fixture. Row/permutation counts are arithmetic inputs/calls, not
that many distinct financial examples. All 2,047 finite exponent fields are
represented with bounded significand shapes and seeded finite-bit inputs.

Generator --check passes. SHA-256: generator
`d20ccf195e276f1a5540da95a5122afa19721c9157ecbe985ef67c2804f67d2e`,
13,713,336-byte fixture
`79f35f8fb75a208a7aa8c54a4cd4a460bb9a2c3391418c2dfbe39ad1ce0f6c3a`;
wrapper `c5b9f990277c4760db621b9987d89473fa09b99db2e45aa5f72dde7bb5dfba2e`,
kernel `a781989372daef776b0a8b5ad0d286d47e1d883f46078c9847c3c7f5ac698673`.

Correct rounding applies to SUPPLIED binary64 factors, including a previously
rounded fraction. NPV reduces finite rounded PV terms with exact_sum; it is
not an exact dot product. Ideal financial comparisons propagate absolute
amount/time/factor/product/reduction error and condition on gross signed flows.
Existing day-count and factor tolerances remain unchanged. Rust elementary
functions have unspecified precision, so calibrated oracle comparisons do not
provide a universal libm certificate or QuantLib parity.

## Resource and public failure evidence

Nine `cashflows_resources.rs` tests pass debug/release. They cover sequential
ICMA overlap charging, first product failure before a later uncovered zero
payment, original indices after filtering, Business/252 calendar-span charging,
overlapping manual partial accruals and cached/structural zero-work paths,
empty anchor guard, an actual maximum 3,652,059-flow duplicate-date scalar leg
and an actual 200,000-reference schedule leg sharing the entire ICMA context.
Count/work caps are checked exactly at the accepted bound and below it.
Two additional grids separately require bit-exact preservation of the tiny
residual after equal-horizon signed cancellation (32 configurations × three
rates × six permutations = 576 NPVs), and direct valuation across every hybrid
switch (two hybrid methods × ten frequencies = 20 cases). Leap-year annual
ICMA gives the first payment 1/366, below even Daily's 1/365 switch. Every
later payment is beyond its switch, and the incorrect chained alternative
differs by more than 16 epsilon; returned-factor sum expectations are bitwise. Coverage inspection identified
an untested reachable factory adapter. A further public case now proves that
28-day/31-day monthly amounts can fail at the second interval (Value index1),
invalid notional fails at index0, and zero scalars still fail at the second
uncovered ICMA interval (Rate index1 and typed root/domain/ICMA sources).
The operation returns no partial leg; no invalid private state is constructed.
No private corruption or forced OOM supplies coverage.

The exhaustive root catalog passes with 89 unique nested variants, including
nine CashFlowError kinds. Wrapper/source/formatting failures are checked
uniformly for every domain. Partial query Rate errors and invalid fraction
wrappers are structurally retained but proof-excluded for today's immutable
conventions after successful full coupon construction; real partial product
underflow is exercised through the production public coupon/leg APIs. Allocation errors are
catalogs, not a claim of a controlled real-OOM test. Arc construction and
ordinary Clone keep their standard system-allocation qualification.

## Independent financial oracle and conditioning

`cashflow_oracles.py` and its offline std-only Rust consumer verify **47,144**
rows across 13 explicitly declared accrual contexts and all 32 discount
configurations: **10,481 coupon**, **5,880 single accrued**, **30,319 NPV** and
**464 leg-accrued** rows. NPV includes 4,992 mixed-coupon/simple-flow cases with
separate coupon/discount contexts, unequal signed amounts and amount-error
propagation; 25,327 use simple flows. There are 47,121 distinct input records
and 23 deliberate repeats, not 47,144 distinct market products.

Civil formulas and checked i128 rational reconstruction independently verify
exact fraction metadata. Existing scalar/termination/Business/252 fractions
retain their correctly rounded E1 expectation; ISDA/ICMA retain four ULP E2.
For each actual permitted binary64 fraction, a Fraction-generated product map
requires the exact rounded product AND its underflow/overflow cause/sign/domain.
No expected value comes from Rust or reference internal code. All 2^53 quotient
input bounds are asserted. Decimal 120/200 independently agree on NPV values,
propagated allowances and factor statuses; the unchanged independent M4 oracle
helper is SHA-pinned. Plain fixed coupons use N*r*YF, not discount compounding.

| Profile | Rows | Required comparison |
|---|---:|---|
| TOL-E0-PRODUCT | 16,361 | Actual admissible fraction maps to exact product bits or typed range status |
| TOL-E0-SUM | 8,429 | Exact rounded-term reduction/identity bits, including cancellation |
| TOL-E5-CASHFLOW-PROPAGATION | 22,354 | 22,236 finite values against propagated absolute error, plus 118 exact structured failures |

Stable classification is 30,665 aggregate values, 79 work errors, 38 Rate
errors (three ICMA, five NumericalBoundary and 30 Value), one final Sum error,
397 product overflows, 895 definite product underflows, four alpha-conditioned
underflow-or-value rows and 15,065 definite product values. The four ICMA-M
quarter-year midpoint rows CF-ANALYTICS-003690/003697/003717/003724 can yield
only underflow or MIN_SUB according to the ACTUAL alpha in the existing four-ULP
profile. Conditional counts are pinned without forcing a platform's permitted
choice. This native snapshot observes 899 underflows and 15,065 product values.
Three E0 sum rows have unrepresentable gross telemetry and zero allowance;
no infinite tolerance is accepted. All comparison allowances are finite and
nonnegative. Inclusion, phases, original indices and work used/needed are exact.

**Material near-pole conditioning is explicitly retained.** Four rows have
propagated allowance/gross above 1e-6. The largest normalized error is
0.9999999999999107 at CF-ANALYTICS-041397: SimpleThenCompounded,
EveryFourthMonth, 30E/360 2024-01-01..05-01, amount 1,
rate nextUp(-3) = -2.9999999999999996. Exact alpha 1/3 versus its returned
binary64 value differs by -1.8503717077085942e-17. The ideal Simple factor is
1.4802973661668754e-16; exact fused arithmetic at returned time gives
2.0354088784794536e-16. Ideal PV is 6,755,399,441,055,744, versus correctly
rounded returned-time PV 4,913,017,775,313,269: a 27.2727272727272633% gross
change. The propagated allowance is 1,842,381,665,742,639.5. Exact Fraction
independently verifies these factor/reciprocal anchors. This is date-time input
conditioning near a Simple pole, not failed elementary arithmetic or a widened
accrual tolerance. Rows 041451/041478 likewise use counts 6/12 and 27.2727%;
041586 uses Daily and 5.2770%. Aggregate PASS does **not** establish universal
financial relative accuracy near these boundaries. The separate 576-case exact
cancellation grid prevents broad gross conditioning from hiding a lost small
cash flow in ordinary equal-horizon reduction.

Generator `--check`, debug/release consumer and strict all-target Clippy pass.
SHA-256: generator
`ee8bd7059f6ca1aead42c67121e3ad30c2a357365c2e153b254757a8d89788b8`,
consumer `2f984d434f8ff8fc4286a7c026b7f547dc98791f6433c36b6d65c66e2092946f`,
14,804,224-byte fixture
`2c61c8fd00dd0feb0f57070b672ae4e2cf89e58a716a9a3680621c1cdc8405a9`.
The 2026 TARGET model uses only the published closure set and queried 2026
half-open days; it is not a historical/projected market oracle. Core public
cases separately retain strict/projection/inception/gap and typed guard tests.
No new QuantLib cash-flow execution/parity is claimed; primary sources and
public headers supply the qualified research inventory. Earlier reference
fixtures remain unchanged and execute in the full regression suite.

## Source review and package

[Independent API review](M7_CASHFLOW_API_REVIEW.md) finds no production defect,
proves ICMA overlap-prefix subtraction and resource bounds, and qualifies
proof-excluded partial Rate/fraction/OOM paths. [Traceability audit](M7_CASHFLOW_TRACEABILITY_AUDIT.md)
maps all FR-CF-01..12 and actual tests. Exactly 64 public struct/enum declarations
have one centralized value/thread assertion each; all domain error traits/root
conversions and 89 unique nested error variants are checked.

Reviewed parent SHA-256 is
`bd67f8fd631f9b02b4d14ead7d9ac4102592b347adb25c21737f63e29fb0ea8f`;
source error/lib identities and focused consumer hashes are retained by the
independent audits. All 65 prior fixture/generator files remain byte-identical
to saved 5ef7af9. The cumulative fixture inventory is 1,171,206 rows in 49 files;
these are declared records/calls, not proofs, features or distinct products.

Actual archive `/private/tmp/sabiquant-m7-cashflow-package-target/package/sabiquant-0.0.0.crate`
has 35 allowlisted files, 148,133 compressed bytes and 642,760 content bytes,
SHA-256 `7defdb6b1f1aa9af55fc11b9422bf6603adadc056146f8d036c380d5d5342b9a`.
It records base HEAD 5ef7af9f16618ec327707618c7ed1dee2256b999, dirty=true.
All 33 original files match the workspace before/after extraction/checks;
normalized manifest and VCS metadata are the generated exceptions. The fresh
actual extracted package passes 78 unit tests, 27 doctests (24 ordinary,
three compile-fail), strict all-target Clippy and warning-denied public/private
docs with zero compiler warnings. Research, generators, integration consumers,
fixtures and native/reference binaries do not ship. Cargo packaging at its
snapshot emits 49 expected omitted-test notices, separately from extracted
compiler warnings. Version remains 0.0.0, publish=false, no dependencies.
Evidence JSON `/private/tmp/sabiquant-m7-cashflow-package-evidence.json` SHA
`e6cb01aedccfbe02008452fb70f0802e6517849ed8c37cf27e1f5bb407652e69`.

## Final full-project checks

Final Rust1.99/macOS arm64 warning-denied full debug/release passes **480 debug**
and **479 release** tests in **53 result groups**, zero failed/ignored/compiler
warnings: 78 debug/77 release units, 375 integrations and 27 doctests (24 ordinary,
three compile-fail). One inherited unit intentionally checks a debug assertion.
Logs are `/private/tmp/sabiquant-m7-cashflow-debug.log` and `...-release.log`.
Strict all-target Clippy, rustfmt and warning-denied public/private docs pass.
Linux GNU and Windows MSVC final all-target warning-denied compilation passes;
it is compile-only, not native execution. Existing reference regressions pass.

Fresh coverage at `/private/tmp/sabiquant-m7-cashflow-coverage-final` executes
all instrumented unit/integration binaries successfully. Excluding embedded
unit-test source, new cash-flow production regions are **688/703 (97.87%)**:
parent 513/527, product domain wrapper 22/22, dependency-free kernel 153/154.
This is a source-region union, including the actual-source private kernel
integration harness; it is not branch, exhaustive-input or numerical accuracy
coverage, and excludes doctests. Initial coverage prompted the real factory
error-adapter test above; final source is unchanged.

The remaining 15 regions represent fallible reserve/OOM propagation, the
proved unreachable partial-query/nonnegative-fraction failures, the factory's
other-error fallback (valid Schedule raw intervals cannot fail interval order),
and the kernel's zero-shift join. For three nonzero finite binary64 operands,
fixed unnormalized significands force a strictly positive discarded-bit shift:
three normals have highest integer bit at least 156; two normals at least 104;
one normal plus two subnormals has exponent sum <=-1177 and shifts by at least
103 at the minimum-normal/subnormal boundary; three subnormals shift by 2148.
Thus the guarded shift==0 path is excluded by real accepted operands. No
private intermediate corruption or forced OOM is used for a percentage target.

The related current Markdown audit resolves 242 local relative targets. It
fixed an inherited `ARCHITECTURE.md` link to nonexistent `LOCAL_VALIDATION.md`
by pointing to the actual M6/ICMA/cash-flow evidence. All 65 prior fixtures/
generators and all 33 packaged original files remain byte-identical to their
recorded snapshots. Full M7 and formal gates remain open; next roll-rule research
is separate from this saved analytical tranche.
