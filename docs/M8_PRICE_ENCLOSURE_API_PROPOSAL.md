# Price enclosure API proposal

Unadopted engineering proposal, recorded 2026-10-26. This adds no public API,
production implementation or accepted contract. The local runtime checkpoint is
`ae7bda7`; existing contracts and frozen references remain unchanged.

The body records the preceding private research checkpoint. Its statements
about repository writes refer to that checkpoint; this transcription adds
proposal documents and the research driver.

[Design](M8_PRICE_ENCLOSURE_DESIGN.md) ·
[API](M8_PRICE_ENCLOSURE_API_PROPOSAL.md) ·
[Resources](M8_PRICE_ENCLOSURE_RESOURCE_PROPOSAL.md) ·
[Research validation](M8_PRICE_ENCLOSURE_RESEARCH_VALIDATION.md)

Companion to `REPORT.md`. Every name/signature/error/type below is a proposal,
not an adopted API, new requirement or production implementation. The complete
model scope stays fixed. Final Rust lifetimes/layouts and public/error census
need compilation/review after mathematical/resource closure and preproduction.

## 1. Entry points and exact targets

Candidate methods on the four existing prepared plan families:

```rust
BondPricingPlan::enclose_price_per_100(
    &self, yield_rate: f64, kind: BondPriceKind, budget: EnclosureBudget,
) -> Result<EnclosureOutcome<'_, '_>, EnclosureError>;

GiltPricingPlan::enclose_price_per_100(
    &self, yield_rate: f64, kind: BondPriceKind, budget: EnclosureBudget,
) -> Result<EnclosureOutcome<'_, '_>, EnclosureError>;

TreasuryPricingPlan::enclose_price_per_100(
    &self, yield_rate: f64, kind: BondPriceKind, budget: EnclosureBudget,
) -> Result<EnclosureOutcome<'_, '_>, EnclosureError>;

CanadianPricingPlan::enclose_price_per_100(
    &self, yield_rate: f64, basis: CanadianPriceBasis, budget: EnclosureBudget,
) -> Result<EnclosureOutcome<'_, '_>, EnclosureError>;
```

The two lifetime positions mean plan-borrow and underlying instrument/convention
lifetime, with the latter outliving the former. This is signature-level research,
not a claim that these placeholder lifetime spellings have passed Rust compilation.
A borrowed proof avoids cloning instruments, prepared vectors or calendars. Source
plan/instrument lifetime handling must be compiled and reviewed before adoption.

Every output is explicitly **price per100 face**. No amount/per100 inference occurs:
generic normalization is exact100/represented positive face applied to the final
model expression; named sources already use per100. Dirty versus clean is explicit.
Canadian retains Clean, ModelDirty and SettlementDirty; no implicit conversion from
ModelDirty to settlement cash is introduced.

Expose a read-only `EnclosureBasis` discriminator with Generic(kind),
ConventionalGilt(kind), UsTreasury(kind), Canadian(basis). Entry-point types prevent
cross-market basis arguments. This is a model discriminator, not a market/legal
compliance stamp. No default basis, yield, quantum, currency, calendar or budget.

Generic target: exact represented prepared A,t,y, frozen branch selection using
RN(1/m), and exact represented cached accrued amount. It certifies neither original
unrounded coupon mathematics nor ideal unrounded day-count/accrual upstream.
Named target: exact retained source coefficients/A/AI, exact coordinates and exact
represented y, including Treasury selected published AI and all Canadian shifts.
No named Ahat, observed delta/g/factor/PV or native elementary call enters the proof.

## 2. Admission policy — decide it explicitly

**Candidate decision:** these new methods calculate the mathematical model on a
valid existing plan. They do **not** call an existing observed `evaluate`/`risk`/
`yield_from_price` as a gate. A valid mathematical input may receive an enclosure
or valid unresolved outcome even when the old evaluator returns native factor/PV
underflow/overflow/NumericalBoundary. This makes the new operation useful without
silently changing a single old result, failure order or native profile.

