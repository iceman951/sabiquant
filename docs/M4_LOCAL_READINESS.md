# Local M4 implementation decisions

The owner delegated continuous local work with broad meaningful tests on
2026-10-11. M3 commit `61a15db` passes 240 debug and 239 release tests, strict
lints/docs, native coverage and qualified reference checks. Local work may
continue while remote CI/milestone review is pending; this is a scheduling
exception, not M3 completion or permission to push or publish.

The agent adopts these engineering decisions under that delegated authority:

- OQ-22: finite rates and nonnegative finite horizons; positive finite factor
  outputs; reject overflow, zero underflow, nonperiodic frequencies and invalid
  periodic bases. Periodic and hybrid quotes require rate greater than minus
  their annual frequency at construction, including at zero/short horizons.
- OQ-23: implement all five specified compounding methods. Each periodic value
  owns its explicitly validated frequency; simple/continuous take no frequency.
- OQ-24: real inputs/results remain `f64`; no new scalar real wrappers. Immutable
  quote and convention types require their financial context explicitly.
- The quote owns a scalar or contextual day-count convention. Date operations
  reject reversed dates before querying accrual; ICMA and guarded TARGET retain
  all context checks, including equal dates.
- Named percent/basis-point helpers explicitly convert to/from decimal rates
  with one division/multiplication, finite inputs/outputs, canonical zero and
  rejection of a nonzero value rounded to zero. No input mode or implicit
  scaling is added to quote construction.
- Equivalent-rate conversions use annual log growth where possible. Positive
  horizons with the same conventions preserve the input rate exactly, after
  validating its active domain. Zero horizon is underdetermined: choose rate
  zero, matching the reference's `C=1` implied-rate rule. Unequal date-based
  year fractions are both explicit and preserve the original period factor.
- Zero target horizon with positive source horizon and nonzero source rate is
  an error, including tiny rates whose computed growth could round away. Discount-factor
  implied rates use the negative logarithm directly, avoiding an unnecessary
  reciprocal overflow. Present values allow finite negative amounts and reject
  overflow or zero underflow of a nonzero product.
- NFR-03/R1 is qualified to match Rust's documented elementary-function
  contract: discrete/basic arithmetic and explicit identity paths remain
  bitwise reproducible; transcendental calls require measured conditioned
  tolerance, including repeated calls in the same execution. The agent records
  this specification correction under delegated authority; it does not claim
  a separate owner approval or a deterministic replacement for platform math.
- The original strict rate-sign invariant is mathematical. Tiny nonzero rates
  can produce a rounded factor exactly one. A correctly rounded implied simple
  rate from a very small positive factor can reach the horizon-specific simple
  boundary. Inference/conversion rejects that rounded result when its active
  simple factor is nonpositive. A positive factor close to that boundary
  remains ill-conditioned with respect to rate representation; reconstructed
  factor comparisons include that derivative rather than assuming a small
  relative error. Round-trip guarantees require representability.
- Range-boundary research found that a rounded log-space exponent can hide
  mathematical overflow or zero underflow. Elementary factor calculations
  conservatively reject logarithms within `8u*(1+abs(logarithm))` of
  `ln(MAX)` or `ln(min_subnormal)-ln(2)`; discounts also check reciprocal
  overflow. This is the unchanged E3 error budget, applied as an explicit
  `rate.numerical_boundary` policy. Exact fused simple paths are exempt.
  The margin is locally measured, not a formal bound on platform math.
  [M4_BOUNDARY_VALIDATION](M4_BOUNDARY_VALIDATION.md) records the separate tier.
  The guard uses the computed binary64 logarithm. A narrow outer-edge tier
  around the real-log oracle's guard uses an explicitly measured conditional
  status contract; no finite result is accepted for a mathematically
  unrepresentable factor and value tolerances are unchanged.

Numerical implementation follows FS-RATE and NUMERICAL_CORRECTNESS §9, using
`ln_1p`, `exp_m1` and a correctly rounded fused simple factor where useful.
[Rust f64 documentation](https://doc.rust-lang.org/std/primitive.f64.html#method.mul_add)
specifies fused arithmetic; elementary transcendental precision is platform
dependent and must be measured rather than assumed identical across platforms.
[Python Decimal](https://docs.python.org/3.9/library/decimal.html)
provides independently rounded high-precision logarithms/exponentials. Oracle
generation uses exact `Fraction` arithmetic for Simple/basic-ratio operations
and exact binary64 Decimal input conversion at two increasing precisions for
elementary formulas. Precision agreement verifies stable rounding on a grid,
not a universal correct-rounding certificate.

Observed public QuantLib 1.43 probes: a daily compounded `1e-20` rate converts
to continuous zero through its rounded factor; a large horizon can return
infinity; `r=-2` with semiannual compounding returns one at zero time; a hybrid
can use its simple branch despite that invalid periodic base. These are specific
reference behaviors to classify, not financial correctness definitions.

Validation must cover every frequency and method, signed zeros, all nonfinite
inputs, hybrid threshold and adjacent binary64 values, tiny/subnormal rates and
times, negative-domain boundaries, extreme factors, conditioned round trips,
all day-count context types, date errors, present values, exact/rational simple
cases, high-precision error distributions and attested public reference calls.
TOL-E3/E4 constants remain provisional until measured. No M4 acceptance,
cross-platform execution or independent owner review is claimed here.

Local review also qualifies NFR-04/E0: error kind strings remain exact once a
variant is chosen, while elementary numerical classification near range/guard
edges uses the declared conditional outcome contract. The original NP-12 and
INV-R-4 assumed rounded hybrid switches were always well-conditioned; near
minus-frequency inputs disprove that relative claim. Independent branch
values remain checked, and the rounded switch behavior is retained and
explicitly documented rather than claimed universally continuous/monotone.
These are agent specification corrections under the original delegated local
authority, not separate owner approvals.
