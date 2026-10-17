# Independent review of the generic bond yield/risk proposal

Review date: 2026-10-17. Scope: the complete
[generic yield/risk proposal](M8_BOND_YIELD_RISK_DESIGN.md), the adopted
[bond core](M8_BOND_CORE_CONTRACT.md), [M6 solver contract](M6_SOLVER_RESEARCH.md),
[date-rate contract](M4_DATE_CONTRACT.md),
[M4 numerical evidence](M4_NUMERICAL_VALIDATION.md), and
[M4 domain decisions](M4_LOCAL_READINESS.md).

**No blocking mathematical or capacity defect was found in this proposal.**
The fixed residual scale, derivative equations, extended term representation
and separate raw/normalized admission policies are coherent. The decisions
listed below still need explicit names and operation order in the future
contract. This review adopts no contract, adds no production implementation,
and supplies no financial, milestone or platform acceptance.

The reviewer did not read Rust bond, quote, rate, sum or solver production.
The proposal's statement that its author inspected those files describes that
author's work, not this review. Only this new review document was added to the
repository. Temporary independent arithmetic checks used Python standard-library
integers, `Fraction` and `Decimal`; no QuantLib source/output or remote write
was involved.

## Residual, solver and cached state

The proposed threshold is correct. With `MAX=2^1024-2^971`, the nearest-even
overflow midpoint is `MAX+2^970`.

- If `abs(T)<2^970`, `abs(D_hat-T)<=MAX+abs(T)` is strictly below that
  midpoint. Every nonzero difference of finite binary64 values is an integer
  multiple of `2^-1074`, so rounding the exact difference cannot create zero.
- Otherwise, halving the exact difference bounds its magnitude by `MAX`.
  The smallest nonzero spacing beside a target of magnitude at least `2^970`
  is `2^917`, so halving cannot underflow or erase its sign.

Thus the exact signed difference followed by one final rounding is finite and
zero iff the supplied computed dirty value equals the target. Separately
rounding two halved operands is not equivalent. An overflowing unscaled
financial residual can remain a separately fallible report view without
invalidating the finite scaled solver residual.

The private fallible Brent boundary retains M6's checked bracket/options,
check-before-call budget, lower-first endpoint order, finite-point invariant,
bracket-width stopping and explicit precision exhaustion. It does not require
a public generic callback API or a NaN sentinel. An attempted callback that
returns its typed financial error must consume one invocation. No date or
context work should occur inside it.

Caching a payload at every retained endpoint/history entry is necessary:
bracket-tolerance termination may return an earlier endpoint rather than the
last trial. Returning its cached dirty amount avoids unbudgeted repricing and
an unsupported assumption about repeated elementary-function bits. Endpoint
permutations and replacement must move the payload with its coordinate/value.
The infallible public lift must preserve all existing M6 observations; this
review has not validated such a refactor because it is not implemented here.

The chosen M6 one-call lower-zero rule correctly means that only queried
coordinates have valid factors. An unqueried upper bracket endpoint need not
be admissible. Coordinate termination and computed callback zero still do not
certify mathematical existence, uniqueness or price accuracy.

## Constants, signs and identifiability

Positive amounts at positive times yield a strictly decreasing mathematical
price on the common fixed-branch domain; amounts at time zero add constants.
That supplies conditional mathematical uniqueness, not strict monotonicity of
the rounded callback, a root-existence certificate or a slope bound.

The proposed constant certificate is sufficient and deliberately incomplete.
Pairing opposite equal magnitudes within identical positive-time factor
classes cancels their real analytic contributions. A general class sum of
zero need not cancel separately rounded PVs, and separate elementary calls
need not return identical bits. A nonpairable cancellation remains
`Uncertified`; failure to obtain this certificate proves no sensitivity claim.

For a certified mathematical constant, one ordinary budgeted lower evaluation
must retain all factor/product failures. Only afterwards compare the exact
time-zero sum `C` with the exact finite target `T`, without first rounding `C`.
The proposed match/mismatch errors then express lack of an identifiable yield,
even if rounding noise creates occasional computed equalities. They do not
claim every other bracket coordinate is numerically admissible.

The proposal's signed example is correct:

