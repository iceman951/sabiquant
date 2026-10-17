# Local settlement boundary contract

| Field | Value |
|---|---|
| Status | **Adopted for local implementation; frozen** |
| Baseline | Local `ea22049`, after the amount phases A/B and bond-core evidence audit |
| Date | 2026-10-16, Asia/Bangkok |
| Design input | [Settlement proposal](M8_SETTLEMENT_ADAPTER_DESIGN.md), including independent pre-adoption review |
| Decision scope | Explicit local NI-01 phase C engineering boundary; no owner, market, milestone, platform or publication acceptance |

Adopted on 2026-10-16 under the owner's continuing authority for autonomous
local engineering. Root reviewed the complete draft and the independent
pre-adoption mathematical findings, resolving component Exact checks, party
roles and common-basis weights. This specifies the complete finite-input
boundary for the next local tranche. It changes no accepted [amount](M8_AMOUNT_CONTRACT.md),
[binary64 boundary](M8_BINARY_BOUNDARY_CONTRACT.md) or
[bond-core](M8_BOND_CORE_CONTRACT.md) result, domain or error. This is a local NI-01 reconsideration, not owner API
sign-off, named-market compliance or release acceptance. Authoritative
FR-SETTLE-01..16 rows are adopted in the requirements register; all earlier
contracts, generators and fixtures remain frozen.

## 1. Scope, units and signs

Add public module `settlement`, containing exactly the eight types in section
2. The four bounded recipes below operate on one strictly positive exact
DecimalAmount of **par nominal** in a common caller-selected monetary unit.
Trade recipes use traded par; coupon distribution uses held par. Nominal is
neither a bond count nor the binary64 pricing face retained by Bond.

For trade-proceeds recipes, positive cash is payable by buyer to seller;
negative cash reverses those parties. For `coupon_per_1000`, positive cash is
payable by issuer to holder and negative cash reverses those parties. Signed
generic price/accrued/coupon values are valid. The sign convention is explicit
generic mathematics, not a claim that every named market permits such quotes.
Never encode trade direction with negative nominal or round an unsigned
magnitude before imposing a party direction: directed modes apply to the
signed expression at their specified stage.

There is no default currency, denomination, final quantum, mode, aggregation,
calendar, settlement lag, coupon generation or entitlement. The caller supplies
every unit consistently; no FX conversion or runtime currency validation is
performed. Source-inspired staging is an explicit recipe, not a complete
DMO/Treasury compliance claim. Preserve their unresolved signed tie, final
invoice aggregation, auction/secondary-market, payment-calendar and financial
acceptance gates in the [source reviews](M8_US_TREASURY_SOURCE_REVIEW.md) and
[gilt review](M8_GILT_SOURCE_REVIEW.md).

All remaining [M8 readiness](M8_LOCAL_READINESS.md) work stays in scope:
source-complete named price/yield models, yield inversion, duration/convexity,
justified analytical error enclosures and independent owner/platform/release
acceptance. Completing this boundary alone closes none of those obligations.

## 2. Types, identity and complete public surface

Public types are SettlementEvidence, SettlementRounding, CashAggregation,
SettlementInstruction, SettlementDecision, SettlementReceipt,
SettlementGuarantee and SettlementError. Struct fields are private; contextual
enum variants are non-exhaustive. All are immutable, Debug, Send+Sync+'static.
The first seven implement Copy, Clone, PartialEq, Eq and Hash. SettlementError
implements Clone, Debug, PartialEq, Eq and std::error::Error. No Default,
panicking arithmetic operator, implicit numeric conversion or generic callback
is added. No Ord/PartialOrd is inferred for evidence, instructions or receipts.

Equality/hashing retain the complete original evidence kind, canonical input
values/bits and every instruction policy; equal numerical bounds do not erase
declared-interval provenance or change decimal evidence into binary evidence.
Binary inputs are stored as finite bits with either zero sign canonicalized
to +0. This makes Eq/Hash defined without storing a NaN or deriving equality
directly over f64 fields. Receipt identity includes its amount, instruction
and guarantee. This is definition equality, not merely equal cash proceeds.

