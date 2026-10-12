# Local M6 requirement-to-test traceability audit

Read-only audit by the solver agent `/root/period`, 2026-10-12. Source, tests,
generator definitions and the local requirements were inspected; this report
changes none of them. “Local evidence” below describes executable coverage of
the stated scope, not formal milestone acceptance, native Linux/Windows
execution or a universal certificate for Rust elementary functions.

## Requirements and executable evidence

| Requirement / promise | Status and executable evidence | Limits of the conclusion |
|---|---|---|
| FR-MATH-01: explicit finite brackets, bounded callbacks and contextual termination | Local evidence: [math_solvers.rs](../tests/math_solvers.rs), especially `callback_counts_never_exceed_any_shared_budget`, `newton_shared_budget_distinguishes_initial_derivative_and_trial_cost`, all nonfinite evaluation stages, endpoint/initial cache tests and callback-panic propagation. Instrumented caps 1–80 cover all three public methods. | A callback is responsible for continuity, accuracy, determinism and its own panic. Endpoint initialization can stop with one observed zero; an unevaluated opposite sign must not be invented. |
| FR-MATH-01: positional accuracy and actual fast-method safeguards | Local evidence: 2,667 exact dyadic solves, continuous unrepresentable adjacent-float roots, sign jumps, full-range contraction to MIN_SUB, zero/wrong/repeating Newton fallbacks and `brent_public_trace_contains_secant_iqi_and_two_step_history_rejection`. [math_solver_roots.rs](../tests/math_solver_roots.rs) checks 21,084 independent two-precision roots against three methods; [math_solver_oracles.rs](../tests/math_solver_oracles.rs) checks 97,851 exact rational stopping cases against three methods. [math_solver_scaling.rs](../tests/math_solver_scaling.rs) observes midpoint fallbacks when distinct small ordinates collapse after normalization by MAX. | Callback zero is not a mathematical-root certificate. The root-location oracle includes declared callback/reference conditioning. The stopping rule is a rounded operational comparison, not exact real interval arithmetic; no residual-only success or hidden epsilon floor is tested as acceptable. |
| FR-MATH-02: checked immutable nodes, explicit methods/policies and identity paths | Local evidence: [math_interpolation.rs](../tests/math_interpolation.rs) tests catalogs/parsing, indexed invalid pairs, exact knots, signed zero, nonfinite queries, all method/policy pairs, constants, two-node reductions, definition equality and concurrent/reversed query reuse. | Equality describes the supplied definition, not byte equality of recomputed elementary coefficients. Exact knots/Flat/constant paths have stronger E0 checks than ordinary evaluation. |
| FR-MATH-02: NaturalCubic/PCHIP definitions, shape and numerical support | Local evidence: 99,072 rational/Decimal cases in [math_interpolation_oracles.rs](../tests/math_interpolation_oracles.rs), separate natural/PCHIP/nonuniform goldens, recovered C1/C2 and endpoint curvature, 625 shape grids with derivative-quadratic extrema, normalized scale regressions. [math_interpolation_boundaries.rs](../tests/math_interpolation_boundaries.rs) checks lost/subnormal required normalized quantities, typed interval/quantity/source context, overflowing differences, extreme queries, real output overflow and subnormal outputs. [math_interpolation_refinement.rs](../tests/math_interpolation_refinement.rs) adds nine analytic PCHIP studies at five sizes. | Natural cubic may overshoot. PCHIP shape promises apply inside segments, not extrapolation. Supported normalized grids are a separate domain from the conditioned evaluation budget; a support error does not prove mathematical output overflow. Dense exact constraints use small node sets; mesh-specific approximation rates do not imply a uniform few-ULP, arbitrary-grid accuracy or universal cubic-order claim. |
| FR-MATH-03: six normal operations and explicit location/scale | Local evidence: [math_normal.rs](../tests/math_normal.rs) checks every public nonfinite-input boundary, location/scale/probability domains, exact median, median neighbors, every binary probability exponent and 512 low subnormals, logarithmic range, recoverable affine density/quantile cancellation, selected Decimal anchors, thread reuse and independent Simpson mass checks. [math_normal_oracles.rs](../tests/math_normal_oracles.rs) checks 19,946 two-precision cases across six operations. | Forward/inverse round trips alone would share errors; they supplement the external oracle and integral relation. Saturated probabilities/density zero are meaningful rounding, while logarithmic outputs must be finite. Small probability support does not make the functions correctly rounded. |
| FR-MATH-03: density/affine range classification | Local evidence: explicit midpoint/nearby/safe-MAX tests and independently classified density/affine guard-edge rows. Allowed finite observations remain conditioned-accuracy checked; true rounded overflow permits only the documented numeric or boundary errors. Both signed affine boundaries are represented. | Guard-edge bands and quantile allowances are local measured contracts. Increasing Decimal precision agreement and convergence bounds do not prove all Rust `ln`/`exp` behavior on every platform. |
| FR-MATH-04: supplied-order compensation and exact one-rounding reduction | Local evidence: [math_sums.rs](../tests/math_sums.rs) consumes 105,307 exact Fraction cases, pins seven independently proved prefix failures, checks exact overflow sign/quantity, final compensated correction overflow, nonfinite-input precedence, 8,000 severe-cancellation permutations, finite singleton/cancellation bit patterns and million-element reductions. [sum_oracles.py](../validation/generator/sum_oracles.py) includes all exponents, tie perturbations and 33-limb carry/borrow chains. | Compensation has no universal correct-rounding or permutation guarantee. The exact accumulator's maximum slice-length capacity and defensive unreachable paths are justified by the independent proof in [M6_LOCAL_NUMERICAL_REVIEW](M6_LOCAL_NUMERICAL_REVIEW.md), not by allocating a maximum-size slice. Inputs are supplied binary64 values; exact products or means are separate operations. |
| FR-MATH-05: finite/domain policy, stable contextual errors and immutable thread-safe values | Local evidence: each suite checks public failures/results; the root [error tests](../src/error.rs) sample 58 distinct nested variants with transparent display/source and stable unique kinds. [public_api.rs](../tests/public_api.rs) contains all 14 new math value/error trait assertions; 49 total declarations are covered jointly with contextual ICMA/day-count/TARGET checks. Policy, lint and docs checks remain executable aggregate gates. | Trait checks are compile-time guarantees for values, not a promise that arbitrary caller callbacks are thread-safe. Coverage and error catalogs do not establish numerical accuracy outside the tested/declared domains. |

