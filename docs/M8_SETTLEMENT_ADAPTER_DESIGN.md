# M8 financial settlement boundary proposal

| Field | Value |
|---|---|
| Status | **Planning proposal; no adopted contract or API** |
| Baseline | Local bond-core commit `5ea07cd`, after amount phases A/B |
| Date | 2026-10-16, Asia/Bangkok |
| Scope | Complete NI-01 phase C: explicit monetary units, source stages, input meaning, final quantum outcome and provenance |
| Preserved scope | [M8 readiness](M8_LOCAL_READINESS.md), including remaining named price/yield models, inversion and risk |

Recommend a small settlement module that evaluates a bounded, explicitly chosen
financial formula on exact supplied values or declared enclosures. Retain the
exact [amount](M8_AMOUNT_CONTRACT.md), [binary boundary](M8_BINARY_BOUNDARY_CONTRACT.md)
and [bond-core](M8_BOND_CORE_CONTRACT.md) contracts. A settlement date, a generic
analytical BondValuation and an exact cash amount remain distinct concepts.

The proposed module must retain every monetary rounding stage. It must also
distinguish exact arithmetic on supplied inputs from the conditional statement
that every ideal value inside a declared interval produces the same cash
amount. It cannot invent an analytical error enclosure from empirical ULP
allowances. This document adopts neither market compliance nor a new numerical
policy; it proposes the remaining decisions and their executable obligations.

## 1. Source evidence and boundaries

