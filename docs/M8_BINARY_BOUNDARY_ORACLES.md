# Independent binary64 boundary oracles

| Field | Value |
|---|---|
| Scope | Frozen local phase B [binary64 boundary contract](M8_BINARY_BOUNDARY_CONTRACT.md): the four named conversions, FR-BIN-02 through FR-BIN-06 and the FR-BIN-08 kinds/operation strings, over the phase A [amount contract](M8_AMOUNT_CONTRACT.md) quantization table |
| Baseline | `0ae9168`; local-only M8 phase B preparation |
| Authority | IEEE 754 bit fields decoded with Python integer shifts; Python integers and reduced Fraction values; no Rust, QuantLib or float-formatting expectations |
| Additional checks | CPython float decomposition, exact `Decimal.from_float` expansion, correctly rounded int/int true division and decimal-string parsing, reduced-Fraction quotients, one Decimal division at 768 and 1024 digits, division-free `Decimal.quantize` for power-of-ten quanta |
| Corpus | 196,773 rows, 176,699 distinct serialized operation/input tuples, 22,676,994 bytes |
| Consumer status | `tests/amounts_binary_oracles.rs` evaluates every row; integrated results are in [M8_BINARY_BOUNDARY_VALIDATION](M8_BINARY_BOUNDARY_VALIDATION.md) |

The [generator](../validation/generator/binary_boundary_oracles.py) was
authored from the frozen phase B contract, its eight FR-BIN requirement rows,
the phase A contract (quantization table and `AmountError` fields), the design
proposal §7 and TESTING_STRATEGY §7.3. The phase A oracle generator, its
fixture header and its report were read for format and style only, together
with the shared reader `tests/support/fixture.rs`. The author did not open any
file under `src/`. The phase B production code, written concurrently, was never
read. No other Rust test and no QuantLib source was read. The generator loads
no library binary, prior expectation, external reference runtime or non-stdlib
package. `--check` reuses only the recorded `generated-at`/`platform` header
metadata.

The contract changed once while this oracle was being written. The first
revision gave `NonFinite` a nested `ValueError::NotFinite` cause. The pinned
revision records `value:NonFinite` and has no nested source. The fixture
encodes only the operation and the classification, so its rows are the same
under both readings. A consumer must check the absence of a source as the
pinned text requires.

The new [fixture](../validation/fixtures/amounts/binary-boundary-oracles.fixture)
covers only the four explicit conversions. It does not test shortest-decimal
reading (which the contract excludes), phase C settlement adapters, currency or
rounding-stage policy, bonds, or FR-BIN-01/07 API, kernel and allocation
properties.

## Reproduction and pins

From the repository root, using Python 3.9 or later:

```sh
python3 -I validation/generator/binary_boundary_oracles.py
python3 -I validation/generator/binary_boundary_oracles.py --check
```

The first command writes only the new fixture. The second recomputes every
expectation and cross-check, verifies both contract pins and compares the
complete fixture byte-for-byte. `--output PATH` selects a different
destination; `--check --output PATH` only compares it. `--check` reuses the
recorded `generated-at` and `platform`, so they cannot cause spurious
differences. On the local Python 3.9.6 runtime (Darwin arm64), the generation
and a subsequent full `--check` each finished in about 7 seconds with exit
code zero. The check reported `check: identical`. Neither command runs a Rust
test.

| Artifact | SHA-256 |
|---|---|
| Frozen phase B contract (hard-coded in the generator) | `7f3c24a089e17115b9d85527797c79617a4e7e599f8bb4474aed5a7be2ccec31` |
| Phase A amount contract (hard-coded in the generator) | `954571eb64be97e71c68877c8e5c705f5ca9312a2109d6b54f6a1883aead7df1` |
| Generator | `346188ff94900c3a793c43248a272bb0e312fb3151518e7135ce743a6ef33e4e` |
| FR-BIN requirement rows only | `f9cf0020de3effe0ec615edc077c659e79bfb76f9036302cfce81efd9c043b7b` |
| Fixture body: column line and records | `e7c816edc5d3c51850e3f89827d27d1cac633db73020940728ad859d4c82bbf9` |
| Complete fixture, 22,676,994 bytes | `b0b7bda628d5eaa8b0118e5752da0b52445784e0a4ec94d3259ddd7f66bec667` |

