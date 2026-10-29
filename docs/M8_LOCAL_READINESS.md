# Local M8 readiness and implementation record

Recorded 2026-10-16 from verified local commit `4da3f43`; updated after phase A (`0ae9168`), phase B (`8bfa292`), the bond core tranche (`5ea07cd` / `0a44c4e`), the phase C settlement boundary (`65ff67e`), and the locally validated phase D generic bond yield-to-maturity inversion and analytical risk tranche. The owner requested
continuous local progress with broad meaningful tests and explicitly retained
local-only work. That authority permits the staged decisions and implementation
below; it does not grant formal milestone, owner API, financial, native-platform
or publication acceptance. No remote action is proposed.

## Complete objective and current evidence

The [M8 completion checklist](M8_COMPLETION_CHECKLIST.md) tracks five fixed
engineering exits and the separate formal acceptance states. At the current
checkpoint E1 is committed and E2 closes with the local signed-off unit described
in the [preproduction report](M8_PRICE_ENCLOSURE_PREPRODUCTION.md); E3–E5 remain
open. No percentage is assigned; helper tests and code coverage do not measure
milestone completion. Earlier dated sections retain their historical status.

M8 retains the entire [roadmap](ROADMAP.md) scope: zero-coupon and fixed-rate
bonds; explicit settlement and coupon entitlement/ex-coupon; clean and dirty
price; accrued interest; yield inversion; duration and convexity. NI-01 exact
settlement amounts are a related numeric obligation, not a synonym for a
settlement date. Completing an arithmetic foundation or generic price kernel
alone must not replace the remaining M8 work or claim its acceptance.

The prerequisite local M3–M7 engineering tranche at `4da3f43` passed 619 debug
and 618 release tests, strict checks and fresh package/coverage verification
([evidence](M3_US_AFB_VALIDATION.md)). Native Linux/Windows runtime, TARGET
maintenance, full SIFMA and historical OldCDS financial gates remain separate.
A local staging exception follows the same continuing-authority approach used
for M6/M7; no external approval or CI result is invented.

