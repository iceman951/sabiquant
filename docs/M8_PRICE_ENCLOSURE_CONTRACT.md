# Complete four-model price enclosure contract

Adopted for local engineering on 2026-10-28 under the owner's continuing
implementation authority. The configuration, controlled evidence and contextual
error API is installed in the library; the four financial producers follow in E3.
The [adoption review](M8_PRICE_ENCLOSURE_CONTRACT_REVIEW.md) records the actual
review and compile-only evidence and its limits. Existing contracts,
REQUIREMENTS, fixtures and generators remain unchanged.

The [finite test matrix](M8_PRICE_ENCLOSURE_TEST_MATRIX.md) preserves all 72
planning obligations and maps them to the 18 contract rows below. E1 closes
with this adopted contract and tested API foundation. Exact case/consumer
bindings and independent expectations are E2 B1–B5; E2–E5 remain open. Local contract adoption is not financial,
owner milestone, native-platform or formal M8 acceptance.

## 1. Exact operation and unchanged scope

Add `enclose_price_per_100(&self, yield_rate: f64, basis, budget)` to the four
existing prepared plan types. Generic/Gilt/Treasury take `BondPriceKind`;
Canadian takes `CanadianPriceBasis`. The result is
`Result<EnclosureOutcome<'plan, 'source>, EnclosureError>` with
`'source: 'plan`. Every finite target is price per 100 face. Existing methods,
construction errors, source policies, risk/yield families and cash behavior do
not change. No new country, calendar or bond product is introduced.

Generic retains all 32 compounding configurations, all publicly admitted
prepared signed amounts and nonnegative direct horizons, both price kinds,
positive finite face and represented cached accrued amount. Named models
retain both EOM choices, all admitted geometries and original indices. Canada
retains f2/4/12, explicit Compound/MM and LastCouponMoneyMarket routing, true T
and all three price bases. No configuration may return Unsupported.

The exact target uses the mathematical real values represented by the generic
prepared A/t/AI/face/y binary bits, with hybrid selection at the existing stored
RN(1/m) threshold. Dirty is the complete discounted signed amount sum; Clean
subtracts cached represented AI; exact 100/face is applied last. This does not
certify the original unrounded coupon or upstream day-count mathematics.

Named targets use exact retained source ratios and coordinates, not observed
source/PV/factor caches. Gilt uses selected dividends and exact lambda with par
once; Clean subtracts raw signed source A. Treasury uses raw analytic coupons,
q=(2s+r*y)/(2s) and b=(2+y)/2 with original j; Clean subtracts the selected
published AI. Canadian Compound uses valuation coupons and lambda:
Clean=D-A, SettlementDirty=D-A+AI. MM uses payable coupons and exact
(1+y*rho_i)/(1+y*tau): Clean=D-AI, SettlementDirty=D. ModelDirty is distinct.

## 2. Public types, control and traits

The final surface has fourteen structs/enums and one result alias:
EnclosureBudget, EnclosureBudgetError, EnclosureBasis, EnclosureStage,
EnclosureContext, EnclosureOrigin, EnclosureEncoding, EnclosureProofKind,
PriceEnclosure, EnclosureUnresolvedReason, PriceEnclosureUnresolved,
EnclosureOutcome, EnclosureDomainCause, EnclosureError, EnclosureResult.
The four research function-pointer aliases are omitted. Origin creation is
private to the actual producer; the public facade's origin constructors were
research scaffolding and are omitted. Origin inspection returns the actual
borrowed applicable prepared plan, preserving its instrument/convention borrow.
No new public prepared-flow view or automatic cash helper is introduced.

Budget/context/basis/stage/encoding/proof/domain/reason values are
Clone+Copy+Debug+PartialEq+Eq+Hash. Origin is Clone+Copy+compact Debug, without
PartialEq/Eq/Hash. Finite, Unresolved and Outcome are Clone+compact Debug,
without Copy/PartialEq/Eq/Hash. Clone preserves the same origin borrow.
Every actual public type must be Send+Sync. Diagnostics traverse no plan vector,
source array or numerical arena. Opaque reports have no public field/constructor,
parser, From<observed value>, From<SettlementEvidence> or caller proof flag.
Unresolved has no evidence conversion. Non-exhaustive enums follow project
policy; a deliberately closed mathematical discriminator documents its reason.

