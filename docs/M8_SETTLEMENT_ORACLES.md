# Independent settlement oracle

This is local numerical evidence for the frozen [settlement contract](M8_SETTLEMENT_CONTRACT.md)
and FR-SETTLE-01..16, not owner API acceptance, named-market compliance, an
analytical price enclosure or platform execution evidence. The generator was
written without reading settlement production source or implementation outputs.
Python standard-library integers and Fraction are authoritative; Decimal and
small independently enumerated source-cell sets provide separate cross-checks.
No QuantLib source or outputs supply these expectations.

The artifacts are [the generator](../validation/generator/settlement_oracles.py)
and [the fixture](../validation/fixtures/settlement/oracles.fixture). Regenerate
with `python3 -I validation/generator/settlement_oracles.py`; verify every byte
with `python3 -I validation/generator/settlement_oracles.py --check`. Check mode
reuses only the recorded generation time and platform. All mathematical rows,
source pins, censuses, body and record hashes are recomputed. No external input,
Rust executable, reference library, network or random seed source is used.

## Authority and fixture structure

The contract SHA256 is
`47f1b08ff1ca646c502c90b1e7cb380fa4d74a4763b3aa9e32d729adc89faeba`.
The 16 authoritative requirement rows, selected in register order by the exact
prefix `| FR-SETTLE-`, joined with LF and no final LF, have SHA256
`f500bb6926d8f38f789817e342397719543d7ffd8f73dc1f114b640da770f637`.
The generator refuses changed contract/requirement bytes. The header records
SHA256 and FNV-1a64 of its own source, contract, requirement rows, record bytes
and complete body. FNV uses standard offset 14695981039346656037 and
prime 1099511628211 with wrapping u64 arithmetic.

The shared-reader layout is schema 1 metadata, one blank line, one tab-separated
column line and LF-terminated records. Body hashes include the column line;
record hashes exclude it. Sequential IDs are `SET-000001` onward. The 17 columns
are:

```text
id profile op recipe nominal primary secondary aggregation price_mode accrued_mode quantum cash_mode expected context lower upper guarantee
```

The displayed column names are separated by spaces here; actual cells use
tabs. Unused cells are exactly `-`. Decimal cells encode coefficient:scale and
may use a noncanonical but valid input; expected amounts are always canonical.
The four evidence encodings are `d:C:S`, `b:hhhhhhhhhhhhhhhh`, `di:C:S:C:S` and
`bi:hhhhhhhhhhhhhhhh:hhhhhhhhhhhhhhhh`. Binary cells preserve original IEEE754
u64 bits, including zero sign and invalid constructor NaN payloads. No `0x`
prefix is used. All encoded hex is lowercase. Identifier input is `x:UTF8hex`,
where `_` denotes empty UTF8 text.

| op | Used input cells | Output meaning |
|---|---|---|
| evaluate | recipe, nominal, primary, quantum, cash_mode; secondary/aggregation for the two component recipes; price_mode/accrued_mode only for StagedPrice100Accrued1000 | Determined amount, ambiguous endpoints, unresolved stage or complete error |
| evidence | primary only | Canonical retained bounds and original kind/declared status, or constructor error |
| aggregation | primary=x:hex only | Parsed canonical CashAggregation identifier, or IdentifierError |
| guarantee | primary=x:hex only | Parsed canonical SettlementGuarantee identifier, or IdentifierError |

For single-component recipes secondary and aggregation are absent, not
invented defaults. Source modes are absent outside the staged recipe. All
evaluation evidence arguments are valid already-constructed values. Evidence
constructor errors are their own rows; the fixture does not impose an order on
a caller's separate Rust argument expressions.

| expected | context | lower / upper / guarantee |
|---|---|---|
| determined | text=canonical amount Display as UTF8hex | canonical amount / - / one canonical guarantee |
| ambiguous | - | strictly ordered canonical amounts / - guarantee |
| exact_unresolved | stage=exact stable stage string | all - |
| evidence | kind=decimal or binary64,declared=0 or 1 | canonical lower/upper decimal cells or canonical-zero finite bit cells; guarantee - |
| identifier | - | canonical enum identifier / - / - |
| identifier.unknown | concept=UTF8hex,text=UTF8hex | all - |
| settlement.nonpositive_nominal | nominal=canonical C:S | all - |
| settlement.decimal_interval_order | lower=C:S,upper=C:S | all - |
| settlement.binary_interval_order | lower_bits=16hex,upper_bits=16hex | all - |
| settlement.non_finite | endpoint=value/lower/upper,value=NaN/PositiveInfinity/NegativeInfinity | all - |
| settlement.arithmetic | stage=stable string,endpoint=point/lower/upper,cause=amount.inexact or amount.out_of_range,operation=quantize or sum; OutOfRange also negative=0 or 1 | all - |

