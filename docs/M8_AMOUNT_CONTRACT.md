# Local exact decimal amount foundation contract

Adopted for local implementation on 2026-10-16, Asia/Bangkok, from `4da3f43`.
The owner's continuing authority covers decisions, implementation and broad
meaningful validation in the local workspace. This is an explicit local
NI-01 reconsideration under [ADR-0006](adr/ADR-0006-numeric-policy.md), not
an assertion of owner API sign-off, financial/milestone acceptance or release.
The [amount design](M8_EXACT_AMOUNT_DESIGN.md) remains a historical proposal.

## Scope and preserved obligations

Implement the complete exact decimal foundation described below as phase A.
Existing binary64 analytics, M4/M7 amount/error/rounding behavior and accepted
H1 numeric policy remain unchanged. No implicit currency, minor unit,
settlement lag, accounting posting, market rounding stage, clock or global
rounding context is introduced.

The full NI-01/M8 objective remains: explicit exact/rounded binary64 boundary
(phase B), financial settlement adapters and their accuracy/provenance choices
(phase C), and zero/fixed bonds with settlement/entitlement, clean/dirty,
accrued interest, yield, duration and convexity. Phase A alone completes none
of those later obligations and is not described as cash settlement support.
The [bond proposal](M8_BOND_API_DESIGN.md), [DMO source](M8_GILT_SOURCE_REVIEW.md),
[Treasury source](M8_US_TREASURY_SOURCE_REVIEW.md) and
[historical ISMA source](M8_ICMA_SOURCE_REVIEW.md) record distinct future models.

## Values, identifiers and traits

Add public module `amounts` with four immutable private-field types:

- `DecimalAmount`: canonical signed coefficient i128 and scale u8; value
  coefficient/10^scale, input scale0..38. Every i128 coefficient is valid.
  Remove trailing coefficient factors of ten while scale>0; every zero is
  (0,0). Equal values have equal hashes regardless of original input spelling.
- `AmountQuantum`: a strictly positive canonical DecimalAmount, constructed
  from explicit coefficient/scale. No default or inferred denomination.
- `AmountRounding`: the seven modes listed below, no default.
- `AmountError`: contextual, non-exhaustive structured domain errors below.

DecimalAmount and AmountQuantum implement Copy/Clone/Debug/Eq/Hash/Ord/
PartialOrd/Send/Sync. Ord compares exact numeric value, not field order.
AmountRounding implements Copy/Clone/Debug/Eq/Hash; AmountError implements
Clone/Debug/PartialEq/Eq/StdError. All are Send+Sync+'static. No panicking
numeric operators or blanket floating/integer conversion traits are supplied.

AmountRounding has ALL in this order, canonical identifiers identical to the
variant names, Display and exact case-sensitive FromStr with no aliases:
TowardZero, AwayFromZero, TowardPositiveInfinity, TowardNegativeInfinity,
NearestTiesEven, NearestTiesAwayFromZero, Exact. Unknown/whitespace/case variants
return existing IdentifierError::Unknown, concept `amount rounding`.

## Frozen public surface

```text
DecimalAmount::MAX_SCALE: u8 = 38
DecimalAmount::MAX_TEXT_BYTES: usize = 128
DecimalAmount::new(coefficient:i128, scale:u8) -> Result<Self,AmountError>
coefficient(self) -> i128
scale(self) -> u8
checked_add(self, other:Self) -> Result<Self,AmountError>
checked_sub(self, other:Self) -> Result<Self,AmountError>
checked_neg(self) -> Result<Self,AmountError>
checked_mul(self, other:Self) -> Result<Self,AmountError>
quantize(self, quantum:AmountQuantum, mode:AmountRounding) -> Result<Self,AmountError>
is_multiple_of(self, quantum:AmountQuantum) -> bool
from_ratio(numerator:i128, denominator:u128, quantum:AmountQuantum,
           mode:AmountRounding) -> Result<Self,AmountError>
multiply_ratio(self, numerator:i128, denominator:u128, quantum:AmountQuantum,
               mode:AmountRounding) -> Result<Self,AmountError>
multiply_divide(self, other:Self, divisor:u128, quantum:AmountQuantum,
                mode:AmountRounding) -> Result<Self,AmountError>
AmountQuantum::new(coefficient:i128, scale:u8) -> Result<Self,AmountError>
AmountQuantum::amount(self) -> DecimalAmount
AmountRounding::ALL: [Self;7]
AmountRounding::canonical_identifier(self) -> &'static str
exact_amount_sum(values:&[DecimalAmount]) -> Result<DecimalAmount,AmountError>
```