Admission is exact and independent of native libm:

| Family/branch | Required mathematical domain |
|---|---|
| Every call | Existing prepared plan was successfully constructed; finite y, canonical zero; retained coefficients/horizons already finite and t>=0. |
| Generic periodic/hybrid global gate | y>-m for the declared m, including empty/time0 and selected Simple flows. |
| Generic selected Simple flow | Exact1+y*t>0, including zero-amount flows. |
| Generic Continuous / selected Periodic | Exact exp argument finite real; positive exact periodic base. Finite binary inputs make the mathematical exp finite and positive as a real even outside IEEE range. |
| Gilt/Treasury | y>-2; q/b strictly positive under exact retained source r/s. |
| Canadian Compound | y>-f, with exact positive base. |
| Canadian MM | Exact1+y*tau>0, with exact retained T,S; required source affine numerator conditions proven/checked using original flow indices. No rounded floor/y*tau gate. |

Validate nonfinite/global quote rules first, then exact per-flow mathematical domains
in original order. No zero coefficient/net cancellation erases a required domain
check. A later exact mathematical domain failure is not preempted by an early
estimated final-price overflow. Existing construction/domain errors remain their
own original types; the new operation has its separately reviewed context ledger.

Work must be charged during admission too. If the budget expires before all exact
domain checks finish, return a **resource/admission-incomplete error**, not an
invalid-domain verdict and not a valid-domain unresolved certificate. Retain the
next original flow index and work used. This avoids claiming an unvalidated later
Simple denominator is valid. No hidden unbudgeted full scan is allowed.

The mandatory core route constructs Simple/MM/Treasury affine denominators EXACTLY
in Raw, proves their sign, then converts them to extended normalized dyadics. A
positive exact rational has a positive p-bit lower endpoint; there is no IEEE range
underflow inside this representation. Exp reciprocals start from[1,2] and positive
squares. Consequently a zero-containing divisor interval is not an admitted-input
unresolved case in this core graph. Keep a private checked invariant for primitive
misuse; do not add an unused public DenominatorEnclosureContainsZero variant.

A future conservative interval-affine alternative could create such ambiguity despite
an exact valid domain; it would need a separately justified route and reachable
preproduction controls. There it would mean refine/exact fallback/unresolved, never
InvalidYield/assert/panic. Near-unit log intervals may cross0 as ARGUMENTS safely;
they are not divisors. The ordinary prototype assert is not an input-facing policy.

## 3. Explicit finite budgets and work inspection

Minimal candidate `EnclosureBudget` has private immutable fields and an explicit
fallible constructor:

```rust
EnclosureBudget::new(precision_bits: u16, max_work: u64)
    -> Result<Self, EnclosureBudgetError>;
```

A fixed precision per call is the initial proposal. Illustrative candidate choices
are64/128/256/512/1024 bits; the exact set/max is not adopted until capacity review.
Unsupported precision and max_work below the explicit entry/terminal minimum are
configuration errors. RESOURCE_PROPOSAL proposes C_init52,656+C_exit1,024=53,680
as an unadopted minimum; this supersedes the earlier merely-nonzero rule. The budget
constructor's fixed field validation is outside the producer run, reports attempted
fields/work0 and creates no arena. A valid call prepays initialization before creating
working storage and reserves terminal-report work that arithmetic cannot consume. Accessors
return both original fields. No Default, automatic precision ladder or hidden retry.
The p96 Fraction prototype is a research choice, not one of these proposed API values.

Charge a deterministic work counter before each admission/normalization/integer-loop/
directed-series/endpoint operation. The final cost table must define indivisible
charged steps so an input cannot cause uncharged variable-width loops. It must
include exact source conversion, both endpoints, log/exp tails, every squaring,
all flow/signed sum/AI/unit work, exact-point detection and endpoint export. A
high-level row/callback count alone is insufficient. This accounting and all exact
primitive bounds are open pre-adoption resource work.