| Work stream | Evidence/current state |
|---|---|
| Conventional gilt primary formulas | [DMO source review](M8_GILT_SOURCE_REVIEW.md): directly downloaded fourth-edition PDF; relevant complete formula/accrued pages visually inspected; explicit source boundaries and amount roles |
| Treasury primary formulas and adopted contract | Historical [Treasury review](M8_US_TREASURY_SOURCE_REVIEW.md), [2026-10-20 source refresh](M8_US_TREASURY_SOURCE_REFRESH.md) and [research validation](M8_US_TREASURY_RESEARCH_VALIDATION.md): all seven examples, 640 daily cells and 116,800 staging cases independently checked. The complete [main](M8_US_TREASURY_CONTRACT.md) and [cash](M8_US_TREASURY_CASH_CONTRACT.md) pair is implemented locally after independent pre-production evidence; [current validation](M8_US_TREASURY_VALIDATION.md) records final verification and acceptance limits |
| Mathematical risk/yield definitions | [Historical ISMA review](M8_ICMA_SOURCE_REVIEW.md): rotated scanned book inspected visually; derivative definitions and independently diagnosed rounded-source example |
| Public reference feasibility | [Pre-contract probes](M8_BOND_REFERENCE_DESIGN.md): 48 attested isolated cases, raw public errors/nonfinite results preserved; no fixture or local financial contract inferred |
| Complete bond API/test proposal | [Bond design](M8_BOND_API_DESIGN.md), including distinct generic/named yield models, explicit date roles, prepared work, typed inversion and all required risk families |
| Exact amount design | [Amount design](M8_EXACT_AMOUNT_DESIGN.md): canonical decimal/quantum modes, full fused operation, independent bound proofs and the binary boundary later adopted as phase B |
| Phase A implementation evidence | [Oracle report](M8_AMOUNT_ORACLES.md), [validation](M8_AMOUNT_VALIDATION.md) and [implementation review](M8_AMOUNT_IMPLEMENTATION_REVIEW.md): 277,577 independent oracle rows and an 18-test blind hand suite pass; full native checks, coverage and package extraction recorded; owner/financial/platform gates open |
| Phase B implementation evidence | [Binary boundary contract](M8_BINARY_BOUNDARY_CONTRACT.md), [oracle report](M8_BINARY_BOUNDARY_ORACLES.md), [validation](M8_BINARY_BOUNDARY_VALIDATION.md) and [implementation review](M8_BINARY_BOUNDARY_IMPLEMENTATION_REVIEW.md): 196,773 independent oracle rows and a 21-test blind hand suite pass; full native checks, coverage and package extraction recorded |
| Bond core implementation evidence | [Bond core contract](M8_BOND_CORE_CONTRACT.md), [oracle report](M8_BOND_CORE_ORACLES.md), [contextual oracle](M8_BOND_CONTEXTUAL_ORACLES.md), [validation](M8_BOND_CORE_VALIDATION.md) and [implementation review](M8_BOND_CORE_IMPLEMENTATION_REVIEW.md): 18,152 retained exact-quote/binary64-plan rows, 11,136 contextual day-count/compounding rows, an additive high-precision audit and 19 public hand/edge/review tests pass across zero-coupon and fixed-rate bullet bonds, ex-coupon entitlement, earned-minus-detached accrual, single-rounding binary64 per-100 quote conversion, and two-stage direct-horizon `BondPricingPlan` / `BondValuation` |
| Phase C settlement boundary evidence | [Settlement contract](M8_SETTLEMENT_CONTRACT.md), [oracle report](M8_SETTLEMENT_ORACLES.md), [hand report](M8_SETTLEMENT_HAND_TESTS.md), [implementation review](M8_SETTLEMENT_IMPLEMENTATION_REVIEW.md) and [validation](M8_SETTLEMENT_VALIDATION.md): 154,241 independent oracle rows and 27 blind hand tests pass across four bounded per-100/per-1,000 monetary recipes, source/cash rounding stages, continuous/discrete `Exact` domains and conditional receipts; 730 debug / 729 release tests, 97.0946% production coverage and 48-member package extraction recorded |
| Phase D generic bond yield and risk evidence | [Yield/risk contract](M8_BOND_YIELD_RISK_CONTRACT.md), [oracle report](M8_BOND_YIELD_RISK_ORACLES.md), [implementation review](M8_BOND_YIELD_RISK_IMPLEMENTATION_REVIEW.md) and [validation](M8_BOND_YIELD_RISK_VALIDATION.md): 141 independent dual-precision (160/240-digit Decimal + exact Fraction) oracle rows and 11 hand tests pass across `BondPricingPlan::yield_from_price`, `BondPricingPlan::risk`, `BondYieldReport`, `BondYieldSensitivity`, `BondRisk`, and `BondYieldError` |
| Independent design challenge | [Review](M8_DESIGN_REVIEW.md) and [yield/risk design review](M8_BOND_YIELD_RISK_DESIGN_REVIEW.md): DMO symbol-role conflict, constant-price signed-flow degeneracy, resolved decimal-product API gap and verified yield/risk residual/derivative bounds |

## Adopted immediate tranche: complete phase A exact decimal foundation

The agent adopts the explicit [amount contract](M8_AMOUNT_CONTRACT.md) under
continuing local authority. This records the NI-01 reconsideration triggered
by [ADR-0006](adr/ADR-0006-numeric-policy.md) without changing accepted H1
binary64 analytics or silently supplying a currency/quantum/rounding policy.
The contract includes canonical i128/scale0..38 values, exact ordering/text,
all seven explicit rounding modes, arbitrary positive decimal quantum,
checked exact arithmetic, cancellation-safe reduction and complete ratio/fused
two-decimal product/divisor quantization. Intermediate i128 overflow cannot
reject a representable final result; the eight-limb proof and signed limits
are part of the contract.

Implementation follows independent contract review and expectation generation.
The producer must not read oracle output to copy expectations into arithmetic.
At least one generator/hand suite author derives results before inspecting any
amount production implementation. All previous fixtures/generators remain
frozen. Required evidence includes exact integer/rational oracles, algebraic
ties, two-precision Decimal cross-checks with a justified precision, all scales,
resource/error ordering, MIN/MAX, fused rescue cases, threads, public/error
catalogs, full strict checks, fresh coverage and actual extracted package.

## Retained next stages and explicit open decisions

1. Phase B is adopted and implemented under the
   [binary boundary contract](M8_BINARY_BOUNDARY_CONTRACT.md), with evidence in
   [M8_BINARY_BOUNDARY_VALIDATION](M8_BINARY_BOUNDARY_VALIDATION.md). Exact
   rounding of supplied bits is still not a certificate for an ideal
   financial value near a quantum boundary.