EnclosureError is Clone+Debug+PartialEq+Display+std::error::Error, Send+Sync and
'static, with no origin borrow or huge source/exponent field. Allocation errors
compare by retained context/requested bytes; their opaque TryReserveError cause
is deliberately not part of equality. Other arms compare context and their
typed cause/operation. This resolves the v2 facade's missing PartialEq policy.
EnclosureBudgetError additionally supports Copy+Eq+Hash. The crate root Error
gets separate Enclosure and EnclosureBudget wrappers, kind delegation, typed
immediate source and From conversions. No existing root variant is repurposed.

## 3. Explicit budget and normative work tariff

`EnclosureBudget::new(precision_bits: u16, max_work: u64)` accepts exactly
64/128/256/512/1024 and max_work >= 53,680. Precision validation precedes minimum
validation. Its constant-time construction reports original attempted fields
and work_used=0, creates no arena and is outside a producer run. There is no
Default, hidden precision ladder, retry, convergence loop or global counter.

Work is a deterministic abstract debit, not CPU cycles or a peak-memory bound.
Every variable traversal is prepaid before its bounded block, with checked u64
arithmetic and an independent mathematical cap. A failed debit does not execute
that block and does not change used work. Existing exact scalar/source/fold/
codec/arena/kernel helper debits are retained, not replaced by a per-row fee.
The complete adapter tariff is:

| Debit | Units | Exact coverage |
|---|---:|---|
| Entry | 64 | Fixed call context, finite quote classification, signed-zero canonicalization and applicable scalar frequency-floor comparison |
| Scalar-frame initialization | 8,560 = 16*531+64 | Prepaid before the one ZERO_FRAME initialization and its fixed bookkeeping |
| Lazy arena initialization | 44,032 | Prepaid before at most one try_reserve_exact(24), 24 typed interval initializations and fixed handle bookkeeping |
| Terminal | 1,024 | Reserved on entry; bounded report/error/evidence staging, release and return bookkeeping |
| Admission flow | 16 | One original-order metadata/branch lookup, its progress bookkeeping, excluding separately charged exact affine work |
| Exact sign inspection | 64 | At most 34 raw numerator words plus sign/zero metadata after a required exact affine construction |
| Fixed cached operand staging | 4,096 | Charged once per selected cached source or coordinate operand, including every AI/par operand and each active semantic denominator role; fixed lookup/selection/borrowed staging only, at most five cached words per magnitude; no calendar or whole-vector traversal |
| Reduction commit | 8 | Once per ORIGINAL price flow, only after all of its parts and accumulator updates succeed; Canadian coupon and separate terminal par must both finish before its single public flow commit |
| Basis bookkeeping | 64 | Fixed selection/progress work, excluding each separately charged source conversion/shift/unit operation |
| Encoding bookkeeping | 256 | Fixed choice, finite endpoint/evidence construction and compact result metadata, excluding charged point/normalization/export math |

The source compatibility bridge may use only bounded Wide blocks of width n<=5.
A copy/add/subtract/compare/checked-small-multiply block debits 16*n+32 before
execution. pow10 accepts scale<=38 and prepays 64+scale*(16*n+32) before the
bounded original helper, at most4,320 units. These fees are separate from the
4,096 fixed lookup fee. Full Wide multiplication, general division and GCD are
not authorized by these small-block fees; any required such arithmetic must use
the already charged Frame helper graph. The actual source adapter needs cached
ratios, small decimal/day scaling and terminal N-100D; adding integer par leaves
canonical denominator gcd unchanged, so extracting it needs no GCD. No old
source helper with hidden uncharged loops may bypass this bridge.

Entry+frame+arena+terminal = 53,680. The minimum permits this initialization
schedule; it does not promise completion or force all debits on every outcome.
Arithmetic Work.max=max_work-1,024 and Work.used begins at 64. Every terminal
result adds exactly the reserved 1,024 once. No Drop arithmetic changes reported
work. Quote rejection before a frame costs 1,088 and performs no reservation.

Frame initialization and lazy arena initialization are distinct. Finite/floor
quote checks occur first. Then debit/init the single frame, complete any exact
MM global check and every original-order admission check without an arena.
These fixed staging fees exclude arithmetic, division, GCD and pow10 reconstruction, which must invoke and debit separately bounded helpers. A fee never authorizes an uncharged variable loop. Only fully admitted calls enter price arithmetic. The mandatory zero-yield
exact fold and point codec can finish without any interval allocation. Other
routes debit arena initialization before the one reservation. There is no second
reservation, growth, PV vector, calendar query or prepared-plan clone per call.
Allocator internals and allocator-chosen capacity are not a byte-exact promise.