Every outcome/error reports original precision/max_work, actual work used, completed
flow count and admission-complete status; stage/original-index context accompanies
resource stops. Work used never exceeds max_work. No diagnostic/report formatting
traverses an unbounded hidden buffer or changes the numerical result.

A caller may explicitly request another call at another bounded precision. Any
future refinement method must make total versus additional budget semantics explicit
and can intersect sound intervals; no unbounded retry is part of this proposal.

## 4. Controlled immutable proof and outcome types

`EnclosureOutcome` has two conceptual variants:

- **Finite(PriceEnclosure):** exact admission completed, the full requested target
  expression has a rigorous finite exportable enclosure, and an existing declared
  evidence value is available. This does not mean final cash is uniquely determined.
- **Unresolved(PriceEnclosureUnresolved):** exact admission completed but the fixed
  precision/work allowance could not produce a finite exportable result. No cash
  recommendation follows. It retains only a small summary of whether complete extended arithmetic finished;
  the huge bounds stay private and a partial sum is never a full-target bound.

`PriceEnclosure` privately retains an immutable borrow of its exact originating plan,
model/basis/unit discriminator, canonical yield bits, precision/budget/work facts,
encoding, exact endpoints, and a closed implementation-owned proof-path identifier.
The plan borrow preserves all original source/instrument/prepared context without
cloning mutable data or rerunning calendars. The full context inspection route,
including generic prepared-bit meanings, must be specified before adoption.

No public raw-field constructor, free-form proof stamp, boolean validated flag,
From<BondValuation>, From<SettlementEvidence>, Deserialize/parser or caller-selected
algorithm identity creates this wrapper. An implementation-owned enum may identify
exact algebraic identity versus reviewed directed-series version. Public display or
Debug must not suggest source/financial/owner acceptance. It should omit scratch.
Send+Sync follows immutable plan/value properties and must be checked; Eq/Hash,
Clone and exact public lifetime spelling remain API-review decisions, not assumptions.

`PriceEnclosureUnresolved` privately retains origin/basis/yield/budget/work and a
reason such as WorkLimitAfterAdmission or FiniteEndpointUnavailable. A core
zero-containing divisor is a private checked-invariant issue, not an invented public
unresolved variant. Huge extended bounds remain private in the single arithmetic arena and are dropped
on return; no33-word exponent/ratio enters an owned unresolved report or error. A
small summary distinguishes completed extended arithmetic from incomplete arithmetic
work, without exposing huge endpoint values or claiming exact price is outside IEEE.
No conversion to SettlementEvidence exists until finite bounds are established.
This supersedes the prior draft's proposed public extended-endpoint limb views.

Every error variant, including budget construction/admission-incomplete, retains
its applicable original budget/precision/work context. A construction error reports
the attempted raw budget fields and work0; a call error reports the constructed
budget and actual charged work. Shared compact context must not disappear on an
early return. The exact layout remains to be measured against the existing161-byte
large-error threshold; the proposed target is<=160bytes and must not hide a huge
exponent or source ratio inside a nested cause.

Candidate non-domain failures include admission-work exhaustion, allocation failure
if the final resource design uses an explicit reservation, and invariant/capacity
failure with exact stage/index context. Exact invalid quote/domain failures carry
model, original quote bits and original flow/affine context. The final error ledger
must separate these classes; a resource stop must not become InvalidYield or a fake
final-price overflow. No unused placeholder error or root wrapper is adopted now.

## 5. Endpoint/evidence kinds — preserve proved decimal points

**Candidate encoding decision:**

1. If a complete exact rational identity is proved within budget and fits canonical
   DecimalAmount (scale<=38, coefficient in range), retain **ExactDecimalPoint**.
2. Otherwise, if that proved rational is exactly a finite binary64, retain
   **ExactBinary64Point**.