2. Phase C is adopted and implemented under the frozen
   [settlement contract](M8_SETTLEMENT_CONTRACT.md), with evidence in
   [M8_SETTLEMENT_VALIDATION](M8_SETTLEMENT_VALIDATION.md):
   four explicit positive-par per100/per1000 recipes, source/cash stages,
   all seven modes, total/component aggregation, exact supplied evidence and
   conditional all-real interval consequences. Currency selection remains the
   caller's responsibility; source-complete named market adapters and analytical
   enclosure producers remain open. No universal market convention is selected.
3. The bond core tranche (`sabiquant::bonds`) is adopted and implemented under
   the [bond core contract](M8_BOND_CORE_CONTRACT.md), with evidence in
   [M8_BOND_CORE_VALIDATION](M8_BOND_CORE_VALIDATION.md) and
   [M8_BOND_CONTEXTUAL_ORACLES](M8_BOND_CONTEXTUAL_ORACLES.md): owned zero-coupon and
   fixed-rate bullet terms, retained schedules/redemption, explicit settlement,
   payment inclusion and ex-coupon boundary, earned-minus-detached accrued
   adjustment, single-rounding binary64 per-100 quote conversion, and two-stage
   direct-horizon `BondPricingPlan` / `BondValuation` with shared accrual work
   budgeting.
4. Price/yield models need explicit capability coverage. DMO fractional powers,
   Treasury fractional simple factors and generic direct-horizon rates are
   different models. A source-complete named adapter must separately verify
   dates, irregular periods, entitlement and examples. No maintained UK/US
   holiday calendar is inferred from WeekendsOnly.
5. Phase D generic bond yield-to-maturity inversion and analytical risk is
   adopted and implemented under the frozen
   [yield/risk contract](M8_BOND_YIELD_RISK_CONTRACT.md) (`FR-YR-01..16`), with
   evidence in [M8_BOND_YIELD_RISK_VALIDATION](M8_BOND_YIELD_RISK_VALIDATION.md):
   bounded source-preserving fallible Brent callbacks (`brent_with_payload`),
   sufficient exact-pair mathematical constant classification, explicit yield
   coordinates, target-selected residual scale 0/1 in 34 limbs, normalized
   53-bit extended-exponent derivative/time-weight terms in 5-limb scalar
   arithmetic, and 243-limb accumulation on `2^-8192` across all six risk views.
6. Freeze new reference fixtures only after the corresponding contract exists;
   retain every raw error, nonfinite reference observation, unsupported context,
   entitlement and rounding difference with an executable classification.

These are remaining work items, not a claim that all implementation is blocked.
They will be resolved and adopted locally in dependency order while the full
objective and independent financial/platform/owner/release gates remain visible.

## Follow-up evidence audit after `5ea07cd`

The [core validation](M8_BOND_CORE_VALIDATION.md) now records698 debug and697
release tests, fresh production-only coverage and actual package extraction.
The [Decimal audit](M8_BOND_CORE_DECIMAL_AUDIT.md) corrects the original sign-only
cross-check limit without modifying frozen expectations. New public regressions
cover signed partial underflow, conditioned cancellation, every normal quote
exponent and successful reopening work caps. No production Rust changed.
The [settlement contract](M8_SETTLEMENT_CONTRACT.md) implements the
next local boundary. Generic yield/risk, analytical enclosure producers and
source-complete named market models remain implementation obligations.

## Validated settlement and contextual evidence tranche (2026-10-17)

The complete frozen FR-SETTLE-01..16 contract adds eight immutable public types
and six contextual errors (catalog97/131). The private forty-limb kernel
covers all finite binary64 and scale0..38 input coordinates without early
public range admission. Independent oracle (154,241 rows) and blind hand (27 tests)
authors derived expectations from the frozen contract before inspecting production.
Full regression (730 debug / 729 release), fresh production coverage
(12,198 / 12,563 = 97.0946%) and extracted 48-member package evidence are
recorded in [M8_SETTLEMENT_VALIDATION](M8_SETTLEMENT_VALIDATION.md).

An additive [contextual bond oracle](M8_BOND_CONTEXTUAL_ORACLES.md) addresses
FR-BOND-04/06 numerical coverage for ICMA regular/irregular periods, terminal
30E/360 ISDA, both US EOM flags, custom and strict TARGET Business/252,
reopening and shifted payments across all32 supported compounding configurations.
Its new 11,136-row fixture is separate from the existing frozen core evidence.

