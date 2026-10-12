# Local M6 numerical validation

This records the locally implemented numerical foundations on 2026-10-12,
following saved M5 commit `018b14c` under the owner's instruction to continue
locally with broad tests. [Readiness](M6_LOCAL_READINESS.md) records delegated
decisions; [numerical review](M6_LOCAL_NUMERICAL_REVIEW.md) separates findings,
fixes and independent review. The
[traceability audit](M6_TRACEABILITY_AUDIT.md) maps each local requirement and
numerical policy to executable evidence. No remote write, owner API approval, formal
milestone acceptance or publication is implied.

## Implemented contracts

`sabiquant::math` provides three bracketed solvers, four immutable
interpolators, six standard normal operations and an explicit location/scale
distribution, and two checked reductions. Four domain error types preserve
typed context through the root error. Fourteen new public values bring the
repository inventory to 49; trait checks cover all of them across the public
API/context suites. Root-error samples cover 58 distinct nested error kinds.

Solvers require finite explicit brackets/options and count every function and
derivative callback before invoking it. Brent uses inverse-linear/quadratic
interpolation with progress safeguards; Newton requires an explicit initial
point and has a bracket-preserving fallback. FunctionZero observes a callback
value, while BracketTolerance uses the pinned rounded width criterion. Neither
certifies continuity or callback accuracy; callback panics propagate.

Interpolation retains exact stored knots and explicit Reject/Flat/outer-segment
extrapolation. NaturalCubic is C2 with zero endpoint curvature; the named
MonotoneCubicHermite uses the pinned weighted-harmonic PCHIP/endpoint rules and
is C1. Natural overshoot and PCHIP extrapolation are distinct from in-domain
shape constraints. Both cubics prepare dimensionless coefficients; required
normalized nonzero quantities must retain supported relative precision.
Construction support errors do not assert that the exact mathematical spline
or its physical derivative overflows.

Normal tails/logarithms avoid subtracting a small probability from one.
Inverse uses centered mass near the median and logarithmic tail residuals
elsewhere, including subnormal probabilities. Finite explicit mean and
positive scale retain recoverable standardization, density and fused affine
cancellation. Rounded zero tails/densities and saturated probabilities are
valid. Unrepresentable logarithmic results and clear output overflows are
errors; unresolved density/affine-quantile boundaries use explicit guards.

Neumaier reduction validates all inputs before supplied-order arithmetic and
rejects a nonfinite partial sum, correction or final result. Exact reduction
accumulates integer multiples of MIN_SUB separately by sign, cancels exactly,
then rounds once, nearest with ties to even. Its capacity proof covers the
binary64 exponent range and every slice length; it remains permutation
invariant even after an overflowing floating-point prefix. Both return
canonical positive zero. It is not an exact dot product or a guarantee for
merging already rounded partial sums.

## Independent fixture evidence

The ten new math fixture files contain **343,260** rows. The unchanged 32
M2–M4 files contain 598,564 rows: **941,824 total**. These counts measure
declared scenarios, not independent features, branch coverage or exhaustive
real input spaces. Rust tests read fixtures offline with only std. Python
3.9.6 is external regeneration tooling and is excluded from the package.

| Layer | Rows | Independent construction and scope |
|---|---:|---|
| Sums | 105,307 | Exact Python Fraction values and one correctly rounded rational-to-float conversion; every exponent, halfway/sticky transitions, 33-limb carry/borrow, cancellation, seeded vectors and final overflow ties |
| Interpolation | 99,072 | Exact dense polynomial constraints for natural cubic, exact Hermite constraints for PCHIP, rational linear and 120/200-digit logarithmic values; four grids, 2–5 nodes, Cartesian ordinates, exact knots, interior and extrapolation |
| Solver tolerance | 97,851 | Exact Fraction rounding of each declared operation and separate real-width allowance; all three methods checked at a strict two-callback cap |
| Polynomial roots | 21,084 | Independent 120/200-digit square/cube roots, 251 binary scales, translated and negative intervals; 63,252 solver executions with independent callback counts |
| Normal | 19,946 | Six operations; independently selected center series, separated continued-fraction convergents, inverse bisection and exact supplied affine inputs at 120/200 digits; density/quantile boundary tiers |

All five committed generators reproduce their final fixtures with `--check`.
The tolerance-ID header updates changed no fixture data row: four before/after
fixture-body SHA-256 comparisons match. Generators record their exact hashes,
precision/environment and preserved generation timestamps under `--check`.
The normal generator's center/tail overlap and mandatory convergent separation
checks remain independent of Rust's crossover and Newton iteration.
Two-precision agreement is empirical evidence, not an interval certificate.

## Accuracy and stopping measurements

