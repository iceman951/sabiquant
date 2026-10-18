# Independent extended generic bond yield/risk oracles

The new [risk generator](../validation/generator/bond_yield_risk_extended_oracles.py)
and [one-flow inverse generator](../validation/generator/bond_yield_inverse_extended_oracles.py)
derive expectations from the frozen [contract](M8_BOND_YIELD_RISK_CONTRACT.md),
M3 civil/calendar contracts and M4 numerical profiles. The producer did not
read Rust bond/rate/solver production or use QuantLib source/output. All earlier
generators, fixtures and adopted contracts remain unchanged. The initial
[evidence audit](M8_BOND_YIELD_RISK_EVIDENCE_AUDIT.md) explains why this is additive.

## Exact kernel corpus

The [24-column fixture](../validation/fixtures/bonds/yield-risk-extended.fixture)
contains **20,201 rows**: 12,997 residuals, 6,128 scalar terms, 282 signed sums
and 794 successful financial flow observations. Exact integer/Fraction
arithmetic is primary. The only Decimal-dependent kernel rows are the flow
observations, whose complete serialized expectations agree at 160/240 digits.

Residual rows include every finite IEEE exponent field 0 through 2046, equal
and adjacent operands, both signs, opposite signs, MIN_SUB, MAX, the signed
`2^970` threshold and neighbors. The expectation decodes exact operands,
subtracts first, selects `s` from the target, and rounds the scaled difference
once. It separately records unscaled amount-residual admission. Exact integer
checks require finite scaled results, preserved sign and zero iff equal.
The 34-word capacity follows from coefficient `<2^2099` on the MIN_SUB basis;
time-zero constants require `<2^2120`, also below 2176 bits. The constant-plan
classification path itself is covered by separate public regressions, not
claimed exercised by a residual-only row.

Term recipes use one through five nonzero finite positive numerator operands,
zero through two positive denominators, explicit sign and exponent shift 0/1.
The generator forms the exact rational, finds its binade by integer comparison,
and rounds its normalized quotient to 53 bits with exact remainder/parity.
All operand arities and MIN_SUB/MAX products occur; scalar decoding also spans
every exponent field. There are four even-half ties, four odd-half ties and
four normalization carries. In particular `(1+2u)/(1+u)^2` is just below 1
and carries to 1. Retained term exponents reach **-7470 and 7216**.
The five-word width uses numerator `<2^265`, denominator `<2^106` and at most
266 alignment bits, below 320. These are valid finite scalar operands;
there is no forged impossible accumulator/report state.

Sum recipes pass those finite factors through the real term-rounding and
accumulation path. Positive/negative prefixes, a tiny retained tail, exact
cancellation, signs, raw subnormal ties/overflow and independent normalized
admission are explicit. Expected exact totals are integer coefficients on
`2^-8192`; those integers are comparison outputs only. The adapter never uses
them to construct production state. Zero dirty gives the typed normalized
error even for zero numerator. Signed underflow distinguishes signed zero and
its strict signed interval; overflow distinguishes signed infinity.

The 243-word proof uses each coefficient `<2^15461` and a slice length
`<2^usize::BITS`, hence `(15461+usize::BITS).div_ceil(64)` words. The corpus
uses at most hundreds of repeated terms, not a physically allocated
`2^usize::BITS`-length slice. It exercises large exponent separation and carry;
it does not claim executable enumeration of the count-width proof's maximum.

Flow rows cover all 32 configurations, each rounded `RN(1/m)` cutoff and its
adjacent times, both amount signs, moderate signed yields and `next_up(-m)`.
The oracle selects branches against the rounded cutoff with the correct
inclusive comparison. Invalid active-Simple candidates whose exact fused
factor is nonpositive are excluded from this success-only tier: 38 of the
768 proposed signed near-boundary combinations are rejected, leaving 730.
Another 64 rows cover zero time or zero amount. Invalid quote/factor/error
ordering is a separate public regression obligation, met by the
[failure-order suite](../tests/bonds_yield_risk_failure_order.rs).

For accepted PV `Vhat`, q is `RN_fma(1+y*t)`, delta is `RN(m+y)`, and w is
`RN(t+RN(1/m))`. Every contribution is the contract's exact rational in those
represented operands, rounded once to an unrestricted-exponent 53-bit term.
For fixed time the multiplier is linear in Vhat, so extremal PV endpoints
bound its rounded contribution monotonically, including negative signs.

## Public risk corpus and source-real anchors

The [56-column public fixture](../validation/fixtures/bonds/yield-risk-extended-public.fixture)
has **2,181 rows**: 34 profiles times all 32 compounding configurations times
two nonzero signed yields, plus five exact controls. The profiles comprise 15
zero-coupon and 19 fixed-rate definitions. Six scalar conventions, both US
EOM flags, both ISDA final-February roles, explicit ICMA regular/short/long
reference periods, custom Business/252 masks/overrides, strict verified TARGET
and both joint rules are present. Reopening before raw accrual start, detached
ex-coupon and included payment-date flows are explicit. Schedule calendars are
Null/unadjusted; yield and coupon calendars remain independent definitions.
This tier covers successful preparation. Budget exhaustion/coverage errors are
already independently exercised by the contextual bond corpus and new public
reuse/error regressions; no such row is silently accepted here.