## Phase D implementation and evidence audit (2026-10-18)

The complete frozen [FR-YR-01..16 contract](M8_BOND_YIELD_RISK_CONTRACT.md) adds
four immutable public types (`BondYieldReport`, `BondYieldSensitivity`,
`BondRisk`, and `BondYieldError`) and eight domain error variants (six in
`BondYieldError`, two in `BondError`), bringing the public type catalog to
**101** and the domain error catalog to **139**. Independent dual-precision
(160/240-digit Decimal + exact Fraction) oracle evidence (141 rows across six
families) and 11 hand tests are recorded in
[M8_BOND_YIELD_RISK_ORACLES](M8_BOND_YIELD_RISK_ORACLES.md),
[M8_BOND_YIELD_RISK_IMPLEMENTATION_REVIEW](M8_BOND_YIELD_RISK_IMPLEMENTATION_REVIEW.md),
and [M8_BOND_YIELD_RISK_VALIDATION](M8_BOND_YIELD_RISK_VALIDATION.md). Named
market yield conventions (UK DMO quasi-period powers, US Treasury fractional
first-period prefixes, Canadian models), analytical enclosure producers, and
owner/platform/milestone acceptance remain open.

The post-commit audit at `a33e8a7` found that earlier phase D reports described
nonexistent error names, an endpoint-selected residual scale and binary64
intermediate risk admission, while the actual implementation follows the frozen
34/5/243-limb contract. Those report descriptions have been corrected. The
frozen 141-row generator also compares a hybrid boundary against exact `1/m`
with strict `<`, unlike the adopted `<= RN(1/m)` rule, and uses blanket numerical
allowances. Existing rows are retained. The additive evidence has completed:
20,201 exact-kernel rows, 2,181 conditioned public risk rows and 992
independent one-payment inversion enclosures. Fresh native checks, coverage
with every uncovered region assessed, and an actual extracted package are
recorded in the [validation report](M8_BOND_YIELD_RISK_VALIDATION.md). The
financial comparisons are conditional on the measured M4 elementary profile,
not analytical enclosures. Named markets and analytical enclosure producers
remain required M8 work.


## Adopted next named-market tranche: complete conventional gilt

After the generic evidence audit and failure-order regressions, the next local
engineering tranche is the source-specific conventional UK gilt model. The
[primary-example comparison](M8_GILT_SOURCE_EXAMPLES.md) independently matches
all eight conventional price cells, eight inverse yields,214 daily accrued
cells, three separate accrued anchors and the published long-first dividend.
The checker uses exact Fraction and160/240-digit Decimal; it reads no Rust or
QuantLib implementation and generates no fixture before a contract exists.

The source separates rounded published first dividends from the unrounded
rational ingredients used for accrued interest. Complete quasi dates must
include a skipped quasi date even where no payment occurs. These are concrete
inputs to the forthcoming named-market design; the generic plan remains the
adopted direct-horizon model. Date admission, exact money stages, numerical
bounds, complete yield/risk and source-capability decisions must be reviewed
and frozen before implementation. Treasury, Canadian, analytical-enclosure
and all independent acceptance gates remain required work.

The complete [model proposal](M8_GILT_MODEL_DESIGN.md), independent
[numerical review](M8_GILT_NUMERICAL_DESIGN_REVIEW.md) and
[52-family test proposal](M8_GILT_TEST_DESIGN.md) are now available. They
distinguish exact rational quasi coordinates from rounded generic ICMA times,
preserve source coefficients beyond a derived i128 decimal view, and derive
new bounded scalar/cash widths. Those proposals adopted no API or requirement.
The complete [contract draft](M8_GILT_CONTRACT_DRAFT.md) was subsequently
integrated and [independently challenged](M8_GILT_CONTRACT_REVIEW.md) across
API/lifecycle, exact cash and numerical proofs. The resulting complete
[conventional gilt contract](M8_GILT_CONTRACT.md) is adopted on2026-10-19 for
local implementation, with authoritative `FR-GILT-01..24`. It includes exact
source publication/AI, price, bounded yield inversion, all six risk views and
published-dividend/AI/dirty-trade/clean+AI cash together. Eleven types and31
domain errors were implemented at the gilt checkpoint, bringing its public
catalog to112 and domain errors to170; the later Treasury catalog is126/199. Complete local [integration validation](M8_GILT_VALIDATION.md) passes827 debug
and826 release tests, strict checks, fresh coverage and package extraction;
this engineering implementation does not establish independent acceptance.