The body FNV-1a64 is `55dfd0bc0911406b`. The generator FNV-1a64 is
`eb62192ad2b8d902`, and the phase B contract FNV-1a64 is `1addd38b2168d5c1`.
The requirement-row pin hashes only the eight `| FR-BIN-nn |` rows, in file
order, joined by newlines, with no final newline. Changing either contract
makes the generator refuse to run until it is deliberately re-pinned. A fresh
generation without `--check` records a new `generated-at`, which changes the
complete-file digest but not the body digest.

## Practical fixture schema

After the metadata, one blank line and the column line, every UTF-8 record
has ten tab-separated fields:

```text
id profile op bits amount quantum mode expected context output
```

IDs are unique sequential `BIN-000001` values. An unused field is `-`, so no
field is empty.

| `op` | Active fields / public operation |
|---|---|
| `from_exact` | `bits` → `DecimalAmount::from_binary64_exact(f64::from_bits(bits))` |
| `from_rounded` | `bits`, `quantum`, `mode` → `DecimalAmount::from_binary64_rounded(f64::from_bits(bits), AmountQuantum::new(c,s)?, mode)` |
| `to_exact` | `amount` → `DecimalAmount::new(c,s)?.to_binary64_exact()` |
| `to_nearest` | `amount` → `DecimalAmount::new(c,s)?.to_binary64_nearest()` |

- **Bits.** A `bits` input and a binary64 output are `0x` followed by 16
  lowercase hexadecimal digits of the IEEE 754 pattern (TESTING_STRATEGY §7.3
  rule 7, readable with `Record::real_bits`). The pattern keeps `-0.0`, the
  sign of NaN and every NaN payload. A consumer must build the input with
  `f64::from_bits` and compare outputs with `to_bits`, never through decimal
  text.
- **Decimals.** `amount` and `quantum` are canonical `coefficient:scale` in
  base-ten ASCII. Every quantum is positive with scale at most 38, and every
  amount is a valid canonical amount.
- **Modes.** `mode` is the exact canonical `AmountRounding` identifier.

`expected=ok` has `context=-`. Its `output` is the canonical result
`coefficient:scale` for incoming operations, or the result bits for outgoing
ones. For an error, `expected` is the stable kind, `output` is `-`, and
`context` lists every structured field as comma-separated `key=value`:

| Kind | Context keys |
|---|---|
| `amount.non_finite` | `operation`; `class` = the variant's `value:NonFinite` (`NaN`, `PositiveInfinity` or `NegativeInfinity`) |
| `amount.result_scale` | `operation`; exact needed `scale` (39 to 1074); `maximum=38` |
| `amount.out_of_range` | `operation`; `negative` as `0`/`1`, the input sign |
| `amount.inexact` | `operation` |

The operation strings are exactly `from_binary64_exact`,
`from_binary64_rounded` and `to_binary64_exact`. `to_binary64_nearest` has no
error. Every NaN pattern has class `NaN`, whatever its sign bit or payload.
These are structured fields, not expected Display text.

## Independent method

**Decoding.** A pattern's sign, 11-bit exponent field and 52-bit fraction are
read by shifting a Python integer. Field `0x7FF` is an infinity (zero
fraction) or a NaN. Field 0 gives `M = fraction` and `e = −1074`. Any other
field gives `M = fraction + 2^52` and `e = field − 1075`. The generator asserts
`M < 2^53` and `−1074 ≤ e ≤ 971`. No float value, float formatting or float
operation feeds an expectation.

**Exact incoming.** A nonfinite pattern fails first; either zero gives `0:0`.
Otherwise the generator removes trailing zero bits so that `M` is odd. For
`e ≥ 0` the coefficient is `M·2^e` at scale 0. For `e < 0` it is `M·5^t` at
scale `t = −e`. A scale above 38 fails `ResultScale` with that `t`, before
range is considered. Otherwise a coefficient outside `[−2^127, 2^127−1]` fails
`OutOfRange` with the input sign.

**Rounded incoming.** The generator forms the contract's unreduced
`A = M·2^e·10^s, B = Q` (for `e ≥ 0`) or `A = M·10^s, B = Q·2^−e`. Python
`divmod(A, B)` gives `k` and `r`. The phase A table is applied with integer
tests `2r` versus `B`, the parity of `k` and the input sign. `Exact` with
`r ≠ 0` fails `Inexact` before any range test. Otherwise the generator applies
the sign, multiplies by `Q`, normalizes at scale `s` and then checks the
signed i128 range. Every row asserts the contract widths: `A < 2^1151`,
`B < 2^127` or `A < 2^180`, `B < 2^1201`, and `k·Q ≤ A + Q < 2^1152`.

