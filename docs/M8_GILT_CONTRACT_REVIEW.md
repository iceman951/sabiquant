# Conventional gilt pre-adoption contract review

Recorded 2026-10-19 against local baseline `0d1c987`, before any conventional
gilt production implementation or new frozen gilt oracle. This records an
independent challenge of the complete [contract draft](M8_GILT_CONTRACT_DRAFT.md),
including source lifecycle, price, yield, all six risk views and all four cash
operations. Its conclusion concerns local engineering completeness; it does
not accept a financial model, implementation, market calendar, platform or M8.

The reviewed final draft has SHA-256
`2d113405b8aadfcf478fb967ac12e0e0d53d5cde81bdf7dcfe426565f49bb4ed`.
Root integrated three separately authored API/lifecycle, numerical and cash
components, then assigned three independent read-only review scopes. No
reviewer edited the repository, altered expectations, or read QuantLib C++.
The [source-example report](M8_GILT_SOURCE_EXAMPLES.md), complete primary-page
visual review and earlier [52-family test proposal](M8_GILT_TEST_DESIGN.md)
remain separate evidence inputs.

## Corrections resolved in the complete draft

| Challenge | Selected correction and consequence |
|---|---|
| Numerical component omitted genuinely zero coupons, contradicting actual-payment identity | Retain each entitled actual zero coupon and observe its factor. Only a zero **source-coefficient bundle** has zero PV/risk. A zero final dividend still shares positive par100 and cannot remove redemption risk. Missing/detached dividends remain absent at their original quasi indices. |
| Cash accessor names differed across components | Select `cash_rounding()`, matching the existing settlement interface. One complete signature list is authoritative. |
| Quantized source views had incompatible error wrappers | Both public source views use `CashArithmetic`, `source dividend` or `source accrued`, endpoint `point`, with `quantize` Inexact/OutOfRange. Remove unused GiltError::Amount. The selected additions are eleven types and 26+5 domain errors, resulting in112/170 only after implementation. |
| AI sketches did not define all date variables and first-period transitions | Define the exact seven AI branches, original issue ratios r1/s1, long second interval r2/s2, actual-payment reset and skipped-quasi nonreset. Entitlement is Ceased, ExCouponDetached or BuyerEntitled in explicit order. |
| Pricing r/s could be confused with issue r1/s1 in cash formulas | Name issue ratios explicitly in both first publications and short AI. A short ex control gives -3/182, whereas substituting remaining settlement r gives the wrong +87/182. |
| Remaining-period inspector had an off-by-one ambiguity | For Qk<=S<Q(k+1), next index=k+1 and n=m-k-1, so the last interval has n=0. Sparse payments do not change this coordinate. |
| Amount-residual and clean/target overflow were claimed reachable | D>0 and T>0 exclude amount-residual overflow. Exact abs(AI)<2^127 and cached abs(AI)<=2^127 put clean/target shifts below MAX+2^970. Keep typed defenses; do not forge financial states for coverage. |
| exp absolute bound was unqualified for infinity | Apply the error comparison only to admitted finite-positive observations. Record zero/overflow classification separately; runtime rejects all nonfinite/nonpositive observations ordinarily. No universal primitive theorem follows. |
| Subnormal exp absolute error could invalidate a relative-only boundary argument | Include MIN_SUB/2 explicitly and derive the below-Z crossing band ln(1+4u)<8u. Keep the outward threshold/exponent/primitive allowances distinct. |
| AI admission and target-shift work order differed | Prepare the exact AI and its binary view once in the plan. Inversion reuses it and performs only its optional target shift before NonPositiveTarget, workspace and callbacks. |
| Source-AI cached rounding had no separate capacity ledger | Prove numerator/denominator below2^144 and alignment at most198bits in five words. Nonzero AI exceeds2^-143. Fix source derivation/publication/AI-round labels and original index meanings. |
| Allocation and defensive error strings were incomplete | Requested allocation is element slots, with phase-defined element meaning. Date uses Some(k)/quasi cycle. Residual uses None, yield residual/yield amount residual and residual difference/round residual. |
| Adoption gates required implementation evidence before a contract existed | Resolve source/API/math review before freezing; author independent expectations from the frozen contract before production; require full runtime/coverage/package evidence before implementation acceptance. |

No remaining blocker was found in the reviewed source/API/cash/math design
after these corrections. Actual implementation still must meet every retained
acceptance obligation. A proof about the selected algorithm does not validate
a different sequence of intermediate multiplications or floating roundings.

## Independently checked arithmetic and primitive evidence

The source-PV operation needs one IEEE rounding of an exact source rational
times an admitted discount. A normalized53-bit rounding followed by IEEE
subnormal admission can double-round. The exact `(100.5+5e-39)*MIN_SUB`
primitive control rounds to101 MIN_SUB; the double-rounded route gives100.
This is a legitimate source coefficient, but it is not claimed as an executed
public-plan case without establishing its actual observed discount.

The reviewers independently checked five-word scalar bounds, early IEEE
binade classification, subnormal clamping, quotient/remainder parity and
carry. The source PV's202bit alignment, risk scalar's176bit alignment and
cached AI's198bit alignment fit320bits. The source term bounds and count give
risk basis coefficients below2^9365; 243 words also cover the10187bit worst
normalization alignment. Implementation review must census helper temporaries
against the explicit logical-storage budget. This is not a compiler stack
frame guarantee.