3. Otherwise retain **OutwardBinary64Bounds**, with finite floor/ceiling endpoints.
   Bounds may be equal, but equal observed values alone are not an identity proof.

Exact identity detection starts with cheap reviewed cases: exact zero/grouped-factor
cancellation, unit factors at y0/t0 after all admission checks, and any approved
bounded rational specialization. It is not a promise to materialize huge rational
integer powers, discover every algebraic simplification or decide every rational/tie.
The identity route has the same work budget; if it cannot complete, fall back to the
rigorous interval route or unresolved. A matching old observed result is never proof.

For example a proved exact100.1 can be canonical decimal coefficient1001/scale1.
Exporting it as the two adjacent binary64 bounds would still be sound, but would
introduce a continuous interval and could keep Exact cash unresolved. The decimal
point preserves this exact known value. If no fitting exact decimal/binary encoding
is proved, that possible loss of determinacy is explicit and permitted; do not
silently narrow the interval. Nonpoint decimal interval export is not in this minimal
proposal; adding it would require an explicit further encoding/rounding contract.

All three encodings use **declared** constructors:

```rust
// exact decimal identity
SettlementEvidence::declared_decimal_interval(point, point)
// exact binary identity OR rigorous outward binary bounds
SettlementEvidence::declared_binary64_interval(lower, upper)
```

Never use supplied_decimal/supplied_binary64 merely because a model value was proved
or endpoints became equal. The evidence retains declared provenance, and existing
receipts continue to use UniqueForDeclaredEnclosure. The wrapper supplies the
separate mathematical containment premise; it does not strengthen or rename
SettlementGuarantee or claim financial acceptance.

Expose read-only endpoint inspection through an encoding enum (DeclaredDecimalPoint,
DeclaredBinary64Point, OutwardBinary64Bounds), original precision/work and the exact
evidence bounds. An exact decimal point is not falsely exposed as a dyadic endpoint.
ExactRationalIdentity and IntervalCollapse are distinct proof facts. A sound interval-collapse proof may establish a binary point, but
it is still a controlled interval proof, not an unproved algebraic identity or a
user-supplied trade quote. Separate these facts in inspection.

## 6. Finite export, signed cancellation and unresolved semantics

Internal signed normalized dyadics retain huge exponent metadata. No final IEEE
range judgment occurs at an individual term. Finish signed reduction, exact source
AI shifts and per100 scaling first. Identical exact factor grouping may prove a
zero/constant, but matching rounded factors do not.

A finite outward interval can cross0 and negative clean/all-in is allowed. Tiny
positive bounds export toward [0,MIN_SUB], negative toward [-MIN_SUB,0]. If an
endpoint cannot be bounded by a finite f64 at this precision, return unresolved.
Wide bounds beyond MAX can contain a finite/zero exact price after cancellation;
never clip them or report false final-price overflow. If a future error claims the
true price is outside a finite range, it needs a separate full-target interval proof
and must retain that proof context; this proposal does not require such a claim.

A completed finite interval is returned even when its eventual exact-cash image is
ambiguous. The existing cash consumer decides quantum ambiguity/Exact unresolved,
source collapse and staged/range errors. This keeps enclosure completeness separate
from cash determination. A midpoint is never substituted for an unresolved value.

## 7. Conversion and concrete price-to-cash composition

Candidate wrapper accessor:

```rust
PriceEnclosure::settlement_evidence(&self) -> SettlementEvidence
```

This is infallible only after the Finite variant has validated its controlled
finite endpoint encoding. It returns copied declared evidence. Basis/unit/origin
inspection remains necessary; SettlementEvidence deliberately has no model binding.
Mixing an enclosure from one plan with another plan's cash factory is not thereby
certified. No existing receipt is made to retain a hidden proof or new guarantee.