DecimalAmount additionally implements FromStr and canonical Display.
No constructor, text parser, financial adapter or arithmetic supplies a
rounding mode or quantum implicitly. Phase B conversion names in the design
are deliberately absent from this phase's API.

## Exact text and error order

Grammar: `[+-]?[0-9]+(\.[0-9]+)?`, ASCII only. Leading zeros and plus accepted;
no whitespace, exponents, separators, Unicode digits or omitted integer/
fractional part. Parsing never rounds. Check byte length first, then complete
lexical validity, then raw fractional places<=38, then normalized coefficient
range. A 39-place fractional zero input is rejected even if its value is zero.
A constructor scale>38 is likewise rejected before normalization, even for0.

Syntax errors retain the first offending byte position; missing digits at EOF
use index=len. Stable reason strings are `missing integer digits`,
`missing fractional digits`, and `unexpected byte`. For example `.1` fails
at0, `1.` at2, `1..2` at2, `1.2.3` at3 and sign-only input at1.
The mapping is state-based: `missing integer digits` whenever the first
required digit after the optional sign is absent or a nondigit; `missing
fractional digits` whenever the first required digit after a decimal point
is absent or a nondigit; `unexpected byte` only after at least one digit
has completed the current integer or fractional part. Thus `abc`, `é` fail
at0 with missing integer digits; `++1` and `--1` at1 with that reason;
`1.a`, `1.+2` and `1..2` at2 with missing fractional digits; `1a` at1 and
`1.2a` at3 with unexpected byte.

Remove textual fractional trailing zeros and leading magnitude zeros before
range admission, so MAX followed by `.0` succeeds. No allocated input copy is
needed. Display emits exact canonical fixed decimal ASCII with no redundant
zeros or exponent; signed zero prints0. Formatter precision must not truncate
or round this representation. Parse(Display(x)) equals x.

## Exact arithmetic and representability

Perform aligned integer arithmetic in a bounded private wide magnitude with
separate sign. Normalize the complete exact result before checking final
precision and signed coefficient range. If both fail, result precision is
reported first. Checked multiplication may have intermediate scale76; after
normalization, a remaining scale>38 is a ResultScale error. Nonzero exact
underflow is an error; exact arithmetic never silently returns rounded zero.

Every i128::MIN input is valid. Avoid signed abs/negation/division traps;
unsigned magnitude2^127 is admitted only for a negative final coefficient.
Checked negation of canonical MIN fails OutOfRange. Bounds are on the final
canonical coefficient, not intermediates or quantum count.

Mixed-scale cancellation example:
A=17014118346046923173168730371588410573, B=−0.3. Although10*A exceeds MAX,
A+B is exactly MAX/10 and must succeed at scale1.

Exact slice sum accumulates positive/negative magnitudes at basis10^-38,
subtracts once, normalizes once and admits only the final result. Empty sum
is0; overflowing prefixes may cancel; result and error class are permutation
invariant. A slice has fewer than2^usize::BITS terms, each <2^254; constant
stack capacity must cover254+usize::BITS magnitude bits. This sums supplied
exact decimal inputs, not arbitrary unrounded rational products.

## Quantization and fused operations

For positive quantum q=Q/10^s, reduce abs(x)/q to quotient k and remainder r
with denominator d, 0<=r<d. Exact multiples keep k under every mode. Otherwise:

| Mode | Increment k |
|---|---|
| TowardZero | never |
| AwayFromZero | always |
| TowardPositiveInfinity | positive input |
| TowardNegativeInfinity | negative input |
| NearestTiesEven | r>d−r, or r=d−r and k odd |
| NearestTiesAwayFromZero | r>=d−r |
| Exact | fail Inexact before result range admission |

Apply the original sign, multiply by Q, then normalize at scale s before the
coefficient range check. Explicit rounding of a nonzero input to canonical zero
is valid. Parity is the quantum count, so with q=.05, .125 ties to .10 and
.175 to .20. No floating calculation decides a tie.