Context is comma-separated key=value, with no duplicate keys. Stage names
contain ordinary spaces. All structured error fields are retained; Display
wording is deliberately not an expectation. `operation=sum` occurs only at
final cash net admission. All other required quantum stages use quantize.
InternalCapacity is excluded by the capacity proof; no invalid private state
is forged to manufacture an oracle row for it.

Receipt inspection can reconstruct every expected field from the evaluation
inputs: family identifier; exact positive par nominal; original evidence;
optional secondary/aggregation; explicit final policy; and staged source
quanta1e-6/1e-5 with their specified modes. Determined guarantees follow declared
evidence first, then supplied binary bits, then supplied decimals. Equal
declared bounds and source-collapsed intervals keep declared provenance.

## Exact arithmetic model

For decimal evidence the authority is Fraction(C,10^S). For supplied binary
evidence it is Fraction.from_float on the exact u64-decoded finite Python float,
whose integer ratio is exact. There is no shortest-decimal interpretation.
Both zero signs canonicalize to zero after finite validation. Binary interval
constructors validate lower then upper before ordering. Declared intervals
denote all real points between their exact endpoints, even adjacent floats.

An internal mathematical image retains its exact extrema, a continuous flag
and every variable source-count increment. A non-Exact source round creates
consecutive integer quantum counts. Positive scaling and addition carry those
increments independently. Exact is defined for a point only at a quantum
multiple. A continuous nonpoint input is unresolved; a discrete nonpoint input
requires its lower base and each variable increment to be multiples. Merely
checking two endpoints would incorrectly accept the reachable set 0,.5,1 at q1.

For each quantization, integer divmod of the magnitude supplies the quotient,
remainder, sign and exact half comparison. All seven modes apply directly to
the signed expression. Independent invariants check signed direction, distance
below one quantum for directed rules, distance at most half a quantum for
nearest rules, exact lattice membership and idempotence. Endpoint monotonicity
is checked on every visited nonpoint non-Exact image.

Stage order follows source price, source accrued, then capital cash before
accrued cash under component aggregation, or combined cash total. Component
rounds are combined exactly without another quantum operation. No intermediate
value receives public range admission. Final canonical lower is admitted
before upper; the reported point/lower/upper context uses the numeric input
image of that stage. Exact failure does not wait for later cancellation.
Different representable final extremes are ambiguous without claiming that
every interior amount fits the i128 decimal format.

## Independent Decimal path and precision proof

Each visited endpoint has a separate Decimal rational expression at 1536 and
2048 significant digits. Decimal input decoding does not use the authoritative
Fraction numerator or denominator. It explicitly decodes IEEE sign/exponent/
mantissa onto D0=2^1074*5^38; a decimal C:scale independently maps to integer
C*2^(1074-scale)*5^(38-scale) on that same denominator. Every input identity is
checked against Fraction. Arithmetic uses exact Decimal **integer** numerator
and denominator pairs, cross-multiplies additions, and multiplies nominal and
explicit100/1000 divisors. Source-round outputs form their own exact integer
count*quantum /10^scale pairs for the next stage.

At a round, it first forms the complete numerator*10^quantum_scale and
denominator*quantum_coefficient; it then performs **one** Decimal division and
the independently selected Decimal signed rounding rule. It never divides
price, nominal, source units and cash units separately. Exact inexactness and
all exact tie ownership remain integer-authoritative. The independently formed
Decimal rational is compared with the Fraction expression before every divide;
the two rounded answers must agree at both precisions.

The contract's universal bounds can be independently reproduced using Python
integers. D0<2^1163 and source magnitude after one source round is below 2^1025,
so each common-basis source integer is below 2^2188. Actual weights1+1 over100
or10+1 over1000 sum to at most 11<16; the combined integer is below 2^2192.
Multiplying the nominal coefficient at most2^127 and 10^cash_scale<2^127 gives
quantizer numerator below 2^2446. D0*10^nominal_scale*divisor*Q is below
2^(1163+127+10+127)=2^1427. Rounded count times Q is below 2^2447. Source quantum
operations and final component sums satisfy the smaller contract bounds.
These justify the production40-limb capacity without assuming cancellation.

The separate Decimal expression intentionally keeps larger unreduced cross
products. A raw common-basis numerator is below 2^2188. Scaling a component
gives numerator below 2^2315 and denominator below 2^1300. Adding two such pairs
then forming quantum units gives numerator below 2^3743 and denominator below
2^2727; conservative bounds2^3750 and2^2730 are sufficient. There are at most
1129 and822 decimal digits respectively. Source-rounded expressions are
smaller; rounded cash components combined at the end are smaller as well.
Therefore **every pre-division operation is exact at 1536 digits**. The generator
also asserts exact integer identities and observed digit limits during the run.