```text
SettlementEvidence::exact_decimal(value:DecimalAmount) -> Self
SettlementEvidence::supplied_binary64(value:f64) -> Result<Self,SettlementError>
SettlementEvidence::declared_decimal_interval(lower:DecimalAmount,upper:DecimalAmount)
    -> Result<Self,SettlementError>
SettlementEvidence::declared_binary64_interval(lower:f64,upper:f64)
    -> Result<Self,SettlementError>
SettlementEvidence::decimal_bounds(self) -> Option<(DecimalAmount,DecimalAmount)>
SettlementEvidence::binary64_bounds(self) -> Option<(f64,f64)>
SettlementEvidence::is_declared_interval(self) -> bool

SettlementRounding::new(quantum:AmountQuantum,mode:AmountRounding) -> Self
SettlementRounding::quantum(self) -> AmountQuantum
SettlementRounding::mode(self) -> AmountRounding

CashAggregation::{RoundTotal,RoundComponentsThenSum}
CashAggregation::ALL: [Self;2]
CashAggregation::canonical_identifier(self) -> &'static str

SettlementInstruction::dirty_per_100(nominal:DecimalAmount,dirty:SettlementEvidence,
    cash_rounding:SettlementRounding) -> Result<Self,SettlementError>
SettlementInstruction::clean_accrued_per_100(nominal:DecimalAmount,
    clean:SettlementEvidence,accrued:SettlementEvidence,aggregation:CashAggregation,
    cash_rounding:SettlementRounding) -> Result<Self,SettlementError>
SettlementInstruction::staged_price100_accrued1000(nominal:DecimalAmount,
    clean:SettlementEvidence,accrued:SettlementEvidence,price_mode:AmountRounding,
    accrued_mode:AmountRounding,aggregation:CashAggregation,
    cash_rounding:SettlementRounding) -> Result<Self,SettlementError>
SettlementInstruction::coupon_per_1000(nominal:DecimalAmount,coupon:SettlementEvidence,
    cash_rounding:SettlementRounding) -> Result<Self,SettlementError>
SettlementInstruction::nominal(self) -> DecimalAmount
SettlementInstruction::primary(self) -> SettlementEvidence
SettlementInstruction::secondary(self) -> Option<SettlementEvidence>
SettlementInstruction::aggregation(self) -> Option<CashAggregation>
SettlementInstruction::cash_rounding(self) -> SettlementRounding
SettlementInstruction::source_price_rounding(self) -> Option<SettlementRounding>
SettlementInstruction::source_accrued_rounding(self) -> Option<SettlementRounding>
SettlementInstruction::recipe_identifier(self) -> &'static str
SettlementInstruction::evaluate(&self) -> Result<SettlementDecision,SettlementError>

SettlementDecision::Determined(SettlementReceipt)
SettlementDecision::QuantumAmbiguous { lower_amount:DecimalAmount,upper_amount:DecimalAmount }
SettlementDecision::ExactStageUnresolved { stage:&'static str }
SettlementReceipt::amount(self) -> DecimalAmount
SettlementReceipt::instruction(&self) -> &SettlementInstruction
SettlementReceipt::guarantee(self) -> SettlementGuarantee

SettlementGuarantee::{ForSuppliedDecimals,ForSuppliedBinary64,UniqueForDeclaredEnclosure}
SettlementGuarantee::ALL: [Self;3]
SettlementGuarantee::canonical_identifier(self) -> &'static str
SettlementError::kind(&self) -> &'static str
```

