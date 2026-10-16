# Local binary64 boundary contract (M8 phase B)

Adopted for local implementation on 2026-10-16, Asia/Bangkok, from `0ae9168`,
under the owner's continuing local authority. It freezes the four named
conversions of [the amount design, §7](M8_EXACT_AMOUNT_DESIGN.md) on top of
the phase A [amount contract](M8_AMOUNT_CONTRACT.md), which stays unchanged.
It is not owner API sign-off, financial or release acceptance. Phase C
(financial settlement adapters, currency and rounding-stage policy) and the
bond models remain open; this phase alone is not cash-settlement support.

## Public surface

```text
DecimalAmount::from_binary64_exact(value:f64) -> Result<DecimalAmount,AmountError>
DecimalAmount::from_binary64_rounded(value:f64, quantum:AmountQuantum,
                                     mode:AmountRounding) -> Result<DecimalAmount,AmountError>
DecimalAmount::to_binary64_exact(self) -> Result<f64,AmountError>
DecimalAmount::to_binary64_nearest(self) -> f64
```

No `From`/`TryFrom` impl, numeric cast, default quantum or mode, global
context, shortest-decimal reading or float text formatting is added. Decimal
text still enters only through `FromStr`. The public type census stays 77.

## Exact meaning of a binary64 input

Every finite binary64 value `x` is exactly `(-1)^σ · M · 2^e` with integer
`M < 2^53` and `-1074 <= e <= 971`, read from its IEEE 754 bit fields
(subnormals have `e = -1074`). The decimal written by float formatting is
irrelevant: binary64 `0.1` denotes `3602879701896397/2^55`, not `1/10`.

Both zeros are canonical zero. NaN of any sign or payload and either infinity
fail first, before any zero, quantum, rounding or range shortcut, with
`AmountError::NonFinite`. It records the existing `crate::error::NonFinite`
classification (NaN, PositiveInfinity or NegativeInfinity).

**Exact incoming.** Remove trailing zero bits so that `M` is odd. For
`e >= 0` the value is the integer `M·2^e` at scale 0. For `e < 0`, with
`t = −e`, it is `M·5^t / 10^t`. That coefficient is odd, so scale `t` is
already canonical. A scale above 38 fails `ResultScale` with the exact
needed scale (at most 1074) before any range check. Then a coefficient
outside the signed i128 range fails `OutOfRange` with its sign. No rounding
ever occurs.

**Rounded incoming.** For a positive quantum `q = Q/10^s`, form the exact
ratio `|x|/q`:

```text
e >= 0: A = M·2^e·10^s,  B = Q
e <  0: A = M·10^s,      B = Q·2^(−e)
```

Reduce `A/B` to quotient `k` and remainder `r` and apply the phase A
quantization table unchanged: all seven modes, ties by the parity of `k`, and
`Exact` failing `Inexact` before range admission. Apply the original sign,
multiply by `Q`, normalize at scale `s`, then check the signed i128 range.
A nonzero tiny input may validly round to zero or to one quantum. Huge
inputs are classified after rounding, never truncated or saturated.

## Exact meaning of a binary64 output

Canonical zero converts to `+0.0`. A nonzero amount `c/10^s` lies between
`10^-38` and `2^127` in magnitude, so its correctly rounded binary64 value is
always a normal, finite number with the amount's sign.

`to_binary64_nearest` returns the binary64 nearest to the exact rational,
with ties to the even significand. Exponent selection, the 53-bit
significand and the tie decision use exact integer comparisons only. It does
not divide two rounded floats.

`to_binary64_exact` succeeds only when the exact rational equals a binary64
value, and then returns the same bits as `to_binary64_nearest`. Otherwise it
fails `Inexact`.

Round trips are exact:

- When `from_binary64_exact(x)` succeeds, `to_binary64_exact` returns `x`
  (except that `-0.0` returns `+0.0`).
- When `to_binary64_exact(a)` succeeds, `from_binary64_exact` returns `a`.

## Integer widths

The private phase A unsigned kernel becomes generic over its fixed limb count.
Its algorithms do not change: retained carries and borrows, checked shifts,
and aligned shift/subtract division. Phase A keeps eight limbs.

The rounded incoming path needs nineteen 64-bit limbs (1216 bits). For
`e >= 0`, `A < 2^53·2^971·2^127 = 2^1151` and `B < 2^127`. For `e < 0`,
`A < 2^180` and `B < 2^127·2^1074 = 2^1201`. The rounded count satisfies
`k·Q <= A + Q < 2^1152`.

Exact incoming needs at most `M·5^38 < 2^142` and `M·2^e` compared with
`2^127`. Outgoing needs the coefficient shifted by at most 180 bits, or a
denominator `10^38·2^74 < 2^201`, so eight limbs suffice. Every loop is
bounded by these fixed widths. No heap allocation, unsafe code, I/O or
global state is used.

## Errors and identity

Add one non-exhaustive variant:

| Variant / stable kind | Context |
|---|---|
| `NonFinite` / `amount.non_finite` | `operation:&'static str`, `value:NonFinite` |

The four new operations reuse the existing variants. Each records its own
operation string:

| Operation string | Possible errors |
|---|---|
| `from_binary64_exact` | `NonFinite`, `ResultScale`, `OutOfRange` |
| `from_binary64_rounded` | `NonFinite`, `Inexact`, `OutOfRange` |
| `to_binary64_exact` | `Inexact` |

Like every phase A amount error, the variant has no nested source. A
`ValueError` cause was considered and rejected: `ValueError` carries `f64`
fields and is not `Eq`, while the phase A contract requires
`AmountError: Eq`. The root `Error::Amount` wrapper, its kind delegation and
its single source layer are unchanged. The domain error catalog becomes 112. Requirements FR-BIN-01..08 record this
contract.

## Required independent evidence

Expectations come from Python bit decomposition (`struct`/`float.hex`) and
`Fraction`, never from Rust output or float formatting.

- **Incoming:** both zeros; NaN payload and sign classes; both infinities;
  every exponent-bin edge and its neighbors; every subnormal leading-bit
  position; minimum and maximum subnormal and normal values; `f64::MAX`.
- **Incoming bounds:** exact scale 37/38/39 and 127-bit coefficient
  boundaries; half-quantum inputs and their immediate neighbors; tiny inputs
  rounding to zero versus one quantum under each mode.
- **Discriminators:** binary64 `0.1` (scale 55); binary64 `2.675` versus
  decimal `"2.675"` at cent quanta.
- **Outgoing:** exact dyadics; decimals with uncancelled powers of five;
  53/54-bit significand transitions; every decimal scale; midpoint ties with
  carry into the next binade; coefficient limits.
- **Round trips** in both exact directions.

The phase A oracle and hand suites must pass unchanged after the kernel
generalization. The usual full native, strict lint/docs, coverage, package
and catalog checks apply. All earlier fixtures and generators stay
byte-identical to `0ae9168`.
