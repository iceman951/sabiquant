# Local M4 numerical evidence

This report covers the local `rates` implementation under the delegated
authority in [M4_LOCAL_READINESS](M4_LOCAL_READINESS.md). It records native
aarch64 macOS evidence, not primary-platform CI, milestone acceptance or
publication. The independent date-context and reference reports are
[M4_DATE_CONTRACT](M4_DATE_CONTRACT.md) and
[M4_REFERENCE_VALIDATION](M4_REFERENCE_VALIDATION.md).

## Equations and operation order

Every quote owns its accrual convention and explicitly validated compounding.
The five methods have 32 supported configurations: Simple, Continuous and
three periodic methods at each of ten frequencies. Periodic and hybrid quotes
require `r > -m` even when the requested horizon would use the simple branch.

- Simple factors use the correctly rounded fused `r.mul_add(t, 1)`.
- Periodic annual growth uses `m*ln_1p(r/m)`. Near the negative boundary,
  `m+r` is exact by Sterbenz subtraction and `m*ln((m+r)/m)` avoids losing the
  distance from minus one in the quotient. Below machine epsilon, the
  first-order annual growth returns `r`, preserving subnormal inputs.
- Factors exponentiate annual growth times the horizon. Discounts require a
  representable compound factor and then one correctly rounded reciprocal.
- Periodic/continuous equivalence converts annual log growth without forming
  a factor or multiplying by the horizon. Equal active phases return the
  original rate after checking the source and target domains.
- Simple inference from supplied factors uses direct arithmetic. Discount
  inference rescales before dividing by a tiny discount. Other discount
  inference uses `-ln(P)`, without requiring a representable reciprocal.
- Simple target conversion uses `expm1`; large positive increments are
  rescaled before exponentiation when division by the horizon can keep the
  final rate finite. Inferred/converted rates are checked after rounding,
  including the active simple factor's positivity.
- Present values multiply a finite signed amount by the discount. A nonzero
  amount whose product overflows or underflows to zero is rejected. Zero
  amount still checks horizon and factor validity.