```text
v = 1/(1+y); P = v^2-v
P(1/3) = P(3) = -3/16
P(1) = -1/4; P'(1) = 0
at y=0: P=0, P'=-1, P''=4, M=1
```

It supplies two roots, an even-multiplicity/tangential root and a zero-price
case with meaningful raw measures. A same-sign bracket can legitimately fail
to bracket the tangential root. A local zero derivative cannot establish a
constant model. Equal positive-horizon coupon/redemption cancellation is also
necessary; testing only time-zero flows would miss it.

## Risk equations and extended arithmetic

All three first/second logarithmic multiplier pairs in proposal §6 are correct
for a fixed supplied time and annual decimal yield. In particular periodic
second derivative uses `t*(t+1/m)/(1+y/m)^2`, and convexity is `P''/P` without
a half. Hybrid branch selection is fixed as yield varies. Time differentiation
through a switch and named fractional-prefix market formulas are separate.

The definitions use observed computed dirty price as the normalized
denominator. Comparisons to the real analytic model consequently need both
numerator and price error. The displayed ratio allowance follows from
`abs(P)>=abs(D_hat)-E_P` and the triangle inequality when
`abs(D_hat)>E_P`. No useful normalized certificate follows from that formula
when this condition fails. Negative dirty prices retain their sign; observed
zero requires the proposed explicit normalization error.

The periodic operational operands are safe under the stated domain:
`RN(m+y)>0` for `y>-m`; the near-boundary subtraction is exact by Sterbenz.
Adding an integer at most 365 to finite positive yield cannot reach the enormous
overflow midpoint gap at `MAX`. Similarly `RN(t+RN(1/m))` is positive finite:
the added value is at most 1, far below that gap. A valid Simple factor supplies
positive finite `q`. These operands approximate the analytic multipliers;
their rounding and the observed factor/PV seed still require the proposal's
explicit error propagation.

Five numerator significands and two denominator significands give products
below `2^265` and `2^106`; a five-word scalar integer kernel has ample capacity
for normalization and carry. For a full generic ratio with five finite nonzero
numerator factors and two denominator factors,

```text
2^-7418 < abs(term) < 2^7268.
```

The Simple second-derivative factor 2 occurs with fewer numerator factors and
fits inside this conservative envelope. A 53-bit normalized rounding retains
no bit below exponent `-7470`; its exponent carry remains below `2^7269`.
Basis `2^-8192` therefore contains every rounded term without underflow.
Accumulating fewer than `2^usize::BITS` terms in separate unsigned sign arrays
requires at most `15461+usize::BITS` coefficient bits. The proposed ceiling to
whole words is conservative: 243 words per unsigned array on 64-bit platforms,
approximately 12 KiB for six arrays.

Raw getters must round only after sign cancellation. Normalized getters must
divide that retained sum directly by nonzero dirty price, without calling a
raw getter first. The proposed public anchors are sound: Continuous at zero
yield with `(PV,t)=(MAX,2)` has overflowing raw moments/derivatives but ratios
`(Modified,Convexity,Macaulay)=(2,4,2)`; `(MIN_SUB,1/2)` has nonzero raw values
rounding to zero but finite ratios `(1/2,1/4,1/2)`. Exact opposite terms can
cancel outside binary64 before a raw getter admits the sum. Genuine final
ratio overflow/underflow remains an error.

## Decisions to pin in the adopted contract

These are implementation-boundary requirements, not discovered mathematical
counterexamples:

1. Specify the extended term's retained significand/exponent convention,
   nearest-even midpoint/carry normalization, signed zero and every rounding
   stage. Terms are rounded to 53 significant bits before exact accumulation;
   the result is not an exact sum of ideal analytic derivatives.
2. Include width proofs for scalar temporaries, normalization/division shifts,
   exact constant sum and residual scratch, not only the final arrays. At basis
   `2^-1074`, the two-price exact residual coefficient is below `2^2099`
   (at most 2,099 bits); summing at most `MAX_FLOWS<2^22` time-zero amounts
   gives a coefficient below `2^2120` (at most 2,120 bits).
   Thirty-four u64 words suffice for those current bounds before any separately
   needed checked temporary margin.
