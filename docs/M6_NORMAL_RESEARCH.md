# Normal-distribution contract and research

This is a local, unreleased M6 engineering decision under the owner's authority
to continue local work. It records inspected primary mathematics and the
proposed numerical contract, not owner API approval, native multi-platform
validation or a correctly rounded transcendental-function certificate.

## Inspected sources and independent design

Sources were inspected on 2026-10-12. [DLMF 7.1](https://dlmf.nist.gov/7.1)
relates the normal probabilities to `erfc`; [DLMF 7.20.1](https://dlmf.nist.gov/7.20.E1)
gives the location/scale transformation. Rust's
[binary64 documentation](https://doc.rust-lang.org/std/primitive.f64.html)
still marks `erf`/`erfc` nightly-only. Stable `exp`, `ln` and `ln_1p` have
unspecified precision, including between calls in one execution. Therefore
this implementation uses its own bounded elementary algorithm and retains
the existing conditional numerical policy.

For the center, [DLMF 7.6.2](https://dlmf.nist.gov/7.6.E2) supplies a positive
series. With `phi(z) = exp(-z²/2)/sqrt(2*pi)`, algebra gives

`Phi(z)-1/2 = phi(z) * (z + z³/3 + z⁵/(3*5) + ...)`.

For positive `z`, recurrence terms have ratio `z²/(2*n+1)` and no alternating
cancellation. Compensated accumulation and a geometric bound on the omitted
terms control truncation below the binary64 rounding scale. A fixed iteration
limit prevents unbounded work. This real-arithmetic truncation estimate does
not bound platform `exp` or all accumulated floating-point rounding.

For the tails, Gasull and Utzet's original
[2013 paper](https://arxiv.org/pdf/1307.3433v1), Theorem 1, Corollary 2 and
equation (6), establishes the Mills-ratio convergents and their alternating
monotone bounds for `z>0`. The independent backward recurrence evaluates

`R(z) = 1 / (z + 1/(z + 2/(z + 3/(...))))`, `Q(z)=phi(z)*R(z)`.

No published implementation or QuantLib code is translated. A fixed depth
256 is proposed for `z >= 1.5`; center arithmetic is used below that threshold.
An exact-real bound between adjacent convergents can be checked independently
in high precision. Rounded binary64 convergents are not themselves certified
interval endpoints.

Wichura's original AS 241 article, [publisher record](https://academic.oup.com/jrsssc/article/37/3/477/6985522)
and [university-hosted original scan](https://csg.sph.umich.edu/abecasis/gas_power_calculator/algorithm-as-241-the-percentage-points-of-the-normal-distribution.pdf),
was inspected to compare scope. Its stated tail approximation range is
`5 <= sqrt(-ln(q)) <= 27`, with the advertised input range above approximately
`1e-316`. The smallest binary64 positive probability lies outside that range.
The scan's publication metadata agrees with the publisher record; hosting
does not independently attest its authenticity. No AS 241 coefficients or
code are imported, and its accuracy claims do not transfer here.

## Temporary convergence experiment

An independent Python/Decimal experiment used exact binary64 input conversion,
200-digit arithmetic, a Machin-identity pi calculation and adjacent Mills
convergents at depths 2048/2049. Their relative separation was below `1e-40`
for the measured inputs starting at 1.5. This is not a formal interval
certificate for the Decimal elementary operations. The temporary experiment
is `/private/tmp/sabiquant-normal-experiment.py`; it is not a committed fixture
generator or evidence of the Rust implementation's accuracy.

| positive z | depth 128 tail ULP | depth 256 tail ULP | center tail-subtraction ULP |
|---:|---:|---:|---:|
| 1.5 | 105 | 1 | 3 |
| 2.0 | 0 | 0 | 16 |
| 2.5 | 2 | 2 | 164 |
| 3.0 | 0 | 0 | 215 |
| 4.0 | 0 | 0 | 8200 |

These sampled observations support the crossover and depth; they do not
establish a universal few-ULP error bound. Tail inputs 5, 8, 10, 20, 30, 37,
38, 38.4 and 38.5 also agreed within two ULP at depth 256. A committed,
precision-checked independent oracle must validate dense grids, crossover
neighbors and subnormal rounding before acceptance.

## Public API and domains

The free functions are `standard_normal_pdf`, `standard_normal_cdf`,
`standard_normal_survival`, `standard_normal_log_cdf`,
`standard_normal_log_survival` and `standard_normal_inverse_cdf`.
`NormalDistribution::new(mean, standard_deviation)` owns finite explicit
location and strictly positive finite scale. Its corresponding methods use
the same six operation names without the prefix; accessors return the supplied
location and scale. There is no implicit financial convention or `Default`.
All public real inputs are checked for finiteness before domain tests. The
inverse requires `0 < p < 1`; endpoint infinities are deliberately excluded.
At `p=1/2` it returns exactly the mean, with canonical positive zero.

PDF and probability tails may round to zero; CDF and survival may saturate at
one. Those are meaningful finite binary64 values, so zero underflow is allowed.
Finite positive scale can produce an unrepresentable density: that overflow
is a typed error. Log probabilities must remain finite; a true extreme lower
tail whose logarithm cannot be represented produces a typed result error.
Upper-tail log probabilities may round to canonical zero. Rounded elementary
overflow boundaries require a measured conditional guard rather than an
unsupported exact-status promise.

Location/scale standardization must avoid premature overflow of `x-mean`:
for `x=MAX`, `mean=-MAX`, `scale=MAX`, the standardized value is 2. Density is
evaluated from the combined log density; dividing an already-underflowed
standard density would lose recoverable results at tiny scale. Affine inverse
uses fused `scale*z+mean`, allowing an overflowing product to cancel to a
finite result. These calculations retain conditioning qualifications near
location cancellation and elementary range boundaries.

## Inverse algorithm and validation requirements

Near probability one half, the solver compares the centered mass directly
with `abs(p-1/2)`. It must not subtract a rounded CDF from one half, whose
plateau would erase the smallest nonzero centered probability. In the tails,
it compares `ln(Q(z))` with `ln(min(p,1-p))` and uses derivative
`d ln(Q)/dz = -1/R(z)`. A finite bracket from zero to 40 contains every
strict binary64 probability's standard quantile, including `MIN_SUB`. Newton
steps remain inside a shrinking bracket and fall back to bisection; a fixed
iteration cap yields an explicit convergence error rather than a hang.

Required tests include all nonfinite inputs; scale and probability domain
boundaries; signed zeros; mean/scale preservation; exact median; center
neighbors of one half; crossover neighbors; both tails; logarithmic tails
after probability underflow; every probability exponent and subnormal
probabilities; symmetry; conditioned probability/quantile round trips;
monotonicity with tolerance for measured elementary rounding; affine density
recovery; standardization and inverse cancellation; output overflow; and
clone/reorder/thread reuse without hidden state.

Independent oracle fixtures must use exact binary64 inputs, increasing
precision and two formulations where practical. Centered series and tail
continued fractions must agree where they overlap. Inverse oracle brackets
must be far narrower than output binary64 spacing, and any ambiguous rounding
must be classified instead of selecting the higher precision arbitrarily.
The numerical budget must include elementary argument rounding and affine
conditioning, with an absolute subnormal floor. A global few-ULP claim or a
bitwise reproducibility claim is not justified by this research.

## Implemented local evidence

`src/math/normal.rs` implements the six standard functions and the explicit
location/scale value. Center iterations are capped at 128, the Mills fraction
has 256 levels and inverse iterations are capped at 256. Near-center inverse
uses probabilities in `[0.25,0.75]`; other probabilities use logarithmic tails.
`tests/math_normal.rs` currently has 14 passing tests: 40,001 dense shape points,
10,000 seeded standard/affine round trips, all 1,074 probability exponents,
512 subnormal probabilities, 256 median-neighbor quantiles, 8,192 observations
across eight threads, 2,001 explicit-standard context comparisons, 65,544
independent Simpson density samples, nonfinite/domain checks and extreme
location/scale/range regressions. These include the affine overflow guard,
finite cancellation, exact medians and safe `MAX` means with tiny scales.
Three additional tests in `tests/math_normal_boundaries.rs` check signed
logarithmic overflow context, canonical saturated tails, the half-spacing
overflow midpoint at both signed `MAX` means, and density error context at
four distinct means. Two normal doctests also pass locally.

The committed generator `validation/generator/normal_oracles.py` is stronger
than the temporary convergence experiment. It checks exact binary64 inputs at
120 and 200 Decimal digits, with a separately calculated Machin pi. The oracle
uses positive series below 10 and adjacent Mills convergents above 10, requiring
relative separation at most `1e-70`; its cross-form overlap check requires
agreement within `1e-100`. Center tail subtraction below 10 retains far more
than 50 significant digits even at the 120-digit setting. Quantiles use an
independent 180-step Decimal bisection, with a tighter centered bracket near
one half. Cached duplicate probabilities reduce work without reusing Rust
values. Every rounded output and status must agree at both precisions.
This supports finite-output evidence, not a formal interval certificate for
Decimal elementary functions or a universal correct-rounding assertion.

Six schema-v1 fixture sets contain 19,946 rows, including dyadic grids,
non-dyadic seeded inputs, both tails, every probability exponent, subnormals,
median/crossover neighbors and affine extremes. The numerical allowance is
specified independently of observed errors, with `u=2^-53`:

- PDF: `8*u*(1+z²/2+abs(ln(scale))+ln(sqrt(2*pi)))*abs(pdf)`.
- Small CDF/survival and upper-tail log probabilities:
  `8*u*(1+z²)*abs(result)`; probabilities at least one half use absolute `8*u`.
- Lower-tail log probabilities of magnitude at least one half:
  absolute `8*u*(1+abs(log_probability))`.
- Standard central quantile: `16*u*abs(z)`; tail quantile:
  `8*u*(1+abs(z)+abs(ln(q))/abs(z))`, using the Mills bound `R(z)<1/z`.
  Affine quantile adds `scale` times this allowance and `u*abs(result)` for
  fused rounding.

Every allowance has at least one `MIN_SUB` absolute step. Rounded zero truth
can differ by that floor; an observed zero must retain canonical positive sign.
Exact input identities such as the median are checked separately.

The density overflow guard is `abs(log_density-ln(MAX)) <= 8*u*S`, where
`S=1+z²/2+abs(ln(scale))+ln(sqrt(2*pi))`. Its conditional edge tier uses
`6*u*S <= distance <= 10*u*S`: finite truth allows an accurate finite value or
`normal.numerical_boundary`; overflow truth allows `normal.value` or boundary,
and never a finite result. The two-unit edge allowance is empirical
classification evidence, not a formal platform-error theorem. At all other
points, the declared guard/status must match. 2,049 adjacent subnormal scales
around the mean-point overflow threshold exercise this policy.

Affine inverse checks the actual nearest-even overflow midpoint
`T = MAX + 2^970`. Guarding at `MAX` would incorrectly reject a finite `MAX`
mean plus a harmless tiny perturbation. For a nonmedian computed standard
quantile `z`, it forms a quarter-scaled product, recovers the product residual
with fused multiply-add, and uses `exact_sum` to subtract the signed midpoint
as separate `MAX/4` and `2^968` terms. This retains the small gap through
cancellation without overflowing the unscaled product. A quarter product
that already overflows cannot be rescued by any finite mean; the public
fused result then supplies the normal result-overflow error.

With signed quarter-scaled gap `g` and standard quantile allowance `A_z`
specified above, the inverse guard is
`abs(g) <= A`, where `A = (scale/4)*A_z + 2*u*abs(g) + 2*MIN_SUB`.
The final two terms cover gap rounding and possible subnormal losses. Near a
genuine overflow midpoint, the quarter product and location are normal;
the two-step absolute floor is negligible there. The exact median bypasses
this guard and returns the mean. The error carries quantity `normal quantile`
and a finite log magnitude calculated from the safely scaled affine value.
The standard-quantile allowance remains a measured conditional numerical
budget, not a universal theorem about platform elementary precision.

The independent Decimal oracle computes the true affine value with its own
bisected quantile and compares its quarter-scaled magnitude with `T/4`.
Its conditional edge tier is `0.75*A <= distance <= 1.25*A`, using the true
quantile in `A_z`. This is the same fractional edge width as the density
`6..10` tier around its coefficient `8`. Finite truth permits an accurate
finite value or `normal.numerical_boundary`; overflow truth permits the
correctly signed result-overflow error or boundary error, and never a finite
value. Outside that tier, the declared status and error kind must match.
Thirty independent midpoint neighborhoods add 7,710 exact-input affine
rows: 257 adjacent scales per neighborhood, eight center/tail probability
choices across both signs, and all five means `-MAX`, `-MAX/2`, `0`, `MAX/2`,
`MAX` where the critical scale is finite and positive.

All observed finite results also satisfy their public domains: nonnegative
density, CDF/survival in `[0,1]`, nonpositive finite log probabilities and
finite quantiles. Validators reject invalid computed results; they do not
clamp them into a domain.

| operation | rows | finite observations | errors | normalized p50 | normalized p95 | normalized p99 | maximum normalized error | maximum raw ULP |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| PDF | 3,571 | 2,257 | 1,314 | 0.017892 | 0.126645 | 0.144712 | 1.000000 | 1,181 |
| CDF | 1,522 | 1,519 | 3 | 0 | 0.038693 | 0.143927 | 0.503746 | 413 |
| survival | 1,522 | 1,519 | 3 | 0 | 0.039040 | 0.125000 | 0.503746 | 404 |
| log CDF | 1,522 | 1,480 | 42 | 0 | 0.181220 | 0.269837 | 0.486725 | 404 |
| log survival | 1,522 | 1,480 | 42 | 0 | 0.180024 | 0.269837 | 0.486725 | 413 |
| inverse | 10,287 | 5,942 | 4,345 | 0 | 0.111111 | 0.140522 | 0.217865 | 79 |

Normalized distributions divide absolute observed error by the declared
allowance and exclude rounded-zero truth; the consumer reports zero and
subnormal counts separately. Counts in the finite/error columns include
the conditional edge tiers.

There are 284 density guard-edge rows: 80 finite observations and 204 errors,
with 11 reference-versus-observed guard/status differences admitted only by
that tier. The inverse has 430 guard-edge rows: 93 finite observations,
337 errors and 53 classification differences admitted only by its tier.
Those rows include 105 finite and 105 overflow truths on the negative side,
and 110 of each on the positive side. Every observed finite value is still
checked for accuracy. The PDF
maximum normalized error equals the subnormal absolute-step allowance; raw
ULP distance does not measure the conditioning of extreme affine results or
tail exponential arguments. All six consumers pass on native macOS ARM64
with Rust 1.99.0. The generator constructs `u` by exact conversion of the
binary64 value `2^-53`; its frozen SHA-256 is
`81714cac690f289a228cb5607a34a5f25d03326135cad0958f4b831181ee5798`.
Reproduction uses `python3 validation/generator/normal_oracles.py --check`
and compares all fixture bytes while retaining their recorded generation
timestamp and platform. Other native platforms and final source-freeze
checks remain separate open gates; this evidence does not grant owner API
approval or release readiness.
