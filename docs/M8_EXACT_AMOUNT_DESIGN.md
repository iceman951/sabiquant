# M8 exact amount boundary proposal

| Field | Value |
|---|---|
| Status | **Historical design proposal.** Phase A was adopted separately as [M8_AMOUNT_CONTRACT](M8_AMOUNT_CONTRACT.md), which governs where they differ; phases B and C are not adopted |
| Baseline | Local `4da3f43`, design branch `feat/m8-bond-readiness` |
| Date | 2026-10-16, Asia/Bangkok |
| Governing decision | [Accepted ADR-0006](adr/ADR-0006-numeric-policy.md), especially decision 6 and its cash-settlement reconsideration trigger |
| Open issue | [NI-01](NUMERICAL_CORRECTNESS.md#10-future-numeric-representation-issues) |
| Existing boundary | [M7 analytical cash-flow contract](M7_CASHFLOW_CONTRACT.md) |

Recommend a small, currency-free exact decimal value and an explicit integer
quantizer before building a cash-settlement interface. Keep bond settlement
**dates**, payment entitlement and analytical prices separate from the exact
**amount** boundary. Their terminology overlaps; their numeric obligations do
not. This proposal changes no existing binary64 amount, coupon or rate API.

The accepted ADR explicitly defers exact settlement amounts. Adopting this
proposal therefore requires recording an ADR-0006 reconsideration and a local
NI-01 contract. Continuing local development authority permits preparing that
decision; this document does not record owner approval, close NI-01, or claim
M8 acceptance. The later settlement contract still needs financial choices
about price units, accrual, entitlement and the stage at which each amount is
rounded.

If adopted, exact decimal arithmetic, lattice choices and discrete errors
belong to existing `TOL-E0` exact-result checks. Outgoing nearest binary64
conversion requires a correctly rounded oracle (0 ULP); incoming quantum
rounding requires the exact integer result. Exact conversions either preserve
the supplied real value or return an error.
Existing accrual, factor and pricing approximation profiles remain in force;
none becomes `TOL-E0` merely because its final observation is quantized.

## 1. The exactness claim

An exact amount stores the exact supplied decimal rational. Quantization
returns the specified multiple of an explicit decimal quantum, with an exact
integer decision at ties. Neither statement establishes that a supplied price
is the correct mathematical bond price, that a binary64 coupon is exact decimal
accrual, or that a market rounds at the chosen stage.

For example, exact conversion and cent rounding of the supplied binary64
`2.675` gives `2.67` under nearest/ties-even: those bits represent a number
slightly below the decimal midpoint. Parsing the decimal text `"2.675"` and
applying the same rule gives `2.68`. Both operations can be exact for their
different supplied inputs. Replacing the former with shortest decimal text
changes the input interpretation; it is not a numerical repair.

If analytics supplies an error interval `[L,U]`, certifying the ideal rounded
settlement amount requires proving that every value in that interval rounds
to the same quantum multiple. An interval crossing a decision boundary cannot
be certified by a blanket float tolerance. Exact rounding of the observed
binary64 value can still be returned under an expressly chosen input contract;
it must not be labelled a certificate for the ideal amount. This distinction
belongs in future settlement result/provenance design, not in a hidden
rounding fallback.

## 2. Value and scale model

Proposed `DecimalAmount` has private `coefficient: i128` and `scale: u8` fields:

`value = coefficient / 10^scale`, with `0 <= scale <= 38`.

Inputs carry no currency, denomination, percentage convention or implicit
minor-unit count. The same exact value can represent a notional, an amount or
a supplied decimal price only when the surrounding API states the units.
There is no default currency or default quantum. A price per 100 is explicitly
converted by an amount/price API; the decimal type never infers that factor.

Normalize every successful value by removing coefficient factors of ten while
scale is positive. Canonical zero is `(0,0)`; negative zero is not retained.
Thus `(100,2)`, `(10,1)` and `(1,0)` compare equal and hash identically.
`coefficient()` and `scale()` inspect this normalized representation, not the
original text's significant figures. Retaining declared precision, presentation
scale, quote provenance or a settlement quantum needs a separate surrounding
value. Do not overload equality to recover that metadata.

All coefficient values, including `i128::MIN`, are valid. Positive and negative
limits are asymmetric:

```text
MIN = -170141183460469231731687303715884105728
MAX =  170141183460469231731687303715884105727
```

At scale 38 these are about minus/plus 1.7014; at scale zero they are about
minus/plus 1.7014e38. The model is a finite set of decimal rationals, not a
fixed universal absolute error or an arbitrary-precision decimal. The smallest
positive value is `1e-38`. Its nonuniform range is an explicit tradeoff.

Use separate signs and unsigned magnitudes for arithmetic. In particular,
never compute `abs(MIN)`, `-MIN`, `MIN/-1` or an unchecked signed cast as a
normalization step. Rust supplies
[`unsigned_abs`](https://doc.rust-lang.org/std/primitive.i128.html#method.unsigned_abs)
for obtaining the magnitude, including `2^127`. After exact cancellation and
normalization, admit magnitudes up to `MAX` for positive results and `2^127`
for negative results. Construct the latter endpoint explicitly. Checked
negation of a normalized `MIN` coefficient correctly fails.

Recommended traits are Copy, Clone, Debug, Eq, Hash, Ord, Send and Sync.
Ordering compares numeric values exactly, using wide aligned magnitudes;
lexicographic `(coefficient,scale)` order is wrong. Deriving Eq/Hash is sound
only because all construction paths enforce canonical representation. No
panicking arithmetic operators, lossy casts, implicit float conversion or
Default rounding policy are proposed.

## 3. Parsing and formatting

Recommend a deliberately small locale-independent grammar:

```text
[+-]?[0-9]+(\.[0-9]+)?
```

There is no whitespace, exponent, separator, underscore, Unicode digit,
currency symbol, NaN, infinity, bare decimal point or omitted integer/fractional
part. Leading zeros and an explicit plus sign are accepted. `-0`, `00.00` and
`+0.0` normalize to zero. `FromStr` is exact and never rounds.

Recommend `MAX_TEXT_BYTES = 128` as a named, documented resource policy. Check
the byte cap first, then lexical validity and its first offending byte, then
the raw fractional-place limit of 38, then normalized coefficient range. The
constructor likewise rejects an input scale above 38 before normalizing,
including for zero. This intentionally rejects `"0."` followed by 39 zeros;
the limit is an input precision policy as well as a representation limit.
Record that choice in a contract rather than allowing parser/constructor
inconsistency. The cap exceeds the longest canonical text (41 bytes).

Trim textual trailing fractional zeros and leading magnitude zeros before
converting significant digits to a magnitude. In particular, `MAX` followed by
`.0` must parse successfully even though concatenating every digit into an
`i128` would overflow. A lexically valid result outside the coefficient range
is a range error, never saturation or a binary64 fallback. Lexical scanning
uses bounded stack state, not an allocated copy of the supplied string.

`Display` emits canonical exact fixed decimal text: no exponent, no redundant
leading/trailing zeros, and `0` for zero. `parse(display(x)) == x`. It preserves
neither input spelling nor trailing zeros. Formatter precision must never
silently round/truncate this text; document that decimal precision flags do not
change the canonical representation. Fixed-width invoice formatting is a
separate explicit feature if needed. It must require a chosen scale/quantum
and reject inexact display unless the caller first invokes quantization.

## 4. Explicit quantum and modes

Proposed `AmountQuantum` stores a normalized, strictly positive DecimalAmount.
Its constructor takes an explicit positive `i128` coefficient and scale
0 through 38. It supports `0.01`, `0.05`, `0.025`, `1`, `100` and any other
positive representable decimal quantum. Zero and negative quanta are errors.
Numeric equality and hashing match normalized quantum values. It has no
Default. Restricting a currency to decimal places alone would exclude valid
non-power-of-ten increments; do not silently make that restriction here.

For exact input `x` and positive quantum `q`, write `abs(x)/q = k + r/d`, with
`k >= 0`, `0 <= r < d`. The result is a signed integer multiple of `q`:

| Proposed `AmountRounding` | Increment nonnegative `k` when `r != 0` |
|---|---|
| `TowardZero` | Never |
| `AwayFromZero` | Always |
| `TowardPositiveInfinity` | Input is positive |
| `TowardNegativeInfinity` | Input is negative |
| `NearestTiesEven` | `r > d-r`, or equality and `k` is odd |
| `NearestTiesAwayFromZero` | `r >= d-r` |
| `Exact` | Return an inexact-operation error |

When `r == 0`, every mode returns the exact multiple. Exact/rounded zero is
canonical zero. Explicit rounding of a nonzero amount to zero is a valid
result, unlike M4/M7's nonzero-underflow error for unavailable analytical
factors/products. This is a scoped decimal rounding rule, not a change to
existing numerical domains. Every mode is supplied at each operation.

Evenness concerns the number of quanta, not a displayed decimal digit or the
quantum coefficient. With quantum `0.05`, `0.125` ties to `0.10` and `0.175`
ties to `0.20`. Negative ties obey the corresponding signed rule. Compare
`r` with `d-r`, avoiding a doubled-remainder overflow. Never decide a tie by
first converting input, quotient or quantum to binary64.

Return an ordinary canonical DecimalAmount. The caller keeps the quantum
and mode as explicit settlement metadata. Do not force the quantum count into
`i128`: quantizing `MAX` to `1e-38` exactly requires a 254-bit count although
the final canonical amount is representable. Returning an `i128` count as
the sole result would wrongly reject this valid operation.

## 5. Exact operations, cancellation and overflow

`checked_add`, `checked_sub` and `checked_mul` either return their exact
representable decimal result or a typed error. They never quantize implicitly.
For multiplication, normalize the full exact coefficient product and combined
scale (at most 76) before testing final scale/range. A mathematically nonzero
product below `1e-38` fails if no normalization makes it representable; it
does not silently become zero. A separate explicit rounded operation can
produce a quantum zero.

Mixed-scale addition/subtraction must align into a wide private integer,
combine signs, normalize, then check output range. A checked `i128`
multiply followed by checked add is insufficient. For example, let

```text
A = 17014118346046923173168730371588410573
B = -0.3
```

Then the aligned `10*A` exceeds `MAX` by three, but `A+B` is exactly
`MAX/10` and is representable at scale one. Intermediate overflow must not
reject it. Conversely, an exact final value requiring a coefficient beyond
the asymmetric range or a remaining scale above 38 is an error even when its
magnitude is smaller than another representable value. There is no
saturation, wrapping, lossy scale reduction or opportunistic float path.

An exact slice reduction should sum positive and negative magnitudes at the
common basis `10^-38`, subtract once and normalize once. This permits
overflowing prefixes to cancel, is permutation invariant and requires no
heap allocation. Each aligned term is less than `2^254`; a slice has fewer
than `2^usize::BITS` elements. Each accumulator therefore needs at most
`254 + usize::BITS` bits, rounded up to whole `u64` limbs (five limbs on a
64-bit target). Check only the final normalized result. The empty sum is
zero. Pairwise checked addition is a different prefix-sensitive operation.

Reducing rounded coupon/price amounts exactly still sums supplied rounded
values. It is not an exact sum of ideal products and does not undo earlier
rounding. Whether to round each payment, each constituent or a final net
amount is a settlement convention that must be stated by the caller's
contract. For instance, cent-rounding two decimal `0.005` values separately
with ties-even gives zero; exact-summing first and rounding gives `0.01`.

## 6. Rational-to-quantum rounding without false overflow

For an input `N/D`, with signed `i128 N` and nonzero `u128 D`, and quantum
`Q/10^s`, form exact unsigned wide integers:

```text
A = abs(N) * 10^s
B = D * Q
(k,r) = div_rem(A,B)
```

Apply the mode above, then form the signed decimal value `(k*Q)/10^s`.
Normalize before the signed range check. A sign is stored separately throughout.
`D` need not fit `i128`; accepting `u128` avoids the signed denominator
`MIN/-1` trap. `D==0` is invalid even when `N==0`. Exact zero is otherwise
valid for every mode.

`A` has at most 254 bits and `B` at most 255 bits, so four `u64` limbs suffice.
The final product also fits: rounded `k <= A/(D*Q)+1`, hence
`k*Q <= A/D+Q <= A+Q < 2^255`. A large quotient by itself is not a range
error. GCD cancellation can optimize operations but is not a substitute for
this width proof: coprime operands may defeat it while the final amount is
representable. Division must retain the exact remainder, including when the
quotient is zero or much wider than 128 bits.

An explicit amount-times-ratio operation handles an exact decimal amount
times a supplied bounded rational: compute
`(C/10^a)*(N/D)` and round once to `Q/10^s`, by dividing
`abs(C)*abs(N)*10^s` by `D*10^a*Q`. Six limbs (384 bits) safely hold the
intermediates under these input bounds. It does not cover every exact decimal
price divided by 100: at price scale 37 or 38, expressing that entire ratio
can require denominator `10^39` or `10^40`, beyond `u128`. Splitting the
calculation into `checked_mul` followed by division also rejects some
representable final rounded amounts. Rejecting a large intermediate product
before dividing is incorrect.

Therefore include a distinct fused two-decimal product/divisor quantizer in
phase A. Proposed `multiply_divide(other, divisor, quantum, mode)` rounds the
exact expression `self * other / divisor` once. Both decimal inputs retain
their exact coefficients/scales; divisor is an explicit nonzero `u128`. For
inputs `C1/10^a` and `C2/10^b`, construct

```text
A = abs(C1) * abs(C2) * 10^s
B = divisor * Q * 10^(a+b)
(k,r) = div_rem(A,B)
```

Use the same rounding rule and return signed `(k*Q)/10^s`, normalizing only
after that final product. The result sign is the XOR of input signs; the
divisor and quantum are positive. Check a zero divisor before any zero-input
shortcut. No decimal-product intermediate has to be representable as a
DecimalAmount. The caller supplies 100 explicitly for a price-per100
conversion; it is not a built-in denomination or implicit price convention.

The complete bounds are finite:

```text
abs(C1), abs(C2) <= 2^127
0 < divisor <= 2^128-1;  0 < Q <= 2^127-1
0 <= a,b,s <= 38
A <= 2^254 * 10^38                          (381 bits)
B <= (2^128-1) * (2^127-1) * 10^76          (508 bits)
k*Q <= A/(divisor*10^(a+b)) + Q <= A+Q < 2^382
```

Eight `u64` limbs (512 bits) suffice for all phase A scalar magnitudes,
including this denominator, division state, remainder comparison and the
rounded coefficient. Recommend one verified eight-limb decimal kernel;
the smaller four/six-limb bounds above explain individual operations rather
than requiring separate arithmetic implementations. Use `r` versus `B-r`
for rounding and preserve the division invariant in the shift/subtract
implementation; the 512-bit capacity is not a license to discard a carry.

A concrete discriminator is notional `MAX` at scale zero and decimal price
`(10^37+1)/10^37`. Their exact product has a 250-bit canonical coefficient
at scale 37 and cannot be a DecimalAmount. Encoding price/100 as `N/D`
needs denominator `10^39` (130 bits); its numerator is coprime to that
denominator, so cancellation cannot repair the `u128` interface. Nonetheless,
the exact product divided by 100, rounded to quantum one with nearest/ties-even,
is the representable integer
`1701411834604692317316873037158841057`. The fused operation must succeed.
This is an arithmetic counterexample, independent of a financial choice to
quote a particular product per 100.

No general-purpose public arbitrary rational or unbounded decimal exponent
is needed. Future exact sums of unrounded rational products would require
another design: multiplying unrelated denominators can grow without the
fixed bounds above. Do not advertise the proposed slice reduction as that
operation.

## 7. Binary64 conversions have distinct meanings

Recommend named methods, with no `From<f64>` or blanket numeric cast:

| Operation | Meaning |
|---|---|
| `from_binary64_exact(x)` | Decode finite supplied bits; succeed only if precisely that rational is in the DecimalAmount set |
| `from_binary64_rounded(x,q,mode)` | Decode finite supplied bits and round their exact rational once to the explicit quantum |
| `to_binary64_exact()` | Succeed only if the decimal rational is exactly representable in binary64 |
| `to_binary64_nearest()` | Correctly round the exact decimal rational once, nearest/ties-even; finite result, canonical zero |

The exact incoming conversion is intentionally not a convenience decimal
parser. Binary64 `0.5` converts exactly. Binary64 `0.1` has a reduced
power-of-two denominator requiring decimal scale 55 and cannot be represented
by this scale-38 model; it must fail instead of returning decimal `0.1`.
The exact outgoing conversion rejects decimal `0.1`, but accepts decimal
`0.5`. Amount magnitudes are between `1e-38` and about `1.7014e38`, so the
nearest outgoing conversion needs neither zero-underflow nor infinity; its
sole rounding is nevertheless part of its documented interpretation.

For incoming exact conversion, reduce a nonzero binary64 value to
`M*2^e`. If the reduced exponent is negative `-t`, its exact decimal form
has coefficient `M*5^t` and scale `t`; check scale and coefficient bounds.
For nonnegative exponent, check the exact shifted integer range. Handle
subnormals explicitly and canonicalize both signs of floating zero. Reject
both infinities and every NaN payload using the existing typed finite-input
policy.

The rounded incoming path cannot be routed through an `i128/i128` ratio:
finite subnormals can have denominator `2^1074`, while large finite values
have more than 128 integral bits. With the conventional finite decomposition
`M*2^e`, `M < 2^53` and `-1074 <= e <= 971`, construct

```text
e >= 0: A = M * 2^e * 10^s,  B = Q
e <  0: A = M * 10^s,        B = Q * 2^(-e)
```

The first numerator needs at most 1151 bits; the second denominator at most
1201 bits. A fixed 19-limb unsigned array (1216 bits) covers both, remainder
comparison and the rounded coefficient product. The same quotient/remainder
rule handles tiny values, exact ties and large unrepresentable final amounts.
Range classification occurs after rounding and decimal normalization; a
nonzero tiny input may validly round to zero or to one quantum. A full
finite-input contract is preferable to an undocumented shortcut rejecting
every subnormal before an explicit rounding request.

For outgoing rounding, derive the binary exponent by exact comparisons,
retain the target significand and decide its final guard/remainder tie in
integer arithmetic. Do not implement it as
`(coefficient as f64) / (10^scale as f64)`: that is a two-input-rounding
computation with possible extra error. Exact conversion can check whether
the exact rounded binary64 bits reconstruct the original rational, or cancel
the denominator's powers of five and check the remaining binary significand.

A shortest-decimal interpretation is a separate possible adapter. Recommend
omitting it from the first contract; decimal text should enter through
FromStr. If later added, call it `from_binary64_shortest_decimal` and specify
the formatting grammar, exponent handling and range checks. Neither generic
float Display nor its parse round-trip proves equality to the represented
real number. The bit decomposition and conversion APIs are supported by the
[Rust binary64 documentation](https://doc.rust-lang.org/std/primitive.f64.html#method.to_bits);
the width arguments here are original derivations for these proposed bounds.

## 8. Recommended staged public surface

The following signatures are a concrete proposal, not declarations that
already exist. All values are immutable with private fields. Domain errors
sit beside the operations; the crate root Error wraps AmountError and keeps
the existing typed `source()` layering. Non-exhaustive conventions/errors
follow the current API principles.

```rust
// Proposed `amounts` module, phase A: exact decimal foundation.
pub struct DecimalAmount { /* canonical i128, u8 */ }
pub struct AmountQuantum { /* strictly positive canonical amount */ }
pub enum AmountRounding {
    TowardZero, AwayFromZero,
    TowardPositiveInfinity, TowardNegativeInfinity,
    NearestTiesEven, NearestTiesAwayFromZero, Exact,
}
pub enum AmountError { /* structured variants described below */ }

impl DecimalAmount {
    pub const MAX_SCALE: u8 = 38;
    pub const MAX_TEXT_BYTES: usize = 128;
    pub fn new(coefficient: i128, scale: u8) -> Result<Self, AmountError>;
    pub fn coefficient(self) -> i128;
    pub fn scale(self) -> u8;
    pub fn checked_add(self, other: Self) -> Result<Self, AmountError>;
    pub fn checked_sub(self, other: Self) -> Result<Self, AmountError>;
    pub fn checked_neg(self) -> Result<Self, AmountError>;
    pub fn checked_mul(self, other: Self) -> Result<Self, AmountError>;
    pub fn quantize(self, quantum: AmountQuantum, mode: AmountRounding)
        -> Result<Self, AmountError>;
    pub fn is_multiple_of(self, quantum: AmountQuantum) -> bool;
    pub fn from_ratio(numerator: i128, denominator: u128,
        quantum: AmountQuantum, mode: AmountRounding)
        -> Result<Self, AmountError>;
    pub fn multiply_ratio(self, numerator: i128, denominator: u128,
        quantum: AmountQuantum, mode: AmountRounding)
        -> Result<Self, AmountError>;
    pub fn multiply_divide(self, other: Self, divisor: u128,
        quantum: AmountQuantum, mode: AmountRounding)
        -> Result<Self, AmountError>;
}
impl AmountQuantum {
    pub fn new(coefficient: i128, scale: u8) -> Result<Self, AmountError>;
    pub fn amount(self) -> DecimalAmount;
}
pub fn exact_amount_sum(values: &[DecimalAmount])
    -> Result<DecimalAmount, AmountError>;

// Phase B: explicitly interpreted analytical/decimal boundary.
impl DecimalAmount {
    pub fn from_binary64_exact(value: f64) -> Result<Self, AmountError>;
    pub fn from_binary64_rounded(value: f64,
        quantum: AmountQuantum, mode: AmountRounding)
        -> Result<Self, AmountError>;
    pub fn to_binary64_exact(self) -> Result<f64, AmountError>;
    pub fn to_binary64_nearest(self) -> f64;
}
```

Phase A also supplies the parsing, canonical Display and exact traits above,
including the fused two-decimal product/divisor path. Its full scalar kernel
has eight limbs (512 bits); its reduction accumulator retains the separate
slice-count proof in section 5.
It is independently useful for explicit decimal inputs and can be adopted,
implemented and verified before the larger finite-binary64 quantizer. Phase
B supplies all four named conversions together, so its exact/rounded meanings
are reviewable as a unit. Phase C is the financial settlement adapter and
its provenance/certification policy. Completing A or B does not close C or
the full NI-01 issue. A fixed-point foundation alone must not be reported as
cash settlement support or substituted for the remaining M8 bond analytics.

Possible operation names can be refined in an API review. Keep the explicit
quantum and mode even if `from_ratio` is renamed `from_ratio_rounded`; do not
add a shorter overload with an implicit mode. No general division operator,
decimal transcendental functions, currency registry or mutable context is
proposed.

## 9. Error and work contracts

Recommended distinct errors, with stable `amount.*` kinds, are:

| Error concept | Structured context |
|---|---|
| Input text too long | Actual byte length and configured maximum |
| Invalid decimal syntax | First offending byte index and a stable reason/component |
| Invalid input/result scale | Supplied or exact remaining scale and maximum |
| Invalid quantum | Supplied coefficient; scale failure retains its own variant |
| Zero denominator/divisor | Operation/parameter identity |
| Inexact operation | Operation identity, including Exact quantization or exact float conversion |
| Result outside coefficient range | Operation identity and sign; no lossy float rendering of the rejected exact integer |
| Nonfinite binary64 input | Existing typed ValueError as nested cause, preserving NaN/infinity classification |

Input scale errors and result precision errors should have separate stable
kinds or an explicit operation field; malformed user text must not be
confused with an arithmetic result needing more precision. Whether to expose
the exact rejected huge magnitude as a public object is not necessary for
diagnostics; a private wide integer must not accidentally become an API.

Required precedence is part of the adopted contract: length, lexical error,
input scale, then input range for text; input scale, then positive-quantum
validation for the quantum constructor; denominator/divisor nonzero before zero
shortcut; nonfinite binary64 before a zero/result shortcut; exact remainder
decision before final result normalization/range. Validated amount/quantum
objects require no repeated construction checks. Root/error catalogs and
their source tests must be updated when an implementation is adopted.

Decimal scalar operations are O(1) in the fixed stated integer/scale bounds;
parsing scans at most 128 bytes. Fixed-limb binary long division has a bounded
iteration count (at most the array's bit capacity), with no input-dependent
unbounded retry or allocation. Exact slice reduction is O(n), with constant
stack space and the count-dependent capacity proof in section 5. No extra
calendar/accrual work charge is needed for these scalar operations; future
settlement batch work must still bound its number of flows. There is no
global rounding context, thread-local mode, I/O, logging, clock or cache.

Prefer one verified eight-limb phase A sign/magnitude/division core with
documented capacities over duplicating superficially similar kernels. Its
helpers may later be shared carefully with the distinct 19-limb phase B
path, after reviewing the larger capacities. Reusing private M7
binary-product/exact-sum ideas is permissible;
their existing rounding/range contracts must remain unchanged. Their
significand products are not already a decimal or rational quantizer.

## 10. Required independent tests

Tests must cite real requirements added by an adopted contract; the names
below are proposed test families, not invented requirement IDs. Exact
integer outputs, discrete errors, hashes, formatting and all quantization
decisions need exact equality, never epsilon. Oracle generators should be
written from this contract before inspecting production algorithms. Use
Python integer/Fraction quotient-remainder as one oracle and independently
formulated Decimal operations at two justified precisions as a second check.
Python documents exact rational conversion in
[Fraction](https://docs.python.org/3/library/fractions.html) and explicit
contexts/rounding in [Decimal](https://docs.python.org/3/library/decimal.html).
For a non-power-of-ten quantum, Decimal `quantize(0.05)` alone does not impose
multiples of 0.05: explicitly round the quotient and multiply, or compare to
the independent rational integer result. Python float `round` is not the
expected-decimal oracle.

| Family | Required evidence |
|---|---|
| Construction/identity | All 39 input scales; both limits and neighbors; zero/sign normalization; every removable zero; normalized Eq/Hash; exact numeric Ord across signs/scales; Send+Sync and concurrent immutable reuse |
| Text | All canonical decimal-point positions and signs; MIN/MAX; long leading/trailing zeros; normalized input whose concatenated raw coefficient overflows; 128/129-byte boundary; 38/39 fractional places; empty/sign-only/dot-only/multiple-dot/whitespace/exponents/separators/Unicode/nonfinite cases; first-error index; parse/display idempotence |
| Exact arithmetic | Small exhaustive signed coefficient/scale grids; wide deterministic inputs at every coefficient bit and scale; carry/borrow chains; normalization before range/precision check; MIN negation; equal/opposite cancellation; multiplication with raw scale 76; genuine overflow and precision failures |
| Exact reduction | Empty/singleton/all-zero; positive/negative/permuted/cancelling prefixes including MIN; basis alignment; maximum accumulator bit/carry transitions; final exact representability versus range/scale errors; no claim to allocate a usize::MAX slice |
| Quantization | Every mode and sign; exact multiples; quotient parity; zero quotient; remainder immediately below/at/above half; arbitrary 0.05/0.025/large quanta; scale0/38; both signed limits; result normalization rescuing a huge count; final overflow caused only by rounding |
| Rational paths | Denominator0/1/MAX_u128 and values above MAX_i128; numerator MIN/MAX/0; coprime large operands; products that overflow i128 but have representable quotients; four/six-limb operand bounds within the shared eight-limb core; same mathematical ratio under representable common-factor scaling |
| Fused decimal product/division | Both operand scales0/37/38 and combined scale76; divisor0/1/100/MAX_u128; 381-bit numerator/508-bit denominator bounds; divisor-zero precedence even for zero products; MIN times MIN and every sign; below/at/above quantum ties; final coefficient normalization/range; MAX-times-scale37-price/100 counterexample; cases inaccessible through the bounded-rational adapter; swap operand invariance |
| Incoming binary64 | Both zeros, every NaN class/sign, infinities; exponent-bin edges and neighbors; every subnormal leading-bit position; min/max subnormal, min normal, MAX; exact scale37/38/39 and 127-bit coefficient boundaries; half-quantum floats and their immediate neighbors; tiny-to-zero/one-quantum mode differences; exact versus shortest-decimal discriminators |
| Outgoing binary64 | Exact dyadics, decimals with uncancelled powers of five, significand53/54-bit transitions, every decimal scale, normal midpoint ties/carry, coefficient limits; exact round-trip where possible; nearest result checked against Fraction rounded once |
| Errors and composition | Stable kinds, complete context/source chains, failure precedence including zero operands, public type census; explicit price-per100 ratio and separate per-item/net rounding controls; existing M7 amount behavior unchanged |
| Bound proofs and execution | Loop/limb capacity review, division invariant checks on valid inputs, debug/release, strict all-target Clippy/fmt/docs, actual cross-platform runtime evidence when available, independent fixture reproduction, coverage and extracted std-only package |

Useful invariants include quantization idempotence, exact-multiple identity,
monotonicity in the input for a fixed quantum/mode on representable outputs,
odd symmetry for TowardZero/AwayFromZero/nearest modes, and
`ceil(-x) == -floor(x)` when both signed results are representable. Nearest
error is at most half a quantum; directed error is less than one quantum
with the declared direction. Test these as exact rational inequalities.
Range asymmetry means a valid negative endpoint need not have a valid positive
mirror; do not turn that into a weakened arithmetic assertion.

The two Decimal precisions must be derived from operand digit/bit limits,
including the 1201-bit binary denominator, or selected high enough with a
documented proof plus stability checks. Agreement between two insufficient
precisions is not a tie certificate. Exact integer quotient/remainder is
authoritative at adversarial ties. Tests should construct those ties
algebraically rather than hoping random inputs hit them.

During design, a local Python integer/Fraction/Decimal calculation confirmed
the displayed i128 limits, the mixed-scale addition discriminator, the
254/255, fused-product381/508 and binary64 1151/1201-bit extrema, the
MAX-times-scale37-price/100 counterexample, the binary64 `2.675` versus decimal
`"2.675"` cent outcomes, and binary64 `0.1`'s required scale 55. This is
arithmetic design evidence only: there is no implementation, fixture battery,
coverage result or passing milestone claim in this document.

## 11. Alternatives and remaining decisions

| Alternative | Assessment |
|---|---|
| One globally fixed minor-unit scale | Small and fast, but requires a denomination policy, loses subunit supplied precision and cannot represent arbitrary settlement increments. Suitable only for a later explicit per-ledger type |
| Retain input scale in DecimalAmount Eq/Hash | Preserves presentation but makes 1.0 and 1.00 different amounts; confuses value and metadata. Prefer canonical values plus explicit quote/formatting context |
| Unbounded decimal/rational arithmetic | Broader exactness, substantially larger algorithm/resource/provenance burden under std-only. Not needed for the finite boundary proposed here |
| Decimal floating-point analytics | Would require another precision/exponent/rounding context and elementary-function design. Does not follow from NI-01; retain existing binary64 analytical policy |
| Integer checked multiply then divide | Rejects valid final results. Wide exact division or a complete overflow-free recurrence is required |
| GCD cancellation alone | Useful fast path but cannot prove intermediate representability for all coprime inputs |
| `float*100`, `round`, cast | Introduces extra binary rounding, invalid overflow behavior and an implicit cent convention. Reject |
| Always interpret float as its shortest text | Sometimes convenient for entered decimal intent, but changes the supplied real value and can change settlement at a tie. Only an explicitly named future adapter |
| Reuse M7 exact rounded binary product as settlement | Correctly rounds supplied binary factors; does not encode decimal inputs, quantum or market rounding stage. Keep its analytical claim |

Before freezing a contract, resolve and record:

1. Adopt or reject the canonical scale0..38 model and raw-input precision
   policy, the 128-byte parser cap, names, traits and full proposed mode set.
   These are local engineering choices, not sourced market conventions.
2. Decide whether phase A should contain general exact multiplication, fused
   two-decimal product/division and exact slice reduction immediately.
   Recommendation: include them with the complete eight-limb scalar proof,
   because their finite bounds avoid later ad hoc settlement arithmetic;
   do not add unrestricted exact rational accumulation. A bounded-rational
   multiplier alone does not cover the complete proposed decimal price range.
3. Decide whether the first adopter needs phase B immediately. Recommendation:
   implement A first with its complete contract and independently generated
   edge battery; then B before any adapter accepts analytical f64 output as
   a settlement input. No promise labels phase A alone NI-01 complete.
4. Record the ADR-0006 reconsideration scope. Preserve binary64 analytics and
   every existing M7/M4 numerical domain; qualify any new exact amount claim.
5. Define each future financial adapter's units, price-per100 divisor,
   permitted signs, settlement quantum/mode, coupon/accrual precision,
   ex-coupon entitlement and per-payment versus net rounding stage. Obtain
   primary financial evidence for its named convention.
6. Choose whether settlement means exact rounding of supplied client decimal
   inputs, exact rounding of supplied observed binary64 outputs, or a
   certificate for an ideal analytical value. For the third, specify an
   error enclosure and an explicit indeterminate outcome near rounding
   boundaries; exact representation alone does not supply that enclosure.
7. Keep local implementation/review evidence, owner API approval, financial
   convention acceptance, cross-platform runtime gates and milestone/release
   acceptance distinct. This proposal authorizes none of those gates itself.
