# Independent exact decimal amount oracles

| Field | Value |
|---|---|
| Scope | Complete local phase A [amount contract](M8_AMOUNT_CONTRACT.md), FR-AMT-01 through FR-AMT-14 |
| Baseline | `4da3f43`; local-only M8 preparation |
| Authority | Python integers and reduced Fraction values; no Rust or QuantLib expectations |
| Additional arithmetic check | Independently formulated Decimal expressions at 384 and 512 significant digits |
| Corpus | 277,577 rows, 276,760 distinct serialized operation/input tuples, 47,836,843 bytes |
| Consumer status | `tests/amounts_oracles.rs` evaluates every row; integrated results are in [M8_AMOUNT_VALIDATION](M8_AMOUNT_VALIDATION.md) |

The [generator](../validation/generator/amount_oracles.py) was authored from
the frozen contract and the 14 FR-AMT requirement rows before reading any
amount or wide-integer Rust production. The author had not read the
then-unwritten implementation. It loads no library binary, prior expectation,
external reference runtime or non-stdlib Python package; `--check` reuses only
the recorded `generated-at`/`platform` header metadata. The existing M7
binary64 exact-product/sum implementations were inspected during the earlier
design task; they do not supply expectations or an integer division routine
to this oracle.

The new [fixture](../validation/fixtures/amounts/oracles.fixture) concerns exact
supplied decimal inputs and explicit rounding of bounded rational expressions.
It neither tests phase B float conversions nor certifies ideal mathematical
bond prices, financial settlement conventions or full NI-01/M8 acceptance.

## Reproduction and pins

From the repository root, using Python 3.9 or later:

```sh
python3 -I validation/generator/amount_oracles.py
python3 -I validation/generator/amount_oracles.py --check
```

The first command writes only the new fixture. The second recomputes every
expectation and cross-check, verifies the frozen contract pin and compares
the complete fixture byte-for-byte. `--output PATH` selects a different
destination; `--check --output PATH` only compares it. The header records
`generated-at` and `platform` as TESTING_STRATEGY §7.3 requires; `--check`
reuses the recorded values, so they cannot cause spurious differences.
The initial generation and a subsequent full `--check` both actually
terminated with exit code zero on the local Python 3.9.6 runtime. The latter
reported `check: identical`. Neither command executes a Rust test; the
consumer `tests/amounts_oracles.rs` evaluates every row through the public API.

The first generation used a nonstandard `---` separator that the shared
fixture reader rejects. The integrator changed only the serialization to the
standard header, blank line and column line. The 277,577 records are
byte-identical to that first generation: their digest equals its body digest
`aa9d4dc530b333b34b3a4754b18d17a26c298050336bb4ead405b9e378da0d6b`.

| Artifact | SHA-256 |
|---|---|
| Frozen contract | `954571eb64be97e71c68877c8e5c705f5ca9312a2109d6b54f6a1883aead7df1` |
| Generator | `6fbc2e883e9fe22735b354df6283315ed9a710c540e6679ca7d8ff17e69bd00f` |
| FR-AMT requirement rows only | `21fea919bf4b23d7793bf8434dabce245856c0f420f2719691319816d6faf226` |
| Records only (unchanged since first generation) | `aa9d4dc530b333b34b3a4754b18d17a26c298050336bb4ead405b9e378da0d6b` |
| Fixture body: column line and records | `41bbc18e6d9f77848f42db5864f714d80c1861da2b07c911b086a02df4d910d3` |
| Complete fixture, 47,836,843 bytes | `9e0709ceb9a7ef539d82d70760ad9beabe24f4ab1a3d6f2a36e466c6ef3f40cd` |

The body FNV-1a64 is `6485fa3c26e8cb66`. Header pins additionally include
generator/contract FNV-1a64. The requirement-row pin hashes only matching
FR-AMT rows, in file order with newlines between them and no final newline;
unrelated later edits to the requirements document do not change it. The
contract digest is hard-coded in the generator: changing the adopted
contract cannot quietly rewrite its oracle evidence.

## Practical fixture schema

After the metadata, one blank line and the column line, every UTF-8 record
has 14 tab-separated fields:

```text
id profile op lhs rhs numerator denominator quantum mode texthex terms expected context output
```

IDs are unique sequential `AMT-000001` values. A decimal input/output cell is
`coefficient:scale`, in base-ten ASCII. An input cell can retain a constructor's
unnormalized coefficient/scale; the Rust consumer must construct the amount
and pass the resulting value to the indicated operation. Only `new` and
`quantum` deliberately pass invalid scales. An unused field is `-`.
`terms` is a semicolon-separated list of decimal cells; `-` means an empty
sum. The longest explicit sum has 8,195 terms.

`texthex` contains UTF-8 bytes encoded as lowercase hexadecimal. `_` denotes
empty text; `-` denotes an unused text field. This preserves embedded NUL,
whitespace, Unicode, punctuation and otherwise empty fields without breaking
the tab/newline record grammar. Syntax error positions count bytes, not
Unicode scalar values. Mode cells use exactly the frozen canonical names.