Pre-adoption native scaled-log/exp and exact outward-guard challenges are
recorded in the contract review. Their finite conditional profiles are not
analytical enclosures or foreign-platform runtime evidence. Independent
[pre-production oracles](M8_GILT_PREPRODUCTION_VALIDATION.md) now supply86,722
rows across exact source/date/cash, private numerical primitives, ideal
price/risk and target-driven inverse references. All regenerate and pass the
actual shared Rust reader before production. Public source/cash and ideal-price/risk/inverse consumers, the private exact
numerical adapter and public hand controls exercise the implementation. The
ideal consumer explicitly records22 uninformative rows with no financial
accuracy assertion ([review](M8_GILT_IDEAL_CONSUMER_REVIEW.md)). Complete native
regression, [coverage classification](M8_GILT_COVERAGE_REVIEW.md), actual extracted
packaging and foreign-target compilation pass. The [evidence map](M8_GILT_EVIDENCE_MAP.md)
qualifies all24 requirements and applicable52 semantic families. Earlier
evidence, local scope and broader named-market, financial/platform/owner/release
gates remain unchanged.

## Adopted complete Treasury tranche (2026-10-20)

The complete [main Treasury contract](M8_US_TREASURY_CONTRACT.md) and
[cash component](M8_US_TREASURY_CASH_CONTRACT.md) are adopted together under
continuing local-development authority after complementary API/cash and
numerical reviews and root reproduction, recorded in the
[integration review](M8_US_TREASURY_CONTRACT_REVIEW.md). The adopted scope is
fixed-principal semiannual notes/bonds, regular/short/long first periods,
price, bounded yield inversion, all six risk views and all four cash recipes.
It retains explicit daily-basis/tie policies and dated-versus-issue roles.

The26 `FR-UST` obligations and [76 semantic test families](M8_US_TREASURY_TEST_DESIGN.md)
are requirements, not passing test counts. The adoption checkpoint had
112 public types/170 domain errors. Four independent corpora were subsequently
accepted and frozen before production, with37,024 reference records and66
blind hand tests. The complete price/yield/six-risk/four-cash implementation
now has126 public types and199 domain errors. The
[validation record](M8_US_TREASURY_VALIDATION.md) distinguishes actual private
primitive checks, public observations, useful conditional ideal bounds,
semantic mutants, final native regression and production coverage. The
actual extracted package passes its std-only and external API checks.
Final guarded native verification passes914 debug/913 release tests and
strict lint/fmt/docs checks. Instrumented production coverage is
19,801/21,088 regions; all493 missing Treasury coordinates have a reason
ledger, without removing exclusions from the denominator. All76 semantic
families have an explicit evidence mapping. These are local engineering
results, with the public-projection, primitive-superset and conditional ideal
accuracy qualifications retained in the linked validation record.
Earlier production arithmetic, frozen contracts, generators and fixtures
remain unchanged. Native foreign-runtime, source-calendar/announcement,
independent financial/owner, analytical-enclosure, Canadian and wider M8
gates remain open.

## Canadian source and model research (2026-10-23)

The [Canadian source review](M8_CANADIAN_SOURCE_REVIEW.md) retains genuine
IIAC1.1/1.2/1.3 and scoped Bank of Canada publications with complete-byte
pins and actual visual inspection. It distinguishes valuation/payable
coupons, valuation/settlement AI, compound/MM quotation and source numerical
errors. Monthly AI, completed quasi pieces, the long-first definition and
the generalized MM terminal sum require explicit local reconciliation.

The [research validation](M8_CANADIAN_RESEARCH_VALIDATION.md) reproduces all
21 Appendix cells (20 agree, one printed error), certifies the seven monetary
rows by exact rational root enclosures, and records446,400 direct clamped
anchor/frequency cases (clamped duplicates possible) plus1,222 daily coordinates. Its reviewed research tool passes14
integrity/reproduction controls, including rejection of differing CRLF bytes.
These are source-research checks, not Canadian Rust tests or coverage.