| Proof target | Existing consumer route and exact source requirement |
|---|---|
| Generic Dirty per100 | `SettlementInstruction::dirty_per_100` with caller's explicit positive nominal and rounding. |
| Generic Clean per100 | `clean_accrued_per_100` additionally needs evidence for exact100*cached_AI_amount/face. Its **observed** accrued_per_100 getter is not that exact rational unless proved. A separately justified interval may widen from lost clean/AI dependence; direct Dirty enclosure is the preferred complete-invoice route. |
| Gilt Dirty | Same originating plan's dirty_cash. |
| Gilt Clean | Same plan's clean_accrued_cash; exact signed source AI is supplied by the existing factory. |
| Treasury Dirty/Clean | Same plan's corresponding factory; Clean uses selected published exact AI and mandatory source/cash stages. |
| Canadian SettlementDirty | Same plan's dirty_cash; explicitly all-in basis. |
| Canadian Clean | Same plan's clean_accrued_cash; Compound gives (D-A)+AI and MM gives (D-AI)+AI. |
| Canadian ModelDirty | A mathematical basis only. It must not be passed directly as SettlementDirty. Obtain a separately certified shifted basis or use its corresponding valid price-only purpose. |

For example: request a Canadian SettlementDirty enclosure, inspect Finite and its
basis/origin, copy its declared evidence into that plan's existing dirty_cash with
explicit nominal/quantum/mode, and evaluate. A determined receipt's existing tag is
still UniqueForDeclaredEnclosure. The price wrapper proves containment; the exact
cash pipeline proves the recipe consequence. If the cash result is unresolved,
no amount is recommended. Coupon/source-accrued-only cash remains its exact source
operation and needs no invented elementary-factor enclosure.

A future typed cash-composition helper could enforce origin/basis matching, but
would need a separately reviewed wrapper/API contract. The minimal proposal does
not pretend existing unbound SettlementEvidence enforces that relationship.

## 8. Required public/admission/evidence controls before adoption

- Four plan entrypoints and every basis/frequency/branch/date/source family; no
  model narrowed into an Unsupported fallback. Preserve all old APIs/results/errors.
- Native factor/PV failure with a valid exact model; empty/zero terms still enforce
  global and active exact domains; invalid later flow versus admission-work limit.
- Invalid exact denominator versus valid denominator whose computed interval includes0.
- Admission-incomplete resource error versus valid unresolved arithmetic/export;
  no partial-net certificate and no early overflow from a huge signed term.
- Exact100.1 decimal point, exact binary point outside DecimalAmount range, zero,
  rational not fitting either point encoding, and irrational/undetected rational
  bounds that legitimately leave Exact cash unresolved.
- Equal declared bounds and no supplied-tag upgrade; false observed-point/algorithm
  stamp constructor attempts; no From<valuation/evidence> or caller-set proof flag.
- Every mode/quantum/source stage/aggregation, signed zero crossing and finite endpoint
  edge; correct Canadian three-basis and generic prepared-cache semantics.
- Budget construction, every consumed work path, exact point detection and export;
  no hidden retry/allocation/formatting traversal. Inspect all original context.

The exact public item/error count, trait/lifetime closure, enum field layout, cost
units and hard resource proof are still open. This concrete proposal is intended
for review and independent preproduction, not immediate production/adoption.

## 9. Resource companion and review boundaries

`RESOURCE_PROPOSAL.md` supplies an unadopted fixed precision/word/liveness/work route.
Its explicit payload is not a Rust stack/heap measurement. Initialization, borrowed
input/output lifetimes, aggregate constructors, return transfers and actual public
report/error layout must be reviewed/measured before adoption. Both the full source
model set and old evaluator behavior remain unchanged.

The prior exact API draft is retained as `API_PROPOSAL.reviewed-v1.md`, SHA256
`efaf74ed61dae2f29548d442e3a7ba3938531515909f39ea3a421f558a077a12`.
This revision follows root's compact unresolved/error requirement and the peer's
encoding/point-proof/budget-context clarification; it is still unadopted.