`from_ratio` rounds exact numerator/denominator once. `multiply_ratio` rounds
exact self*numerator/denominator once. `multiply_divide` rounds exact
self*other/divisor once, preserving both decimal scales. For each ratio form,
reject denominator/divisor0 before any zero operand shortcut. Ordinary quantize
has no denominator-validation error because its quantum is already validated.

The two-decimal fused operation is necessary: MAX at scale0 times
DecimalAmount(10^37+1,37), divided by100 and nearest/ties-even rounded to q=1,
must return1701411834604692317316873037158841057. The unquantized product is
not representable and price/100 cannot fit a u128 denominator; rejecting that
intermediate would be a defect.

## Bounded original integer kernel

Eight u64 limbs provide512bits for phase A scalar magnitudes. For the widest
fused operation, numerator A=abs(C1)*abs(C2)*10^s has at most381bits and
denominator B=divisor*Q*10^(a+b) has at most508bits. Rounded k*Q<=A+Q<2^382.
The smaller ratio/alignment/product paths fit the same kernel. Slice sum's
count-dependent bound must separately cover the target usize width.

Use original std-only integer arithmetic with retained carries, borrows and
exact quotient/remainder. The private division implementation must not assume
an unchecked doubled remainder fits its width; aligned shift/subtract division
or an explicitly retained carry is valid. GCD cancellation is optional and not
an intermediate-range proof. Scalar loops are bounded by fixed bit/scale limits;
DecimalAmount parsing<=128bytes, exact sum O(n) with fixed stack space.
Decimal arithmetic and decimal parsing allocate no heap. Mode identifier
parsing preserves the existing IdentifierError::Unknown contract: its full
rejected input is copied into String, with O(text length) work/allocation
and ordinary system-OOM behavior; the decimal parser's 128-byte cap does
not apply to a mode identifier. No unsafe, callbacks, I/O, clock or
global/interior mutable state are required.

## Typed errors and root integration

AmountError has these eight variants (all contextual variants non-exhaustive):

| Variant / stable kind | Context |
|---|---|
| TextTooLong / amount.text_too_long | length:usize, maximum:usize |
| InvalidSyntax / amount.invalid_syntax | index:usize, reason:&'static str |
| InvalidScale / amount.invalid_scale | scale:usize, maximum:u8 |
| InvalidQuantum / amount.invalid_quantum | coefficient:i128, scale:u8 |
| ZeroDenominator / amount.zero_denominator | operation:&'static str |
| Inexact / amount.inexact | operation:&'static str |
| ResultScale / amount.result_scale | operation:&'static str, scale:u16, maximum:u8 |
| OutOfRange / amount.out_of_range | operation:&'static str, negative:bool |

Operation values are `parse`, `add`, `subtract`, `negate`, `multiply`,
`quantize`, `ratio`, `multiply_ratio`, `multiply_divide`, `sum` as applicable.
Quantum new checks scale first, then strict positivity, preserving raw input
coefficient/scale on InvalidQuantum. AmountError::kind is stable; Display
wording is not. Phase A domain errors have no nested cause. Root Error::Amount
wraps the typed domain error, delegates kind/Display and exposes that exact
AmountError as its next source layer, with From conversion.

The public-type census becomes77; the domain-variant error catalog becomes111.
No stub future error or API is added merely for a planned phase.

## Required independent evidence

Add authoritative FR-AMT requirements when implementation begins. Oracle authors
must derive their expectations from this contract before reading production.
Exact integer/discrete/format/hash checks use TOL-E0, with no epsilon. Include
all39scales, sign asymmetry and MIN/MAX, normalization rescue, every rounding
mode/tie direction/arbitrary quantum, full text grammar/resource/error order,
all rational/fused bounds/coprime cases, permutation-invariant sum and numeric
Ord. Derive algebraic ties, not only random samples. Python Fraction/integer
quotient-remainder and separately formulated high-precision Decimal checks
must agree under justified precision; Decimal.quantize(.05) alone does not
impose multiples of .05. Retain deterministic corpus census and exact failures.

Keep all existing fixtures/generators byte-identical to4da3f43. Require focused
and full debug/release/strict lint/docs, extracted actual package checks,
coverage with honest unreachable-region limits, public/error catalogs and
platform evidence scoped to execution versus compile checks. No forced OOM,
forged invalid private state or universal branch/input coverage claim.
