# Numerical Correctness Policy

| Field | Value |
|---|---|
| Milestone | M0 |
| Document status | **Accepted** with [ADR-0006](adr/ADR-0006-numeric-policy.md) on 2026-10-10. TOL-E3/E4 are calibrated in M4; §12 records operation-specific local M6 extensions under delegated authority |
| Related | [REQUIREMENTS](REQUIREMENTS.md), [TESTING_STRATEGY](TESTING_STRATEGY.md), [ADR-0008](adr/ADR-0008-validation-strategy.md) |

This policy defines how SabiQuant computes and validates real-valued results
**before** any code exists. It covers the floating-point model, where `f64` is
and is not adequate, error sources, policies, determinism, tolerances, and
domain invariants.

---

## 1. Scope

The policy applies to every real-valued computation in SabiQuant. In H1 these
are year fractions, compound and discount factors, implied and equivalent
rates, and present values. Later iterative and stochastic methods inherit the
policy and extend it in their milestones.

## 2. Floating-point model

SabiQuant uses IEEE 754 **binary64** (`f64`).

| Quantity | Value |
|---|---|
| Significand precision | 53 bits (about 15.95 decimal digits) |
| Machine epsilon ε = 2⁻⁵² | 2.220446049250313 × 10⁻¹⁶ |
| Unit roundoff u = 2⁻⁵³ | 1.1102230246251565 × 10⁻¹⁶ |
| Largest finite value | 1.7976931348623157 × 10³⁰⁸ (natural log ≈ 709.78) |
| Smallest positive subnormal | about 4.94 × 10⁻³²⁴ |
| `exp(x)` overflows to +∞ | for x above about 709.78 |
| `exp(x)` underflows to 0 | for x below about −745.13 |

**Guarantees** from Rust's [`f64` documentation](https://doc.rust-lang.org/std/primitive.f64.html),
retrieved on the research date:

- `sqrt` and `mul_add` are "guaranteed to be the rounded infinite-precision
  result". `mul_add` is a fused multiply-add with a single rounding.