The operation-specific profiles are pinned in
[numerical policy §12](NUMERICAL_CORRECTNESS.md#12-local-m6-operation-profiles),
with complete equations and primary-source limits in
[solver](M6_SOLVER_RESEARCH.md),
[interpolation](M6_INTERPOLATION_RESEARCH.md) and
[normal](M6_NORMAL_RESEARCH.md) research. Existing budgets were not increased
to accommodate the corrected cubic arithmetic or guard observations.

Percentiles use the sorted zero-based element `floor(n*p/100)`. The following
are absolute error divided by the declared conditioned budget. Interpolation
uses 18,216 non-knot values per method; the 6,552 exact knots per method are
checked separately with zero-ULP equality. Root ratios use 21,084 per method.

| Operation | p50 | p95 | p99 | Maximum |
|---|---:|---:|---:|---:|
| Linear | 0 | 0 | 0 | 0 |
| LogLinear | 0 | 0.060670 | 0.060670 | 0.060670 |
| NaturalCubic | 0 | 0.018567 | 0.027397 | 0.045341 |
| MonotoneCubicHermite | 0 | 0.020512 | 0.034162 | 0.076923 |
| Bisection root | 0.136448 | 0.232763 | 0.250879 | 0.250879 |
| Brent root | 0 | 0.002144 | 0.004216 | 0.004216 |
| Safeguarded Newton root | 0 | 0.002144 | 0.008026 | 0.008026 |
| Compensated sum | 0 | 0.250000 | 0.250000 | 0.250000 |
| Normal PDF | 0.017892 | 0.126645 | 0.144712 | 1.000000 |
| Normal CDF | 0 | 0.038693 | 0.143927 | 0.503746 |
| Normal survival | 0 | 0.039040 | 0.125000 | 0.503746 |
| Normal log CDF | 0 | 0.181220 | 0.269837 | 0.486725 |
| Normal log survival | 0 | 0.180024 | 0.269837 | 0.486725 |
| Normal inverse | 0 | 0.111111 | 0.140522 | 0.217865 |

Exact reduction has **105,295 zero-ULP finite values** and 12 signed final
overflow errors. Compensation has 105,288 accepted finite values and seven
named finite-final-sum prefix errors. The generator independently proves each
first overflowing prefix; the consumer checks its exact index, quantity,
infinity sign and source. A separate both-sign regression distinguishes final
corrected overflow from partial overflow. Its 105,288 finite error ratios
supply the reported compensated distribution; exact finite values have zero
ULP error throughout the independent grid.

Solver operational decisions contain 97,108 full-width and 743 overflow-half
rows: 59,429 accepts and 38,422 rejects. There are 6,734 rounded accepts outside
the exact real criterion and three rounded rejects inside it. Every accepted
case satisfies the declared real rounding allowance; no configured tolerance
is changed. Root-grid maximum shared callback counts are **46 / 38 / 97**.
Trace tests independently demonstrate genuine IQI, rejected otherwise-valid
proposals and historical progress safeguards. Tests also cover all caps 1–80,
nonfinite callback stages, adjacent coordinates and discontinuous callbacks.
An independent continuous piecewise-linear callback has MAX-scale endpoint
residuals and MIN_SUB-scale interior residuals. Its public trace checks the
midpoint fallback when common scaling erases distinct interpolation ordinates,
while retaining finite in-bracket calls and the exact central root.

The two new PCHIP analytic refinement tests cover **45** preparations,
**13,422** approximation queries, **75,888** interval shape/direction queries
and **4,509** exact knot checks. Smooth `x³+x` has observed third-order behavior
on uniform and graded meshes, but second-order on fixed alternating 3:1
widths. Aligned/unaligned quadratic extrema retain the independent `H²/4`
envelope; unaligned halving ratios range roughly 1.96–15.17, so no universal
rate is asserted. All pinned approximation errors remain over 393,000 times
the gross-scale rounding proxy. These studies separate approximation from
rounding and do not enlarge the evaluation budgets.

Normal ordinary-domain rows have strict statuses. The density conditional
edge tier has 284 rows (80 values/204 errors, 11 declared/observed guard-status
differences); affine inverse has 430 (93 values/337 errors, 53 differences).
True rounded overflow never permits a finite result. True finite edge values
may be an accurate finite observation or NumericalBoundary, never an ordinary
overflow error. Every allowed finite edge observation remains accuracy checked.
The quantile guard uses the actual `MAX+2^970` overflow midpoint, a quarter-
scaled FMA residual and exact reduction; safe MAX means and exact medians
are retained. Log errors preserve negative-infinity cause context.

Raw maximum interpolation ULP distances are **0 / 1 /
4,368,491,638,549,381,120 / 144**. The large natural distance is to a near-zero
exact value, where ULP/relative accuracy is uninformative. Normal raw maxima
range from 79 to 1,181; the largest normalized PDF ratio is a MIN_SUB rounding
floor case. Neither layer claims uniformly few-ULP composite/transcendental
accuracy. Rust elementary functions retain unspecified precision, even
within one execution; native platform evidence is required for R3.

## Explicit fused operation order

These are intentional NP-06/NP-11 specifications of the implemented order,
not implicit optimizer contraction. `fma(a,b,c)` means `a.mul_add(b,c)`;
other products/additions remain separate rounded operations.

| Path | Implemented order |
|---|---|
| Interpolation logarithmic blend | Opposite-signed logs: `fma(1-t,left,t*right)`; same sign with `t<=1/2`: `fma(t,right-left,left)`; otherwise `fma(t-1,right-left,right)`, then exp |
| Cubic Horner and ordinate restoration | `p=fma(fma(a,t,b),t,c)`; scale one uses `fma(p,t,d)`, otherwise `fma(S,p*t,d)` |
| Natural back-substitution | `fma(-upper[i],curvature[i+1],rhs[i])`, checked for supported normalized precision |
| PCHIP endpoint candidate | Compute separately rounded weight and `-weight*next`, then `fma(first,1+weight,-weight*next)` before sign/three-secant clipping |
| Tail inverse Newton proposal | `fma(log_residual,Mills_ratio,x)` before finite bracket/progress selection |
| Affine inverse and boundary residual | `fma(scale,z,mean)`; quarter-product `p=scale*(z/4)`, residual `fma(scale,z/4,-p)`; exact reduction evaluates the signed midpoint gap. Boundary diagnostic uses `fma(scale,z/4,mean/4)` |

Supplied-order Neumaier operation order follows its two documented correction
branches. Exact reduction uses checked input decomposition and integer limbs;
neither algorithm relies on reassociation or fast-math.

## Native checks and coverage

The source-frozen aggregate passes **389 debug tests / 388 release tests**,
including **22 doctests** (19 ordinary and three compile-fail), over 41 result
groups with zero failures or ignored tests.
The debug-only internal invariant check explains the one-test difference.
Formatting, strict all-target Clippy and warning-denied public/private
documentation pass. All-target Linux GNU and Windows MSVC compile checks pass;
they do not execute native tests or establish cross-platform CI.

Native LLVM instrumentation in `/private/tmp/sabiquant-m6-closed-coverage`
excludes embedded test modules and unions source regions across instantiations:

| M6 production source | Covered / regions | Coverage |
|---|---:|---:|
| Interpolation | 924 / 971 | 95.16% |
| Normal | 479 / 498 | 96.18% |
| Solver | 655 / 655 | 100% |
| Summation | 256 / 258 | 99.22% |

New finite public-input boundary tests cover unsupported normalized cubic
ratios, natural/PCHIP construction failures, extreme policy ordering,
recovered affine extrapolation and separate signed/log result domains.
Remaining uncovered regions include defensive error propagation, pathological
construction paths, normal convergence defenses,
compensation-correction overflow and the proof-excluded zero-magnitude
rounding fallback. Coverage measures source regions, not branches, exact-real
range certification or every possible input. Private invalid states are not
fabricated to inflate it. M2–M5 date/calendar/day-count/rate coverage and
unchanged fixtures retain their separately documented scope.

## Package and remaining gates

The offline package has **31 files**, **120,303 compressed bytes** and
515,880 content bytes. SHA-256:
`7ef39aa9ea5f0da9916e3a3fc8e46a169bb3e75764f00901eac1714bd7010bd4`.
It captures `018b14c` plus dirty local M6 changes, not a clean release tag.
All 29 original archive files match the frozen workspace, including the
original manifest/readme/changelog; the normalized manifest and VCS metadata
are generated package files. Extracted verification passes **73 unit tests**,
**22 doctests**, strict all-target Clippy and warning-denied public/private
documentation. The manifest remains version `0.0.0`, standard-library-only and
`publish=false`; integration suites, external generators/references and all
fixtures remain excluded. No release artifact or remote availability is
claimed from a local archive.

Native Linux/Windows execution and fresh primary-platform CI remain open.
Public QuantLib solver/interpolation probes in the research records identify
specific distinctions, not full M6 parity; the mathematical fixtures are
independent oracles. Formal dependency acceptance, independent owner API
review, security-setting verification and an explicit release decision remain
pending. The earlier US 30/360/AFB source questions, original ISDA/QR-01
inspection and TARGET maintainer/annual review are still unresolved. M6 is
locally reviewable, not declared formally complete.