The reviewed [UK DMO fourth edition](https://www.dmo.gov.uk/media/334d05fo/yldeqns_v4.pdf#page=3)
specifies trade-level penny rounding without intermediate price rounding;
printed p.17 separately applies penny rounding to accrued interest on traded
nominal. Its negative accrued-interest cases make signed ties relevant. This
task inspected the complete existing renders of printed pp.2,17,28 and verified
the PDF digest against the [gilt review](M8_GILT_SOURCE_REVIEW.md).

For non-indexed Treasury securities,
[Appendix B I.D.4](https://www.ecfr.gov/current/title-31/subtitle-B/chapter-II/subchapter-A/part-356/appendix-Appendix%20B%20to%20Part%20356)
specifies five-place accrued interest per 1,000 before nominal scaling;
II defines six-place price per 100. I.A coupon scaling instead uses unrounded
per-1,000 amounts. The source uses normal-rounding language, and its examples
sometimes round intermediates for exposition. The current HTML was rechecked;
the [Treasury review](M8_US_TREASURY_SOURCE_REVIEW.md) retains scope/version detail.

Do not infer a universal final invoice aggregation rule, signed tie convention,
secondary-market entitlement or holiday calendar from those statements.
Before a source-labelled conformance claim, resolve each remaining choice
against its own primary evidence. Explicit engineering staging can progress
while these financial gates stay visible. Index-linked instruments, FRNs,
bills, currency conversion, tax and fees need separate contracts.

## 2. Units, signs and authoritative inputs

The adapter's nominal is an exact strictly positive DecimalAmount of par in
one caller-selected monetary unit. Trade-proceeds recipes use traded par;
the coupon-distribution recipe uses held par. It is neither a count of bonds
nor the binary64 `Bond::face()` used to normalize an analytical valuation. Price per 100 face and accrued interest per 100 or 1,000 par must
be explicit in constructor names/argument roles. No currency, minor unit,
par redemption, basis conversion or cash quantum is inferred.
Generic recipes do not validate currency identity or perform FX conversion.
A future named sterling/US-dollar factory must state that coordinate in its
contract and name/metadata, rather than deducing it from the number 0.01.

For trade-proceeds recipes, positive cash is payable by buyer to seller;
negative cash reverses that direction. For `coupon_per_1000`, positive coupon
cash is payable by issuer to holder, with negative cash reversing those parties.
This is an explicit generic distribution orientation, not a named-market
coupon-generation or payment-calendar rule. Permit signed generic price and
accrued components so negative accrued adjustments and explicit correction
transactions remain defined. This mathematical sign domain does not establish
that a named market permits every such quote. Reject nonpositive nominal;
do not encode trade direction by a negative nominal. A payer/receiver view is
an explicit application conversion, with checked endpoint negation.

Apply each rounding mode to the signed expression at that stage. Do not round
a magnitude and then reapply an accounting direction: directed modes would
change meaning. Exact source input zero and every rounded zero are canonical
zero. Negative accrued interest is a signed component, not a rejected
nonfinite/underflow observation.

Input evidence has three meanings:

1. **Supplied decimal:** the exact decimal rational the caller supplied.
2. **Supplied binary64:** the exact finite represented bits, independent of
   shortest decimal text. Finite negative zero becomes canonical zero.
3. **Declared enclosure:** the caller asserts that the relevant ideal
   quantity lies in a specified closed interval. Endpoints are exact supplied
   decimals or exact finite binary64 values. The adapter proves a consequence
   of that interval; it does not prove the caller's assertion.

Separate decimal and binary interval constructors preserve endpoint meaning
without inventing a general public rational type. Both endpoints of a given
constructor use the same representation. A point enclosure remains marked
as a declared enclosure: provenance cannot be upgraded by equal endpoints.
NaN and infinities are invalid endpoints; ordered finite endpoints, including
subnormals, are valid. A zero-width interval at a rounding tie must execute
the actual tie rule, not be rejected merely for touching a boundary.

Using `valuation.dirty_price_per_100()` as a supplied binary64 input rounds
exactly that observed per-100 quote. It does not reconstruct an ideal price,
and need not equal an operation on the earlier rounded `dirty_amount` and
pricing face. The call site must select the observation/coordinate deliberately.
No implicit `From<BondValuation>` or conversion through float formatting is
recommended.

## 3. Bounded financial formulas and required stages

Write `R(x;q,m)` for the already adopted signed quantum rule. Every policy
supplies an explicit positive quantum `q` and mode `m`; there is no default.
Let `N` be positive traded nominal; `P`, `C`, `A100`, `A1000` are explicitly
dirty/clean/accrued components in the indicated per-par units.

### Direct dirty and clean/accrued routes

The direct route is `cash = R(N*P/100; cash_policy)`, with no intermediate
admission or quantization of P. This is the necessary arithmetic shape for
DMO trade-level scaling. A separately requested accrued amount is
`R(N*A100/100; accrued_cash_policy)`; computing it does not change the direct
dirty cash result. The two original DMO `d1` roles in the gilt review remain
separate; the settlement boundary never regenerates or detaches coupons.

An explicit clean/accrued route needs a chosen aggregation policy:

```text
RoundTotal:
    cash = R(N*(C+A100)/100; cash_policy)

RoundComponentsThenSum:
    cash = R(N*C/100; cash_policy) + R(N*A100/100; cash_policy)
```

The second formula exact-sums the staged rounded components; it does not apply
another undocumented final rounding. The two results can differ by a quantum.
Neither is selected from the bare words clean price or accrued interest.
Reporting clean cash as dirty cash minus rounded accrued cash is a third
derived reporting identity, not a silent claim that independently rounded
clean-price proceeds equal that residual.

The full expression must remain exact until its declared round points.
`checked_add(C,A100)` can reject a sum that becomes representable after
nominal scaling; independently rounding the two scaled values can change
RoundTotal. Existing phase A `multiply_divide` is the correct direct exact
decimal primitive, but it does not by itself supply a fused scaled sum.

### Explicit Treasury-inspired source stages

The source-specific candidate stages are:

```text
P6 = R(C;     quantum=0.000001, explicit price_mode)
A5 = R(A1000; quantum=0.00001,  explicit accrued_mode)

capital   = N*P6/100
accrued   = N*A5/1000
RoundTotal cash = R(capital+accrued; cash_policy)
RoundComponentsThenSum cash = R(capital; cash_policy)+R(accrued; cash_policy)
```

Treat this as an explicit **staging recipe**, pending acceptance of its final
aggregation/tie policy for a named invoice. Supply both source modes explicitly;
do not translate normal rounding into a silent nearest-even default. A named
source-compliant adapter may eventually fix a proven mode/quantum in its
contract; that is a declared convention, not a hidden generic default.

Coupon-payment scaling is a separate recipe and must not reuse A5 merely
because the input is expressed per 1,000. The candidate complete coupon path
scales the unrounded supplied coupon-per-1,000 value, then applies its explicit
payment policy. It does not derive the financial coupon from a rounded M7
binary64 YF. A later rational coupon producer can provide exact input or a
justified enclosure under its own period/date contract.

DMO source-labelled penny validation should check its applicable quantum and
supported rounding/sign regime explicitly. A different quantum or unresolved
negative tie rule remains an explicitly chosen generic recipe, with that
limitation in provenance. No factory labelled compliant may quietly accept
an arbitrary caller policy. Named invoice acceptance remains a gate until
the complete stage sequence is sourced.

## 4. Intermediate range and bounded exact implementation

Only externally returned final cash must be admitted to DecimalAmount's
canonical i128/scale range. Required source rounding is an exact lattice
operation internally; its intermediate price/quantity need not fit the
public amount type. This matters even when every endpoint is a finite float.
For example, observed price `1e39`, exact nominal `1e-38`, divisor 100 and
cent rounding give ordinary representable cash. Converting the price into
DecimalAmount first fails and wrongly narrows the settlement domain.

Likewise, a component rounded as an internal part of one net invoice can
exceed the final cash representation and cancel with another component.
If an application needs each component as a separately payable amount, it
must request those separate payments and admit each result individually.
The net-invoice formula is not permission to rescue a failed independent
payment. Do not expose a public arbitrary-width integer to solve this issue.

A small private extension of the generic unsigned kernel suffices for the
proposed formulas. One conservative common denominator for every input is
`D0 = 2^1074 * 5^38`: it contains every finite binary64 denominator and every
phase A decimal denominator. Convert signs and magnitudes exactly into this
basis. Fixed source quantum rounding also returns a rational on this basis.

There are at most two components, with integer weights at most ten when
combining per-100 and per-1,000 terms over a common 1,000 divisor. A finite
binary input has magnitude below `2^1024`; a single source-rounding step adds
less than its quantum, which is below `2^127`. Thus conservatively each
source result is below `2^1025`, `D0 < 2^1163`, and the weighted combined
numerator is below `2^2192`. Multiplying by nominal coefficient `<=2^127`
and final quantum scale factor `<2^127` stays below `2^2446`; the final
rounded-count-times-quantum bound is below `2^2447`. The corresponding
denominator, including nominal scale, basis divisor and quantum coefficient,
stays below `2^1427`.

Forty u64 limbs (2,560 bits) therefore provide a conservative fixed capacity
for these complete finite-input formulas. More explicitly, a source rounding
division uses numerator below `2^2314` and denominator below `2^1290`.
Its rounded `k*Q`, measured at source scale `s`, is below `2^1152` because
the rounded source value is below `2^1025`. Reconstructing that value on D0
by multiplying `k*Q` by the integer `D0/10^s` stays below `2^2188`.
For RoundComponentsThenSum, each rounded scaled component has magnitude
below `2^1153`; their exact sum is below `2^1154`, or below `2^2317` on
the D0 basis. These are full exact-product bounds, including unsigned carry
paths, not permissions to truncate an intermediate. Phase A's eight limbs and
phase B's nineteen must remain unchanged for existing operations. The
proposed extension is private, std-only, stack-based and bounded; no general
expression graph, arbitrary term count, denominator multiplication chain or
unbounded precision retry is proposed.

An implementation can optimize exact-decimal-only inputs through the smaller
fused kernel after proving identical stage/error behavior. It must not use an
optimization that rejects an intermediate merely because a public amount
constructor would reject it.

## 5. Interval consequences and honest accuracy

For every adopted non-Exact mode the quantum map is monotone. Positive nominal,
positive basis divisors and positive component weights preserve monotonicity
through each bounded recipe, even when an accrued component is negative.
Evaluate the **complete** stage sequence on the vector of lower endpoints
and on the vector of upper endpoints, using exact arithmetic and identical
rounding policies. With independent component intervals these bound the entire
declared rectangular input set.

If both final outcomes are the same representable amount, all admissible
values yield that amount. Return a unique-for-declared-enclosure result.
If final amounts differ, return an explicit unresolved decision with both
amounts; never pick the midpoint, nearest observed float or one endpoint.
An interval can touch a tie and still have a unique result; equality at a
boundary must use the selected rule, including quantum-count parity.

Do not stop at an ambiguous intermediate source stage. Different admissible
P6/A5 values may still collapse to one final cash amount at the selected
nominal/quantum. The final decision concerns the whole recipe. Conversely,
separate component cash decisions cannot be combined into a stronger result
by assuming unknown correlation. If clean and accrued bounds are correlated,
the rectangular enclosure is conservative; a caller can provide a tighter
dirty-price enclosure through the direct route. This proposal adds no
dependency-tracking interval framework.

`Exact` is a partial map and needs separate treatment. A point at any required
Exact stage succeeds only for a true lattice multiple, otherwise Inexact.
A nonzero-width **continuous** interval reaching an Exact stage contains
nonmultiples: return an unresolved-exact-stage decision even when both
endpoints happen to be multiples. A preceding source quantizer instead has
a discrete image: every quantum count between its endpoint counts is
reachable. Do not confuse its interval hull with that discrete set.

For a later Exact cash stage fed only by such source-count ranges and point
components, test the lower-count base value and each variable component's
one-count cash increment. Every admissible cash value is an exact multiple
iff the base and all those increments are multiples of the cash quantum.
This is a bounded modular check on at most two components, not an enumeration
of a potentially huge count range. A nonmultiple step yields an unresolved
Exact stage, including when both extreme endpoints are multiples. When all
steps are exact, differing endpoint amounts are ordinary quantum ambiguity.
Under `RoundComponentsThenSum`, apply this test separately at each required
component cash quantizer, in capital-then-accrued order. A combined base cannot
rescue an Inexact component stage. With nominal100, C=.5, A100=.5 and cash
quantum1 Exact, the combined total is1, but each required component is.5 and
the first component stage must fail Inexact. `RoundTotal` instead quantizes
that combined total and succeeds. Wide final-range cancellation never waives
an earlier required Exact-stage failure.

A source stage may collapse an input interval to one point, allowing a
unique later Exact result. Retain point/continuous/count-range propagation
internally; no general public interval algebra is needed.

An endpoint whose required **final** amount cannot be represented produces
a typed stage/endpoint range failure. Do not narrow or clip the interval to
representable values. A required source intermediate stays wide as section 4
specifies. Closed input intervals and monotonicity suffice for the decision;
no epsilon, floating subtraction or estimated boundary distance is needed.
Different representable endpoint amounts do not establish representability
of every interior lattice value: canonical i128/scale representation has
precision holes. The ambiguity result makes no such assertion. Equal endpoint
amounts avoid that issue because the entire final image is the same value.

Guarantee labels must say one of:

- Exact recipe result for supplied decimal inputs.
- Exact recipe result for supplied binary64 inputs (possibly mixed with
  exact decimal inputs).
- Unique recipe result for every value in the **caller-declared** enclosure.

The third is conditional on the supplied enclosure and financial recipe.
No existing BondValuation uncertainty is automatically converted into such
a certificate. Its rate kernels' conditioned calibration and Rust elementary
precision qualifications remain in force. Naively constructing bounds as
`x-error` and `x+error` in binary64 is not a guaranteed enclosure; any future
producer must derive outward-rounded bounds and justify all upstream error.
Published rounded source values likewise need their correct quantization
preimage, conservatively enclosed, rather than being treated as ideal points.
An upstream producer can justify bounds using an exact rational formula or
a reviewed outward-rounded error analysis. A solver bracket alone is not a
price enclosure unless model monotonicity and the boundary evaluations are
also bounded. A producer-specific ideal-price guarantee needs its own proof,
version and retained tests; adding a free-form proof label or a boolean
validated flag would not supply one. Exact agreed transaction quotes can
use the supplied-input route without claiming any ideal-price calculation.

## 6. Concrete minimal public surface

Names below are proposals. Use private immutable fields and contextual
non-exhaustive errors; retain Send+Sync. No implicit conversions/defaults are
added. Existing BondSettlement remains the date/entitlement input type and is
not repurposed as a monetary result.

```text
SettlementEvidence::exact_decimal(DecimalAmount) -> Self
SettlementEvidence::supplied_binary64(f64) -> Result<Self,SettlementError>
SettlementEvidence::declared_decimal_interval(lower,upper) -> Result<Self,SettlementError>
SettlementEvidence::declared_binary64_interval(lower,upper) -> Result<Self,SettlementError>

SettlementRounding::new(quantum:AmountQuantum, mode:AmountRounding) -> Self
CashAggregation::{RoundTotal,RoundComponentsThenSum}  // no Default

SettlementInstruction::dirty_per_100(nominal, dirty_evidence, cash_rounding)
SettlementInstruction::clean_accrued_per_100(nominal, clean_evidence,
                                          accrued_evidence, aggregation, cash_rounding)
SettlementInstruction::staged_price100_accrued1000(nominal, clean_evidence,
    accrued_evidence, price_mode, accrued_mode, aggregation, cash_rounding)
SettlementInstruction::coupon_per_1000(nominal, coupon_evidence, cash_rounding)
    -> Result<Self,SettlementError>
SettlementInstruction::evaluate(&self) -> Result<SettlementDecision,SettlementError>

SettlementDecision::{Determined(SettlementReceipt),
    QuantumAmbiguous{lower_amount,upper_amount},
    ExactStageUnresolved{stage}}
SettlementReceipt::amount() -> DecimalAmount
SettlementReceipt::instruction() -> &SettlementInstruction
SettlementReceipt::guarantee() -> SettlementGuarantee
SettlementGuarantee::{ForSuppliedDecimals,ForSuppliedBinary64,
                     UniqueForDeclaredEnclosure}
SettlementError  // structured stage/input/endpoint/domain context
```

These four factories expose the complete proposed finite recipe set and
its units at the call site; all four return `Result<Self,SettlementError>`.
The staged factory pins its source-stage quanta
in the frozen recipe but still requires explicit source modes; no claim
labels it a complete Treasury invoice convention before its final aggregation
and domain are verified. A later named DMO/Treasury factory should accept
only a source-complete supported policy and carry a versioned profile ID.
It can delegate to these same recipes without a separate pricing engine.

Receipt provenance must retain the exact original evidence, traded nominal,
recipe/version, every source/cash quantum and mode, aggregation choice and
conditional guarantee. A cloned immutable instruction inside the receipt
is enough; no stringly typed provenance object, source URL fetch, clock,
database, currency registry or public generic pipeline is necessary.
Expose original evidence/parameters through unambiguous inspection. Do not
require an unrepresentable intermediate source price to be an inspectable
DecimalAmount merely to produce the final receipt.

The two unresolved variants are valid outcomes, not a guessed cash value or
an excuse to silently widen an enclosure. Error means invalid terms or a
required result outside the bounded output domain. There is no callback or
automatic retry: a caller can supply a tighter justified enclosure and
evaluate a new instruction. Every evaluation performs at most two endpoint
pipelines plus two bounded Exact-stage increment checks, with the fixed
bit/stage limits.

## 7. Failure order and error proposals

Evidence constructors first reject nonfinite lower/upper operands in argument
order, then interval reversal. Decimal endpoints are already validated;
their ordering uses exact numeric comparison. Instruction construction checks
positive nominal before any numeric evaluation; the initial four factories
already encode their unit/recipe shape, and rounding values are validated.
No zero quote/zero result waives an invalid nominal or interval. A later named
profile wrapper must check its supported policy/domain before numeric work.

Evaluation follows recipe order: required source-price stage, required accrued
stage, scaled component combination, cash stage, final representation and
receipt construction. For an enclosure, validate both inputs before any
endpoint evaluation, then process lower before upper consistently. Detect a
nonpoint required Exact-stage interval at that stage; do not return an early
generic ambiguity merely because a non-Exact source interval has width.
If another precedence is selected, freeze it and test the same mixed-failure
cases; do not let optimization change which field/stage is reported.

Proposed stable `settlement.*` errors need these separate variants:

| Variant / stable kind | Context |
|---|---|
| `NonPositiveNominal` / `settlement.nonpositive_nominal` | Exact `nominal:DecimalAmount` |
| `DecimalIntervalOrder` / `settlement.decimal_interval_order` | Exact lower/upper DecimalAmount values |
| `BinaryIntervalOrder` / `settlement.binary_interval_order` | Original finite lower/upper u64 bits, with canonical zero |
| `NonFinite` / `settlement.non_finite` | Endpoint label (`value`, `lower`, `upper`) and existing typed NonFinite class |
| `Arithmetic` / `settlement.arithmetic` | Stage, point/lower/upper endpoint and typed AmountError cause; point Inexact or final OutOfRange |
| `InternalCapacity` / `settlement.internal_capacity` | Stage and private operation; an impossible-under-proof capacity failure |

Private wide capacity failures must be distinguished from final i128 admission
so a programming/proof defect does not masquerade as an ordinary financial
range limit. Root Error should wrap the settlement domain layer and expose
its typed next source; nested amount errors remain inspectable beneath it.
Stage and endpoint labels can be documented stable static strings initially;
extra public enums are unnecessary unless callers need more than matching
structured context. A named conformance wrapper will need a reachable
unsupported-policy/domain error when it is adopted; do not add an unused
future variant to the first generic recipe boundary merely for that plan.
The exact final error variants/type census need an API review after the
recipe/domain decisions, not placeholder catalog entries now.

## 8. Independent executable obligations

Generate expectations from frozen formulas before reading production. Use
integer/Fraction endpoints and lattice-cell membership as the authority;
independently cross-check point/staged expressions with Decimal at two
rigorously justified precisions. Finite binary endpoints need exact bit
decomposition. Sixteen-digit float rendering and printed rounded examples
cannot supply exact expectations.

Useful derived Fraction controls (not printed market examples) are:

| Inputs and explicit policy | Required distinction |
|---|---|
| N=10,000,000, P=100.00000049, cent nearest/ties-away | Direct trade rounding gives 10,000,000.05; first rounding P to six places gives 10,000,000.00 |
| N=10,000,000, A1000=.0000049, cent nearest/ties-away | Scaling before final rounding gives .05; the five-place source stage first gives 0 |
| Source-price interval [100.0000004,100.0000006], source six-place nearest, N=1,000, final cent nearest | Source outcomes differ, final cash is uniquely 1,000 |
| Source quantum 1e-6, source counts 0..2, N=50, basis 100, cash quantum 1e-6 Exact | Extreme cash values are exact multiples, while the reachable middle value is not |

Local Python integer/Fraction checks verified the first two controls and the
large-source-price/small-nominal rescue example in section 4. They also checked
the common-basis/conservative-width bounds; this is design arithmetic, not
an implementation, fixture or conformance test result.

Required test families include:

- Exact decimal and supplied binary input distinctions, including `2.675`;
  all source/final modes, signs, zeros, arbitrary final quanta and MIN/MAX.
- Per-100 versus per-1,000 scaling; trade nominal distinct from pricing face;
  scaled nominal below/above basis; coupon versus accrued source-stage paths.
- Algebraic staged/double-rounding discriminators and final aggregation
  differences; explicit signed ex-coupon accrued adjustments.
- Source-round prices outside DecimalAmount range with representable final
  cash; scaled-sum cancellation before final admission; final overflow and
  normalization rescue; every fixed-width carry/division boundary.
- Closed intervals wholly inside a quantum cell, crossing it, touching a
  tie from either side, exact tie points, even/odd counts, intervals containing
  zero and adjacent binary64 endpoints; all directed-mode asymmetries.
- Non-Exact source-stage ambiguity that disappears at final cash; propagated
  point source stages followed by Exact; continuous Exact intervals whose two
  endpoints are multiples; discrete source-count ranges with integral versus
  half-quantum cash steps and equal-looking endpoint divisibility;
  declared-correlation conservatism.
- Final equal endpoint amounts as a conditional certificate, differing final
  amounts as explicit ambiguity, and endpoint range failures without clipping.
- Treasury literature anchors as their explicitly stated staged precision;
  DMO unrounded price scaling as a separate profile. A source-supported
  positive example does not validate an unresolved negative tie rule.
- Every invalid nominal, nonfinite endpoint, reversed interval, unsupported
  profile/policy and mixed-failure precedence; typed source chains, retained
  original evidence, exact guarantee labels and immutable concurrent reuse.
- All previous amount/binary/bond fixtures unchanged; full debug/release,
  strict lint/docs, fresh region coverage with honest proof exclusions,
  extracted package and scoped platform evidence.

For intervals, independently enumerate every nearby quantum threshold and
side/tie ownership on small rational grids rather than relying only on the
production endpoint method. For staged profiles, enumerate every admissible
intermediate quantum count on bounded small grids, then compare the complete
set of final outcomes to the endpoint conclusion. Exact-mode undefined points
must remain undefined. Wider cases use symbolic boundary constructions and
exact integer inequalities; arbitrary finite sampling is not an interval proof.

## 9. Decisions required before adoption

1. Freeze unit/sign semantics, the four bounded recipes, aggregation policy,
   strict positive nominal and endpoint/input interpretation. Preserve all
   existing binary64 contracts; phase C is an additional boundary.
2. Verify source-specific signed tie behavior and the final Treasury cash
   aggregation stage before exposing a named compliance claim. Retain DMO
   and Treasury formula/calendar/entitlement scope as independent gates.
3. Review the complete conservative 40-limb width proof, including source-count
   reconstruction and component-round-then-sum paths, and the bounded Exact
   stage modular checks. Do not substitute an early DecimalAmount range
   rejection or a continuous hull for discrete source-count reachability.
4. Decide exact public inspection/error/receipt names and store enough stage
   provenance to reproduce the calculation without a framework or global state.
5. Keep supplied-input exactness, unique-for-declared-enclosure consequences,
   actual upstream analytical enclosures and ideal financial acceptance
   separate. No current empirical tolerance becomes a formal enclosure.
6. Retain the complete M8 yield/risk and named-market work after this phase.
   A precise settlement boundary alone does not complete the bond milestone,
   settlement-calendar maintenance, owner/platform or publication gates.

## Independent pre-adoption review

A separate read-only Fraction review found no formula, fixed-capacity or signed
monotonicity defect. It checked15,552 small-grid staged endpoint cases and3,600
RoundTotal Exact base/step cases against bounded count enumeration, plus exact
power-of-two capacity inequalities. The two clarifications above separate
per-component Exact checks and coupon/trade party roles. These finite grids
support the algebraic review; they are neither universal interval proofs nor
implementation/market-conformance tests.
