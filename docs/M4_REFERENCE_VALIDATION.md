# M4 public reference validation

Status: all 77,446 local reference rows pass the native aarch64 macOS
consumer after independent value/domain checks and explicit guard-edge
classification. This does not accept M4, certify platform elementary functions
or claim native Linux/Windows execution.

## Provenance and independence

`validation/generator/rate_quantlib_fixtures.py` calls the installed QuantLib
1.43 public `InterestRate` constructor, `compoundFactor`, `discountFactor`,
`impliedRate` and `equivalentRate`. Before querying, it compares every installed
`QuantLib/` wheel member byte-for-byte by SHA-256 with
`quantlib-1.43-cp39-abi3-macosx_11_0_arm64.whl`; the wheel digest is
`85d70b28778afca84b9b6ae74ef05a1069b2dca00e80fc74aa8ddf438259dc0c`.
The runtime is Python 3.9.6 on macOS arm64. The wheel's exact source commit
has not been independently attested; source baseline metadata records that
limitation. Public upstream rate equations are visible in
[QuantLib's InterestRate implementation](https://github.com/lballabio/QuantLib/blob/6b57206e04598f092efee66e3b367efc84771995/ql/interestrate.cpp).
The fixtures are runtime observations, not values reconstructed from that
source, and contain no copied implementation text.

The generator imports `rate_oracles.py`, records that dependency's SHA-256,
and uses exact Fraction arithmetic for Simple/basic-ratio operations.
Elementary formulas evaluate exact binary64 inputs at 120 and 200 Decimal
digits; rounded outputs, mathematical statuses and guard scores must agree.
Precision agreement is stability evidence rather than an interval certificate.
`Decimal.from_float` preserves the exact input; Decimal logarithm and
exponential functions provide a separate arithmetic implementation. See the
[Python Decimal specification](https://docs.python.org/3/library/decimal.html).
Series evaluations preserve tiny `log1p`/`expm1` increments. No Rust output
is read by either generator.

There is a second, deliberately different oracle for source rounding:
already-rounded binary64 base/exponent arguments are evaluated with Decimal
at both precisions. Python's public libm power result must satisfy unchanged
E3 against that model. The recorded subtraction and multiplication then
reproduce the public QuantLib inferred rate exactly. For equivalent rates,
an additional public `compoundFactor` followed by public `impliedRate` must
match public `equivalentRate` exactly. These source-stage checks explain a
reference discrepancy; they never replace the exact-input project oracle.

## Coverage and comparison rules

The four `*-quantlib.fixture` files contain 77,446 observations: 8,848
compound, 8,848 discount, 3,810 implied and 55,940 equivalent cases. All 32
configurations and all 1,024 source/target configuration pairs appear.
Hybrid source and target thresholds include the rounded `1/m` value and
both adjacent binary64 values. Inputs cover negative and zero rates,
subnormals, tiny rates, the quote `-m` boundary and its next valid neighbor,
largest finite values, invalid finite inputs and NaN/infinities.

A separate actual-reference range grid uses all applicable exponential
configurations at overflow, round-to-zero and reciprocal-overflow logarithms.
For rates `0.05`, `1` and `-0.05`, times surround each center by 0,
±1, ±4, ±8, ±16, ±32 and ±64 binary64 steps. Long-horizon
`CompoundedThenSimple` is correctly classified as Simple and is outside this
exponential grid. Independent exact-rational boundary work is recorded in
[M4_BOUNDARY_VALIDATION](M4_BOUNDARY_VALIDATION.md).

`tests/rates_quantlib.rs` checks every project result against the independent
oracle before checking any reference classification. E3 remains
`8u(1+|ln C|)` relative, with one minimum-subnormal absolute rounding unit
only for subnormal factors. E4 retains `k=8` and its declared `u|r|+u/t`
scale, adding the derived argument sensitivities `(m+r)|ln(1+r/m)|` for
periodic inference and `exp(x)|x|/t` for equivalent Simple inference. These
are conditioning terms multiplied by `u`; no blanket tolerance increase
absorbs QuantLib's rounded base or factor. Canonical project zero and typed
errors are checked separately. Header counts, dependency hash, actual wheel
digest, rounding stages, configuration coverage and discrepancy predicates
are enforced by the consumer.

## Narrow discrepancy taxonomy

| Class | Required mechanism or domain predicate | Representative evidence |
|---|---|---|
| D5 rounded base | Periodic public factor matches Decimal of rounded `1+r/m` and rounded `mt`, but exact-input comparison exceeds E3 or loses a positive factor | Annual `r=1e-12,t=100`: reference `1.000000000100009`, oracle `1.0000000001` |
| D5 base erasure | Nonzero periodic quote rounds its base to 1; exact growth is unrepresentable but reference returns 1 | Tiny positive periodic rate with maximum finite horizon |
| D5 rounded-base range | Valid periodic quote; exact-input factor/range fails while rounded-base public pipeline returns a positive finite result | Semiannual `r=.05,t=14372.369677280592`: reference `1.79769313485908e308`, exact factor overflows |
| D5 pow subtraction | Implied/equivalent periodic inference matches checked rounded power followed by subtraction of 1; result exceeds E4 or an exact nonzero rate becomes zero | Annual implied `C=.5,t=100`: reference `-.006907504562964073`, oracle `-.006907504562964098` |
| D5 factor erasure | Equivalent source quote is nonzero, horizon positive, public intermediate factor exactly 1, inferred reference rate exactly 0 | Daily `r=1e-20,t=1`: oracle equivalent Continuous rate `1e-20`, reference 0 |
| D5 intermediate factor | Equivalent source intermediate factor is zero/nonfinite even when the direct growth conversion can return a finite valid rate | Continuous `r=.05,t=f64::MAX` converted to Continuous preserves `.05`; reference returns infinity |
| D5 rounded-base roundtrip | Source is periodic, target active phase is not; checked compound-to-implied pipeline exceeds original E4 | Semiannual `.05,t=100` converted to Continuous |
| D9 input/quote | Nonfinite rate/time/factor, or periodic/hybrid input quote `r<=-m`; reference accepts a value | Semiannual `r=-2,t=0`: reference factor 1; project rejects quote |
| D9 output domain | Nonfinite/nonpositive factors, nonfinite inferred rates, inferred `r<=-m`, rounded active Simple factor `<=0`, or unrepresentable nonzero implied rate | Extremely small positive factor may infer exactly `-m`; project rejects unusable quote |
| D9 numerical boundary | Exponential range is unresolved under the explicit conservative E3 band; project returns `rate.numerical_boundary` | Annual `r=f64::MAX,t=1`; exact/fused Simple paths are exempt |

Agreement includes matching rejection; D5 also explicitly records loss of
nonzero rates even when a broad factor-conditioned E4 bound would hide that
loss. Each label has a checked predicate. Unknown exceptions, output statuses
or numerical mechanisms abort generation. A source rounding explanation is
not permission for the project to violate its own oracle or domain contract.

## Reproduction

Run with the attested wheel already installed in a separate validation
environment:

```sh
python3 validation/generator/rate_quantlib_fixtures.py --quantlib-wheel /path/to/quantlib-1.43-cp39-abi3-macosx_11_0_arm64.whl
python3 validation/generator/rate_quantlib_fixtures.py --quantlib-wheel /path/to/quantlib-1.43-cp39-abi3-macosx_11_0_arm64.whl --check
cargo test --offline --test rates_quantlib -- --nocapture
cargo clippy --offline --test rates_quantlib -- -D warnings
```

`--check` re-attests the installed runtime, reproduces every row and stable
header field, and ignores only generation time and reported host details.
The generator digest and imported-oracle digest are embedded in every header.
Native primary-platform CI and milestone/owner review remain outstanding.
Rust supplies no platform elementary-function error proof; the local error
budgets and edge allowance are measured acceptance criteria.

## Computed guard and the independent financial gate

The production guard operates on its computed binary64 logarithm; exact-real
membership in that guard is not a portable discrete-output guarantee. A
separate **D5-RANGE-GUARD-EDGE** field records only the annulus
`6S <= abs(L-B) <= 10S`, with `S=u*(1+abs(L))`, around the unchanged `8S` guard.
All applicable compound and reciprocal guards are checked: an edge of one
cannot excuse the strict central region of another.

For `H=abs(L-B)-8u*(1+abs(L))`, argument/boundary drift changes `H` by at most
`abs(delta_L)+abs(delta_B)+8u*abs(delta_L)`. The additional `2S` edge allowance
is supported by the local observed mismatch scores 6.4437–7.9902; it is not
a formal consequence of Rust's unspecified precision or the original `8S`
budget. The implementation guard and finite-value E3/E4 tolerances stay
unchanged.

There are 232 annotated edge rows: 99 compound and 133 discount. Their true
financial results comprise 102 finite values and 130 out-of-range results.
For finite truth, the project may return an E3-accepted positive finite value
or a checked numerical-boundary error; an ordinary value error fails. For
true overflow or zero underflow, only a value or numerical-boundary error is
accepted, never a finite result. Discount truth rounds the compound factor
first and then takes an exact Fraction reciprocal, matching the API policy.
Exact power-of-two anchors and Simple arithmetic retain strict checks.

On this native run, 38 edge outcomes differ from exact-real guard membership:
32 return an ordinary range error and six return a valid E3 finite value.
All other edge rows have the recorded ideal outcome. Specifically, compound
has 15 range errors and four finite values instead of the ideal boundary
error; discount has 17 range errors and two finite values. Every case passes
the independent financial gate. These differences are openly recorded rather
than counted as exact kind agreement or hidden by a wider value tolerance.

## Local results

| Operation | Rows | Reference agreement | Classified D5 | Classified D9 |
|---|---:|---:|---:|---:|
| Compound | 8,848 | 5,567 | 878 | 2,403 |
| Discount | 8,848 | 5,442 | 742 | 2,664 |
| Implied | 3,810 | 2,454 | 328 | 1,028 |
| Equivalent | 55,940 | 34,852 | 19,646 | 1,442 |
| **Total** | **77,446** | **48,315** | **21,594** | **7,537** |

The guard-edge annotation is orthogonal to this reference discrepancy table;
it does not add rows or replace source-rounding explanations. The consumer
checks every classification predicate, recorded count and imported oracle
hash. The regenerated fixtures and generator `--check` pass against the
attested wheel. The factor comparator forms its relative allowance before
multiplying by a large expected value, avoiding intermediate overflow that
would otherwise make the comparison vacuous. Rust independently enforces
the same finite allowance and catches incorrect reference classifications.
