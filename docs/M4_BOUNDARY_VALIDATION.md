# Local M4 factor boundary evidence

This report supplements [M4_NUMERICAL_VALIDATION](M4_NUMERICAL_VALIDATION.md)
with a separate range-boundary tier for FS-RATE/FS-DISC. It records native
aarch64 macOS, Rust 1.99 and Python 3.9.6 evidence. It does not establish a
portable elementary-function error bound, primary-platform CI success, M4
acceptance or publication.

## Defect and conservative policy

Before the boundary guard, a separate 2,538-case investigation found 92
compound/discount status disagreements near overflow or rounding to zero.
Of these, 91 had stable independent Decimal outcomes: 40 compound and 51
discount cases. One was an unstable Decimal midpoint and must not be counted
as established oracle evidence. For example, Annual Compounded `r=1,t=1024`
has exact factor `2^1024`, yet logarithmic evaluation returned a finite value.
The analogous reciprocal case is `r=-0.5,t=1024`.

For the 91 stable disagreements, the largest distance between the independent
logarithm and the relevant boundary divided by `8u(1+|x|)` was
`0.2553995793849702`; the median was `0.04326484218186016`, with `u=2^-53`.
Thus the proposed margin covered this investigation. This is measured
evidence on that scenario set, rather than proof for all binary64 inputs.

Production now reports `rate.numerical_boundary` when its **computed binary64**
logarithm lies within `8u(1+|x|)` of `ln(MAX)` or
`ln(MIN_SUB)-ln(2)` before exponentiation. Discounting separately checks
`-ln(computed compound factor)` near `ln(MAX)` before taking one reciprocal.
These are conservative policy centers. The exact round-to-overflow midpoint
is `2^1024-2^970`; the exact halfway-to-zero threshold is `2^-1075`.
Fused Simple arithmetic is exempt. A definite infinite exponent receives
`rate.value`, not an uncertainty error. Rate conversion may continue in log
space without requiring a representable intermediate factor.

Rust documents unspecified precision for elementary functions, while fused
`mul_add` has one rounding. Consequently, membership of the mathematical
logarithm in the guard band cannot universally predict membership of a
platform's computed argument. This report's exact-status claim covers the
clear inner/outer grid below. See the primary
[Rust f64 documentation](https://doc.rust-lang.org/std/primitive.f64.html).

The separate reference tier limits conditional statuses to true-log distance
`d` between `6S` and `10S`, where `S=u(1+|x|)`. A mathematically finite factor
there may produce an E3-accepted value or a numerical-boundary error; a true
overflow or zero-underflow must produce `rate.value` or
`rate.numerical_boundary`, never a finite result. The additional `2S`
argument/threshold allowance is
empirical, not a formal libm bound. The production guard remains `8S` and no
finite-value tolerance was widened.

## Independent oracle and coverage

[rate_boundaries.py](../validation/generator/rate_boundaries.py) imports only
the standard-library equation helpers in
[rate_oracles.py](../validation/generator/rate_oracles.py). It records both
source hashes, exact binary64 inputs, precision, platform and generation time
in [boundaries-oracle.fixture](../validation/fixtures/rates/boundaries-oracle.fixture).
It does not import Rust or QuantLib outputs.

Exponential probes use independently computed Decimal annual growth and
120/200-digit agreement on status and rounded output. Simple probes use exact
`Fraction` arithmetic, round the compound factor once, and then round one
reciprocal. An expanded Simple case (`r=100`,
`t=1.797693134862315e306`) exposed different rounded results at 120 and 200
Decimal digits; Fraction resolves the near-midpoint instead of selecting a
precision. Decimal agreement alone is not a certified interval proof. Python
documents [Decimal rounding](https://docs.python.org/3.9/library/decimal.html)
and [exact float-to-Fraction conversion](https://docs.python.org/3.9/library/fractions.html).

| Scenario family | Rows | Independent evidence |
|---|---:|---|
| Logarithmic boundary neighbors | 13,020 | All frequencies; positive/negative rates, near-minus-frequency and maximal rates; horizons offset by 0, ±1, ±2, ±32, ±64, ±128, ±512 and ±4,096 binary64 neighbors |
| Frequency switch neighbors | 720 | Each periodic configuration at the predecessor, exact rounded `1/m`, and successor; both hybrid directions and extreme rates |
| Simple range boundaries | 286 | Exact rational fused cancellation and maximal-factor cases |
| Fused residual | 2 | Exact positive `2^-104` retained where separate multiplication loses it |
| Exact powers of two | 40 | Annual Compounded and SimpleThenCompounded; `2^1024`, `2^-1075` and adjacent representable/range cases |
| **Total** | **14,068** | 7,034 compound and 7,034 discount rows |

The tier covers all 32 configurations and both active phases of every hybrid.
Long-horizon probes for CompoundedThenSimple use its Simple phase naturally;
its bounded periodic phase cannot reach negative logarithmic range limits
with a valid binary64 rate. Maximal rates and switch neighbors exercise the
reachable positive extreme instead.

Inside the uncertainty band, mathematical rounding is deliberately
unclassified unless an exact rational anchor exists. `2^-1075` is exactly
half the smallest subnormal and rounds to even zero; two Decimal precisions
previously disagreed there. The Rust consumer independently rebuilds the
anchor's integer numerator/denominator by digit carries and constructs finite
powers directly from IEEE exponent/significand bits. The overflow and
halfway-zero anchors reject with `rate.numerical_boundary` on this native
clear-grid run; the mathematical oracle does not assign them a finite value.

## Results and reproduction

[rates_boundaries.rs](../tests/rates_boundaries.rs) has five passing tests.
The fixture outcomes were 6,008 finite values, 2,542 numerical-boundary
errors and 5,518 value errors, with exact kinds on this native clear-grid run.
Finite factors use the original E3 bound `8u(1+|ln(expected)|)` plus one
minimum-subnormal absolute unit for subnormal expectations. Maximum
normalized normal-factor error was `2.307315542243634`, below 8; maximum ULP
distance was 1,640. There were 2,427 subnormal finite values and 152 exact
Simple finite values matching bit for bit.

Three controlled tests additionally scan every adjacent computed Continuous
argument at offsets -64 through +64 around both compound boundaries and the
reciprocal boundary: 387 probes. They check the specified computed-argument
policy, distinguish ordinary range errors, and verify one reciprocal of the
actual positive factor. These policy tests complement the independent
financial-value oracle; they do not certify libm accuracy.

```sh
python3 validation/generator/rate_boundaries.py --check
cargo test --offline --test rates_boundaries -- --nocapture
cargo clippy --offline --test rates_boundaries -- -D warnings
```

Use the fixture's recorded Python version when reproducing its metadata.
Generator/source hashes must agree, and `--check` must reproduce every row;
the timestamp and platform header are the only ignored fields. The focused
Rust test run took about 0.14 seconds on this machine. The original 92-case
disagreement count describes the pre-guard audit, not the current generator's
outputs or a full-domain accuracy claim.