The platform elementary functions have unspecified precision, including
within one execution. Rust documents correctly rounded fused/basic arithmetic
but does not supply a worst-case error bound for `ln`, `exp`, `ln_1p` or
`exp_m1`. The numerical bounds below are measured acceptance criteria for
these scenario sets, not a proof for every binary64 input. See the primary
[Rust f64 documentation](https://doc.rust-lang.org/std/primitive.f64.html).

## Independent oracle

[rate_oracles.py](../validation/generator/rate_oracles.py) uses only Python's
standard library and the FS-RATE equations. Simple factors, direct simple
inferences, reciprocal outputs and active simple domain checks use exact
`Fraction` arithmetic with binary64 inputs. Elementary inputs use
`Decimal.from_float`, preserving the exact binary64 value. Decimal logarithms
and exponentials use 120 and 200 digits. Near zero, independently summed
convergent Taylor series retain increments that decimal addition to one would
otherwise lose. The two precisions must agree on the complete domain status,
error kind and final rounded binary64 value for every row.

The generator never imports Rust or QuantLib. Its hash, precision, environment
and rounded hybrid switch rule are recorded in each fixture. Matching two
high precisions is evidence of stable rounding on this grid, rather than an
interval-arithmetic certificate. Python documents its elementary Decimal
functions in the primary
[Decimal reference](https://docs.python.org/3.9/library/decimal.html).

| Operation | Rows | Finite values | Exact typed-error statuses |
|---|---:|---:|---:|
| Compound factor | 7,404 | 5,694 | 1,710 |
| Discount factor | 7,404 | 5,692 | 1,712 |
| Implied compound rate | 2,816 | 1,532 | 1,284 |
| Implied discount rate | 2,816 | 1,533 | 1,283 |
| Equivalent rate | 42,624 | 38,724 | 3,900 |
| **Total** | **63,064** | **53,175** | **9,889** |

The grid includes all configurations and all 32² conversion pairs, signed
zero, minimum subnormal rates/times/factors, tiny rates, negative rates,
largest finite inputs, each negative periodic boundary and its next valid
value, rounded hybrid switches and both adjacent binary64 values. An exact
simple cancellation case retains a `2^-104` fused factor that separate
multiplication/addition can lose. Additional regressions exercise rates whose
rounded target lands at its active simple boundary.

## D5-RATE-EXPONENTIAL-CONDITIONING

The original E4 proposal `8u*(abs(r)+1/t)` modeled near-unit factor rounding
and continuous inference. It omitted the sensitivity of an exponential
target to error in its argument. Classification is **D5**, a tolerance model
correction supported by derivatives; the multiplier remains **8**.

For periodic inference, `r=m*expm1(x)`, so
`dr/dx=m*exp(x)=m+r`. An argument error of scale `u*abs(x)` therefore contributes
`u*(m+r)*abs(x)`. Here `x=ln(1+r/m)`. For a simple rate obtained through log
growth, `r=expm1(x)/t`, so `dr/dx=r+1/t` and `x=ln(1+r*t)`.

Two local counterexamples demonstrated the missing terms:

- Infer an annual compounded rate from `C=1.1`, `t=1/365`. The implementation
  returned `1283305580313382.8`, compared with the oracle
  `1283305580313389.3`: difference 6.5, or 26 ULP. The original bound was
  approximately 1.1398. The additional periodic derivative term accounts for
  this sensitivity without changing `k`.
- Convert a monthly compounded rate of 0.05 to Simple at `t=100`. The values
  were `1.4587944942491362` and `1.4587944942491375`: six ULP. The original
  bound was about `1.30455e-15`, below the observed difference; the simple
  exponential derivative supplies the missing scale.

The fixture consumer uses E4 scale
`s=u*abs(r)+u/t+u*A`, with the active target branch's derivative term `A`
specified in [NUMERICAL_CORRECTNESS §7](NUMERICAL_CORRECTNESS.md#7-tolerance-policy).
It applies `u` before potentially overflowing arithmetic. Direct simple
inference and continuous targets have `A=0`; the exact zero-rate rule is
checked separately.

## Rate representation and factor reconstruction

Independent audit found a separate domain defect in the prototype:
`implied_rate(min_subnormal,1,Simple)` and conversion of a continuous rate
`-40` to Simple at `t=1` rounded to rate `-1`, then failed when used because
the reconstructed factor was zero. The implementation and independent oracle
now reject that rounded result. Dedicated tests also cover implied discounts,
both active simple hybrid branches and date-based conversion.

A valid rounded rate can still have a large relative factor reconstruction
error near cancellation. With continuous rate `-34.5`, `t=1`, the source factor
is about `1.039538011670222e-15`. The correctly rounded simple rate is
`-0.999999999999999` (bits `bfef_ffff_ffff_fff7`); its factor is about
`9.992007221626409e-16`, roughly 3.88% lower. This does not violate the rate's
rounding accuracy. Simple has `dC/dr=t`, so factor reconstruction includes
the absolute term `t*abs(delta_r)` in addition to each factor evaluation's
E3 allowance. The regression uses `u*abs(r)` at `t=1` for the nearest-rate
representation contribution. An unconditional small relative reconstruction
bound would be false.

For other targets, `dC/dr=t*C` for Continuous and
`dC/dr=t*C/(1+r/m)` for Compounded. Rounded-factor inference additionally
propagates source-factor uncertainty through the inverse derivative; this is
distinct from comparing an exact supplied factor to the high-precision oracle.
Reference discrepancies are classified explicitly in the reference report.

## Measured distributions

Native Rust 1.99.0 debug execution of `rates_oracles` passed all rows after the
active-domain correction. Errors are compared exactly by kind; no error row
is treated as a floating-point value. Normalized factor error is
`abs(delta_C)/C / (u*(1+abs(ln(C))))`; normalized rate error is `abs(delta_r)/s`.
Acceptance is at most 8 in either case.

| Metric | Samples | Median | 95th percentile | 99th percentile | Maximum |
|---|---:|---:|---:|---:|---:|
| E3 normal factors | 11,331 | 0 | 0.5084128793 | 1.1088487047 | 2.1089603203 |
| E4 nonzero rates with finite bounds | 27,782 | 0 | 0.5249669502 | 0.8958166399 | 2.0 |

Another 55 positive subnormal factor rows use a one-smallest-subnormal absolute
step. There are 13,979 exact canonical zero rate rows and 28 rate rows whose
mathematical bound exceeds binary64 range. Those 28 pass domain/sign checks,
but their saturated comparison gives no useful accuracy guarantee and they
are excluded from the distribution. Separate exact identity/extreme-value
regressions supply additional evidence where applicable.

Maximum raw ULP distances are 655 (compound), 934 (discount), 694 (implied
compound), 400 (implied discount), and 6 (equivalent). Raw ULP grows at highly
sensitive exponentials; these results do not justify a blanket few-ULP claim.
`k=8` is retained as the locally measured acceptance constant. Native Linux
and Windows execution is still required before accepting it across the
primary platforms.

## Reproduction and remaining gates

```sh
python3 validation/generator/rate_oracles.py --check
cargo test --offline --test rates_core --test rates_oracles -- --nocapture
cargo test --offline --test rates_dates --test rates_properties
```

The complete native check and coverage evidence is recorded below. Remote
writes remain unauthorized. Native
cross-platform CI, independent milestone/owner review and M3's outstanding
convention/source/maintenance gates remain unresolved; this report does not
declare M4 complete.

## Separate range-boundary tier

The coarse grid above is not an exhaustive range-classification claim.
Independent adaptive audit found 92 finite/error disagreements in 2,538 cases
around overflow, reciprocal overflow and zero-rounding surfaces. One involved
an unstable Decimal tie: the exact factor `2^-1075` rounds to zero, but
120/200-digit logarithmic formulas landed on opposite sides. The generator's
two-precision guard detected this; exact rational anchors resolve such ties.

The local policy now returns `rate.numerical_boundary` when the elementary
factor's computed logarithm lies within the unchanged E3 allowance
`8u*(1+abs(logarithm))` of `ln(MAX)` or `ln(min_subnormal)-ln(2)`. Discounts also
guard reciprocal overflow. Simple fused arithmetic is exempt and receives
exact bit comparisons against its independently rounded oracle, including
each hybrid's active simple phase. Percent/basis-point helpers likewise use
one correctly rounded basic operation and reject a lost nonzero result.

This conservative error is an explicit **D9** domain policy, justified by the
observed uncertain classification, rather than an increased value tolerance.
It can reject a mathematically finite result very close to a range surface.
The margin remains a locally measured acceptance budget, not a proof of
platform `libm` behavior. The expanded independent tier and exact integer-power
anchors are recorded in [M4_BOUNDARY_VALIDATION](M4_BOUNDARY_VALIDATION.md).

## Complete local check evidence

On 2026-10-12, Rust 1.99.0 on aarch64 macOS passed **281 debug tests** and
**280 release tests**, including seven doctests, with zero failed or ignored
checks. The existing debug-only internal invariant assertion accounts for
the difference. Strict all-target Clippy, formatting, warning-denied public
and private rustdoc, offline dependency inspection and both x86_64 Linux GNU
and Windows MSVC all-target compile checks pass. Cross-target compilation is
not native execution or R3 CI evidence.

All three M4 generators reproduce their committed evidence with `--check`;
the QuantLib generator re-attests its installed wheel. M4 adds **154,578**
fixture records (63,064 equation-oracle, 14,068 clear-boundary and 77,446
actual-reference), bringing the repository to **598,564** records. Record
counts measure scenarios, not distinct financial features or exhaustive
binary64 coverage. The date suite tests **1,245,184** source/target/context/date
conversions and **24,320** quote configurations; seeded properties cover
20,000 cases and concurrent reuse checks 8,192 observations.

Native LLVM instrumentation in
`/private/tmp/sabiquant-m4-complete-coverage` measures production source regions
with embedded test modules excluded:

| Source | Covered / regions | Coverage |
|---|---:|---:|
| `rates.rs` | 778 / 785 | 99.11% |
| `numeric.rs` | 38 / 38 | 100% |
| `error.rs` | 292 / 295 | 98.98% |

The seven uncovered rate regions are formatter-failure propagation and
invariant-only errors for already validated scalar/termination dates,
nonnegative finite owned date accrual and date-based annual growth. The
reachable arbitrary-horizon negative growth overflow and huge-product
rescaling are separately tested. An unused private inference branch was
removed because both public Simple inference paths use direct arithmetic;
no impossible private values were fabricated to inflate coverage. These
are source-region measurements, not branch coverage or exhaustive-input proof.

A further exact Simple regression covers subnormal discounts: with compound
factor about `1.739769535547968e308`, `P*C` differs from one by `4u`, despite a
correctly rounded reciprocal. The usual `2u` identity allowance assumes a
normal reciprocal; subnormal rounding contributes the absolute term
`C*MIN_SUB/2`, plus final-product rounding. The invariant now states this
condition and also includes E3 variance for separately evaluated elementary
factors.

Remaining acceptance gates are native primary-platform CI and independent
milestone/owner review. M3's unresolved US/AFB sources and TARGET maintenance
appointment also remain release gates. Local M5 API/package/validation review
may proceed under the same delegated authority, without marking M4 accepted
or publishing the crate.

## Rounded hybrid switch conditioning

M5 independent review exposed another false unconditional invariant in the
original numerical policy. For frequency 13, `r=(-13).next_up()` and
`t=rounded(1/13)`, SimpleThenCompounded gives about `8.113168256876145e-17`;
at `t.next_up()`, its periodic phase gives about `1.3664283380001849e-16`.
The factor rises about 68.4% despite a negative rate. Both individual branch
formulas agree with their independent oracle; the rounded switch sits away
from the exact rational equality point and the tiny factor magnifies that
input difference. The coarse oracle already covers every near-minus-frequency
rate and both threshold neighbors; the monotonic property grid is a normal,
well-conditioned domain rather than a full-domain proof.

The policy now qualifies computed monotonicity and E3 switch continuity by
conditioning. A true exact rational hybrid remains mathematically continuous;
the current operational rounded-threshold contract does not promise relative
continuity near a zero simple factor. No implementation, formula value
tolerance or reference predicate is changed by this specification correction.