The fixture's `ideal_*` cells are **source-real ideal anchors**: coupon amount
is exact face times the supplied binary rate times the exact raw rational DCC,
and horizon is the exact rational DCC. These are distinct from contract
`P(y)` at the *actually prepared binary64* amounts/times, and from the staged
computed dirty price `Dhat`. The kernel flow tier instead fixes actual binary
amount/time inputs and verifies the contract's numerical term model directly.

ICMA/ActualActualISDA/AFB uncertainty admits their declared four-ULP YF
profiles; exact profiles use zero ULP. The generator enumerates the at most
nine allowed time values and bounds the exact supplied coupon triple product.
It then bounds each accepted PV independently. Simple and zero-yield identity
paths use exact Fraction staging. Elementary discounts use the measured M4
criterion `8u*(1+abs(ln d))` times `abs(d)`, with one MIN_SUB absolute floor
and a separate Decimal-reference uncertainty margin. This is a conditional
comparison criterion, **not a rigorous theorem about platform libm**.

PV endpoint intervals are propagated through the exact q/delta/w formulas,
normalized-term rounding and exact signed sums. Dirty is rounded only after
the PV reduction. Numerator intervals are divided directly by the nonzero
dirty interval, so raw overflow/underflow does not block normalized results.
Correlation between flows is conservatively discarded; errors therefore add
on gross terms instead of granting two ULP on a nearly cancelled net value.
All finite ratio intervals in this corpus exclude zero dirty; the exact
zero-dirty control explicitly requires the three normalized errors.

Every finite source-real RN anchor has an upward-rounded absolute comparison
allowance equal to the greatest distance from that anchor to the staged
getter enclosure. This includes coupon/YF representation, factor/PV staging,
term rounding and final reduction/division. It is not an unqualified relative
accuracy or exact-prepared-P claim. Nonadmittable source-real cells explicitly
encode `overflow+`, `overflow-`, `underflow+`, `underflow-`, or `zero_dirty`;
their error cell is `-`. Representable normalized anchors remain numerically
checked even when raw anchors are outside binary64.

The five exact controls include Simple one-year coupon `-100`, redemption
`next_up(100)` and yield `.25`: staged dirty is `2^-46`, while source-real
dirty is `.8*2^-46`. The difference is expected staged behavior, not a code
bug. Other controls cover zero/negative dirty and Continuous MAX/t=2 and
MIN_SUB/t=.5 raw-range rescue, with finite normalized values.

**Frozen-header erratum:** `schedule-metadata` broadly says explicit 6M.
The 34 ordinary profiles use 15 zero-coupon definitions and 19 three-boundary
fixed-rate schedules; the latter use `build_explicit`, tenor 6M, Forward,
first=penultimate middle, Ignore EOM, cap 3. The three additional two-boundary
fixed-rate controls must use the public `build_zero(2)` endpoint schedule,
because `ExplicitStubDates(None,None)` is forbidden. The other two controls
are zero-coupon bonds. Root adopted that valid construction after the producer
freeze; the consumer checks actual raw/pay vectors. Frozen fixture/source bytes
and all financial expected cells were preserved. This is a construction
metadata correction, not an exception to a numerical assertion.

## Decimal precision and independent one-flow inversion

Decimal ln/exp are correctly rounded nearest-even primitives. Exact Fraction
formation removes near-boundary subtraction before Decimal conversion.
Simple and identity controls remain rational. Each other bounded per-flow
calculation uses at most 24 rounded operations, `abs(log d)<1000`, and the
finite corpus's moderate multiplier magnitudes. A coarse absolute/relative
uncertainty `max(1,abs(value))*10^(-p+24)` exceeds the conversion, primitive and
product propagation at p=160 or 240. Sums and final ideal ratios operate on
exact Fractions of these Decimal results; interval endpoints propagate that
uncertainty. The script requires both ends to determine the same IEEE rounding
or signed range status. Complete 160/240 record agreement is an additional
check, not the sole precision argument. There is no 28-digit Decimal aggregation.

The separate [26-column inverse fixture](../validation/fixtures/bonds/yield-inverse-extended.fixture)
has **992 rows**, 31 per compounding configuration: 960 across the 15 contexts
and two signed yields, plus 32 near-minus-m ICMA controls. It is independently
generated from the already pinned new risk helper; the earlier two fixtures
are not rewritten when inverse work changes.

For positive redemption A, converted dirty target T and prepared time t, the
closed forms are `(A/T-1)/t`, `ln(A/T)/t`, and
`m*(exp(ln(A/T)/(m*t))-1)` for Simple, Continuous and Periodic respectively.
Hybrid branch selection is fixed by each admissible prepared time. Targets
come from independently rounded source-real price per 100, with exact
single-round quote-to-amount conversion; target rounding and the source-real
ideal root have separate precision checks. Face 100/300 and redemption 105/1000
exercise that conversion; Clean/Dirty alternate with cached AI exactly zero.