## Numerical-policy mapping

| Policy | M6 evidence / qualification |
|---|---|
| NP-01 finite inputs; NP-07 no NaN output | All NaN/±infinity classes are exercised at solver constructors/callback stages, interpolation nodes/queries and each standard/context normal API; both reductions retain input index/class. Oracle consumers reject nonfinite successful values. |
| NP-02 meaningful finite in-domain output | Positive LogLinear outputs versus signed cubic outputs are tested separately. Normal probability/density saturation and finite nonpositive logs have distinct domains. Exact-sum overflow midpoint and normal conditional range tiers are checked without permitting a false finite observation. |
| NP-03 stable formulations | Direct survival/log tails, centered inverse near the median, affine density/quantile recovery, exponent-scaled linear contributions, normalized cubic coefficients and exact cancellation reductions have public counterexamples. This is focused stability evidence, not proof of optimal conditioning everywhere. |
| NP-04 exact discrete arithmetic; NP-05 year-fraction rounding | M6 introduces no date or day-count computation and does not change the existing year-fraction contract. Callback counters/caps are bounded integers; exact-sum limbs use explicit carry/borrow with a capacity proof. |
| NP-06 explicit FMA; NP-11 operation order | Solver full-width/overflow-half order and exact-sum extraction/rounding are pinned. [M6_NUMERICAL_VALIDATION's explicit fused-order table](M6_NUMERICAL_VALIDATION.md#explicit-fused-operation-order) now specifies interpolation fused Horner/scale restoration, log blending, spline back-substitution/endpoint slopes, normal log-tail Newton and affine/boundary residual paths. The reviewer checked that table against the actual source; the documentation gap is closed without changing numerics. |
| NP-08 signed zero | Constructor/node canonicalization, exact/compensated zero, observed callback zero, normal mean/median/log-zero and subnormal interpolation scenarios have bit checks. |
| NP-09 supplied order; NP-10 determinism | Compensation retains supplied order; exact sum is independently permutation invariant. Immutable interpolation/distribution reuse is checked across clones/threads. Solver traces repeat for deterministic test callbacks. Elementary repeated results use conditioned checks where precision is unspecified; no global-state or thread-order source was introduced. |
| NP-12 explicit real branches | Exact per-operation Fraction stopping oracle separates real and operational tolerance. PCHIP direct sign/plateau conditions, interpolation numerical-support boundaries and normal narrow range tiers are tested. None of these branches authorizes increasing a numerical allowance to hide a discrepancy. |

The operation-specific profile IDs and scopes are now pinned in
[NUMERICAL_CORRECTNESS §12](NUMERICAL_CORRECTNESS.md#12-local-m6-operation-profiles).
The completed metadata-only profile-label regeneration is recorded in
[M6_NUMERICAL_VALIDATION](M6_NUMERICAL_VALIDATION.md#independent-fixture-evidence):
four before/after fixture-body hashes match and all four affected generators'
`--check` runs pass. The normal generator's final `--check` also passes.
Those checks attest reproducible fixtures, rather than universal accuracy.

## Independence and feasible follow-ups

The largest fixtures inspect public behavior rather than importing private
production helpers. The exact-sum oracle retains rational inputs independently
of the limb design. Natural-spline oracles solve dense polynomial value/C1/C2
constraints, independently of the normalized tridiagonal preparation. PCHIP
and log interpolation necessarily share the pinned mathematical definition,
but compute rational/Decimal results without reusing Rust coefficient code.
Normal oracles use independently converged arbitrary-precision identities,
adjacent tail-convergent bounds and quantile bisection; independent integration
and probability identities supplement them. Solver root oracles solve the
polynomial analytically at two precisions. The tolerance oracle deliberately
implements the documented stopping policy with exact rational rounding: it
tests that policy, rather than serving as an independent solver algorithm.

The audit found no new arithmetic defect. Two feasible coverage/documentation
gaps were sent promptly to the root and are **resolved locally**:

1. The two frozen tests in
   [math_interpolation_refinement.rs](../tests/math_interpolation_refinement.rs)
   separate strictly monotone cubic behavior on uniform, smoothly graded and
   abrupt 3:1 alternating meshes, and aligned/unaligned quadratic maxima.
   Across 45 preparations they inspect 13,422 analytic approximation queries,
   75,888 shape observations and 4,509 E0 knots. The selected monotone function
   shows mesh-specific third-order uniform/graded and second-order abrupt
   refinement. Quadratic maxima use the separate `H²/4` envelope plus a named
   rounding floor; alignment/clipping is explicit. Quarter-sample measurements
   are not a supremum certificate or a universal PCHIP convergence order.
   The owning agent's debug/release/strict-Clippy checks pass; this reviewer
   independently reran both debug tests successfully.
2. The source-matching fused/order table linked above is complete. Its
   specifications distinguish separately rounded products from explicit
   `mul_add`, including the two cubic ordinate-restoration paths.

The reviewer also inspected and independently ran the new public scaling
regression successfully. Its continuous piecewise-linear callback has MAX
endpoint residuals and only a few MIN_SUB centrally. The first midpoint and
two further midpoint fallbacks are recorded through the callback, and the
known `5/8` zero, finite arguments and actual callback counts are checked.
This is a feasible numerical-degeneracy case, not a call to a private helper
whose implementation is repeated by the test.

Useful limits that should remain visible instead of spawning artificial tests:
unreachable defensive accumulator paths; reductions long enough to overflow
the compensation accumulator; arbitrary ill-conditioned cubic grids excluded
by the declared support boundary; and normal convergence/elementary precision
outside proved truncation or measured domains. These do not justify loose
fallback epsilons, silently accepted errors or implementation-mirroring tests.

This record incorporates the completed follow-up tests and reproducible
metadata. Native Linux/Windows execution, formal owner API/milestone review,
source/maintenance and publication gates remain open. Local tests, coverage,
cross-compilation and packaged-source verification are different evidence and
must retain their names.