The complete [local design proposal](M8_CANADIAN_LOCAL_DESIGN.md),
[numeric/resource and72-family test proposal](M8_CANADIAN_NUMERIC_AND_TEST_DESIGN.md)
and [independent numerical review](M8_CANADIAN_NUMERICAL_DESIGN_REVIEW.md)
retain all three frequencies, regular/odd first/odd
last and composite periods, both yield models, all price bases, six risk
views and four money operations. Source conflicts and broader odd-period
generalizations must remain explicit local choices. No Canadian API,
contract, requirement or production oracle is adopted at this checkpoint;
The [API closure](M8_CANADIAN_API_REVIEW.md),
[conditional native profile](M8_CANADIAN_COMPOUND_PROFILE_REVIEW.md),
[bounded guard](M8_CANADIAN_COMPOUND_GUARD_DESIGN.md),
[MM staged range proof](M8_CANADIAN_MM_RANGE_REVIEW.md) and
[numeric resource route](M8_CANADIAN_NUMERIC_RESOURCE_REVIEW.md) now supply
reviewed inputs, including actual observed/ideal MM image differences. Complete
cash-resource/integration review, contract adoption and independent pre-production
expectations remain necessary before implementation. Current126 public
types/199 domain errors and every prior frozen contract/fixture are unchanged.


## Adopted complete Canadian local contract (2026-10-23)

The complete [Canadian contract](M8_CANADIAN_CONTRACT.md) is now adopted for
local engineering after the [whole-contract adoption review](M8_CANADIAN_CONTRACT_REVIEW.md).
It binds fourteen frozen components and FR-CAD-01..72 together: all f2/4/12,
regular/short/long/composite periods, compound/MM/explicit last-coupon strategy,
three price bases, bounded inversion, six independent risk getters and four
exact-money recipes. Source conflicts and local generalizations remain explicit.
This supersedes the earlier research checkpoint's unadopted status without
rewriting its historical proposals/review evidence.

The immutable contract and its complete [preproduction protocol](M8_CANADIAN_PREPRODUCTION_PROTOCOL.md)
now govern independent source/date/cash, represented numerical-stage, certified
ideal price/six-risk and target-driven inverse producers plus blind hands.
All must be independently reviewed, reproduced, parsed with the unchanged
shared reader, attested and frozen before any Canadian production. Native
primitive allowances, exact seeded stages, public classification stability,
inverse basis/conditioning and informative ordinary cases retain their
explicit qualifications. No Canadian fixture or implementation is accepted
at this adoption checkpoint. Actual implemented catalogs remain126types/199
errors; actual layout/lifetime/allocation and runtime/coverage/package checks
follow implementation. Financial/owner, foreign-native-runtime, maintained
calendar/legal, analytical-enclosure, release and wider M8 gates remain open.

The [blind numerical hands](M8_CANADIAN_NUMERIC_HAND_DESIGN.md) now freeze
eight reviewed future test functions and14 arithmetic groups before
production. Their199 exact rounding certificates reproduce and passed a
separate author's mathematical/API review. Rustfmt parsed the body; no
Canadian Rust compilation/runtime is claimed. Separate reviewed
[source/cash hands](M8_CANADIAN_SOURCE_CASH_HAND_DESIGN.md) now freeze14 future
test functions with154 exact arithmetic records and7,308 signed quantizer
checks. Their proof reproduces and Rustfmt parses the unchanged reviewed body;
no Canadian compilation/runtime is claimed for those hands either. The four
corpora have now passed the complete
[preproduction artifact gate](M8_CANADIAN_PREPRODUCTION_VALIDATION.md) and are
frozen: actual producer reproduction,115 input guards and unchanged reader
acceptance of81 files/3,159,951 rows. Canadian implementation and semantic
runtime acceptance remain ahead. Separately
reviewed [API/metadata hands](M8_CANADIAN_API_HAND_DESIGN.md) freeze seven future
functions and92 negative trait obligations, with syntax-only verification.
The complete [72-family binding ledger](M8_CANADIAN_TEST_BINDINGS.md) identifies
each independent reference route and the actual implementation checks still due.

## Complete Canadian local implementation (2026-10-24)

The adopted Canadian scope is now implemented locally in `bonds::canadian`
and verified against the frozen references without editing any of them
([validation](M8_CANADIAN_VALIDATION.md)). Eighteen public types and38 domain
errors bring the catalogs to144/237. The S/P/Y corpora run through public
consumers, all3,546 numerical rows replay through the actual private stages,
the29 frozen hands pass and all92 negative API probes fail with the intended
diagnostic. Every one of22 declared semantic mutants is killed by a runtime
assertion; one survivor of the first catalogue run led to a postproduction
residual-scale boundary control rather than to an edited expectation.
Financial/owner, foreign-native-runtime, maintained-calendar/legal,
analytical-enclosure, release and wider M8 gates remain open.