| `op` | Active input fields / public operation |
|---|---|
| `new` | `lhs` supplies raw `DecimalAmount::new(coefficient,scale)` |
| `quantum` | `lhs` supplies raw `AmountQuantum::new`; inspect its `amount()` |
| `parse` | Decode `texthex`, then DecimalAmount FromStr |
| `mode` | Decode `texthex`, then AmountRounding FromStr |
| `add`, `sub`, `mul` | Construct `lhs`/`rhs`, then checked_add/sub/mul |
| `neg` | Construct `lhs`, then checked_neg |
| `sum` | Construct every `terms` cell in order, then exact_amount_sum |
| `cmp` | Construct `lhs`/`rhs`, then exact numeric Ord |
| `multiple` | Construct `lhs` and positive `quantum`, then is_multiple_of |
| `quantize` | Construct `lhs`, `quantum` and `mode`, then quantize |
| `ratio` | Signed `numerator`, unsigned `denominator`, quantum/mode, then from_ratio |
| `multiply_ratio` | `lhs`, signed `numerator`, unsigned `denominator`, quantum/mode |
| `multiply_divide` | `lhs`, `rhs`, unsigned `denominator` as the explicit **divisor**, quantum/mode |

All fields unused by the selected operation are `-`; there are no hidden
input defaults. `expected=ok` requires canonical output `coefficient:scale`;
`context=text=HEX` supplies its exact expected canonical Display text. For
mode-parser success, `expected=mode`, output is the canonical identifier and
the context records its Display text. `expected=bool` has output `0` or `1`;
`expected=cmp` has output `-1`, `0` or `1`. Their context is `-`.

For errors, `expected` is the stable kind, output is `-`, and context is a
comma-separated list of all structured `key=value` fields:

| Kind | Context keys |
|---|---|
| `amount.text_too_long` | `length`, `maximum` |
| `amount.invalid_syntax` | `index`, `reason` (reason is hex-encoded UTF-8) |
| `amount.invalid_scale` | `scale`, `maximum` |
| `amount.invalid_quantum` | Raw `coefficient`, raw `scale` |
| `amount.zero_denominator` | `operation` |
| `amount.inexact` | `operation` |
| `amount.result_scale` | `operation`, exact normalized `scale`, `maximum` |
| `amount.out_of_range` | `operation`, `negative` as `0`/`1` |
| `identifier.unknown` | `concept`, `text` (both hex-encoded UTF-8) |

These are structured fields, not expected Display messages. The operation
strings are exactly the contract's `parse`, `add`, `subtract`, `negate`,
`multiply`, `quantize`, `ratio`, `multiply_ratio`, `multiply_divide` and
`sum`. The mode-parser concept decodes to `amount rounding`.

## Independent arithmetic and precision argument

The authoritative model represents each supplied amount as reduced Fraction
`coefficient/10^scale`. Exact arithmetic and sums operate on those reduced
rationals. To recover a canonical decimal, factor its reduced denominator
into powers of two and five, take the larger exponent as the minimal decimal
scale and reconstruct its exact coefficient. This differs from the adopted
production implementation's aligned wide integer/carry/division approach.

Quantization forms the exact reduced ratio `x/q`. Python integer
quotient/remainder decides sign, integer parity and ties. The `Exact` mode
checks a nonzero remainder before final decimal range admission. Exact
arithmetic checks the minimal result scale before signed coefficient range.
Both signed endpoints, raw syntax/scale order and denominator-zero-before-zero
semantics are retained. No finite float is created in this generator.

For the separate Decimal check, form the complete signed numerator and
positive denominator directly from the original decimal coefficients/scales,
ratio inputs and quantum using Decimal products/powers. Perform **one**
Decimal division and apply Decimal's own integral rounding; multiply the
integral result by the decimal quantum. Repeated divided intermediates could
approach an exact tie from the wrong side and are deliberately avoided.
Decimal `quantize(.05)` is not used as a nickel-lattice operation.

The widest corpus expression is the two-decimal fused operation:

```text
A <= 2^254 * 10^38                         < 10^115
B <= (2^128-1)*(2^127-1)*10^76            < 10^153
```

Every exact predivision Decimal product needs at most 153 significant digits;
the absolute quotient has at most 115 integer digits. The distance of a
nonintegral rational `A/B` from an integer is at least `1/B > 10^-153`;
its distance from a distinct half-integer is at least `1/(2B) > 5e-154`.
At precision 384, its one-division absolute rounding error is at most
`0.5*10^(115-384) = 5e-270`, far smaller than either separation. At precision
512 it is smaller still. Exact integers/half-integers have at most 116
significant digits and are represented exactly at both precisions. Thus
neither check can manufacture an integer or cross the relevant rounding
boundary. All exponents stay far inside the Decimal contexts' exponent range.
The final integral-times-quantum product needs at most 154 digits and is
exact at both precisions.