If a debit stops before all admission is complete, return AdmissionIncomplete;
this includes an exact global MM check at QuoteAdmission with next index None.
After admission, any debit stop returns WorkLimitAfterAdmission, with no usable
partial bound. It must not silently switch to another cheaper numerical route.
An allocation failure remains AllocationFailed even after admission. Impossible
checked capacity/protocol defects remain InternalCapacity, never InvalidYield.

## 4. Exact admission and failure precedence

The new operation uses an already valid prepared plan and does not call observed
evaluate/risk/inverse as a gate. Retain attempted quote bits in every error,
including nonfinite payloads and negative zero. Successful finite/unresolved
contexts canonicalize yield signed zero to positive zero.

Validate nonfinite y and the applicable global periodic floor before Frame
initialization or arena reservation. Compiler prologues may reserve/probe stack
earlier; this is not a promise about physical stack allocation order. Generic periodic/hybrid requires y>-m even for selected Simple,
empty plans, zero amounts and time zero. Gilt/Treasury require y>-2, Canadian
Compound y>-f. Canadian MM uses exact 1+y*(T-S)/365>0, not the compound floor.
Selected Generic Simple requires exact 1+y*t>0 on each original prepared flow,
including zero coefficients. Exact affine sign is inspected before normalization.
Required checks run in original order; cancellation never waives admission.

Validated Treasury 0<r<=s and y>-2 imply q>0. Canadian 0<=rho_i<=tau and positive
exact d imply positive exact MM numerators. These are retained geometry/source
premises of the actual adapters, not fabricated reachable public domain errors.
Positive exact raw inputs have positive extended normalized lower endpoints;
zero-containing divisors are checked private graph invariants, not a new public
valid-input unresolved reason.

## 5. Complete numerical route

Use one borrowed Frame (531 u64 words) shared by exact admission, source seeds,
fold/point and scalar/kernel math. Raw magnitude capacity is 34 words, mantissa
16, signed exponent magnitude 33, arithmetic scratch width 52. Every helper
returns small metadata/status and writes large numbers through disjoint borrows.
No heap integer, unsafe, native libm containment premise or enormous absolute
exponent shift is permitted. The bounded log/exp route uses the reviewed atanh
and Taylor tail certificates, all squarings and both interval endpoint paths.

For zero yield, exact admission still runs first. Stream the full selected sum
through the consuming Fold Session. Generic uses the represented dyadic grid,
cached AI subtraction and exact face normalization. Named roles are fixed
semantic cache roles with model caps Gilt3/Treasury3/Canadian5, structural aliases
omitted; each term denominator must divide its assigned role. The private union
Fold alone proves only D dividing the common product; actual source adapters
must establish the stronger own-role/source/cap premises. Treasury uses selected
published AI only. Canadian terminal par is counted once, with all basis shifts
and explicit zero AI terms included in checked declared counts.

The complete exact raw result prefers fitting canonical DecimalAmount, then
exact finite binary64. NoFittingPoint is a successful codec result: normalize
that complete raw value and export rigorous finite outward bounds if possible.
The successful complete Fold finish establishes complete_extended_arithmetic=true
before codec, fit or export begins. If NoFittingPoint requires an arena, preserve
and normalize the staged complete Raw0/E0 target into its destination interval
BEFORE initialize_unit or another helper can clobber those raw fields.
An exhausted identity attempt does not certify a partial sum. Other identities
may be recognized only by bounded reviewed exact work; finding every algebraic
cancellation is not required.

General arithmetic builds the exact affine/base operands, rigorous factors,
complete signed streaming sum, source clean/AI shifts and exact final unit scale.
Do not reject an individual huge term or round a prefix into IEEE range. Only
after the full target is formed export finite outward bounds. Positive tiny
values may need [0,MIN_SUB], negative tiny values [-MIN_SUB,0]. Missing finite
endpoints yield FiniteEndpointUnavailable; no clipping, midpoint or recommendation.

## 6. Proof encoding, context and errors