For cash, eight words cover the source-only combined-decimal-power algorithm
(all intermediates below405bits). Forty words cover every finite binary64 or
admitted decimal endpoint using `R0=2^1074*5^38` and `Dc=R0*dayD`. The largest
final-scale component net coefficient is below2^2460, inside2560bits. This
depends on storing rounded components at the final quantum scale rather than
reconstructing them on Dc. There is no intermediate i128 range admission.

Actual pre-adoption scratch measurements are:

| Check | Observed result and qualification |
|---|---|
| Scaled log | 16,943 distinct valid quotes:10,695 tiny identities and6,248 ln/ln_1p cases. Decimal160/240 nearest-bit references agree; maximum error is1.1666666558012366 units of u*abs(g*), below the selected4u finite profile. |
| Strict exp profile | Reused12,459 native observations without rerunning them. All12,390 finite-positive values meet4u*exp(z)+MIN_SUB/2;36zero and33overflow observations separately match dual-precision IEEE classification. Maximum finite-budget fraction0.9999999999999003 occurs just above the half-MIN_SUB threshold. |
| Outward guard | Exact Fraction arithmetic checks10,913 nonzero normal exponents, including4,097 float neighbors about each threshold. All17 certified inner-band cases are rejected;26 total guard rejections. Both160/240 Decimal computations reproduce the threshold bits and2^-44 constant errors. |

These finite native measurements support conditional comparison profiles.
They do not establish uniform libm accuracy, native Linux/Windows behavior,
correctly rounded transcendental functions or analytical settlement enclosures.
No Rust gilt implementation test was executed during this design review.

| Scratch artifact | SHA-256 |
|---|---|
| `/private/tmp/sabiquant-gilt-log-profile-v1/exp-evidence-numeric-review-v3.json` | `5ac343ffecd7da27e16f2405d3e0d068383bca0da9ee52ed4e8298e8e1b38a12` |
| Strict exp checker | `b77bd766abef08ec91b39d8e6f57166df93800fab813964c432aeea4e8ba8775` |
| `/private/tmp/sabiquant-gilt-guard-review-v1.json` | `888f444b06c8ba787033c4737fc4710cb1ba32fbba6280c511e22cfe9382ef25` |
| Guard checker | `e82e95947d9608a590021dc1a4c60d329fe202997b516ed4258aff15b4ebd119` |

## Exact cash challenges and retained acceptance gates

Three source-valid Fraction controls were independently derived. They are
review controls, not a Rust fixture or implementation result:

- Long first example AI=-1/61, N=6100, q=1 Exact yields cash-1. A source view
  at q=1 Exact fails Inexact, while the final source cash succeeds. Substituting
  nearest binary AI would instead yield a nonintegral rational and fail Exact.
- A regular December7/June7 interval with S at its midpoint, explicit cutoff
  at issue and C=2^126 gives exact ex AI=-2^124. With N=1000 and supplied clean
  binary price2^124, component Exact at q=1 yields two opposite amounts outside
  i128 whose exact net is0. Final-only range admission must allow that zero.
- Dirty interval [2^100,next_up(2^100)], N=100, q=2^48 has divisible endpoints
  but continuous interior nonmultiples. Exact returns ExactStageUnresolved at
  cash total. Endpoint divisibility does not establish universal definedness.

The review confirms monotonic endpoint evaluation for all non-Exact modes,
point versus continuous Exact behavior, capital-before-accrued precedence,
final-only range admission, original declared-evidence provenance and compact
snapshot reproducibility. Equal cash does not erase definition identity.

After contract adoption, the independent oracle authors must cover exact
source/date/cash decisions, scalar/accumulator staging, conditional ideal
price/risk and target-driven inverse references before production is assessed.
Public hand tests must cover every accessor, reachable error/source chain,
zero-coupon factor order, solver traces/caps/cached payloads, real range
controls, threads and all52 proposed semantic families as applicable to the
selected explicit API. Unsupported Schedule adapters or arbitrary redemption
inputs have no fictitious runtime path in this API.

Full debug/release regression, strict lint/fmt/public+private rustdoc, policy,
cross-target compilation, production-region coverage with honest exclusions,
actual extracted-package checks and byte preservation of old contracts and
frozen evidence remain required. UK calendar maintenance, auction ownership,
signed legal tie/aggregation conventions, ideal-price enclosure producers,
Treasury/Canadian/other named models, native foreign-platform runtime and
financial/owner/release/full-M8 acceptance remain open.

## Verification of the documentation and adoption change

The pinned primary-example checker was actually rerun and passed all retained
source cells and zero-yield controls. Its new create-only report is
`/private/tmp/sabiquant-gilt-source-examples-contract-review-v1.json`, SHA-256
`fe6e6f9d09a49ec0ed075ed52fe387f3fe8837941b19e85b3bd3f802f25fae6e`.
The six changed/new documents have228 valid relative file links. The selected
contract error ledger has26 domain rows and the requirements register has
exactly24 consecutive FR-GILT rows. `git diff --check` passed.

All127 tracked Rust source/test files and110 pre-existing validation artifacts
were byte-compared against `0d1c987` and are unchanged. No prior contract was
edited. The complete new frozen contract has SHA-256
`03f0a7a96dd590bf3adb9fc2db26490866791efc6e2ed574ea6153dec595d292`.
The create-only check record is
`/private/tmp/sabiquant-gilt-contract-adoption-v1.json`.
No unchanged multi-million-row Rust regression suite was rerun for this
documentation-only change; its prior native evidence remains separately
recorded in the generic yield/risk validation report.
