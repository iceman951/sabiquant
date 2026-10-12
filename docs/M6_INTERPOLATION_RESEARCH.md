# Local M6 Interpolation Research and Contract Proposal

This is agent research dated 2026-10-12 for F-MATH-02 in the
[feature matrix](FEATURE_MATRIX.md) and [M6 roadmap](ROADMAP.md). It proposes a
reviewable implementation contract; it is not owner approval, implemented
behavior, M6 acceptance or cross-platform evidence. No C++ implementation was
read, copied or translated. QuantLib observations below use its public Python
API only. The initial implementations should be derived from the mathematical
definitions and checked against independent arithmetic.

## Primary evidence

| Source inspected | Establishes | Limits |
|---|---|---|
| [NIST DLMF §3.3, equations 3.3.14–15](https://dlmf.nist.gov/3.3.E14) | Linear interpolation is the two-node affine combination; the smooth-function error has coefficient 1/8 times interval width squared and a second-derivative bound. | Approximation error differs from floating evaluation error. The bound assumes the stated smoothness. |
| [Fritsch–Carlson, 1980, DOI 10.1137/0717021](https://epubs.siam.org/doi/10.1137/0717021) | Original publication develops monotonicity conditions for piecewise cubic interpolation. | Publisher metadata/abstract inspected; full publisher PDF redirected to restricted access. No full-paper inspection or formula extraction is claimed. |
| [Fritsch–Butland, 1984, DOI 10.1137/0905021](https://epubs.siam.org/doi/10.1137/0905021) | Original publication concerns a local monotone cubic Hermite construction. | Same full-text limit. Its name alone does not pin every implementation's weights or endpoint rule. |
| [Cleve Moler, Numerical Computing with MATLAB, interpolation chapter](https://www.mathworks.com/content/dam/mathworks/mathworks-dot-com/moler/interp.pdf) | Author's description of Hermite interpolation, weighted harmonic interior slopes, and one-sided endpoint treatment; distinction from a twice-continuously-differentiable spline. | Mathematics supplies the specification; no MATLAB program is to be translated. Endpoint formula below can be derived by differentiating the three-node quadratic and imposing shape limits. |
| [Official SciPy PCHIP documentation](https://docs.scipy.org/doc/scipy/reference/generated/scipy.interpolate.PchipInterpolator.html) | Explicit interior weights, zero slopes at sign changes/plateaus, C1 smoothness and shape preservation; cites Fritsch–Butland and Moler. | This is the primary contract for that named software scheme, not evidence that QuantLib's similarly named scheme is identical. Retrieved manual identified v1.18.0. |
| [Official SciPy cubic-spline documentation](https://docs.scipy.org/doc/scipy/reference/generated/scipy.interpolate.CubicSpline.html) | Natural means zero endpoint second derivatives; clamped and not-a-knot are different conditions. Construction boundary conditions and extrapolation are separate choices. | No default boundary condition should be inferred for SabiQuant. |
| [Rust f64 documentation](https://doc.rust-lang.org/std/primitive.f64.html#method.ln) | Basic/fused arithmetic and elementary-function precision have different guarantees. | `ln`, `exp` and related precision is unspecified, including within an execution; calibration is required. |

These sources define numerical interpolation, not a market curve convention.
Interpolation of discount factors, zero rates and forward rates remains an
explicit later curve choice. Positive nodes do not impose a discount-factor
upper bound of one: negative-rate markets can have factors above one.

## Proposed public API

Use an owned immutable value rather than borrowed node lifetimes or an open
user-implementation trait:

```text
InterpolationMethod::{Linear, LogLinear, NaturalCubic, MonotoneCubicHermite}
Extrapolation::{Reject, Flat, ExtendBoundarySegment}

Interpolator::new(nodes: Vec<(f64, f64)>,
                  method: InterpolationMethod,
                  extrapolation: Extrapolation)
    -> Result<Interpolator, InterpolationError>
Interpolator::value(&self, query: f64) -> Result<f64, InterpolationError>
Interpolator::{nodes, domain, method, extrapolation}(&self) -> borrowed/copy views
```

The method and policy catalogs should be non-exhaustive, `Copy/Eq/Hash`, with
exact identifiers and no `Default`. The value should be `Clone/Debug/PartialEq`,
`Send/Sync/'static`, with private fields and no mutable last-segment cache.
Equality compares canonical input pairs, method and policy; it should not
depend on independently recomputed `libm` coefficients. Do not add floating
`Ord` or `Hash`. Derivatives, primitives, periodic/not-a-knot/clamped splines,
log-cubic methods, multidimensional interpolation and user callbacks are
separate scope decisions, not implied by this first value API.

Construction is O(n) time/storage, including cubic preparation; queries locate
a segment in O(log n) and evaluate it in O(1). Node pairs eliminate separate
array-length mismatches. Require at least two pairs, finite x/y, and strictly
increasing x; do not sort, merge duplicates, pad missing values or infer units.
Canonicalize signed zeros, so −0/+0 abscissas are duplicates. LogLinear requires
strictly positive ordinates. All methods accept nonmonotone data; the monotone
method preserves each data segment's shape rather than requiring one global
direction.

Validate query finiteness before any node identity or extrapolation shortcut.
At an exact stored abscissa return its canonical stored ordinate directly,
including the last node. Constant segments return their stored value. These
are E0 identities even for LogLinear: `exp(ln(y))` is not an exact-node policy.
Two-node NaturalCubic and MonotoneCubicHermite reduce to the ordinary linear
interpolant; they should not require a representable derivative merely to
return the two values or their finite interpolation.

Proposed non-exhaustive error variants/kinds:

| Variant | Structured context / stable kind |
|---|---|
| `NotEnoughNodes` | actual/minimum counts; `interpolation.not_enough_nodes` |
| `InvalidNode` | index and typed `ValueError` source, whose parameter names abscissa/ordinate and domain; `interpolation.invalid_node` |
| `NotStrictlyIncreasing` | interval/index and offending finite adjacent x values; `interpolation.not_strictly_increasing` |
| `OutsideDomain` | finite query and lower/upper bounds; `interpolation.outside_domain` |
| `NumericalGrid` | interval, computed quantity and typed numeric cause; `interpolation.numerical_grid` |
| `Value` | typed query/result numeric cause; `interpolation.value` |

Errors implement `Error/Display/Debug/Clone/PartialEq`, retain typed causes and
have non-exhaustive structured variants. A `NumericalGrid` failure means a
declared algorithm cannot represent a required intermediate. It must not be
described as proof that the mathematical interpolated value overflows.

## Pinned methods

For one interval, write `h=x1−x0`, `t=(q−x0)/h` in mathematical real arithmetic.

- **Linear:** `p=(1−t)y0+t y1`. Signed ordinates and exact zero are valid.
- **LogLinear:** `p=exp((1−t)ln(y0)+t ln(y1))`. This defines geometric
  interpolation, not a spline or a linear discount-rate interpolation.
- **NaturalCubic:** the unique C2 piecewise cubic through the nodes, with
  zero second derivatives at the first and last node. It is not guaranteed
  monotone, positive, convex, or free of overshoot.
- **MonotoneCubicHermite:** pin the weighted-harmonic PCHIP scheme described
  below; C1 with possibly discontinuous second derivatives. This is not
  QuantLib `MonotonicCubicNaturalSpline` and must not be labeled as such.

NaturalCubic can be derived independently by integrating a linear second
derivative on each interval and matching first derivatives. If `M_i` is the
knot second derivative and `δ_i=(y_(i+1)−y_i)/h_i`, interior equations are

```text
h_(i−1) M_(i−1) + 2(h_(i−1)+h_i) M_i + h_i M_(i+1)
    = 6(δ_i−δ_(i−1)),       M_0=M_(n−1)=0.
```

A tridiagonal solve gives O(n) preparation; its equations are the definition,
not a requirement to use QuantLib's operation order. The local prototype
sets `H=max(h_i)`, `S=max(|y_(i+1)−y_i|)`, `r_i=h_i/H` and
`d_i=(y_(i+1)−y_i)/S`, then solves for dimensionless `K_i=M_i H²/S`:

```text
r_(i−1) K_(i−1) + 2(r_(i−1)+r_i) K_i + r_i K_(i+1)
    = 6(d_i/r_i−d_(i−1)/r_(i−1)),       K_0=K_(n−1)=0.
```

Segment curvature contributions are `K_i r_i²/6`; the polynomial retains
dimensionless change coefficients and its original base ordinate. `S` is
restored at query evaluation, avoiding premature physical-curvature or
subnormal-coefficient rounding. Compare with an independently solved rational
constraint system. Natural endpoints do not make the outer
polynomial linear: zero second derivative at a point is not zero curvature
throughout an extrapolated interval.

For the proposed monotone scheme, let `h_i>0` and secants `δ_i` be as above.
At an interior knot set `m_i=0` when either neighboring secant is zero or their
signs differ. Otherwise use

```text
w1 = 2 h_i + h_(i−1),       w2 = h_i + 2 h_(i−1),
m_i = (w1+w2)/(w1/δ_(i−1)+w2/δ_i).
```

The left provisional endpoint derivative is
`m0=((2h0+h1)δ0−h0δ1)/(h0+h1)`; the right formula is mirrored. Set an endpoint
slope to zero if its sign differs from its adjacent secant. If the next secant
has the opposite sign and the endpoint candidate's magnitude exceeds
`3|δ_adjacent|`, limit the
endpoint slope to `3δ_adjacent`. Two nodes use the single secant at both ends.
This derivative and clipping rule is part of the named method, not an
optional undocumented adjustment. Compare signs directly; do not multiply
secants merely to test their signs, because the product can overflow/underflow.

Evaluate the resulting Hermite polynomial on normalized t, using endpoint
values and interval-scaled slopes. The basis is uniquely determined by its
endpoint values and derivatives; derive it from those four constraints.
Scale positive h weights before forming weighted harmonic means so that
`2h_i+h_(i−1)` does not overflow needlessly. Preserve plateaus and local extrema;
do not repair natural-spline overshoot by silently clipping its output.

## Extrapolation and numerical support

`Reject` errors outside the closed node domain. `Flat` returns the respective
stored endpoint value. `ExtendBoundarySegment` evaluates the same first/last
segment formula beyond its interval, including the outer cubic for cubic
methods and its log affine function for LogLinear. No monotonicity/positivity
claim follows for cubic extrapolation. LogLinear still requires a finite,
strictly positive result; overflow or rounding to zero is a typed failure.
Any optional future linear-tangent extrapolation needs a distinct policy name.

Finite increasing x values do not guarantee representable differences,
secants or coefficients. The first implementation contract should state its
supported numerical grid explicitly:

| Case | Required response or implementation decision |
|---|---|
| `x=[0,min_subnormal]`, `y=[0,1]` | Endpoint queries are finite although a cached secant is infinite. Linear/log-linear must not require cached slopes. |
| `x=[−MAX,MAX]`, query 0 | The real weight is 1/2 although direct h overflows. Use a safely scaled coordinate difference, or return an explicitly declared support error; never a silent zero weight. |
| Opposite signed extreme y | `y1−y0` may overflow although the convex result is finite. In-domain evaluation needs a sign-aware convex combination/scaling. |
| Very small normalized weight, large y span | A weight rounded to zero/subnormal can lose a representable contribution. Calibrate a rescaled multiply/divide path or account explicitly for weight quantization; a blanket relative error bound is false. |
| Positive log nodes spanning min-subnormal to MAX | `y1/y0` overflows. Blend logarithms rather than forming that ratio; exact nodes and constant segments bypass exponentiation. |
| Tiny/huge cubic intervals | A finite value can have an unrepresentable derivative, `h²M`, or polynomial coefficient. Prefer normalized scaled data; otherwise use `NumericalGrid`, not a fabricated mathematical-overflow claim. |
| Widely unequal adjacent h | Normalize harmonic weights and spline rows; track conditioning. A computed zero pivot or lost nonzero secant is a supported-grid failure, not a duplicate-node error. |
| Far segment extrapolation | Intermediate weight/derivative products can overflow even when a rearranged answer is finite. Specify support and typed failures; Flat remains an exact bounded alternative. |

The initial physical-secant restriction was superseded by the dimensionless
prototype. Both cubic methods require finite positive adjacent widths and
finite ordinate differences. Required nonzero normalized widths, ordinate
differences, slopes, curvature/slope contributions and change coefficients
must retain normal binary64 relative precision; a lost or subnormal required
quantity is `NumericalGrid`. The natural solve also requires finite pivots and
intermediates. Physical secants and curvatures are not formed and need not be
representable. PCHIP applies its stated weights/endpoint rules to normalized
widths and secants, with the same `S` restored during evaluation.

Constant/two-node identity paths avoid unused cubic intermediates. Full-range
cubic support is not established: extreme nonuniform ratios or unsupported
required coefficients still reject even when individual mathematical answers
are finite. This numerical-support condition is distinct from duplicate-node
validation and does not impose an arbitrary epsilon node spacing.

Result validation checks the rounded binary64 computation, not the exact-real
range of the mathematical interpolant. For example, extending Linear nodes
`[(0,MAX),(1,MAX/2)]` to `−min_subnormal` yields finite MAX although the real
value is slightly above MAX; the tiny increment rounds away. A coarse query
`−1` clearly overflows and is rejected. No exact-real range certificate is
claimed for an accepted finite result or a numerical-support failure.

## Accuracy and independent tests

Separate the error of evaluating the mathematically defined interpolant from
the error of approximating an underlying function. Use E0 for exact nodes,
all-constant data and Flat boundaries. Equal adjacent ordinates make a linear
or log-linear segment constant; a natural cubic between equal ordinates can
overshoot and is not an identity path. Linear evaluation is composite rational/basic
arithmetic, with an absolute condition scale built from weighted ordinate
magnitudes, weight uncertainty times ordinate difference, and subnormal
quantization. Cancellation can make output-relative ULP claims meaningless.
LogLinear adds elementary error: if `L=(1−t)l0+t l1`, perturbations include
`(1−t)|Δl0|+t|Δl1|+|l1−l0||Δt|`. Its tolerance therefore cannot depend only on
`|ln(output)|`; opposing large logs can cancel. Cubic solve/evaluation needs a
named E5 algorithm/conditioning budget measured separately. No constants or
uniform cubic convergence order are accepted by this document.

The meaningful validation matrix should include:

1. Every method/policy, 0/1/2 nodes, nonfinite x/y/query, exact and rounded
   duplicates including signed zero, unsorted input, positive-log domain,
   boundary queries and immediate predecessor/successor queries. Match exact
   typed errors and causes, without unsafe QuantLib-invalid-input probes.
2. Exact stored nodes over hostile finite bit patterns; constant data;
   power-of-two affine anchors and log-geometric anchors. Linear affine
   reproduction is numerical outside exact anchors. Natural reproduces
   affine data, not every cubic under its natural boundary conditions.
3. Linear exact-input `Fraction` oracle. For natural splines, solve the full
   polynomial value/C1/C2/boundary constraint system with rational Gaussian
   elimination on small grids, independently of production's tridiagonal
   curvature solve. For monotone Hermite, rational slopes and four Hermite
   constraints give an independent polynomial oracle. LogLinear uses exact
   binary64 inputs and increasing Decimal precisions, with exact power/tie
   anchors; agreement alone is not an interval certificate.
4. Strictly increasing/decreasing data, plateaus, alternating directions,
   sign crossings and irregular spacings. For monotone Hermite, check each
   derivative quadratic's endpoint/vertex minimum analytically and test its
   range, not only a sampled visual grid. Natural overshoot examples must
   remain supported and must not be tested as monotone.
5. Tiny/subnormal/extreme x/y, opposite MAX endpoints, rounded-zero secants,
   overflowing unused derivatives, huge spacing ratios and finite-answer
   intermediate overflow. Classify support exclusions separately from value
   accuracy and actual mathematical out-of-range results.
6. Refinement on smooth functions using the same region and declared boundary
   conditions. Verify linear `h²` behavior on a quadratic and a fourth-order
   natural-spline target with compatible endpoint curvature, such as sin(πx).
   Measure monotone Hermite on smooth monotone data, nonuniform grids and
   extrema separately; do not infer one order from the word cubic or demand
   cubic-polynomial reproduction from clipped data-estimated slopes.
7. Clone/concurrent/reversed query order, definition equality, read-only node
   views and absence of mutable lookup caches. Basic-only evaluation uses R1;
   separately computed log coefficients/results use conditioned tolerances.

## Attested public QuantLib 1.43 probes

These local observations were executed on 2026-10-12 using Python 3.9.6,
Darwin arm64, `/private/tmp/sabiquant-validation-venv/bin/python` and
`QuantLib.__version__ == "1.43"`. The imported binding was
`.../lib/python3.9/site-packages/QuantLib/__init__.py`, SHA-256
`6daeb4d8978fd7a227434dc67ddc2b2713d55a7c0f6ba4cc2d4011eecc5757a2`.
This identifies the public proxy file, not the native binary or a complete
build attestation. Extreme cases ran in separate child processes; all exited
normally. No C++ source was inspected and these observations are not fixtures
or a compatibility certification.

| Public call/data | Observed value | Consequence |
|---|---|---|
| Linear, x=[0,1,2], y=[2,4,8], q=0.5/1.5 | 3 / 6 | Ordinary linear reference anchor. |
| Same Linear, q=−1/3 with `allowExtrapolation=True` | 0 / 12 | Outer segment extension; default call rejects outside. |
| LogLinear, same data at supplied last knot q=2 | 7.999999999999998 | Proposed exact-node return is an intentional stronger contract. |
| NaturalSpline, x=[0,1,2], y=[0,1,1], q=1.5 | 1.09375 | Natural cubic overshoot is expected. |
| MonotonicCubicNaturalSpline, same data, q=0.5/1.5 | 0.65625 / 1 | Distinct from the proposed local PCHIP rule. |
| FritschButlandCubic, same data, q=0.5 | 0.6875 | Matches the proposed rule on this uniform example only. |
| FritschButlandCubic, x=[0,1,4], y=[0,1,2], q=0.5/2.5 | 0.5708333333333333 / 1.725 | Proposed weighted PCHIP independently gives 305/528 and 75/44; nonuniform method discrepancy must be classified. |
| Linear, x=[−MAX,MAX], y=[−MAX,MAX], q=0 | NaN | Reference arithmetic failure, not the mathematical answer 0. |
| Linear, x=[0,min_subnormal], y=[0,1], q=0/min_subnormal | NaN / infinity | Avoid cached-slope dependence and preserve exact nodes. |
| LogLinear, x=[−MAX,MAX], y=[1,2], q=0 | 1 | Incorrect normalized coordinate; mathematical value is √2. |
| LogLinear, x=[0,1], y=[min_subnormal,MAX], q=0.5 | 2.980232238769562e−8 | Finite wide-range case needs a high-precision oracle. |
| LogLinear with first ordinate 0 or −1 | RuntimeError naming index 0 | Supports explicit positive-log domain; error wording is not SabiQuant's contract. |

Compact reproduction of ordinary distinctions:

```python
import QuantLib as ql
assert ql.__version__ == "1.43"
x, y = [0.0, 1.0, 2.0], [0.0, 1.0, 1.0]
for name in ("CubicNaturalSpline", "MonotonicCubicNaturalSpline",
             "FritschButlandCubic"):
    f = getattr(ql, name)(x, y)
    print(name, f(0.5), f(1.5))
f = ql.FritschButlandCubic([0.0, 1.0, 4.0], [0.0, 1.0, 2.0])
print("nonuniform", f(0.5), f(2.5))
```

## Proposed implementation sequence and open decisions

Start with owned nodes, errors, exact-node lookup and explicit extrapolation;
implement Linear/LogLinear without cached slopes. Next add NaturalCubic with
its exact boundary definition and independent constraint oracle, then the
pinned weighted-harmonic MonotoneCubicHermite with endpoint/shape regressions.
These are reviewable batches, not a claim that half of F-MATH-02 completes M6.

Before production, record: accepted method/policy names; conservative supported
numeric grids versus rescaled full-range evaluation; output conditioning and
measured acceptance budgets; log-space range handling without silent clipping;
and the intentional QL endpoint/nonuniform-method differences. No market curve
semantics, source-code licensing exception or owner approval is inferred.

## Subsequent local prototype evidence

Under delegated local work, the four proposed methods/policies were implemented
in [interpolation.rs](../src/math/interpolation.rs), with the error variants
above. This later prototype record does not turn the original research into
owner approval or a full-range accuracy proof. The bare method label `Cubic`
now rejects as ambiguous with natural/monotone candidates; method/policy
identifiers remain exact and case-sensitive.

Linear uses binary exponent scaling for multiply/divide contributions, exact
identical-factor cancellation and a finite centered distance for overflowing
raw widths/value differences. It rescales ordinates if a large affine
contribution would overflow before cancellation. The identity over
`[(-MAX,-MAX),(MAX,MAX)]` is bit-exact at ±min-subnormal, ±minimum-normal, ±1
and ±MAX/2 for the three signed two-node methods. Extending
`[(0,-MAX),(1,-MAX/2)]` to query 4 returns MAX rather than losing the finite
answer to intermediate overflow. These anchors are stronger than the initial
conservative grid proposal, without claiming all ill-conditioned input grids
have a few-ULP bound.

Cubic construction retains declared support failures for nonrepresentable
raw widths/differences or required normalized quantities. Natural solves use
dimensionless `K`; PCHIP uses normalized slopes. Both retain normalized change
coefficients and restore `S` during evaluation. This fixes the reproduced
three-node case `h=2^500`, `Y=2^-74`, values `[0,0,Y]`: storing physical
`M_1=1.5 min_subnormal` previously rounded it to `2 min_subnormal`, giving
`−Y/8` at the first midpoint rather than exact `−3Y/32` (a 33⅓% error).
The scaled solve now gives the dyadic target exactly.
Constant and two-node data avoid unused derivatives. Cubic segments whose
higher coefficients are zero use the robust linear evaluator. No
natural-spline output clipping or default policy was added.

Native Rust 1.99 checks passed **17 core tests** in
[math_interpolation.rs](../tests/math_interpolation.rs), plus the independently
constructed **99,072-row** constraint/Decimal oracle in
[math_interpolation_oracles.rs](../tests/math_interpolation_oracles.rs).
The core includes 85,050 integer-weight linear cases, 625 nonuniform four-node
data sets with segment bounds and reconstructed derivative-quadratic extrema,
natural C2/boundary conditions, separate natural/PCHIP goldens, compatible
refinement, typed errors, subnormal rounding-unit boundaries and **16,512**
concurrent clone/query observations.
Independent three-node curvature constraints add **446 Natural** midpoint
checks (normal/subnormal ordinate scales, uniform and both nonuniform mirrors).
Exact-rational Hermite constraints add **1,248 PCHIP** value checks across
uniform/nonuniform grids and physical slopes that underflow or overflow.
The separate [math_interpolation_boundaries.rs](../tests/math_interpolation_boundaries.rs)
adds **9 public boundary tests** for actual finite-input support failures,
query policy/coordinate ordering, recovered centered extrapolation, distinct
signed/log result domains and near-MAX rounded-range behavior.
Basic-only repeated results are bitwise; logarithmic repeats use conditioned
budgets. The unchanged 32u absolute-conditioned independent oracle budget has
measured maximum fractions 0.04534 for Natural and 0.07692 for PCHIP after
scaling; raw near-zero ULP distances are not useful error summaries.
Focused strict Clippy passed. Final aggregate coverage, cross-platform
execution, numerical-budget calibration and remaining M6 acceptance gates
belong to the later milestone evidence, not this research record.