Finite output retains origin, context, existing DECLARED SettlementEvidence,
encoding and an implementation-owned mathematical proof kind. All declared
encodings, including equal endpoints, retain declared provenance and the existing
UniqueForDeclaredEnclosure receipt guarantee. The wrapper's containment premise
does not imply market/source/owner acceptance.

Preferred exact point encodings are Decimal then Binary64. ExactRationalIdentity
means a complete bounded exact expression was proved. IntervalCollapse means the COMPLETE FINAL requested price, after every basis
and unit conversion, has equal controlled extended endpoints or equal final
finite binary64 export endpoints. Intermediate or prefix equality does not qualify. It proves one
exact value and triggers a budgeted Decimal-then-Binary fit check. An internal
singleton such as 2^100+1 may fit Decimal even when its binary64 outer endpoints
differ; the fit check therefore precedes classifying final binary bounds. DirectedArithmetic denotes
a nonpoint rigorous enclosure. A collapsed interval is never inferred from old
observed equality. The permitted encoding/proof combinations are:

| Encoding | Proof facts |
|---|---|
| ExactDecimalPoint or ExactBinary64Point | ExactRationalIdentity or IntervalCollapse, with the original proof fact retained |
| OutwardBinary64Bounds, strictly lower<upper | ExactRationalIdentity or IntervalCollapse after NoFittingPoint; DirectedArithmetic for a general non-singleton internal interval |

A direct exact-rational point keeps ExactRationalIdentity. A controlled collapse
uses the preferred exact point when one fits; otherwise its strict outward
bounds may retain IntervalCollapse because proof fact and encoding are distinct. Exhaustion
during that fit check gives WorkLimitAfterAdmission with complete arithmetic=true;
it does not downgrade to an equal-bound encoding. An INTERNAL extended dyadic singleton need not fit a binary64 or decimal
point: for example (2^53+1)*2^-53 has valid strict outward bounds
[1,1+2^-52]. After its known-point fit returns NoFittingPoint it remains
OutwardBinary64Bounds with IntervalCollapse. It is not an invariant failure
or finite-endpoint unavailability. Existing SettlementEvidence
continues to support equal declared bounds unchanged. No huge bound leaves the arena.

Context fields are budget, charged work, admission_checked_flows,
reduction_completed_flows, next_original_index, yield_bits, basis, stage,
admission_complete. Stage values are Initialization, QuoteAdmission,
FlowAdmission, Source, Kernel, Reduction, BasisAndUnit, Export, Completion.
Admission and reduction counts are independent. Before a current flow commits,
next_original_index identifies that original source index. Global arithmetic setup (role/common-base Source/Kernel), basis/export and
terminal stages use None. A source stop counts only earlier completed ORIGINAL
price flows; setup has zero completed flows. Fold-internal AI/par commits do not
advance the public flow count.
Final source/unit phases have every price-flow commit complete. Additional AI
or par operations in an identity stream do not inflate price-flow counts.
Unresolved additionally exposes reason and complete_extended_arithmetic. That
flag becomes true immediately after the complete requested target, basis and
units exist, BEFORE codec/fit/export. A later codec, arena-fallback, fit or export
stop retains true. It is false for any earlier stop.

| Stable kind | Additional fields | Immediate typed source |
|---|---|---|
| enclosure_budget.precision | attempted p/max_work, work0 | None |
| enclosure_budget.minimum_work | attempted p/max_work, work0 | None |
| enclosure.non_finite_yield | context/original quote | ValueError |
| enclosure.yield_not_above_floor | context, payments_per_year | EnclosureDomainCause |
| enclosure.nonpositive_simple_denominator | context, exact horizon bits | EnclosureDomainCause |
| enclosure.nonpositive_money_market_denominator | context, days_to_true_maturity | EnclosureDomainCause |
| enclosure.admission_incomplete | context/next original index | None |
| enclosure.allocation_failed | context, requested_bytes=19,200 | TryReserveError |
| enclosure.internal_capacity | context, operation tag | None |

Operation tags distinguish source, exact_fold, point_encoding, normalize, add,
multiply, divide, log, exp, export, initialization and progress. They expose no
source ratio or new public cause graph. The final catalogue must distinguish
proved unreachable invariant guards from executed reachable outcomes.

## 7. Cash composition and resource evidence