The reduced quantizer ratio has numerator below 2^2446 and denominator below
2^1427. Its magnitude has at most 737 decimal digits. A noninteger ratio is at
least1/B from an integer, and a nontie is at least1/(2B) from a half integer;
B<10^430 makes the latter distance greater than 5e-431. A 1536-digit division
has absolute rounding error at most5e-800 for this magnitude, far smaller than
that separation;2048 digits is stricter. Exact integer or half-integer results
fit comfortably in the chosen precision. Consequently these division checks
cannot falsely create or erase an integer or half tie. This is a finite-domain
precision proof, not agreement used as an accuracy heuristic.

## Independent bounded cell enumeration

For the small staged families, a second rounder independently selects among
nearby quantum lattice candidates by signed inequality, minimum distance and
parity. It does not call the divmod authority. Source enumeration places both
interval endpoints, every integer-quantum and half-quantum threshold inside
the interval, and a midpoint in every open cell. This covers all output cells
and boundary tie ownership for all seven signed modes, including zero.

The entire resulting finite source-output sets are combined as independent
Cartesian sets using the recipe formulas. Cash rules are then applied to
**every reachable cash input**, with required component Exact checks kept
separate and in order. An undefined Exact member makes a nonpoint stage
unresolved; a sole undefined value is an Arithmetic/Inexact point failure.
The complete finite-set decision and extrema must equal the image model.
Intervals cover at most 12 source quantum cells so enumeration is bounded.
Continuous source Exact intervals are rejected explicitly, rather than
mistakenly treating a finite sample as the real interval.

This is independent finite evidence for the small families, not a universal
replacement for the contract's monotonicity/lattice proof. Arbitrary large
declared intervals still use exact algebraic images and the proven base/step
test. No output claims ideal analytical price accuracy or validates the caller's
enclosure against an underlying financial model.

## Corpus scope and reproduction evidence

The deterministic families cover all four recipes; all seven cash rules;
all 49 source-rule pairs; both aggregation choices; decimal scales 0..38;
signed i128 limits; arbitrary positive quantum coefficients; algebraic cash
ties and neighbors; all 2047 finite exponent fields with mantissa 0/1/MAX under
both signs; and subnormal leading-bit neighborhoods. This is exponent/boundary
coverage, not enumeration of every finite bit pattern.

Explicit families retain source/intermediate range rescue, wide component
cancellation, Exact-before-range precedence, no Exact net rescue, exact
discrete endpoints with an undefined middle, two variable count steps,
continuous adjacent-float intervals, source ambiguity collapsing to unique
cash, declared-point failures, post-source pointness, cash collapse to an
unrepresentable result, signed lower/upper admission, and precision holes
between different representable endpoints. Mixed evidence/provenance and
identifier case/whitespace/NUL/nonASCII inputs are included.

Generation completed and a separate terminal `--check` returned `identical` on
2026-10-17, Darwin arm64. The frozen generator is 39,164 bytes and fixture is
38,656,756 bytes. There are **154,241 rows and 154,063 distinct serialized inputs**;
identity excludes ID/profile/expectations and includes op plus all nine input
cells. Duplicate serialized inputs are counted honestly. The header contains
every profile, operation, expectation, recipe, cash mode, stage, guarantee and
structured-error census, observed arithmetic widths and cross-check count;
these are recomputed rather than inferred from intended loop bounds.

| Frozen bytes | SHA256 |
|---|---|
| Generator | dda662320a74b485438b23be7a06124567d0dd6794fc85584006aae3c5043a69 |
| Complete fixture | fa97ed1298693f364eccc903d4763829693e7320606bc6e95a0733cbe1477f4e |
| Body including columns | 577cf4f455eae23725b5dc96abbf56588a0435625a3b729ca1eb6ea0495089fd |
| Records only | 8e9d1fac93e94234cab2541a1b0151e2dd011133b4166ef3bd1de1807979dc46 |

Body FNV64 is `1477ce8b9fff859b`; records FNV64 is `7969a6ddeff2dec6`;
generator FNV64 is `58b8eb0530325482`; contract FNV64 is `63fb87ff5504b2b1`;
requirement-row FNV64 is `ceab07dcdae7b6a4`. These pins refer to the frozen bytes
after the maximum-width family was added; preliminary generation is not evidence.

