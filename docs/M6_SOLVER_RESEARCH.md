# M6 one-dimensional solver research and local contract

| Field | Status |
|---|---|
| Date | 2026-10-12 |
| Scope | F-MATH-01: bisection, Brent and safeguarded Newton |
| Evidence | Primary mathematical publications, Rust 1.99 documentation and attested public QuantLib 1.43 probes |
| Stage | Local research/specification and bounded prototype, under delegated authority. No owner milestone acceptance, publication approval or cross-platform CI claim |

## Mathematical basis

Solve `f(x)=0` on a finite real interval. A sign change plus continuity
establishes existence, not uniqueness. Reliable evaluations are a separate
caller responsibility; discontinuities and rounding-induced callback zeros
can invalidate the mathematical interpretation of numerical termination.

| Method | Mathematical definition and conditions |
|---|---|
| Bisection | Repeatedly evaluate the midpoint and retain the sign-changing half. In exact arithmetic the interval width after `n` reductions is the initial width divided by `2^n`. Multiple roots may be present; a same-sign even-multiplicity root is not bracketed |
| Newton | Trial `x-f(x)/f'(x)`. For a sufficiently close initial point and a simple zero of a suitably smooth function, convergence is locally quadratic; multiple roots generally reduce the order. This local result alone provides no global bracket guarantee |
| Brent | Combine inverse linear/quadratic interpolation with bracket-preserving bisection and historical-step safeguards. The fast trial is optional; the maintained sign bracket and fallback supply robustness |