Other exact operations use at most the aligned decimal operand width plus
the corpus's term-count growth; even the 8,195-term sum is below 82 decimal
digits. Accepted parse text has at most 128 bytes/digits. Their two Decimal
checks are exact, not empirical high-precision approximations. Python integer
quotient/remainder remains authoritative at algebraic ties; agreement of two
unjustified precisions would not constitute evidence.

Each successfully rounded expression additionally checks exact quantum
integrality, less-than-one-quantum error, nearest half-quantum bounds or the
appropriate directed inequality, idempotence and applicable signed symmetry.
A separate sorted small grid checks monotonicity for each mode at five
quanta; Exact compares only its defined multiples. These are generator
invariants, not claims that a Rust implementation passed them.

## Corpus census and limits

| Operation | Rows |
|---|---:|
| new / quantum | 30,586 / 1,336 |
| parse / mode | 38,693 / 47 |
| add / subtract | 18,314 / 18,314 |
| multiply / negate | 18,274 / 7,751 |
| comparison / multiple predicate | 18,274 / 2,000 |
| exact sum | 2,881 |
| quantize / ratio | 28,343 / 33,397 |
| multiply_ratio / multiply_divide | 24,353 / 35,014 |

Every one of the 39 scales is paired with all signed bit-neighborhood
coefficients in the constructor corpus (29,718 rows). All 217 invalid u8
scales are checked under zero, positive and both signed endpoint coefficients,
including quantum scale-before-sign precedence. Explicit MIN/MAX/zero
negation covers every valid scale.

The short text corpus contains every length-zero-through-five string over
the eight-symbol alphabet `0`, `1`, `+`, `-`, `.`, `a`, space and `é`:
37,449 rows. This is exhaustive only for that stated alphabet/length grid.
Additional cases cover canonical point positions, normalized range rescue,
128/129-byte and 38/39-place boundaries, sign/Unicode/NUL examples, rejected
mode spellings and invalid suffixes that must precede scale/range failures.

Exact arithmetic has the full 68-by-68 small operand grid under four
operations (18,496 rows), four selected coefficient-pair profiles at every
39-by-39 scale pair (24,336), rotating all-bit/all-scale cases (29,718) and
8,192 deterministic wide examples. It does not claim every coefficient
pair or every arbitrary expression is exhausted.

Rounding includes a 100-value/20-quantum grid under all seven modes;
two 9,996-row algebraic half-quantum families exercise bounded-rational and
fused forms. Denominator zero, one and u128 endpoints have separate small
and signed-limit controls, including coprime MAX/u128MAX. All scale pairs
exercise the simultaneous extreme fused operands under all modes (10,647).
The explicit MAX-times-scale37-price/divisor100 discriminator, huge quantum
counts whose final decimal normalizes into range, Exact-before-range cases,
and 57,344 deterministic wide rounded cases remain separate named profiles.

Observed unreduced maximum widths are 254/254 bits for quantize numerator/
denominator; 254/255 for ratio; 381/382 for multiply_ratio; **381/508** for
multiply_divide. These are observed public-input expression widths; they do
not prove which private optimized division path a Rust implementation visits.
The generator also proves the fixed bounds algebraically in the accompanying
contract/design; it does not allocate a usize::MAX slice or forge private
integer states.

The corpus contains 131,947 successful amounts, 18,274 comparisons, 2,000
predicates, seven mode successes and 125,349 errors: 15,012 Inexact,
156 InvalidQuantum, 1,742 InvalidScale, 37,237 InvalidSyntax,
58,569 OutOfRange, 6,612 ResultScale, ten TextTooLong, 5,971
ZeroDenominator, and 40 existing IdentifierError::Unknown results.
The full profile and operation/error-operation censuses are embedded in the
fixture header and recomputed by the generator. Distinct-input counting
excludes ID/profile/expectations and counts the serialized active op/input
fields, not distinct rational values; deliberate cross-profile repetitions
and alternate constructor spellings remain visible.

Actual generation performed 133,948 exact Decimal expression checks and
230,272 Decimal quotient checks (both totals include both precisions).
It checked 100,124 rounding inequality/idempotence/symmetry cases, 15,012
Exact rejections and 2,945 ordered monotonicity cases. The exact-half corpus
has 6,402 mode-specific observations (2,940 even and 3,462 odd quantum counts);
22,230 nonzero inputs validly round to zero. Canonical Display additionally
agrees with exact Decimal fixed formatting at both precisions for every
successful amount.

The generator and fixture are now frozen after that actual byte-identical
regeneration. This report's pins match those generated bytes.
Subsequent Rust consumer, independent hand-suite, full regression, strict
lint/docs, coverage, package extraction and platform results are in the
[implementation validation report](M8_AMOUNT_VALIDATION.md). Neither row counts nor an oracle that
checks itself establish those later results.