- The basic operations `+ − × ÷` are correctly rounded under IEEE 754
  round-to-nearest. Rust's floating-point semantics follow IEEE 754 except for
  NaN bit patterns. **Re-confirmed at the start of M1 (2026-10-10, Rust
  1.99):** the Rust Reference's chapter on operator expressions does not
  state the rounding of float arithmetic. The standard library documents it:
  the [`f32` page](https://doc.rust-lang.org/std/primitive.f32.html), to which
  the `f64` page refers, states that the result of addition, subtraction,
  multiplication or division "is rounded according to the roundTiesToEven
  direction defined in IEEE 754-2008". Its section on NaN bit patterns
  describes the only exception.
- **Transcendental functions** (`exp`, `ln`, `powf`, `powi`, `ln_1p`,
  `exp_m1`, …) have **unspecified precision**. The documentation states that
  precision "is non-deterministic", that it "varies by platform, Rust
  version, and can even differ within the same execution from one invocation
  to the next". Several functions currently call the platform C library.
- NaN bit patterns are not guaranteed to survive arithmetic.
- `ln_1p` (`log1p`) and `exp_m1` (`expm1`) exist and are documented as
  accurate near zero.
- `erf`, `erfc`, `gamma` and `ln_gamma` are **nightly-only** (`float_erf`,
  `float_gamma`). Stable SabiQuant cannot use them. Special functions must be
  written in-house when needed (M6).

## 3. Where `f64` is and is not suitable

| Use | Suitable? | Reasoning |
|---|---|---|
| Year fractions | Yes | Built from exact integers. One or a few roundings |
| Rates, compound and discount factors | Yes | Relative precision near 10⁻¹⁶ is far finer than market quotes, which are typically given to 10⁻⁶ or coarser |
| Prices and risk analytics | Yes, with care | Errors must be tracked through conditioning (§7) |
| Exact settlement amounts in decimal currency units | **No** | Decimal fractions such as 0.01 are not exactly representable. This is a recorded design issue (§10, NI-01) |
| Long sums (millions of terms) | With care | Needs a deterministic order and compensated summation (NP-09) |
| Values implied from nearly-equal inputs (short horizons, tiny rates) | With care | Cancellation magnifies error (§9) |
| Equality tests on computed values | **No** | Only exactly-specified results may be compared for equality |

## 4. Error sources

| Source | Description | Example in SabiQuant |
|---|---|---|
| Representation | Decimal inputs are rounded to binary | 0.05 is not exactly representable |
| Rounding | Each operation rounds | Act/Act ISDA sums up to three rounded terms |
| Cancellation | Subtracting close numbers exposes earlier rounding | `C − 1` for short horizons; `(1 + r/m)^m − 1` |
| Conditioning | The problem itself amplifies input error | Implied rate from `C` near 1: κ ≈ 1/abs(ln C) |
| Library variance | `libm` differs by platform and version | `exp` and `pow` results can differ in the last bit |
| Overflow / underflow | The result is outside the representable range | `exp(r t)` for `r t > 709.78` |
| Accumulation order | Floating-point addition is not associative | Future parallel reductions |
| Algorithmic (future) | Truncation and convergence error | Solvers, Monte Carlo, finite differences |

## 5. Numerical policies

| ID | Policy |
|---|---|
| NP-01 | **Finite inputs.** Every real input is checked for finiteness at the public boundary (FR-CORE-03). |
| NP-02 | **Finite, in-domain outputs.** Outputs are finite and satisfy their domain, for example a strictly positive compound factor. Overflow or underflow that makes a result meaningless is an error, not ±∞ or 0. |
| NP-03 | **Stable formulations.** Use `log1p`/`expm1` and algebraic rearrangement wherever they remove cancellation. Avoid subtracting nearly-equal computed quantities. |
| NP-04 | **Exact discrete arithmetic.** Dates and day counts use checked integer arithmetic. There is never a floating-point date calculation. |
| NP-05 | **Minimal rounding for year fractions.** Where a convention is a ratio of integers, compute it as **one** division of exact integers. |
| NP-06 | **No fast-math.** No reliance on floating-point contraction or reassociation. `mul_add` only where a specification calls for it explicitly, because it changes results (single rounding) and may be slow without hardware FMA. |
| NP-07 | **No NaN output.** NaN is never returned. NaN inputs are rejected. |
| NP-08 | **Signed zero.** −0.0 behaves exactly like +0.0 wherever sign has no financial meaning. Results do not depend on the sign of zero. |
| NP-09 | **Deterministic summation.** Sums use a fixed order. Compensated (Kahan–Babuška/Neumaier) summation is used for composite accrual; M6 exact_sum and M7 cash-flow reductions retain supplied finite binary64 terms exactly and round once. Exact reduction is not an exact dot product and cannot rescue individually invalid terms. |
| NP-10 | **Determinism.** No randomness, no thread-dependent ordering, no global state (NFR-03). |
| NP-11 | **Documented operation order.** For composite formulas, the specification or implementation documentation records the order of operations that determines rounding. |
| NP-12 | **Careful branch conditions.** Branch conditions on computed reals are explicit. The current hybrid switch compares against rounded binary64 `1/m`; branches agree at the exact rational threshold. Rounded-switch differences are small only in well-conditioned regions. Near a negative periodic-base boundary, relative continuity and monotonicity require separate switch/input conditioning, as documented in M4 numerical evidence. |

## 6. Determinism and cross-platform reproducibility

| Level | Scope | Guarantee |
|---|---|---|
| R1 | Same binary, same machine, repeated or concurrent runs | Discrete results, basic arithmetic and explicit identity paths are bitwise identical. Elementary transcendental results use the §7 bounds; Rust does not guarantee even same-execution bitwise identity (NFR-03, qualified during local M4 research) |
| R2 | Same target and toolchain on different machines | Bitwise identical for integer and basic floating-point arithmetic. Transcendental results are identical only if the same `libm` and the same optimizer decisions apply. **Not promised** |
| R3 | Different platforms (Linux, macOS, Windows; x86_64, aarch64) | Date/enumeration and explicit-input discrete outputs **identical**. Elementary numerical range/guard outcomes use their scoped conditional contract; finite floating-point outputs **within the tolerances in §7** |

Known sources of cross-platform difference:

1. Different C math libraries (glibc, Apple's libm, Microsoft's CRT) behind
   `exp`, `ln`, `powf`, `ln_1p`, `exp_m1`.
2. Changes in Rust or LLVM versions.
3. Compile-time evaluation of math calls compared with run-time evaluation.
   The Rust documentation warns that precision may differ "within the same
   execution".
4. 32-bit x86 targets without SSE2 (x87 extended precision). These are
   excluded from the primary platform tier (OQ-08).

Consequences for tests:

- Expected values are **never** specific to a platform.
- Assertions on transcendental results use tolerances. Bitwise assertions are
  reserved for exactness classes E0 and E1.
- The cross-platform test matrix (OQ-08) runs the same tests on each primary
  platform ([TESTING_STRATEGY §11](TESTING_STRATEGY.md#11-cross-platform-testing)).

**Option recorded for later (NI-03).** SabiQuant could write its own
deterministic elementary functions using only basic operations, which would
give bitwise cross-platform reproducibility. This is deferred. The trigger
would be a user requirement for bitwise reproducibility across platforms.

## 7. Tolerance policy

### 7.1 Principles

1. **No single global epsilon.** QuantLib's default closeness test uses a
   fixed 42ε (B-20). SabiQuant does not adopt any equivalent default.
2. A tolerance comes from four things: the computation's **exactness
   class**, the **condition number** of the quantity, the **error bounds of
   both implementations** being compared, and the **precision of the
   reference value**.
3. Every tolerance used in a test refers to a policy ID below, and every
   fixture records it.
4. A tolerance is **never widened to make a failing test pass** unless the
   discrepancy has first been classified and documented
   ([TESTING_STRATEGY §7.5](TESTING_STRATEGY.md#75-discrepancy-classification)).

### 7.2 Comparison metrics

| Metric | Use |
|---|---|
| Exact equality | Discrete values; correctly rounded results checked against a correctly rounded oracle |
| ULP distance | Results expected to be within a few roundings of the exact value |
| Mixed absolute/relative: `abs(a − b) ≤ max(atol, rtol × max(abs(a), abs(b)))` | Values near zero, where relative error is meaningless, and quantities with a known absolute error scale |

### 7.3 Exactness classes

| Class | Quantities | Against the independent oracle | Against QuantLib 1.43 | Policy ID |
|---|---|---|---|---|
| E0 Exact | Dates, weekdays, booleans, integer day and business-day counts, enumerations, error kinds | Equal | Equal | TOL-E0 |
| E1 Single division | Year fractions for Actual/360, Actual/365F, the 30/360 family, Business/252; discount factor as `1/C` given `C` | 0 ULP (correctly rounded quotient) | 0 ULP expected. Any difference is investigated, not tolerated | TOL-E1 |
| E2 Composite rational | Year fractions for Actual/Actual ISDA and ICMA; AFB remains under research | ≤ 4 ULP from the exact rational value. ISDA has bounded positive terms; the ICMA compensated-sum derivation is in [M3_ICMA_CONTRACT](M3_ICMA_CONTRACT.md) | ≤ 8 ULP (both implementations' errors combined) | TOL-E2 |
| E3 Well-conditioned elementary | Compound and discount factors | `rtol = k·u·(1 + abs(ln C))`, with `k = 8` initially | Same bound, applied to both sides | TOL-E3 |
| E4 Ill-conditioned derived | Implied and equivalent rates | `abs(Δr) ≤ k·u·(abs(r) + 1/t + A)`, with `k = 8` initially and the derivative term `A` below | Same conditioning, with independently classified QuantLib factor/base rounding included (see §9.3); no blanket multiplier | TOL-E4 |
| E5 Iterative or approximate | M6 solvers/interpolation/compensated reductions; future quadrature, Monte Carlo and finite differences | Algorithm tolerance plus convergence evidence; M6 profiles in §12 | Per algorithm | TOL-E5-* |

**Rationale for E3 and E4.** For `C = exp(x)`, a relative error δ in `x`
becomes a relative error of about `abs(x)·δ` in `C`, plus the `libm` error. So
the relative tolerance grows like `1 + abs(ln C)`. For an implied rate,
`r = ln C / t`. An absolute error of about `u` in `C` near 1 gives an absolute
error of about `u / t` in `r`, which dominates when `r t` is small. Hence the
`abs(r) + 1/t` scale for continuous inference. The local M4 oracle found that
this scale alone omits exponential argument conditioning for other targets:

- A periodic target uses `r = m·expm1(x)`, where
  `x = ln(1 + r/m)` and `dr/dx = m + r`. Its additional scale is
  `A = (m + r)·abs(x)`.
- A simple target obtained through annual log growth uses
  `r = expm1(x)/t`, where `x = ln(1 + r·t)` and
  `dr/dx = exp(x)/t = r + 1/t`. Its additional scale is
  `A = (r + 1/t)·abs(x)`. The test evaluates the conservative equivalent
  `(abs(r) + 1/t)·abs(x)` to avoid cancellation. Direct simple inference
  from a supplied factor does not take this exponential path and uses `A=0`.
- Continuous targets use `A=0`. Hybrids use their active target branch at
  the supplied time. At `r=0`, exact canonical zero is tested separately.

The `u` factors are applied before large divisions/products to avoid
intermediate overflow in the bound. Subnormal factor comparisons also allow
one smallest-subnormal absolute step. A mathematically unrepresentable rate
bound is recorded as saturated and supplies no useful accuracy evidence;
finite-domain, sign and explicit identity checks still apply. See the local
[M4 numerical evidence](M4_NUMERICAL_VALIDATION.md) for the D5 derivation,
counterexamples to the original scale and measured distributions. The value
`k=8` has not been increased to hide these discrepancies.

The constants `k` are **initial proposals**. M4 measures the actual error
distribution against the high-precision oracle and fixes the constants, with
the evidence recorded.

### 7.4 Reference precision

Oracle values come from exact rational arithmetic (day counts, simple
compounding) or high-precision arithmetic with at least 50 significant digits
(elementary formulas). Exact rational values are rounded once to the nearest
binary64 target. Elementary approximations verify stable rounding at increasing
precision and classify unresolved ties rather than choosing a precision.
Precision agreement alone is not an interval certificate
([ADR-0008](adr/ADR-0008-validation-strategy.md)).

## 8. Domain invariants

The invariants for dates, periods, calendars, business-day conventions and
day counts are defined next to their specifications:
[INV-DATE](REQUIREMENTS.md#5-fs-date--dates),
[INV-PER](REQUIREMENTS.md#6-fs-per--periods-and-frequencies),
[INV-CAL](REQUIREMENTS.md#7-fs-cal--calendars),
[INV-BDC](REQUIREMENTS.md#8-fs-bdc--business-day-conventions), and the
day-count property table in
[REQUIREMENTS §9.1](REQUIREMENTS.md#91-general-specification). That table
lists properties that **fail** for some legitimate conventions.

Rate invariants (FS-RATE). `C` is the compound factor and `P` the discount
factor. Every invariant holds only inside the domain of FS-RATE.

| ID | Invariant | Valid for | Check |
|---|---|---|---|
| INV-R-1 | `C(0) = 1` | Every valid quote | Exact (TOL-E0). Holds for `1 + r·0`, `exp(0)`, and `exp(0·log1p(·))` |
| INV-R-2 | `r = 0` ⇒ `C(t) = 1` for all `t ≥ 0` | Every compounding | Exact, provided `log1p(0) = 0` on each platform (verified by test). If not, the implementation must special-case it |
| INV-R-3 | `P(t)·C(t) ≈ 1` | Representable factors using the same evaluated `C` | Normal `P`: `abs(P·C−1) ≤ 2u`; subnormal `P`: allowance `u + C·MIN_SUB/2`. Separate elementary evaluations additionally include their E3 variance |
| INV-R-4 | Mathematically, `C` is non-decreasing in `t` if `r > 0`, non-increasing if `r < 0`, with hybrid continuity at exact rational `1/m` | Simple, Compounded, Continuous and both hybrids | TOL-E3 for evaluated fixed branches and well-conditioned switches. A rounded hybrid threshold near `r=-m` requires additional switch/input conditioning; unconditional relative monotonicity is not promised |
| INV-R-5 | Implied rate of the rounded `C(r, t)` at `t` recovers `r` within the factor-conditioned bound | `t > 0`, inferred quote and reconstructed factor representable | TOL-E4 plus supplied-factor uncertainty; near-zero factors can round an inferred simple rate to the rejected domain boundary |
| INV-R-6 | Mathematically, `C₂(t; r₂) = C₁(t; r₁)` for equivalent rates | `t > 0`, both factor outputs representable | TOL-E3 for evaluating each quote, plus the target rate-rounding sensitivity `abs(∂C₂/∂r₂)·abs(Δr₂)`. Near Simple cancellation this absolute term can dominate the factor's relative error |
| INV-R-7 | Equivalent rates between Compounded and Continuous do not depend mathematically on `t` | Compounded ↔ Compounded and Compounded ↔ Continuous | The implementation omits `t` from log-space conversion; transcendental evaluations use TOL-E4, rather than a platform-wide bitwise promise. Identity conversions that return the input rate are exact. **Fails with Simple**: 5% continuous gives a simple rate of 0.0506302410… at `t = 0.5` but 0.0525854590… at `t = 2` |
| INV-R-8 | `C(t₁ + t₂) = C(t₁)·C(t₂)` | Compounded and Continuous only | TOL-E3. **Fails for Simple**: `1 + r(t₁+t₂) ≠ (1 + r t₁)(1 + r t₂)` when `r ≠ 0` |
| INV-R-9 | Mathematically, with `t > 0`: `r > 0` ⇒ `C > 1`; `r < 0` ⇒ `C < 1`. Rounded factors can equal one for either sign. `r = 0` ⇒ exact `C = 1` | All, in domain | Non-strict direction within TOL-E3; exact zero-rate result. The converse `C=1` ⇒ `r=0` is false in binary64 |
| INV-D-1 | `P(S, S) = 1`; PV is linear in the amount | FS-DISC | Exact; TOL-E3 |

## 9. Analyses of foundational computations

### 9.1 Year fractions

Conventions defined as one ratio of integers (Actual/360, Actual/365F, the
30/360 family, Business/252) are **correctly rounded** when computed as one
division. That makes them bitwise reproducible everywhere, and bitwise equal
to any other implementation that also does a single division.

Composite conventions accumulate several roundings, and results depend on the
order of operations. QuantLib 1.43 computes Actual/Actual (ISDA) for a
same-year interval as `(Y2 − Y1 − 1) + a/B + b/B`. That is algebraically equal
to `(E − S)/B`, but it is rounded differently (B-12). An independently derived
formula can therefore differ from QuantLib by a few ULPs. That is the reason
for class E2.

### 9.2 Compounding near zero rates

Computed directly, `(1 + r/m)^(m t)` first rounds `1 + r/m`. For `r/m = 10⁻¹⁰`,
the absolute rounding error of the base (up to `u ≈ 1.1 × 10⁻¹⁶`) is a
**relative** error of up to about 10⁻⁶ in `r/m`. So about six significant
digits of the rate's contribution are lost before the power is taken. The
formulation `exp(m t · log1p(r/m))` keeps `r/m` intact.

### 9.3 Implied rates and cancellation at short horizons

Take `r = 0.05` and `t = 1/365` (one day under Actual/365F). Then
`r t ≈ 1.37 × 10⁻⁴`. Recovering `r` from a computed `C` amplifies the error in
`C` by about `1/(r t)`. A representation-level error in `C` of about
`ε ≈ 2.2 × 10⁻¹⁶` becomes a relative error of about `ε/(r t) ≈ 1.6 × 10⁻¹²`
in `r`. That is four orders of magnitude above `u`. M4 measures the actual
error distribution. TOL-E4
scales with `1/t` for this reason. QuantLib's equivalent-rate computation goes
through `C(t)`, so its own error has the same scale. Differential tolerances
must include it.

### 9.4 Equivalent rates without forming `C`

Example: a 5% semiannual rate converted to annual compounding. The exact value
is `(1 + 0.05/2)² − 1 = 0.050625`. Evaluated naively in binary64,
`(1.025)² − 1` gives **0.05062499999999992** (verified). The subtraction
exposes the rounding of `1.025` and of the square. The log-space formulation
`m₂ · expm1((m₁/m₂) · log1p(r₁/m₁))` has no subtraction of nearly-equal values.
Its error is expected to be a few ULPs. M4 measures it.

### 9.5 Extreme inputs

- Continuous: `exp(r t)` overflows above `r t ≈ 709.78` and underflows to 0
  below `r t ≈ −745.13`. Both are errors under NP-02.
- Compounded with Daily frequency over 100 years: `m t = 36,500`. This is fine
  in log space and must stay finite for realistic rates.
- As `r` approaches `−m` from above, `log1p(r/m) → −∞`, so `C → 0` and
  `P → ∞`. Reaching the representable limit is an error. The domain boundary
  `r = −m` itself is rejected.

Near the range surfaces, log/exponential error can change finite/error
classification. Local M4 rejects the E3 logarithmic uncertainty band through
`rate.numerical_boundary` rather than returning an ambiguous factor. Exact
simple fused arithmetic is exempt. This conservative policy and exact
integer-power anchors are documented in
[M4_BOUNDARY_VALIDATION](M4_BOUNDARY_VALIDATION.md); the nominal E3 budget does
not establish a universal platform elementary-function bound.

### 9.6 Hybrid compounding at the switch point

The current comparison `t ≤ 1/m` uses a rounded binary64 reciprocal and a
supplied binary64 time. Both branches agree at the exact rational threshold,
not necessarily at its rounded representation. TS-RATE-09 checks the declared
branch and independent values at the threshold and both neighboring inputs.
Relative continuity within E3 applies to well-conditioned switch cases.
Near `r=-m`, division/threshold representation errors are amplified by the
small factor and may cause a large relative jump or an active simple domain
error. See [M4_NUMERICAL_VALIDATION](M4_NUMERICAL_VALIDATION.md) for a concrete
frequency-13 counterexample and the qualified invariant. A future exact
rational switch is an API/numerical behavior change requiring its own evidence.

## 10. Future numeric representation issues

These are recorded design issues, not decisions.

| ID | Issue | Notes | Decision point |
|---|---|---|---|
| NI-01 | Exact decimal amounts for settlement | Options include fixed-point integers in minor currency units (std `i128`) or a decimal floating type, written in-house | Before cash settlement features (M7/M8), [OQ-06](IMPLEMENTATION_READINESS.md#5-decision-register) |
| NI-02 | Automatic differentiation or generic scalars | Conflicts with concrete `f64`. Retrofitting generics across a large codebase is expensive | Before pricing engines (M10/M11), ADR-0006 trigger |
| NI-03 | Deterministic in-house elementary functions | Bitwise cross-platform reproducibility | When a user requirement appears |
| NI-04 | Extended-precision accumulation (double-double) | Long sums, Monte Carlo statistics | M13 |
| NI-05 | In-house special functions (`erf`, inverse normal CDF) | Accuracy targets and published algorithms needed | M6 |
| NI-06 | Calibrating the TOL-E3 and TOL-E4 constants | Measured against the oracle | M4 |

## 11. Evidence required

Each implementation milestone delivers, for its numerical outputs:

1. The exactness class and tolerance policy ID of every public real-valued
   output.
2. Measured error distributions (maximum and percentile ULP or relative
   error) against the oracle on documented input grids, including the
   boundary grids from §9.
3. Differential results against QuantLib 1.43, with every discrepancy
   classified.
4. Results from the cross-platform matrix showing agreement at R3.

## 12. Local M6 operation profiles

These profiles extend the policy under the owner's instruction to continue
locally, recorded in [M6_LOCAL_READINESS](M6_LOCAL_READINESS.md). They are
specified acceptance/calibration rules, not owner acceptance or universal
certificates for Rust elementary functions. Aggregate measured evidence and
remaining R3/differential gates are in
[M6_NUMERICAL_VALIDATION](M6_NUMERICAL_VALIDATION.md).

| Output / policy ID | Check and scope |
|---|---|
| Exact reduction, stored knots, identity paths — `TOL-E0` | Exact supplied binary64 sum rounded once, ties to even; stored input/explicit identity bits; exact contextual statuses. This exactness class also includes correctly rounded results, not only discrete values |
| Compensated reduction — `TOL-E5-SUM` | Empirical absolute allowance `8u*abs(exact)+8n*u²*sum(abs(inputs))`, floored at MIN_SUB. Checked intermediates may reject an independently proved overflowing prefix even when the final exact sum is finite; no universal rounding/reordering guarantee |
| Solver root location — `TOL-E5-SOLVER-ROOT` | Explicit positional tolerance plus callback/reference conditioning. Polynomial-oracle check uses configured absolute tolerance plus `16u*max(abs(input endpoints))`; no residual-only or mathematical-root certificate |
| Solver tolerance decision — `TOL-E0`, with `TOL-E5-SOLVER-BRACKET` | Independently round each pinned full-width/overflow-half operation exactly. Accepted real width is bounded by `(1+4u)T+2MIN_SUB` or `(1+6u)T+4MIN_SUB`; these explain rounding and never change the configured operational stopping rule |
| Composite signed interpolation — `TOL-E5-INTERPOLATION` | Independent rational polynomial evaluated-term conditioning: `32u*(sum(abs(terms))+max(abs(node ordinates)))`, floored at MIN_SUB. Supported construction grids are separate from this empirical evaluation check; no uniform output-relative ULP claim |
| Log interpolation — `TOL-E3` | `8u` weighted-log sensitivity times the output magnitude, with MIN_SUB floor. Positive output domain and explicit extrapolation remain mandatory |
| Normal density/probabilities/logarithmic tails — `TOL-E3` | Operation-specific `k=8` argument/tail/log sensitivity and absolute rounding floors, pinned in [normal research](M6_NORMAL_RESEARCH.md). Rounded zero/saturated probabilities are valid; logarithmic outputs must be finite and nonpositive |
| Standard/affine normal inverse — `TOL-E4` | Standard allowance `16u*abs(z)` centrally or `8u*(1+abs(z)+abs(ln(q))/abs(z))` in tails; affine propagation adds scale conditioning and output rounding. The explicit overflow-midpoint guard retains conditional edge statuses and forbids a finite observation for true rounded overflow |

Required cubic intermediate precision and normal range guards are domain
contracts rather than permissions to enlarge an accuracy allowance. The
fixtures record these policy IDs or their existing E3/E4 profiles and all
finite allowed edge observations remain accuracy checked. Same-execution
elementary results retain the qualification in §6. Maximum/percentile errors
are reported as fractions of conditioned budgets; near-zero raw ULP distances
can be large and do not establish meaningful relative error.

## Local M7 analytical cash-flow profiles

The [adopted cash-flow contract](M7_CASHFLOW_CONTRACT.md) extends analytical
binary64 scope without closing NI-01 decimal settlement. Its private three-factor
integer product rounds exact supplied binary64 factors once (TOL-E0, 0 ULP),
including final subnormal/overflow decisions. The year fraction itself retains
its existing rational/elementary profile. Leg reductions use exact_sum of rounded
per-flow PV/accrued results. Ideal signed financial NPV comparisons propagate
absolute amount, factor, product and final reduction errors, conditioned on gross
flow magnitude; relative-to-net/ULP-only assertions are unsuitable near cancellation.
Factor-required M4 domains and existing accuracy allowances remain unchanged.