CashAggregation and SettlementGuarantee implement Display and exact
case-sensitive FromStr. Canonical identifiers are their variant names, ALL
has the listed order, no aliases/trimming are accepted. Unknown input retains
the existing IdentifierError::Unknown with concept `cash aggregation` or
`settlement guarantee`. Parsing a guarantee enum does not construct a receipt
or establish a financial guarantee. Mode parsing remains AmountRounding's
existing contract. Unknown identifier parsing retains its existing String
allocation/system-OOM behavior; arithmetic and decimal-evidence handling
introduce no allocation.

There is no public Recipe enum and no parser/Display serializer for contextual
evidence, instructions, decisions or receipts. Recipe identifiers inspect a
family; they cannot reconstruct its input evidence/policies.

## 3. Evidence meaning and construction order

Supplied decimal evidence denotes exactly its canonical decimal rational.
Supplied binary64 evidence denotes exactly the finite represented real value,
including subnormals; it is never read through shortest decimal text.
Declared intervals are closed sets of **all real numbers** between their
exact endpoints, not merely the finite float values lying between them.
A binary interval between adjacent floats is therefore nondegenerate.

`decimal_bounds()` returns Some((value,value)) for supplied decimal evidence,
Some((lower,upper)) for declared decimal evidence and None for binary kinds.
`binary64_bounds()` has the analogous behavior for binary kinds and None for
decimal kinds. `is_declared_interval()` is true for both declared constructors,
even when endpoints are equal; it is false for both supplied-value constructors.
There is no automatic upgrade from a point enclosure to supplied evidence.

Binary constructors reject nonfinite arguments before any zero/order shortcut.
For a single supplied value use endpoint label `value`; for an interval check
`lower` then `upper`. Only after both finite checks canonicalize zero and
compare exact finite endpoint values. Decimal constructors compare validated
exact amounts. A lower endpoint greater than upper is an order error; equality
is valid. Constructor failure does not build a partial evidence value.

Evidence has no hidden analytical accuracy allowance. Declared provenance
means the caller supplied an enclosure claim; the library proves only the
specified consequence of that claim. Existing empirical rate/price tolerance
budgets are not automatically enclosures. No implicit From<BondValuation>,
error-bracket widening, source-document fetch or accuracy-validation flag exists.

## 4. Instructions and exact recipes

Every factory checks nominal.coefficient()>0 before any evaluation work. Its
evidence/rounding arguments are already valid values; this does not reorder
the caller's own preceding argument-constructor evaluations. Zero price,
coupon or eventual cash never waives positive nominal or valid evidence.

Inspector roles and family IDs are fixed:

| Factory | `recipe_identifier()` | primary / secondary | aggregation | source price/accrued rounding |
|---|---|---|---|---|
| dirty_per_100 | `DirtyPer100` | dirty per100 / None | None | None / None |
| clean_accrued_per_100 | `CleanAccruedPer100` | clean per100 / accrued per100 | Some(caller choice) | None / None |
| staged_price100_accrued1000 | `StagedPrice100Accrued1000` | clean per100 / accrued per1000 | Some(caller choice) | Some(q=1e-6,price_mode) / Some(q=1e-5,accrued_mode) |
| coupon_per_1000 | `CouponPer1000` | coupon per1000 / None | None | None / None |

The fixed source quanta are explicit definitions of the staged recipe; the
two source modes remain required arguments. No policy is inferred from the
words normal rounding. Cash policy is always caller-supplied. Single-component
recipes retain absence as None, rather than inventing an aggregation default.

Let R(x;q,m) be the adopted seven-mode signed quantum function, retaining a
wide exact result internally. N is nominal, P dirty per100, C clean per100,
A accrued per100, B accrued per1000 and K coupon per1000. Write Rc for the
cash policy. The recipes are:

```text
DirtyPer100:     cash = Rc(N*P/100)
CouponPer1000:   cash = Rc(N*K/1000)

CleanAccruedPer100:
    capital = N*C/100; accrued = N*A/100

StagedPrice100Accrued1000:
    P6 = R(C;1e-6,price_mode); A5 = R(B;1e-5,accrued_mode)
    capital = N*P6/100; accrued = N*A5/1000

For each two-component recipe:
    RoundTotal:                 cash = Rc(capital+accrued)
    RoundComponentsThenSum:     cash = exact_sum(Rc(capital),Rc(accrued))
```