| Profile | Rows |
|---|---:|
| ALGEBRAIC-CASH-TIES |21756|
| ALL-SCALES-SIGNED-LIMITS |2184|
| ALL-SCALES-TINY-NOMINAL |2184|
| BINARY-ALL-EXPONENT-BOUNDARIES |12282|
| BINARY-ALL-EXPONENT-RECIPES |85974|
| BINARY-EVIDENCE-SPECIAL |19|
| BINARY-INTERVAL-ORDER-FINITE-FIRST |361|
| DECIMAL-DECLARED-POINT |429|
| DECIMAL-EVIDENCE-39-SCALES |429|
| DECIMAL-INTERVAL-ORDER |195|
| ENUMERATED-SOURCE-CELLS |4802|
| EXACT-DISCRETE-BASE-STEP |1176|
| EXACT-DISCRETE-TWO-STEPS |1176|
| FULL-WIDTH-COMMON-BASIS |1260|
| IDENTIFIERS |44|
| NONPOSITIVE-NOMINAL |112|
| PROVENANCE-POINT-CONTINUOUS |1092|
| STAGE-PRECEDENCE-RANGE-RESCUE |2142|
| SUBNORMAL-INTERVAL-CONTINUOUS |312|
| SUBNORMAL-LEADING-BITS |312|
| WIDE-ROTATING-INDEPENDENT |16000|

There are 140,170 instruction evaluations, 14,027 evidence constructions and 44
identifier parses. Evaluations distribute as DirtyPer100=49156,
CleanAccruedPer100=27671, StagedPrice100Accrued1000=36253 and CouponPer1000=27090.
The corpus contains 60,489 determined decisions, 3,942 ambiguous decisions,
2,911 Exact-unresolved decisions, 72,716 Arithmetic errors, 112 nonpositive nominal
errors, 78 decimal-order errors, 77 binary-order errors, 198 nonfinite errors,
13,674 valid evidence constructions, 5 valid identifiers and 39 unknown identifiers.
Arithmetic causes are 19,203 Inexact and 53,513 OutOfRange; operations are
57,474 quantize and 15,242 sum. Every reported stage and every applicable endpoint
label occurs. Determined guarantees distribute as 34,312 supplied-binary,
22,927 supplied-decimal and 3,250 declared-enclosure receipts.

The completed run checks 498,566 Decimal quotients (249,283 visited rational
rounds at each of two precisions), 21,382 exact half-quantum rounds,
230,080 rounding inequalities/idempotence results and 21,149 nonpoint monotone
endpoint pairs. Independent finite enumeration checks 7,154 instructions,
96,320 source-cell samples and 25,136 cash inputs. Nonzero input rounds to zero
36,104 times. Counts include visits before earlier-stage failures; they do not
assert that every source/cash stage executes for every row.

Observed common-basis cash numerator width is 2,443 bits, denominator 1,426 bits and
rounded count*Q 1,271 bits. Source numerator/denominator maxima are 2,207/1,163 bits.
The independently unreduced Decimal quotient maxima are 3,735/2,718 bits, below
the separate 3,750/2,730-bit proof. Reduced authoritative quotient maxima are
2,309/1,423 bits. Observed maxima support boundary coverage; the algebraic bounds
above provide the full-domain capacity argument.

## Rust consumer

The new [consumer](../tests/settlement_oracles.rs) is wired from the public
contract without reading settlement production. It consumes every row and
checks exact canonical amounts, Display text, constructor bounds/zero bits,
declared provenance, identifiers, all retained instruction policies and receipt
instruction identity. It compares every structured error field and the typed
root-to-SettlementError-to-AmountError source chain. It also checks sequential
IDs, all active/unused cells, distinct serialized inputs, all row-derived
censuses, duplicate-context rejection and frozen source/contract/requirements/
body/record fingerprints. Full-fixture FNV64 `1d74fd3a8cd8ce4d` pins header
metadata too, including generator-only width/cross-check counters.

The stable consumer SHA256 is
`f1e1b7231476f786c4ea2f1bc3416953927c7f115112d4449517b7e992da2bb9`.
Focused release on this exact source passes both tests, including all 154,241
rows, in 8.15 seconds. Focused debug on the same exact source passes both tests
in 331.65 seconds. Strict scoped Clippy with `-D warnings`, rustfmt check and
`git diff --check` pass. Each Cargo run used a separate target directory under
`/private/tmp`, Rust 1.99 and `--offline --locked`. An earlier redundant debug
run, started before the final header-only fingerprint assertion, was cancelled
with exit130 and is not counted as a pass. Final consumer runs are the evidence.

These numerical rows and focused consumer checks do not supply the full trait,
Eq/Hash, allocation, concurrency, all-target lint, cross-platform execution or
package evidence. Those remain separate implementation/integration obligations
under FR-SETTLE-14..16.
No earlier generator/fixture bytes are edited by this tranche. Yield/risk,
source-complete named markets, calendar/entitlement gates and financial/owner
acceptance in M8_LOCAL_READINESS remain open.