The first two descriptions are supported by
[NIST DLMF §3.8(ii–iii)](https://dlmf.nist.gov/3.8). Conditioning depends on
the slope: a small function error can produce a large root error near a flat
zero; a small residual alone is not a positional guarantee. See
[DLMF §3.8(vi)](https://dlmf.nist.gov/3.8.vi).

Brent's original mathematical description was visually checked in the
author-hosted [1973 book, Chapter 4 §2, printed pages 48–52](https://maths-people.anu.edu.au/~brent/pd/rpb011i.pdf),
with [the author's publication/errata record](https://maths-people.anu.edu.au/~brent/pub/pub011.html).
Maintain a best bracket endpoint, the opposing endpoint and the previous best
point. Distinct ordinates permit inverse quadratic interpolation; otherwise
use the inverse linear trial. Reject an interpolation that fails to improve
the function magnitude, travels beyond three quarters of the way across the
bracket, or does not shrink sufficiently relative to the step before the last.
The two-step history is significant. Its magnitude must shrink by at least
half every second interpolation, with bisection when progress is inadequate.
The book's rounding discussion explicitly distinguishes the computed callback
from the mathematical function. Its stopping rule is adapted to the explicit
project tolerances below; no book or QuantLib program is copied or translated.

Safeguarded Newton is a project proposal built from Newton's mathematical
trial and the sign-bracket invariant. Only finite, strictly interior,
non-repeating proposals with adequate progress are used; otherwise bisect.
Zero derivative falls back to bisection. A nonfinite derivative is a typed
evaluation error. Do not advertise unconditional quadratic convergence for
the safeguarded method: safeguards may legitimately keep selecting bisection.

## Pinned public boundary

Small, immutable checked values, private fields and no `Default`:

```rust
RootBracket::new(lower: f64, upper: f64) -> Result<RootBracket, SolverError>
SolverOptions::new(absolute: f64, relative: f64, max_evaluations: u32)
    -> Result<SolverOptions, SolverError>

bisect(function, bracket, options) -> Result<SolveReport, SolverError>
brent(function, bracket, options) -> Result<SolveReport, SolverError>
safeguarded_newton(function, derivative, initial, bracket, options)
    -> Result<SolveReport, SolverError>
```

Initial callbacks are `FnMut(f64) -> f64`; the solver stores no callable or
hidden state after the call. Root handles module/root-error integration:
`src/math/solver.rs` defines `SolverError`, wrapped by `Error::Solver`.

Requirements:

- Check all inputs before invoking callbacks: finite `lower < upper`, finite
  `absolute > 0`, finite `0 <= relative < 1`, nonzero callback budget and a
  finite Newton initial point in the closed input bracket. Reuse cached
  endpoint values when the initial point equals an endpoint.
- Canonicalize irrelevant signed zero. Evaluate the lower endpoint first;
  if its callback value is exactly zero, return immediately. Then evaluate
  the upper endpoint and handle its exact zero. All other endpoint values
  must be finite and have opposite signs. Compare signs directly; never
  multiply function values to decide this condition.
- Every callback argument is finite and inside the original bracket.
  Nonfinite function/derivative outputs identify argument and evaluation
  stage in a typed error. Nonfinite internal trial arithmetic is discarded
  in favour of a valid midpoint; it is never sent to a callback.
- Maintain an ordered coordinate bracket and cached values. A callback-zero
  result retains the current non-collapsed bracket; `RootBracket` does not
  acquire invalid equal endpoints merely to represent a successful point.

## Tolerance and machine resolution

There is no residual-only or small-step success. Operational bracket
termination uses the full width `w = upper-lower` and
`T = absolute + relative*max(|lower|,|upper|)` in the specified binary64
operation order. If `w` is finite, compare `w <= T` directly. Do not halve a
finite subnormal width: width `5*MIN_SUB` and tolerance `4*MIN_SUB` must fail.
A rounded infinite `T` here represents a genuinely large allowance.

If subtraction overflows, compare the safe half-width
`upper/2-lower/2` against
`absolute/2 + relative*(max_abs/2)`. This is an explicitly rounded operational
criterion, not exact real interval arithmetic. For finite-width acceptance,
a conservative round-to-nearest bound is true width at most
`(1+4u)*T_real + 2*MIN_SUB`, where `u=2^-53`; a conservative overflow-half
allowance is `(1+6u)*T_real + 4*MIN_SUB`. These allowances describe rounding,
not permission to change the user's configured tolerance. Independent exact
Fraction tests must validate both branches and their transitions.

The finite-width allowance follows the usual round-to-nearest binary64
basic-operation model with gradual underflow: a rounded finite operation has
relative error at most `u` plus absolute error at most `MIN_SUB/2`. Bounding
the product, tolerance addition and width subtraction gives coefficients
`(1+u)^2/(1-u) < 1+4u` and `(3+u)/(2(1-u)) < 2`. A tolerance addition that
overflows is handled using its exact overflow threshold, rather than an
infinity error estimate. The overflow-half branch has exact coordinate
halving at the enormous magnitudes needed for overflowing subtraction;
its extra multiplication/addition rounding is covered conservatively by
the stated `6u` allowance. These basic-arithmetic bounds do not cover callback
error or certify arbitrary platform transcendental functions.

Use a safe midpoint formulation across the entire finite range. Rust 1.99
provides [f64::midpoint](https://doc.rust-lang.org/std/primitive.f64.html#method.midpoint);
extreme and subnormal cases still need direct tests. Reject interpolation
denominator degeneracy, repeated points and overflow by falling back. If the
midpoint is an endpoint and the bracket criterion has not passed, return
`PrecisionExhausted`, rather than silently raising tolerance to epsilon.
Changing derivative/interpolation step floors must never change success.

`SolveReport` contains a finite evaluated point and its finite cached callback
value, final bracket, iteration count, separate function/derivative counts and
`FunctionZero` or `BracketTolerance`. Bracket termination may choose the
cached endpoint with smaller absolute callback value. `FunctionZero` means
the callback returned zero; neither reason certifies continuity, callback
accuracy, uniqueness or existence for an arbitrary user function.

## Evaluation budget and diagnostics

One explicit maximum counts **all** function and derivative invocations,
including endpoints and nonfinite callback results. Discarded arithmetic
proposals make no function invocation and consume no callback count. Check the remaining budget
before invoking. Separate counters explain cost, while their sum never exceeds
the shared maximum. An endpoint zero can succeed with a one-call budget;
otherwise that budget stops before the upper endpoint. No automatic bracket
expansion, wall clock, randomized restart or global cache is introduced.

Implemented stable error categories: value/input rejection, invalid bracket,
invalid budget, not bracketed, nonfinite evaluation with stage/point,
evaluation limit with consumed counters and last known bracket, and precision
exhaustion. Preserve diagnostic context and any numeric source chain. On a
failure before both endpoint values exist, describe known state rather than
inventing a verified function bracket. All finite loop work is bounded by the
callback cap and mandatory progress/precision checks.

The solver itself generates no panics on input. User callback panics propagate;
callbacks must obey the stated contract. Do not install `catch_unwind` as a
routine failure channel: [Rust's panic documentation](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html)
distinguishes unwind from abort and recommends `Result` for regular failures.
A future fallible callback layer can retain `E` in a generic `SolveFailure<E>`
while leaving algorithm `SolverError` nongeneric; that extension needs its
own API decision and is outside this first callback surface.

## Actual reference probes

Local public-API probes used the existing QuantLib 1.43 wheel
`quantlib-1.43-cp39-abi3-macosx_11_0_arm64.whl`, SHA-256
`85d70b28778afca84b9b6ae74ef05a1069b2dca00e80fc74aa8ddf438259dc0c`.
All three installed `QuantLib/` members matched the wheel. Python 3.9.6,
macOS arm64; the precise wheel source commit remains unattested.
Use `Bisection/Brent/NewtonSafe.solve(callback, accuracy, guess, lower, upper)`;
Newton's callable object also exposes `derivative(x)`. Count both methods in
the object. These are numeric runtime observations, not copied test tables.

| Probe | Observed QuantLib behavior | Project implication |
|---|---|---|
| `x*x-2`, `[0,2]`, guess 1, accuracy `1e-12`, budget 100 | Bisection 44 function calls; Brent 10; NewtonSafe nine plus six derivative calls | Compare root accuracy; do not require matching iteration paths |
| Same problem, budget three | All three invoke the function four times before reporting exhaustion; Newton also invokes the derivative twice | Strict check-before-call/shared counting is an explicit divergence |
| `1e-300*(x-1)`, `[0,2]` | All three return lower endpoint zero after one function call, although its callback value is nonzero | Reject residual-near-zero shortcuts; independent root is 1 |
| Newton derivative is infinity for `x*x-2` | NewtonSafe returns 1, with residual −1 | Nonfinite derivative must be a typed error |
| `f(x)=x`, `[-MAX,MAX]` | Bisection reaches its 100-call limit after 101 calls, with 99 nonfinite callback arguments | Safe midpoint and finite callback arguments are required |
| Sign jump at zero | All three return near zero with residual magnitude 1 | Positional termination does not establish a root without continuity |

## Meaningful validation plan and remaining checks

- Independent exact dyadic/Fraction roots and high-precision irrational roots;
  endpoint, interior, negative, translated/scaled and multiple-root examples.
  Check final bracket, point containment, cached residual and declared reason.
- Enumerate tiny/normal/maximum coordinate and tolerance grids; opposing and
  same-sign endpoints; adjacent floats; `5*MIN_SUB` versus `4*MIN_SUB`; full
  finite range; overflow-half transition. No widening of TOL-E5 to hide failure.
- Instrument every function/derivative callback: caps 0, 1, 2, 3 and boundaries,
  no out-of-bracket/nonfinite arguments, exact attempted counts, early endpoint
  short circuits and deterministic evaluation order.
- All NaN/infinity input and output stages; wrong-sign bracket; zero/nonfinite
  derivative; overflowing, escaping, tiny or repeated fast trials; flat roots;
  inverse-quadratic ordinate collisions; midpoint representability exhaustion.
- Verify bisection contraction against an independently tracked interval;
  require Brent's two-step interpolation history and Newton's safeguard to
  trigger on adverse cases. Confirm callback panic propagation separately.
- Source callback rounding, transcendental variation and discontinuity limit
  mathematical claims. The implementation documentation pins the exact
  safeguard algebra and operation order; public-reference differences above
  remain explicit divergences. Native cross-platform behavior is still a
  separate release gate.

## Local prototype validation checkpoint

`src/math/solver.rs` implements the three public algorithms and checked
values. `tests/math_solvers.rs` contains 32 focused tests, including 2,667 exact
dyadic-root solves, every shared cap 1–80 for all three methods, all nonfinite
callback stages and propagation of caller panics. Observed Brent trials are
checked against a separate direct inverse-Lagrange expression; the trace
contains linear and quadratic trials and an otherwise-valid interior trial
rejected specifically by the step-before-last safeguard. Tests also cover a
continuous root strictly between adjacent floats and contraction from
`[-MAX,MAX]` to a one-subnormal-unit bracket with a strict 5,000-call cap.
Tiny scaled flat callbacks expose real callback underflow; their
`FunctionZero` reports deliberately carry no invented mathematical accuracy
claim. On the well-conditioned square-root polynomial both fast methods use
fewer than half bisection's function evaluations.

The std-only external generator
`validation/generator/solver_oracles.py` independently rounds each pinned
tolerance operation from exact Python `Fraction` values and separately checks
the unrounded real allowance. Its schema-1 fixture
`validation/fixtures/math/solver-tolerance-oracle.fixture` has 97,851 rows:
97,108 full-width and 743 overflow-half. All three public methods match every
row using two callbacks and no derivative allowance. There are 59,429
operational accepts and 38,422 rejects. Rounding produces 6,734 accepts beyond
the exact real tolerance and three rejects inside it; every accepted real
width satisfies the conservative allowance above. This is useful boundary
evidence, rather than treating an operational comparison as exact interval
arithmetic. Fixture intermediate bits and statuses are E0 checks; no solver
tolerance was widened.

The final fixture records generator SHA-256
`5995120804ac1a7ba50fb5b391a265b6772d48cc96c6d78a0ed2150e9117002a`.
Adding explicit tolerance-profile IDs changed metadata only; every fixture
data row retained its independently checked value/status bits.
Reproduce with `python3 validation/generator/solver_oracles.py --check`;
`tests/math_solver_oracles.rs` consumes those retained data rows. Rust 1.99
focused debug/release checks, three solver doctests and strict focused Clippy
pass locally. These results do not establish milestone acceptance, owner
review or native Linux/Windows CI.