3. Pin quote-validation, allocation, flow/factor, dirty-reduction and risk-term
   order, along with getter operation labels. Allocation must not accidentally
   change the selected failure when the public contract requires quote
   validation first. Internal capacity failure must remain distinguishable from
   an ordinary final financial range limit.
4. Freeze exact constant-case diagnostics/counts and callback payload ownership
   rules. Constant handling intentionally reports model identifiability rather
   than letting the ordinary lower-zero solver result imply a unique yield.
5. Keep price-only inversion and each risk view independently fallible as
   proposed. Unused clean/per-100/raw-risk range failures must not reject an
   otherwise valid dirty solve or normalized measure.

## Actual independent arithmetic checks

The temporary script `/private/tmp/sabiquant-yield-risk-review.py` and JSON
`/private/tmp/sabiquant-yield-risk-review.json` completed with exit 0. They are
scratch mathematical checks, not repository implementation tests or a
substitute for future frozen oracles.

| Check | Actual evidence |
|---|---|
| Fixed-scale residual | 212,888 exact `Fraction` cases; all 2,047 finite exponent fields, four selected significands, both signs, 13 zero/subnormal/MAX/threshold targets; all finite, exact sign and zero equivalence |
| Scaled-term extremes | 128 corners of five numerator/two denominator factors chosen from MIN_SUB and MAX; lowest retained bit -7470, highest 7267, maximum coefficient 15,460 bits |
| Sum capacity | Maximum tested coefficient times MAX_FLOWS needs 15,482 bits; the generic usize-count bound also fits the proposed 32/64-bit array capacities |
| Fixed-time derivatives | 576 cases: all 32 configurations, six zero/ordinary/switch-neighbor time controls and three negative/zero/positive yields; 100/160-digit analytic agreement |
| Supplementary central differences | Steps `1e-8`, `5e-9`, `2.5e-9`; first/second derivative errors contract by approximately four on every nonzero-time case; maximum final absolute errors about `1.23e-16` / `6.11e-16` |
| Exact signed/edge anchors | Two roots, tangent, zero-price raw measures, raw overflow/underflow with finite normalized outputs verified with rational arithmetic |

The finite-difference checks hold dates/times and branch choice fixed. They
use high-precision mathematical factors, not repeated binary64 repricing, and
make no convergence claim as a binary64 step approaches machine resolution.
The residual and range arguments above are algebraic; selected finite grids
support them without being advertised as exhaustive over all binary64 pairs.

Scratch script SHA-256:
`2497d8261f0e6f8f6e7e2e85d831c9acd1de944a5dc23c1d4d2931cb2ec1446f`.

## Reviewed source snapshot and limits

| Document | SHA-256 |
|---|---|
| Yield/risk proposal | `6415f72064a9ab80b37e6e4bff65f0f53730e2ec7602f6414b310d4b20c191e4` |
| Bond core contract | `009f5da71d5a3b1af0b8d258754350f8810dff327a20d2c77efee17ce9fdd69c` |
| M6 solver contract | `bdbbf00b6f83bef259c82777bdc62544a616dbc7a13fe4734522d08239040b42` |
| M4 date contract | `47216e578082eb56daaffb6b49c45951a416f21fe36372baede2bd0a29cdea98` |
| M4 numerical evidence | `05a0c0416e9daf83a240088e149e087d7c5b7acb729f66a532a8c90cb325fe42` |
| M4 domain decisions | `ca5059ab51bff8cf31e59b54a432171809ee8dd8f6b4709a165719278fcd28b9` |

No fallible Brent implementation, risk kernel, financial derivative fixture,
public API or error census was validated by this review. Future implementation
still requires exact integer residual/term/sum/divide oracles, independent
elementary and signed-cancellation pricing/risk evidence, full M6 parity,
evaluation-budget/error tests, and the complete verification gates in the
proposal. M4's empirical precision qualification stays in force.

This generic tranche does not supply UK DMO quasi-period pricing, Treasury
fractional-first prefixes, Canadian models, automatic legal settlement or
ex-date calendars, or market denomination/rounding acceptance. Source and
capability gaps remain explicit obligations toward the full M8 objective.