## Complete-model enclosure research proposal (2026-10-26)

The Canadian resource/solver/coverage followup is committed locally as
`ae7bda7`, with980 debug and979 release tests and corrected coverage
([record](M8_CANADIAN_RESOURCE_FOLLOWUP.md)). The next engineering obligation
has a reviewed, unadopted [complete-model design](M8_PRICE_ENCLOSURE_DESIGN.md),
[API proposal](M8_PRICE_ENCLOSURE_API_PROPOSAL.md),
[resource route](M8_PRICE_ENCLOSURE_RESOURCE_PROPOSAL.md) and
[research validation](M8_PRICE_ENCLOSURE_RESEARCH_VALIDATION.md).

The scope retains all32 generic compounding configurations and complete
Gilt/Treasury/Canadian models, exact named source roles, signed cancellation,
explicit budgets and declared settlement-evidence provenance. The research
CLI reproduces148 ordinary price-cell controls. Its Fraction storage,
borrowed Canadian metadata and Gilt/Treasury coordinate examples retain
their explicit limits.

The proposed26,264-byte logical payload and53,680 minimum work budget are
unadopted design values. Actual bounded scalar code, typed layouts,
borrows/copies, initialization/return paths and work charges still need
verification, followed by a complete contract and independent frozen
preproduction before new production. Public APIs and accepted contracts
remain unchanged; formal M8 and external acceptance gates remain open.

## Bounded scalar research checkpoint (2026-10-26)

The separate [research crate](../validation/research/enclosure_scalar/README.md)
now implements normalization, signed Mul/Div/coarse Add and finite IEEE export
with fixed Raw34/W52/M16/E33 storage. Independent preimplementation references
pass11,727 inputs per native debug/release mode;9 separate defense/work controls
pass, and all8 compiled arithmetic mutations are detected
([validation and limits](M8_PRICE_ENCLOSURE_PRIMITIVE_VALIDATION.md)).

These helper inputs are not full financial scenarios or additional library tests.
Actual scalar layouts/borrowed helpers and a single-frame caller are inspected;
whole-arena placement, complete log/exp/source/price composition, adopted API/error/
resource contracts and independent dated preproduction still precede a full
four-model producer. The26,264-byte/53,680-work proposal remains unadopted, and
old runtime APIs, financial behavior and frozen references remain unchanged.


## Enclosure contract policy adopted (2026-10-28)

The complete four-model [contract](M8_PRICE_ENCLOSURE_CONTRACT.md) now fixes the
exact targets, admission, API, encoding/proof, work and error/progress policy.
The [adoption review](M8_PRICE_ENCLOSURE_CONTRACT_REVIEW.md) records the reviewed
choices and isolated compile/runtime metadata evidence; the
[finite matrix](M8_PRICE_ENCLOSURE_TEST_MATRIX.md) preserves all 72 test obligations.
The tested configuration/evidence/error API foundation is installed and E1 closes.
No public financial producer exists yet. Exact case and consumer bindings and
the finite B1–B5 preproduction work keep E2 open.
The actual producers, originating cash link, whole-graph verification and separate
acceptance gates remain required by the [completion checklist](M8_COMPLETION_CHECKLIST.md).

## Enclosure preproduction complete (2026-10-29)

The finite [B1–B5 preproduction](M8_PRICE_ENCLOSURE_PREPRODUCTION.md) is complete
for the local E2 unit: unchanged independent S/P/Q/H expectations, one minimal
boundary supplement, all 72-to-18 consumer bindings, symbolic budget/error/proof
protocol and strict containment/usefulness policy are frozen in portable inputs
and FS1 fixtures. The B2 generator and separately expressed checker share one
agent author; no two-author or producer-author-blindness claim is made.

Actual public boundary-source and seven-table fixture-integrity tests pass
debug/release; focused strict lint and format checks pass. These tests construct
sources and validate input integrity, without a public financial producer.
E1 and E2 engineering closure does not imply M8 acceptance. The four producers
(E3), actual originating cash composition (E4), and whole-producer resources and
integrated verification (E5) remain. The owner scope is to finish those exits
and stop; this checkpoint opens no M9 work or new research family.