**Outgoing.** `to_binary64_nearest` uses only integers. The binary exponent
comes from bit lengths corrected by one exact comparison. The 53-bit
significand is an integer quotient. The remainder is compared with half the
divisor, ties go to the even significand, and a carry to `2^53` moves into the
next binade. The generator asserts that every nonzero result is normal and
finite. `to_binary64_exact` returns those bits only when their exact rational
equals the amount. Otherwise it fails `Inexact`. Canonical zero gives `+0.0`.

**Round trips.** Each `from_exact` success in a primary profile is followed
by a `ROUNDTRIP-INCOMING` row. That row expects `to_exact` to return the
original bits, with `−0.0` returning `+0.0`. Each primary `to_exact` success is
followed by a `ROUNDTRIP-OUTGOING` row, which expects `from_exact` to return
the original amount. Each pair is asserted when it is generated.

### Cross-checks performed

| Check | Count | What is compared |
|---|---:|---|
| Finite decode | 121,727 | Field decoder versus `float.as_integer_ratio`, a parsed `float.hex`, the sign bit and an unchanged struct round trip |
| Nonfinite class | 3,486 | `math.isnan`/`math.isinf`/sign versus the field classification |
| Exact expansion | 23,212 | `from_exact` coefficient/scale versus reduced-denominator factoring and the exact `Decimal.from_float` digits |
| Fraction quotient | 97,050 | Contract A/B result versus a restated phase A table over the reduced Fraction `x/q` |
| Rounding invariants | 87,137 | Less than one quantum of error, half-quantum bound or directed inequality, idempotence, signed mirror symmetry |
| Decimal quotient | 194,100 | One division of the exact `Decimal.from_float(x)` by the exact decimal quantum, then Decimal integral rounding, at 768 and 1024 digits |
| Decimal quantize | 59,173 | Division-free `Decimal.quantize` of the exact expansion for the 77 power-of-ten quanta `10^−38…10^38` |
| True division | 72,134 | Integer rounding versus CPython's correctly rounded int/int `/`: every nonzero outgoing row, 562 bracket selections and 18 discriminator texts |
| Decimal-string parse | 71,554 | Outgoing bits versus `float("<c>e-<s>")` (CPython's correctly rounded string conversion) |
| Nearest definition | 71,554 | Neither adjacent binary64 is closer, and an equidistant result has an even significand |
| Representability | 36,539 | `to_exact` success versus a power-of-two reduced denominator with an odd numerator part of at most 53 bits |
| Brackets | 562 | Selected inputs: down/up neighbours are adjacent and strictly bracket the target, or both equal it |
| Round trips | 6,833 | Both exact directions, as described above |
| Monotonicity | 97,808 | Ordered outgoing values, and ordered rounded inputs per quantum/mode, give nondecreasing results |

**Decimal precision argument.** The finite ratio satisfies
`|x|/q = A/B < 2^1152 < 10^347`. A non-integral `A/B` lies at least `1/B` from
every integer, and a non-tie lies at least `1/(2B)` from every half-integer.
Since `B < 2^1201`, both distances exceed `2^−1202 > 10^−362`. The operands
are exact: `Decimal.from_float(x)` has at most 767 significant digits, and the
quantum `Q·10^−s` has at most 39. One division at precision `p` has an
absolute error of at most `½·10^(347−p)`. That is at most `5·10^−422` at 768
digits and `5·10^−678` at 1024 digits, against the 710 digits the separation
requires. Integers and half-integers below `10^347` have at most 348 digits,
so they are exact at both precisions. Decimal integral rounding therefore
cannot cross an integer or a tie. `quantize` needs at most 347 result digits
and is exact at 1024 digits. Python integer quotient/remainder remains the
authority; agreement between precisions is not itself the evidence.

## Corpus census and limits

| Profile | Rows | Content |
|---|---:|---|
| `IN-ZERO` | 1,458 | `+0.0` and `−0.0`: exact, and rounded under all 104 quanta × 7 modes |
| `IN-NONFINITE` | 3,480 | Both infinities and 59 NaN patterns per sign: the quiet bit alone, 51 single-bit signalling payloads and 7 multi-bit quiet/signalling payloads. Exact, plus rounded at 4 quanta × 7 modes |
| `IN-BIN-EDGE` | 49,134 | All 2,047 finite exponent fields × both signs × fractions `0, 1, 2^52−2, 2^52−1`, which includes both bit-increment neighbours of every bin edge, plus ±inf above `f64::MAX`. One exact row and two rounded rows per pattern: a 17-quantum rotation and a magnitude-fitted `10^−s`. Across its four patterns, every field/sign group meets all seven modes |
| `IN-SUBNORMAL-LEAD` | 8,932 | 203 subnormal fractions per sign covering every leading-bit position 0–51 with in-position neighbours. Exact, plus 3 quanta × 7 modes |
| `IN-EXTREMES` | 5,832 | Minimum/maximum subnormal, minimum normal and `f64::MAX`, both signs. Exact, plus all 104 quanta × 7 modes |
| `IN-EXACT-SCALE` | 5,904 | Odd `M·2^−t` for t = 0–45 and 18 larger t up to 1074. M ∈ {1, 3, 2^52+1, 2^53−3, 2^53−1}, the largest odd M whose `M·5^t` fits, and its odd neighbours. Exact, plus `10^−38` × 7 modes |
| `IN-I128-BOUNDARY` | 8,236 | Integers `2^127 ± 2^z` (z = 74–126), `2^127`, ±3-bit neighbours of `2^126`, `2^127` and `2^128`, and largest-fitting odd significands ±2 at scales 32–38. Exact, plus 4 quanta × 7 modes |
| `IN-HALF-QUANTUM` | 27,412 | 52 quanta, each with up to 12 counts k: 0–3, the dyadic exact ties `(f−1)/2` and `(3f−1)/2` (even and odd k), `2^52−1`, `2^52`, `2^53+1`, one SplitMix64 count and the two largest in-range counts. Rows cover the binary64 values at or bracketing `(k+½)q` with their outer neighbours, both signs × 7 modes |
| `IN-TINY` | 2,842 | 19 quanta: subnormal extremes, minimum normal, and the neighbourhoods of `q/2` and `q`, both signs × 7 modes |
| `IN-HUGE` | 4,068 | `f64::MAX` and its predecessor; `2^p` for p ∈ {1023, 1000, 971, 970, 600, 256, 200, 192, 129, 128}; the values bracketing `10^38`, `10^308` and `1.5·(2^127−1)`. Exact, plus 16 quanta × 7 modes |
| `IN-DISCRIMINATOR` | 2,600 | Nearest binary64 of 18 short decimals, with ±2 bit neighbours for `0.1` and `2.675`. Exact, plus 7 quanta × 7 modes |
| `OUT-DISCRIMINATOR` | 66 | Zero and ±16 short decimals (including `0.1` and `2.675`) under both outgoing operations |
| `OUT-SCALE-COEFFICIENT` | 30,710 | Every scale 0–38 with MIN, MIN+1, MIN+2, MAX−2, MAX−1, MAX, ±1, ±2, ±3, ±5^s, ±3·5^s, ±(10^s±1), and `2^k−1`, `2^k`, `2^k+1` for k = 0–127 (alternating sign) |
| `OUT-SIGNIFICAND` | 11,046 | 26 integers around `2^52`…`2^55` times `2^j` (j = −38–73), wherever the decimal fits, both signs |
| `OUT-MIDPOINT` | 12,220 | Every binade from `2^−126` to `2^126` with five significands. Each row is an exact tie with its coefficient neighbours when the tie is decimal, otherwise the two decimals bracketing it at the finest fitting scale; both signs |
| `OUT-RANDOM` | 12,000 | 6,000 SplitMix64 amounts with uniform coefficient bit length, sign and scale |
| `OUT-RANDOM-DYADIC` | 4,000 | 2,000 SplitMix64 amounts that are exactly representable in binary64 |
| `ROUNDTRIP-INCOMING` | 1,518 | `to_exact` of each primary `from_exact` success |
| `ROUNDTRIP-OUTGOING` | 5,315 | `from_exact` of each primary `to_exact` success |

| Operation | Rows | Outcomes |
|---|---:|---|
| `from_exact` | 23,339 | 6,833 ok; 8,948 ResultScale (every needed scale 39–1074 occurs); 7,436 OutOfRange; 122 NonFinite |
| `from_rounded` | 101,874 | 64,935 ok; 23,662 OutOfRange; 9,913 Inexact; 3,364 NonFinite |
| `to_exact` | 36,539 | 6,833 ok; 29,706 Inexact |
| `to_nearest` | 35,021 | 35,021 ok |

The corpus uses 22,670 distinct input bit patterns, 104 distinct quanta and
31,132 distinct outgoing amounts at all 39 scales. Its 97,050 finite nonzero
rounded rows observe 27,655 exact multiples, 48,967 remainders below half,
18,184 above half and 2,244 exact half-quantum ties. The ties split into 1,060
with even and 1,184 with odd quotient counts. Nonzero inputs round to zero in
19,118 rows, and tiny inputs round to one quantum in 10,984. In 3,950 rows,
`k·Q` exceeds i128 but the normalized result fits. 23,662 rows fail
`OutOfRange`, each classified after rounding and normalization. Of 9,913
`Exact` rejections, 1,198 would also have exceeded range after truncation, so
they show `Inexact` taking precedence over range. Among nonzero `to_nearest`
rows, 5,314 are exact, 12,646 have a remainder below half a unit in the last
place and 14,117 above. 2,943 are exact ties: 1,363 round down and 1,580 round
up to the even significand. 1,368 roundings carry into the next binade, 631 of
them ties.

The observed maximum widths are 1,151/127 bits for `e ≥ 0` A/B, 180/1,201 bits
for `e < 0` A/B, 1,151 bits for `k·Q` and 142 bits for an exact fractional
coefficient. Outgoing values reach a 179-bit numerator shift, a 180-bit
shifted numerator and a 127-bit divisor. The generator asserts that the
corpus reaches the contract's 1,151- and 1,201-bit maxima. These figures are
public-input expression widths. They do not show which private path an
implementation takes.

The corpus is not exhaustive over binary64 patterns, quanta or amounts. Exponent
bins are sampled at four fractions with two rotating rounded rows. Quanta
are 104 named values, and the random profiles are deterministic samples. Row
counts and generator self-checks do not establish any Rust result.

## Contract readings

- **Non-finite key.** The contract field is `value:NonFinite`. The fixture
  key is `class`, so it cannot be confused with the input value. Its values
  are the contract's `NaN`, `PositiveInfinity` and `NegativeInfinity`. A NaN
  has no signed class.
- **OutOfRange sign.** `negative` is the input sign bit. An out-of-range
  result is never zero, so it has the input's sign.
- **ResultScale.** Only `from_exact` reports `ResultScale`. Its scale is the
  odd-significand `t`. `from_rounded` cannot exceed its quantum scale and so
  never reports it, matching the contract's operation table.
- **Zeros.** Both zeros give `0:0` under every quantum and mode, including
  `Exact`. Zero is an exact multiple, and only nonfinite inputs precede the
  zero result. `to_exact(0:0)` succeeds with `+0.0`.
- **Signed i128 range.** `-2^127` (`0xc7e0000000000000`) succeeds exactly and
  `+2^127` fails. In the rounded path the range applies to the normalized
  coefficient. For example, `-2^127` at quantum `10^−38` returns
  `-170141183460469231731687303715884105728:0`.
- **Tie parity.** Parity is the truncated quotient count `k`, as in phase A.
- **Neighbours.** "Neighbours" means bit-increment magnitude neighbours of the
  same sign. The neighbour across a bin boundary is the adjacent bin's edge,
  the increment above `f64::MAX` is infinity, and zero has no smaller
  magnitude neighbour.
- **Bit encoding.** The brief asked for 16 lowercase hexadecimal digits. The
  `0x` prefix is added for TESTING_STRATEGY §7.3 rule 7 and the shared reader.
- **Decimal 2.675.** The decimal `"2.675"` rounded half-up to cents (2.68) is
  a phase A quantize result. This fixture has only the four conversion
  operations, so it records the decimal side as `to_nearest(2675:3) =
  0x4005666666666666` and `to_exact(2675:3)` failing `Inexact`. It records the
  binary64 side as `from_rounded(0x4005666666666666, 1:2,
  NearestTiesAwayFromZero) = 267:2`, with the next binary64 above rounding to
  `268:2`.