RoundComponentsThenSum has no further quantization after its exact sum.
Required component Exact checks occur separately in capital-then-accrued
order. With N=100, C=.5, A=.5 and cash q=1 Exact, RoundTotal yields1;
RoundComponentsThenSum fails Inexact at capital. Net cancellation never
waives a required earlier Exact stage.

Only returned final cash is admitted to DecimalAmount's canonical range.
Source prices, combined unrounded components, quantum counts and rounded
components inside one net invoice remain wide even when not representable
as public DecimalAmount. Range cancellation can rescue that net invoice;
it cannot rescue a separately requested payment or an Inexact stage. For
example, supplied binary64 price1e39, exact nominal1e-38 and cent rounding
can produce ordinary representable cash. No finite valid input is rejected
merely because an intermediate DecimalAmount conversion would fail.

## 5. Stage labels, numeric images and Exact

Stable stage strings are `source price`, `source accrued`, `cash total`,
`cash capital`, `cash accrued`, `cash coupon` and `cash net`. Endpoint labels
are `point`, `lower` and `upper`. An endpoint label describes the numeric
input image of the **reported stage**, not the evidence constructor kind.
If that stage input image is a single exact value, use point. Otherwise
evaluate/report lower before upper. Declared interval provenance never changes.

In particular, equal declared endpoints form a point numeric image. A previous
non-Exact source quantizer can also collapse a nondegenerate original interval
into a point image. Both use point at the later stage. Conversely, a nonpoint
input whose cash quantizer collapses to one unrepresentable output still uses
lower for its first final-admission failure: that cash stage's input was not
a point. At cash net, inspect the exact net image of the staged cash components.

Retain sufficient exact image structure internally, with no public interval
algebra:

- A supplied value or equal declared bounds is a point.
- An unrounded nondegenerate declared interval contains continuous variation.
- A non-Exact source quantizer maps an interval to every consecutive integer
  quantum count between its endpoint counts, including exact tie ownership.
- Positive affine scaling/combination preserves endpoints, along with each
  variable source-count step and any nonzero continuous variation.

Every non-Exact mode is monotone. At a required Exact quantizer:

1. For a point, test divisibility exactly. A nonmultiple is
   Arithmetic(Inexact{operation:"quantize"}) at that stage/point.
2. For a nonpoint image with any continuous variation, return
   ExactStageUnresolved at this stage: it contains undefined Exact inputs.
3. For a nonpoint image made only of independent source-count ranges and
   point components, test the lower-count base and each variable component's
   one-count scaled increment against this stage's quantum. If the base or
   any increment is a nonmultiple, return ExactStageUnresolved. Otherwise
   every reachable value is Exact-defined; retain its endpoint image.

The third rule is a bounded base/step divisibility test, not enumeration.
A variable count range always contains neighboring counts, so base plus
each one-step value is reachable. Checking only the two extremes is invalid:
cash inputs0,.5,1 at q1 have Exact endpoints but an Inexact middle. The same
rule applies separately to each required component cash stage; a combined
base may not rescue an undefined capital or accrued stage.

Do not assume cash-quantized images of a discrete source range are themselves
consecutive cash counts: positive scaling can skip counts. This recipe set
has no later quantizer after the two component cash rounds, so exact endpoint
images suffice for their final sum. Source-count ranges and continuous sets
are independent between the two evidence inputs; unknown correlation is not
invented. A caller can supply a tighter direct dirty enclosure when justified.

## 6. Complete evaluation and failure order

For an already constructed instruction, perform these ordered stages:

1. In the staged recipe, compute the complete source price image, including
   its Exact-domain test and lower/upper rounding. Then compute the complete
   source accrued image. Neither source output receives public range admission.