Copy finite declared evidence only into the compatible existing same-origin
factory. Generic Dirty uses dirty_per_100; Clean additionally needs separately
justified exact 100*cached_AI/face evidence, not the rounded accrued_per_100 getter.
Its actual cached AI is publicly inspectable through accrued_amount. Gilt Clean
adds raw A; Treasury Clean selected published AI; Canadian Clean settlement AI.
Canadian ModelDirty is never consumed as all-in. All seven modes, arbitrary
positive quantum/nominal, applicable component/total aggregation, mandatory
source publication order, universal Exact and honest ambiguity remain unchanged.
No new automatic cash method is required by this contract.

The arena has 24 typed 800-byte intervals and at most one reservation; a successful
exact-point route may have none. The scalar frame is 4,248 bytes. These are typed
logical storage facts, not a universal stack/allocator/peak guarantee. Actual
whole-producer constructor, ordinary/identity/error/return paths, every helper
and typed report/Result ABI must be reviewed on the built implementation. Target
EnclosureError <=160 bytes at the native checkpoint; do not embed old large
market errors. Entry and terminal coverage apply equally to no-arena paths. The actual
constructor/transfer/call-graph proof of the fixed entry/init/terminal charges
remains an implementation check under FR-ENCL-13/14; opaque facade layouts alone
do not prove the terminal reserve. No 32KiB promise is adopted. Foreign compilation is not native
execution evidence. All source/test/package/coverage claims bind current bytes.

## 8. Finite requirement and validation mapping

The following contract rows group the existing scope; none enlarges it.

| ID | Obligation | Required evidence |
|---|---|---|
| FR-ENCL-01 | Four prepared models, all32 generic configurations and existing bases | Actual public entrypoints and complete dated family matrix |
| FR-ENCL-02 | Exact represented/source targets and final per100 normalization | Independent source/cache projection and full target equations |
| FR-ENCL-03 | Exact admission and global/original-order failure precedence | Frozen logical domains, literal boundary/error/budget cases |
| FR-ENCL-04 | One explicit p/budget, no default or hidden retry | Constructor, repeated/threaded controls and actual work inspection |
| FR-ENCL-05 | Actual source-role selection and retained original indices | Frozen V3 inputs, actual cached-source adapter consumers |
| FR-ENCL-06 | Rigorous bounded log/exp/affine factors | Independent exact certificates plus actual integrated consumers |
| FR-ENCL-07 | Complete signed sum, shifts and range rescue | Cancellation, huge prefixes, AI/basis/face discriminators |
| FR-ENCL-08 | Bounded zero-yield identity and preferred fitting points | Actual full-source folds, exact point and fallback controls |
| FR-ENCL-09 | Finite outward export or honest unresolved | Signed subnormal/extreme, completed-target and work-stop cases |
| FR-ENCL-10 | Controlled proof/origin/encoding and two lifetimes | Actual opaque types, positive/negative external compilation |
| FR-ENCL-11 | Full context, independent progress and compact typed errors | Final catalogue, every reachable arm and honest exclusions |
| FR-ENCL-12 | Compatible unchanged exact cash composition | End-to-end originating-model/evidence/cash oracle cases |
| FR-ENCL-13 | Exact normative tariff and every bounded loop prepaid | Matched actual source/helper counter ledger and cut points |
| FR-ENCL-14 | One shared frame and lazy bounded typed arena | Actual allocation/liveness/layout/early-return graph |
| FR-ENCL-15 | Explicit traits, compact diagnostics and root conversions | Actual item/error catalogue and trait/source-chain controls |
| FR-ENCL-16 | Independent financial expectations frozen before producer | Dated S/Q/P/C/H attestations, exact row-to-consumer mapping |
| FR-ENCL-17 | Strict integrity and useful containment comparison | Create-only reproducibility, certificate checks and corrupt controls |
| FR-ENCL-18 | Old behavior and complete current integrated verification | Regressions, semantic faults, scoped coverage, docs and extracted package |

The existing 72 EP planning obligations remain the finite detailed test inventory.
Their contract-group mapping is recorded in the companion matrix; bind every
row to exact frozen cases and consumer assertions before closing E2.
E2 requires the missing independent financial/control expectations to be frozen;
helper success is insufficient. E3–E5 require actual full public production,
originating cash composition and whole-graph verification. Independent financial,
native-primary-platform, owner/dependency and formal review/merge acceptance
remain separate. More tests do not create additional exit conditions.