The generator derives a uniform absolute price error over each explicit
bracket. Simple propagates q rounding, reciprocal rounding and PV rounding
with exact rational lower-denominator bounds. Other branches propagate the
conditional M4 E3 discount allowance and final product rounding. Monotonic
fixed-branch real discounts put extrema at bracket endpoints. Invert `T+E`
and `T-E` for every admissible time, then enlarge by the M6 full-width step
tolerance (including its binary operation allowance) and round outward.
These are independent numerical root enclosures, not planted-yield round trips.
Returned dirty enclosures are propagated separately. They retain the solver's
qualifications: a computed zero is not proof of mathematical exactness, and
the evidence is conditional on the elementary comparison profile.

## Reproduction, pins and actual checks

```sh
python3 -I validation/generator/bond_yield_risk_extended_oracles.py --check
python3 -I validation/generator/bond_yield_inverse_extended_oracles.py --check
python3 -I validation/generator/bond_yield_risk_kernel_check.py --output /private/tmp/sabiquant-yield-risk-kernel-reproduction
cargo test --offline --locked --test bonds_yield_risk_extended_oracles
```

Use the repository's required Rust 1.99 toolchain. The private checker copies an
exact source snapshot into a temporary crate, injects a cfg(test) child adapter
only there, and verifies every row through actual private operations. It never
constructs internal states from expected outputs. Original source files remain
unchanged; the harness/fixtures are excluded from the published crate.

Final exact-kernel run passed all **20,201 rows**, one test, zero ignored, in
0.11s after release build. Strict Clippy on that same temporary copy passed.
All 47 original copied files were unchanged. Evidence:
`/private/tmp/sabiquant-yield-risk-kernel-final-v2/evidence.json`, SHA-256
`0e43e82734b6c098888c38f08f5e359e97f1431f405055a093eb6dd726bbd67a`.
The final shared generator `--check` completed with exit 0; log
`/private/tmp/sabiquant-yield-risk-extended-check.log`.

| Artifact | SHA-256 |
|---|---|
| Risk generator | `9c5e77b68dbd9c2212b4ae0fb2cd7cdbc9b420505a55d29c01afa0b559541b85` |
| Kernel fixture | `7c36d7bb9b04ad62295205f209ae7f7c6527b158a60ef7f7f1326945a25fba13` |
| Public risk fixture | `095d4d138be98a71b5c3b89307afcc529a63360e8d31e948d051c335886655d3` |
| Kernel runner | `be607eb014d8f98e1442b6fdeb88743ef6f06d8ade89a389998149ba0eb55f77` |
| Kernel Rust adapter | `efdc66adf5ddebb9cad27c7e9f24e25421ef2aabbc8c4217ef7f3f7546df9adc` |
| Inverse generator | `70723437953a120d18e92bc608a9b2a61112270a3b600b25f01b44a36a4ff258` |
| Inverse fixture | `00c710b1fa82805f23cb8ab9d422d7d5f83df5288a9dbb7039e4db9598dd547b` |
| Public risk consumer | `3cdf425394645604904388ade7330cf76efff59a751d959d3b2136dfcd47281c` |
| Public inverse consumer | `68eefc4acffd098a83b5406c981e07ce6bac54731c6887c67c5947f90b55ac8e` |

Kernel body FNV is `40ed43d920fb5efc`, public risk body FNV is
`9b331d64f6547739`. Headers pin nine contract/policy inputs plus the exact
FR-YR rows, whose SHA is
`eaecaec76703c6e7195684cf2f17fe93eec3ed8f00343361676f839379a24e08`.
Inverse body FNV is `d15325d95af524d8`; it pins the unchanged risk helper in
addition to its own source. Its final generator `--check` completed with exit 0,
log `/private/tmp/sabiquant-yield-inverse-extended-check.log`.

The normal public integration suite consumed all 2,181 risk rows and all 992
inverse rows. Final combined debug and release runs passed three tests with
zero failures/ignored; strict Clippy on both consumers passed. Release times
were 0.27s for the two risk tests and 0.02s for the inverse test. Logs are
`/private/tmp/sabiquant-yr-extended-combined-{debug,release,clippy}.log`.
Their SHA-256 values are respectively
`ed3c647e42e13d66195b3727632b234c569947efdb93761782eb64770692610f`,
`62cfa076dcb477f772093949a67450dea24cc8c560ede0dcd5e87e9608baf272`,
and `f84d582b8c00a954f3427bd1244675d78d928cdd36ea8ab4eeaf5b089d3573a4`.
The cap-one same-host pricing reuse exercise is operational evidence, separate
from the 992 independent inverse accuracy enclosures; repeated elementary
evaluations are not assumed to reproduce identical bits universally.

Finite corpus evidence does not close named-market conventions, legal-calendar
or settlement certification, formal analytical enclosures, financial approval,
other-platform native execution or full M8 acceptance. Existing M6 trace suites,
root's production review/coverage/package checks and separate error/resource
regressions remain necessary complementary evidence.