2. Compute the scaled capital/accrued or single-component exact image.
   Under RoundComponentsThenSum, execute the complete cash capital quantizer
   first, then cash accrued; retain wide results. Under RoundTotal execute
   cash total on the combined image. DirtyPer100 also uses cash total;
   CouponPer1000 uses cash coupon.
3. For RoundComponentsThenSum, exact-combine the two rounded cash images;
   this is stage cash net and is not an additional quantum operation.
4. Normalize/admit the final lower amount, then upper if the reported final
   stage's numeric input image was nonpoint. A point needs one admission.
   For total/coupon admission, retain the stage-input point distinction from
   before that quantizer. For cash net, use its exact net input image.
5. If normalized final amounts are equal, return Determined and its receipt.
   Otherwise return QuantumAmbiguous with strictly ordered lower/upper amounts.

ExactStageUnresolved or an error at an earlier required stage returns before
any later stage. Do not prevalidate future Exact domains or final ranges;
source price precedes source accrued, and capital precedes accrued under
component aggregation. Do not reject ordinary non-Exact intermediate width
as ambiguity: distinct source outputs can collapse to unique final cash.

An endpoint final range failure is Arithmetic at the final reported stage and
point/lower/upper context defined above. Never clip or narrow the input set.
Two different representable extremes do not assert representability of every
interior lattice value, because i128/scale has precision holes. QuantumAmbiguous
is not a certificate that every interior outcome is representable. Equal final
amounts avoid that issue because the entire final numeric image is one value.

All computations are exact integer/rational decisions. Canonical final zero
is valid even for nonzero source inputs. No float tie test, midpoint fallback,
nearest-observation fallback, epsilon or hidden refinement loop is permitted.

## 7. Receipt accuracy and provenance

A determined receipt retains the complete original instruction and amount.
Choose guarantee in this strict order:

1. If either evidence value is declared interval evidence, use
   UniqueForDeclaredEnclosure, even for equal endpoints or a collapsed image.
2. Otherwise if any evidence is supplied binary64, use ForSuppliedBinary64.
3. Otherwise use ForSuppliedDecimals.

The first guarantees the recipe outcome for every value in the **declared**
rectangular enclosure, conditional on the caller's enclosure and recipe.
It proves neither that the ideal financial price lies there nor that the
recipe is a source-complete named convention. The other labels denote exact
arithmetic/rounding of supplied values, not ideal analytical price accuracy.
None is called an ideal-price certificate.

Inspectors preserve exact original evidence, par nominal, unit-bearing family
identifier, aggregation and all source/cash quanta/modes. No intermediate
source amount must fit DecimalAmount simply to produce a receipt. There is
no clock, registry, currency lookup, source URL fetch or mutable audit log.
An unresolved outcome makes no cash recommendation; the caller may construct
a new instruction with a tighter independently justified enclosure.

## 8. Bounded original kernel and work

Reuse the private fixed-width unsigned kernel with a new 40-limb (2,560-bit)
width for this module. Existing phase A eight-limb and phase B nineteen-limb
operations retain their contracts and results. No public wide integer, general
expression graph, arbitrary term count or unbounded rational arithmetic appears.

One exact common denominator is D0=2^1074*5^38 <2^1163. Every finite binary64
and scale0..38 decimal input is an integer multiple of 1/D0. Signed source
results round to quanta whose denominators also divide D0. A source magnitude
is below2^1024; after its single source round it is below2^1025.

The actual recipe common bases matter: clean/accrued per100 uses weights1+1
over100; staged price100/accrued1000 uses weights10+1 over1000. The sum of
weights is at most11, **not** an assumed10+10<16. Coupon/dirty use one weight.
Thus each source common numerator is below2^2188 and the weighted combined
common numerator is below2^2192. Nominal coefficient is at most2^127 and
final 10^scale is below2^127, so a final quantizer numerator is below2^2446.
Its denominator, including nominal scale, common basis, divisor<=1000 and
quantum coefficient, is below2^1427. Rounded count times quantum coefficient
is below2^2447.

A source quantum division has numerator below2^2314 and denominator below2^1290.
Its rounded count times source quantum coefficient is below2^1152 because the
rounded value is below2^1025. Reconstructing on D0 multiplies by integer
D0/10^s and remains below2^2188. Each rounded scaled component magnitude is
below2^1153; their exact sum is below2^1154, or below2^2317 on D0. The bounded
Exact base/step tests require only these same ratios and at most two increments.
All fit 40 limbs with retained carries/borrows/remainders; no early public
range test substitutes for the full proof.

Evaluation needs at most two endpoint pipelines plus two variable-component
Exact divisibility tests. Scalar loops have fixed bit/scale/stage bounds;
no input-dependent allocation, callback, unsafe code, I/O, environment/clock,
global/interior mutable state or indefinite precision retry is used. Identifier
error String allocation remains the explicit existing exception in section2.

## 9. Errors and root integration

SettlementError has exactly these six contextual non-exhaustive variants:

| Variant / stable kind | Fields / source |
|---|---|
| NonPositiveNominal / `settlement.nonpositive_nominal` | nominal:DecimalAmount; None |
| DecimalIntervalOrder / `settlement.decimal_interval_order` | lower:DecimalAmount, upper:DecimalAmount; None |
| BinaryIntervalOrder / `settlement.binary_interval_order` | lower_bits:u64, upper_bits:u64 (finite, canonical zero); None |
| NonFinite / `settlement.non_finite` | endpoint:&'static str (`value`,`lower`,`upper`), value:crate::error::NonFinite; None |
| Arithmetic / `settlement.arithmetic` | stage:&'static str, endpoint:&'static str, cause:AmountError; Some(cause) |
| InternalCapacity / `settlement.internal_capacity` | stage:&'static str, operation:&'static str; None |

Arithmetic causes in this contract are only AmountError::Inexact and
AmountError::OutOfRange. Every source/cash quantum operation uses existing
operation string `quantize`, including final range admission after cash total
or cash coupon. **Only** final RoundComponentsThenSum admission uses operation
`sum`, with stage `cash net`. Source/component range is not admitted early.
No new AmountError variant or operation label is introduced. Final quantum
scale is already <=38, so ResultScale is not a reachable settlement cause.

InternalCapacity distinguishes an impossible-under-proof private capacity
failure from ordinary final i128 range failure. Its semantic operation labels
are `decode evidence`, `source rounding`, `scale cash`, `combine cash`,
`cash rounding`, `exact lattice` and `normalize cash`; stage is the required
stage being computed. Preserve typed failure rather than panic/truncation;
do not forge invalid private states solely to execute this proof-excluded kind.

Display wording is not stable. kind, fields, stage/endpoint/operation strings
and the typed source chain are stable. Root Error::Settlement wraps this domain
error, delegates kind/Display and exposes the exact SettlementError as its next
source; Arithmetic then exposes AmountError. Add From<SettlementError> for root
Error. The public type census becomes97 (from89); domain error variants become131
(from125). Do not add an unused named-profile-policy error or future API stub.

## 10. Adopted requirements and independent evidence

These FR-SETTLE rows are authoritative for this complete local tranche and
are recorded in the requirements register:

| ID | Requirement | Required evidence |
|---|---|---|
| FR-SETTLE-01 | Exact positive par nominal, explicit per100/per1000 units and distinct trade/coupon party signs; no currency/default/clock inference | All signs/zeros, positive nominal limits, roles, units and named-gate scope |
| FR-SETTLE-02 | Immutable canonical decimal/binary/declared evidence; full provenance Eq/Hash and exact bounds inspection | Both zero signs, finite bits/subnormals, decimal endpoints, same-value different-kind identity |
| FR-SETTLE-03 | Evidence finite checks lower then upper before order; equal declared endpoints retain provenance | Every NonFinite class/sign, reversed/equal endpoints, mixed failure order |
| FR-SETTLE-04 | Explicit quantum/mode and all seven adopted signed rules; exact family IDs and complete inspection | Full identifier registries, parameter retention, arbitrary quanta and no default |
| FR-SETTLE-05 | Four exact bounded recipes, source-price6/source-accrued5 stage quanta and unrounded coupon scaling | Independent Fraction formulas, per100/per1000 and staged/direct discriminators |
| FR-SETTLE-06 | RoundTotal versus component-round/exact-sum explicit; no hidden extra round or intermediate range admission | Double-rounding differences, source/cash range rescue, signed net cancellation |
| FR-SETTLE-07 | Required Exact checks precede later work; capital before accrued, no Inexact net rescue | N100/C.5/A.5/q1 control; source and component mixed failures |
| FR-SETTLE-08 | Complete non-Exact interval pipeline is monotone; intermediate ambiguity can collapse at final cash | Endpoint inequalities and independently enumerated small staged image sets |
| FR-SETTLE-09 | Exact point/continuous/discrete images differ; base plus every variable increment determines Exact-defined source-count ranges | Adjacent-float intervals, exact endpoints with undefined interiors, integral/nonintegral steps |
| FR-SETTLE-10 | Stage endpoint label follows numeric input pointness, not evidence provenance; first required stage and lower-before-upper admission stable | Declared point failures, source collapse, cash collapse, source/capital precedence |
| FR-SETTLE-11 | Determined/quantum-ambiguous/Exact-unresolved outcomes preserve semantics; no midpoint, clipping or interior-representability assertion | Same/different final extrema, i128 precision holes, endpoint range errors |
| FR-SETTLE-12 | Receipt retains full instruction and conditional guarantee; supplied bits never become ideal-price certificate | Mixed evidence kinds, equal declared bounds, source-policy inspection and provenance tests |
| FR-SETTLE-13 | Forty-limb exact common-basis arithmetic covers full finite input domain, positive weights sum<=11 and every mandatory stage | Independent width derivation, valid carry/division/extreme tests and bounded work review |
| FR-SETTLE-14 | Six errors, stable contextual fields/root typed sources and reused AmountError operations only quantize/sum | Complete reachable catalog/precedence/source tests; honest InternalCapacity proof exclusion |
| FR-SETTLE-15 | New values Send+Sync, no unsafe/dependencies or hidden state; allocation-free arithmetic and immutable concurrent reuse | Traits/type census, strict policy/lint/docs/package and repeated thread checks |
| FR-SETTLE-16 | Exact TOL-E0 independent oracle and full regression preserve all earlier result/fixture contracts and independent acceptance gates | Integer/Fraction plus justified two-precision checks, fresh coverage/package, platform scope and byte audit |

Oracle authors derive expected values from this frozen text before reading
production. Use exact integer/Fraction endpoint and cell/step membership as
authority; independently cross-check Decimal arithmetic at two justified
precisions. Algebraic ties, every mode/sign, all scales, both signed limits,
source/cash cancellation, unreduced width extremes, actual IEEE bit endpoints,
provenance, unknown identifiers and all reachable structured errors are required.

For bounded small cases, independently enumerate admissible source quantum
counts and nearby cash thresholds, including exact tie ownership. Check the
entire final set against endpoint conclusions; Exact undefined inputs remain
undefined. Finite enumeration supports algebraic review but is not a universal
interval proof. Printed source examples retain their published precision and
are not exact binary64 oracles or named-market acceptance by themselves.

Require focused/full debug and release, strict all-target lint/fmt/docs,
fresh production-region coverage with honest proof exclusions, source/type/error
catalogs, actual extracted std-only package checks and byte-identical earlier
fixtures/generators. Native execution and target compilation are separately
reported. No forced OOM, private-state forgery or universal coverage claim is
required or authorized by this contract.
