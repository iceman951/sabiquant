# Complete Canadian fixed-coupon local contract

Adopted2026-10-23 Bangkok for local engineering under the owner's continuing
authority, after complementary complete-model reviews and independent final
candidate verification. The [adoption review](M8_CANADIAN_CONTRACT_REVIEW.md)
records the exact reviewed candidate and this status-only promotion. This
contract covers the entire chosen source/date/API/price/yield/six-risk/four-cash
model together. It changes no existing generic, gilt or Treasury contract.

All normative definitions below and their exact incorporated component bytes
are frozen. Earlier proposal/status wording in those immutable components is
historical; the precedence below selects the complete local definitions.
Independent preproduction expectations and blind hands must be authored,
reviewed and frozen from this contract before Canadian production begins.
Adoption adds requirements; it does not add implemented public types/errors
or establish current-market, financial/owner or foreign-runtime acceptance.

Read the incorporated [complete signatures/source/error definitions](M8_CANADIAN_LOCAL_DESIGN.md),
[numeric/test detail](M8_CANADIAN_NUMERIC_AND_TEST_DESIGN.md),
[bounded guard](M8_CANADIAN_COMPOUND_GUARD_DESIGN.md),
[numerical resource route](M8_CANADIAN_NUMERIC_RESOURCE_REVIEW.md),
[cash resource route](M8_CANADIAN_CASH_RESOURCE_REVIEW.md) and
[complete preproduction protocol](M8_CANADIAN_PREPRODUCTION_PROTOCOL.md)
with this envelope. All18types/6metadata-enums/33+5errors and FR-CAD-01..72
are one adopted scope; their required runtime evidence remains future work.

## C0. Incorporated definitions, precedence and exact identity

This envelope incorporates the exact byte-pinned component sections below. All public
signatures, structured fields, source formulas, stages and algorithm restrictions in
the normative components form one contract. A hash mismatch is a different input and
requires review; a moving filename alone is insufficient. No fixture or build may
depend on a private PDF/image/report path. The required literal constants appear here.

| Component | Bound sections/role | SHA256 |
|---|---|---|
| M8_CANADIAN_API_REVIEW.md | Informative source/research/review provenance | f68dca447eb12a0ec2b5dc945c6b27c672a9cf2cc4ce3c204ea2b149a12ca5a5 |
| M8_CANADIAN_CASH_RESOURCE_REVIEW.md | A5: complete source/view/cash widths and call-chain schedule | ee4036fed5561950108cc55e9bd4c149ad47e1728ff8f836aeab00c8e17e6201 |
| M8_CANADIAN_COMPOUND_GUARD_DESIGN.md | A3: exact closed-box guard, constants, typed mapping and liveness | eda561678408453539a246f79a9f47c84c7c3253ce1470fd201b011a63f5e2eb |
| M8_CANADIAN_COMPOUND_PROFILE_REVIEW.md | Informative source/research/review provenance | 3c36b9c4e6100739c584a9b46db9f7b7247031748dd22eb834b5acc1a2af5b3c |
| M8_CANADIAN_CONTRACT_INTEGRATION_REVIEW.md | Informative source/research/review provenance | 299ab65b1a2331e7c9b575b348bb3a4b97fd2f7b22bd291f4d83c892f97414c5 |
| M8_CANADIAN_LOCAL_DESIGN.md | A1: complete source/date/API/cash/error definitions, §§2–8 | bb0e1e65d2e59fc8ab320821fe1e313374bf2ecb40b78d55c303761ae4cff9fc |
| M8_CANADIAN_MM_RANGE_REVIEW.md | Informative source/research/review provenance | 93a08ffd40e67c5daf64c6a061a9edb3320cdb553c6cdf50aeef4700f45a27a2 |
| M8_CANADIAN_NUMERICAL_DESIGN_REVIEW.md | Informative source/research/review provenance | 61ea7f10f2ae183cd7a69ee3784c1e15637844ac02959112657b4deff73fcb27 |
| M8_CANADIAN_NUMERIC_AND_TEST_DESIGN.md | A2: N1–N6/N8 observations/resources/full semantic obligations | a7dc604450a390770aca267715a43a23fe12a0ae7b1bcf1ca0531388f33953dd |
| M8_CANADIAN_NUMERIC_RESOURCE_REVIEW.md | A4: selected exact numeric routes, correction addenda and constructive schedule | 2a06fe6041311a4d4e55d5be8e3fefce6973015713be78ffce31a1c1d1f63beb |
| M8_CANADIAN_PREPRODUCTION_PROTOCOL.md | A6: complete independent evidence and acceptance protocol | 3189e114460a34b09986db8853500929003b8fc7f3d7a2ac3645e531c506340c |
| M8_CANADIAN_RESEARCH_VALIDATION.md | Informative source/research/review provenance | ad26673245b4c27e3c0684c1649f219f38d5ad33aa881f6d405aa4798ce7ca14 |
| M8_CANADIAN_RESOURCE_INTEGRATION_REVIEW.md | Informative source/research/review provenance | 33cc02ba08bac46cd022b80df0721e72beb9c1b21a069d260f128f7e4854e9cb |
| M8_CANADIAN_SOURCE_REVIEW.md | Informative source/research/review provenance | dcc9efe40595bf09a4e107c454c9d8cdc875efff3557dcf10f729864726fff2b |

Normative detail: LOCAL_DESIGN §§2–8; NUMERIC_AND_TEST_DESIGN N1–N6/N8; the exact
guard mechanism/typed mapping in COMPOUND_GUARD_DESIGN; the selected arithmetic and
constructive lifetime route in NUMERIC_RESOURCE_REVIEW; the complete cash route in
the final cash-resource component; and the entire independent PREPRODUCTION_PROTOCOL.
Other linked source/review documents provide derivation, evidence and source-deviation
provenance. Historical numerical tables remain their stated ideal/observed type.

Precedence is this envelope, then the final cash/numeric/guard mechanism and explicit
correction addenda, then the main and numeric detail, then informative historical
reviews. In the incorporated normative sections, proposal/recommend/candidate/future
phrasing becomes the selected local definition upon adoption; it does not leave a
menu of implementation choices. Earlier status prose saying a now-closed proof must
still be supplied does not reopen that proof. Research test counts never become runtime
acceptance. The historical incorrect strict comparison lambda>1/184 is replaced by
**lambda>=1/184**, with dated equality admitted and the later bound lambda>2^-8 intact.

CanadianSourcePolicy::CanonicalIiac13LocalV1 has exact source_version() literal
`CA-IIAC13-fixed-2026-10-23-local-v1`. That date identifies this local engineering
version, not IIAC publication/effectiveness/latest guidance. Every instrument, plan,
valuation, risk, yield report and cash instruction uses that same mapping. Another
interpretation requires another policy variant/version, even if some results coincide.
No parser, identifier registry, default or implicit source-policy inference is added.

## C1. Complete supported model and source decisions

The immutable named instrument is CanadianBond: exact nonnegative annual coupon C per
100 par, fixed terminal principal100, original accrual D, issue I, stated M and
explicit true terminal payment T>=M. Annual yield y is decimal: C5 is5 per100/year,
y.05 is5% annually. Exact strictly positive cash nominal N is separate; no binary face
parameter or implicit percentage/currency conversion exists. All DecimalAmount C
coefficients/scales0..38 remain admitted, including C/f requiring scale39/40 or a
nonterminating rational. Public decimal source-view representability cannot gate a
representable fused cash operation or numerical evaluation.

All frequencies2/4/12, regular periods, short/long first and last, combined ends,
entirely off-cycle single intervals and arbitrary full virtual periods within the date
cap are included. Long last and complete composite composition are explicit local
extensions. Compound, explicit MM at any admitted tenor and explicit last-coupon-MM
selection are all supported; compounding frequency equals coupon frequency. No hidden
yield routing/default is permitted. Amortizers, floaters, indexed/optional instruments
and FX are separate models.

The adopted local reconciliation is wholly determined:

| Source issue | Selected local rule |
|---|---|
| Valuation coupon versus payable coupon | Keep separate exact source amounts; compound coefficients use valuation, MM/coupon-cash use payable. |
| Valuation A versus settlement AI | Keep separate caches. Compound clean subtracts A; MM clean subtracts AI; exact cash uses AI. |
| Monthly settlement conflict | AI=A for f12, preferring repeated general sections; p42 fixed365 departure remains explicit. Odd monthly payable still uses the payable rule. |
| Completed virtual periods | Exactly C/f; partial branches apply only to proper fragments. |
| Literal interior late branch | Preserve the184-day C5 elapsed182→183 drop of1/146. No cap/smoothing/fixed365 replacement. |
| Long-first DIC1 ambiguity | Use interval/quasi intersections, counting full pieces separately; missing virtual dates are not actual payments. |
| MM terminal double count | Every actual coupon once, terminal coupon plus par once. |
| Printed p16/p17/Appendix errors | Preserve the source observations/classifications; compute local expectations from the selected formulas. |
| Bond versus MM security rounding | An MM-quoted bond stays this bond; no3dp price prequantization. |
| Money modes/aggregation/ties | Explicit caller quantum, all seven signed modes, both aggregations; no source-implied default. |
| True maturity | Supplied T is terminal payment evidence. It changes MM horizons only, adds no coupon accrual/delay interest/calendar certification. |

The four primary PDF hashes and historical source scopes in SOURCE_REVIEW remain
provenance. Selecting this explicit engineering interpretation does not assert that
every contradiction is resolved as a matter of market practice or law. No additional
owner-approval requirement is inferred for already authorized local work.

## C2. Dates, complete schedule, ownership and geometry

The exact constructor/validation order and error fields are incorporated from main
§§3/6/8. Q is supplied, strictly increasing,2..20,000 dates. The last date is its direct
anchor; every earlier Q is one direct anchor-minus-multiple-of12/f-months result under
the supplied EOM policy, never a recursively rolled successor. Minimal coverage is
Q0<=D<Q1 and Qpenultimate<M<=Qlast. Actual dates are nonempty, increasing, first>D,
last=M; nonterminal dates lie on Q and every interior actual interval joins adjacent Q.
Only first/last can skip virtual dates. Zero coupons remain actual dates.

Adjacent-Q full intervals are Regular. Both-off-Q single intervals are Composite,
including exact quasi length1; other nonregular intervals are Short/Long according to
exact length below/above1. First and last refer to the same interval for one coupon;
coupon/principal is not duplicated. D<=I<F; T>=M. Prepared I<=S<M remains explicit even
if T>M. Nonterminal payments use contractual dates; terminal uses T. No legal calendar,
lag, record/ex-coupon rule or post-M trading window is inferred.

coupon_entitlement(index) takes an ORIGINAL actual ordinal in the full instrument,
including ceased coupons. It returns None only out of that full count, Ceased when
date<=S, BuyerEntitled when date>S; ExCouponDetached is not returned. In contrast,
bundle_coupon_index(index) takes a compressed future position and returns the original
ordinal. All per-flow numerical errors retain that original ordinal. Q-cycle Date
errors alone use original Q ordinals and phase `quasi cycle`.

Exact chi differences use full cells plus at most two proper edge fractions, with a
single fragment if both interval endpoints lie in one Q cell. Per-frequency actual
quasi day ranges are f2:181..184,f4:89..92,f12:28..31. The global minimum future
lambda is1/184 inclusive; lambda<20000. Broad cross-frequency storage supersets are
not automatically dated-reachable inputs.

Every common report/cash snapshot retains L=last_coupon_start_date, including dirty-only
and first-coupon cash. Anchor/f/EOM plus minimal D/M fixes Q; F=M forces sole coupon[M]
and L=D; otherwise actual dates are the contiguous Q slice F..L followed by M. Thus
D,I,M,T,C,F,L,anchor,f,EOM,policy reconstruct the full financial definition. Caps and
caller spare capacity are resource properties, not financial Eq/Hash identity.

## C3. Exact source amounts and observations

For an interval let k be complete cells and (d_j,s_j) its proper fragments. Define

`V=(C/f)*(k+sum d_j/s_j)`.

Each payable complete cell is C/f. A proper fragment is C*d/365 if f*d<365 or
C*(1/f-(s-d)/365) if f*d>365; zero contributes0. Equality is unreachable for integer
d and f2/4/12. A same-cell composite applies this once to its actual fragment. Never
apply a proper-fragment formula at d=s. Canonical exact zero is0/1.

At S, start accrued at latest actual date<=S or D. A=V(start,S). AI is the payable
piece sum for f2/4, A for f12. Actual coupon dates reset A/AI; skipped virtual dates do
not reset. No daily9/accrued5/coupon6dp or other publication quantizer exists. Cache
exact valuation/payable first/last and regular C/f. Independently observe Ahat/AIhat by
one IEEE nearest/even conversion; nonzero values are finite normal throughout C's full
domain. Optional q/m source views do not participate in these caches.

Source rationals use at most five64-bit words EACH numerator/denominator, with the
reviewed numerator<2^160, denominator<2^148 forms including exact terminal par addition.
No unchecked uncancelled alternate expression may silently replace the reviewed form.
Every stored/copied exact source ratio b/d is canonical: nonzero b,d are positive and
coprime after division by gcd(b,d); zero is exactly0/1. This includes valuation/payable
coupons, A/AI caches, selected bundle coefficients including par, and instruction source
values. Equal source values therefore have one exact-ratio representation for Eq/Hash.
Do not erase their separately retained source/date/fragment provenance. Canonicalizing
can only decrease magnitude widths. For the actual numerator<2^160,denominator<2^148
source forms, Euclid starts with(b,d); its first step leaves(d,b mod d), with the first
operand EQUAL to the prior denominator and the second strictly below it. Both are below
2^148. Each two further steps at least halve the active maximum, giving at most297
remainders and at most161-bit scratch. Separately, a generic five-word primitive can
order positive magnitudes first and use the conservative640-remainder cap; its broader
320-bit primitive controls are not source-form attainments. Both routes use fixed
five-word storage and no heap/retry. Cash factories
copy the canonical cache and do not rerun source construction; their zero-allocation
rule remains unchanged. Arbitrary transient numeric/cash quotient forms need not
gcd-reduce merely to satisfy this stored-source identity rule.

## C4. Prices and six independent risk views

For compound B_i is exact valuation coupon plus100 at terminal,
lambda_i=chi(contractual_coupon_i)-chi(S), and ideal
D=sum B_i*(1+y/f)^(-lambda_i), y>-f; clean=D-A; settlement dirty=D-A+AI.
Ideal raw derivatives/moment are -sum W_i*lambda_i/(f+y),
sum W_i*lambda_i*(lambda_i+1)/(f+y)^2 and sum W_i*lambda_i/f.

For MM let tau=(T-S)/365, rho_i=(T-payment_i)/365, h_i=tau-rho_i>0;
B_i is payable plus100 terminal. Exact d=1+y*tau>0, n_i=1+y*rho_i>0,
D=sum B_i*n_i/d, clean=D-AI and settlement dirty=D. Ideal raw derivatives are
-H/d^2 and2*tau*H/d^3 with H=sum B_i*h_i>0; local actual-year moment is
sum B_i*n_i/d*h_i. A familiar periodic duration identity cannot replace that clock.

The three typed bases are Clean, ModelDirty, SettlementDirty; CanadianPrice admits all
finite signed values, canonicalizes zero, rejects NaN/inf, and carries its basis. No
implicit BondPrice conversion exists. Model-dirty remains positive; clean/all-in may
be signed. Numerical reports observe clean with one exact_sum[Dhat,-quote_accrued]
and settlement with one exact_sum[Dhat,-quote_accrued,AIhat], not two rounds. MM returns
Dhat directly as settlement dirty. quote_accrued is Ahat compound, AIhat MM.

Risk terms use the SAME accepted IEEE per-flow PV seeds. Compound uses exact rational
lambda/lambda+1 and deltahat=RN(exact(f)+exact(y)); MM uses exact affines d/n/h/tau.
Each full rational term rounds once to normalized53 bits with a separate exponent,
then exact accumulation on2^-8192 retains three immutable signed sums. No early IEEE
range gate, source-H substitution, rounded horizon or rounded raw getter participates.
Raw getters round retained sums once; Modified/Macaulay/Convexity divide retained sums
directly by positive Dhat then round once. Convexity has no half. No getter's range
failure gates another. Exact zero is+0; nonzero-to-signed0/inf is a typed named Value
failure where reachable.

## C5. Prescribed numerical execution and closed guard

N2's ordered scaled-log/factor/source-PV observations, N3's exact selected terms and
the incorporated bounded guard/resource algorithms are required, not interchangeable
routes selected from passing tests. y0 has g0; abs(y)<2^-50 has g=y. Near floor use
exact(f+y)/f then its RN and ln; otherwise RN(y/f) and ln_1p; RN final f product.
For each source z=RN(-exact(g)*lambda/f) once; z0 gives discount1 without exp. Tiny
analytic remainder and exponent RN distance remain in accuracy comparisons even when
the discount is the identity. Validate factor before source PV, including zero coupons.
Observe exact source times represented factor DIRECTLY once to IEEE, then exact_sum
all accepted PVs once. Extended53-then-IEEE double rounding is forbidden.

Here ln, ln_1p and exp mean the ACTUAL native primitive observations. In the ordinary
branches ell_hat=actual_native_ln(bhat) or actual_native_ln_1p(xhat), followed by
g=RN_even(f*ell_hat); a nonzero unguarded exponent uses actual_native_exp(z). They do
not denote correctly-rounded ideal transcendental values. Their exact argument/product
stages and validation order are fixed, while legitimate native result bits may vary
across platforms within the separately declared conditional profile. This contract
does not silently promise correctly-rounded libm or require macOS primitive bits on
other platforms.

The compound profile is explicitly conditional on stated primitive ln/ln1p2u and
positive-finite exp budget2u*exp(z)+2^-1075, u2^-53, with no flush-to-zero/reassociation.
It implies shared-log2u for f2/4,6u for f12, and non-tiny radius
R=(beta*abs(z)+2^-1075)/(1-beta), beta4u/8u. Finite native measurements do not prove
universal or foreign libm accuracy. Actual zero/inf/NaN validation remains mandatory.

For non-tiny0<abs(z)<2^24 construct the signed endpoints and comparands exactly with
GuardWide20 as incorporated. Use E2^1075,A=abs(z)*E,B=2^53-k,C=A*B,N=k*A+2^53,k4/8;
endpoints are C-N,C+N for positive z and -(C+N),N-C for negative. Compare against
T*B*2^819 for each signed threshold numerator T. Closed intersection, including
touching, with either threshold box gives RateError::NumericalBoundary before exp.
Disjoint intervals, including certain zero/overflow intervals, call exp once then
validate the actual outcome. Tiny/far finite branches bypass wide construction by
their proved disjointness; no inward radius, threshold midpoint, arbitrary retry or
unbounded integer is permitted. The capacity operation is `guard discount range`.

The exact certified boxes have common denominator2^256 and signed decimal numerators:

| Endpoint | Signed numerator |
|---|---|
| Z0 lower | -86280532199940657026901447021783325824039967682459192601582082568326453749384799 |
| Z0 upper | -86280532199940657026901447021783325824039967682459192601582082568326453749384798 |
| Zinf lower | 82187223230455100268499765730225550946232633676658463179948381980536898214648393 |
| Zinf upper | 82187223230455100268499765730225550946232633676658463179948381980536898214648394 |

Five u64 magnitude limbs, LITTLE-ENDIAN, are respectively:

```text
Z0lower: [b4f9156b7ef8865f,bd82fa733745013a,c72b27589541e1ed,221aa5a60a3bec60,00000000000002e9] negative
Z0upper: [b4f9156b7ef8865e,bd82fa733745013a,c72b27589541e1ed,221aa5a60a3bec60,00000000000002e9] negative
Zinflwr: [df08189956935a49,cd0c99ca4d8360d2,8ece600fcbd2bd03,c85fdf473de6ab27,00000000000002c5] positive
Zinfupr: [df08189956935a4a,cd0c99ca4d8360d2,8ece600fcbd2bd03,c85fdf473de6ab27,00000000000002c5] positive
```

These are source literals, not runtime text parsing. Horner reconstruction traverses
REVERSED limb order for exactly five steps; forward traversal is a different integer.
The linked exact atanh/geometric-tail certificate, not Decimal agreement, proves the
boxes enclose -1075ln2 and ln(2^1024-2^970).

MM uses no elementary functions: E2^1074,Y=exact(y)*E and integer day counts build
d_raw=365E+YD and n_raw=365E+YR. Exact denominator positivity is the domain check;
no rounded floor or y*tau admission is used. The factor is one direct ratio RN,
including permitted nonzero subnormals. Do not apply the compound y>-f gate to MM.

## C6. Inversion and range/error classification

Resolve ModelDirty directly, Clean by one exact_sum[value,quote_accrued],
SettlementDirty by one exact_sum[value,-AIhat,quote_accrued]. Reject only a nonpositive
RESOLVED model-dirty target at zero evaluations, not a signed external scalar. Target
shift precedes allocation/trials. Keep original typed target and represented shifts.
Residual s0 when abs(Tdirty)<2^970 else1; each callback returns one RN of exact
(Dhat-Tdirty)*2^-s and cached Dhat. amount_residual uses the unscaled cached difference;
positive finite Dhat/Tdirty make overflow and nonzero-to-zero impossible. Retain a
defensive Result without inventing a financial witness.

Use unchanged fallible shared Brent, caller bracket/options, lower-first, exact caps,
failed invocation counts, derivative count0, cached selected payload and no final
callback. A lower-zero success does not evaluate an invalid unqueried upper quote.
One reserved PV workspace is reused; callbacks allocate zero and do no date/source/cash
work. Ideal strict decrease is a mathematical sensitivity classification, not observed
monotonicity, ideal existence, coordinate accuracy or a financial enclosure.

No ideal-MM image-floor precheck may reject an observed target. The reviewed two-flow
MAX witness with target bits3fd5375375375375 lies below ideal floor181/546 and is a
valid observed protocol control: Explicit MM, ModelDirty, bracket[0,MAX],
SolverOptions(1e-12,0,2), cached upper FunctionZero after2 function/0 derivative calls.
It receives no ideal-root accuracy assertion. Uninformative conditioning remains
explicit, not an expanded tolerance chosen from production mismatches.

Under the complete date/source coupling, MM factors/PVs/dirty cannot zero/overflow;
nonzero subnormals are admitted. No MM risk getter can overflow; raw moment and
Macaulay stay normal finite. Raw derivatives and Modified/Convexity underflow remain
real. The observed Macaulay lower bound is (1-u)^2/((1+u)*365)>1/730, not ideal1/365.
For both conventions clean/settlement bridges and typed target shifts cannot overflow
or turn a nonzero sum to zero under their one-exact-sum staging. Defensive checks stay,
but those impossible dated failures are handled by proof/primitive coverage. Compound
interior-callback exclusions from Treasury are not inherited without proof.

## C7. Four exact cash recipes, staging and provenance

For the instruction's seven-mode signed quantizer R, exact payable U, exact AI, positive
nominal N, clean evidence X and all-in settlement evidence Z:

| Factory / exact recipe_identifier() | Formula |
|---|---|
| coupon_cash / PayableCouponPer100 | R(N*U/100), excluding principal |
| accrued_cash / SettlementAccruedPer100 | R(N*AI/100) |
| dirty_cash / SettlementDirtyPricePer100 | R(N*Z/100) |
| clean_accrued_cash / CleanPriceAndSettlementAccruedPer100 | RoundTotal:R(N*(X+AI)/100); components:exact signed sum at q scale of R(N*X/100) and R(N*AI/100) |

These identifiers have no implicit prefix/suffix. Dirty means ALL-IN settlement
evidence, never silently compound model dirty. Evidence retains exact-decimal,
exact-supplied-binary64 or continuous-declared-interval meaning. No price/display/AI/
coupon intermediate quantizer is inserted. Positive N is required even for zero cash.
Signed trade cash reverses direction; nominal sign does not encode direction.

Coupon factory validates original ordinal then N; other factories N first. Capture
cached exact source and immutable metadata with no allocation/calendar/date arithmetic.
Coupon stage is `cash coupon`; AI `cash accrued`; dirty/total invoice `cash total`.
Components complete `cash capital` Exact before `cash accrued`, then exact `cash net`
without another quantizer. Retain intermediates wide; only final amounts are admitted.
Net cancellation cannot waive an earlier Exact failure.
The component net adds the two signed wide integer coefficients at the SAME quantum
scale, then normalizes and admits the final amount. It never calls the binary64 exact_sum
function used by numerical price/target reductions, and forms no component LCM.

A point Exact stage uses divisibility and can fail Inexact. A nonpoint continuous
Exact stage returns ExactStageUnresolved even when both endpoints are multiples.
Equal declared bounds are points retaining declared provenance. Non-Exact stages are
monotone, allowing endpoint image propagation. Final endpoint labels reflect THAT
stage image, including component-to-net point collapse. Admit lower then upper for a
nonpoint final image. Equal extrema give Determined; unequal QuantumAmbiguous. Never
clip, suggest midpoint or infer interior representability across decimal range holes.

Coupon snapshots select exact U and full original coupon/geometry/payment metadata;
AI/invoice snapshots select exact AI and interval[actual accrual start,S], plus NEXT
actual coupon/date/payment. At [S,S] retain right Q pair twice, full0/no fragments.
C0 preserves nonempty geometry. Dirty-only has S/selection/convention/common identity
but no selected source coupon/geometry/value. None means genuine absence, not fake0.

source_value_per_100(q,m) is exactly the payable_coupon_per_100(i,q,m) view wrapped
Some for coupon, or settlement_accrued_at_quantum(q,m) wrapped Some for accrued/invoice,
including quantization errors. Stages are `source payable coupon` with capacity index
Some(original i), and `source settlement accrued` with None; endpoint `point`,
AmountError operation `quantize`. Dirty-only returns Ok(None) without quantizing.
q/m is the independent view policy and is not retained invoice rounding. Pure source
receipts have no price guarantee; price receipts label only the supplied ingredient
using existing SettlementGuarantee variants, including declared equal-bound points.

Instructions/receipts retain complete reconstructible instrument identity, recipe,
source ratio, selected geometry, nominal/rounding/evidence kind/bits/bounds/aggregation
and selection where applicable. Equality does not erase provenance when cash matches.
Determined retains its instruction via receipt. Nondetermined decisions have only their
listed diagnostics; callers retain the instruction. No old SettlementReceipt is forged.
A binary bridge's observed AIhat is not exact-AI money, so no cross-recipe fine-quantum
equality or ideal-price enclosure is promised.

## C8. Exact public surface and error transport

Exactly18 new public types are required:

`CanadianBond`, `CanadianCouponSchedule`, `CanadianFrequency`, `CanadianPeriodKind`, `CanadianSourcePolicy`, `CanadianYieldConvention`, `CanadianYieldSelection`, `CanadianPriceBasis`, `CanadianPrice`, `CanadianPricingPlan<'a>`, `CanadianValuation`, `CanadianRisk`, `CanadianYieldReport`, `CanadianError`, `CanadianYieldError`, `CanadianCashInstruction`, `CanadianCashDecision`, `CanadianCashReceipt`.

The six metadata enums are `CanadianFrequency`, `CanadianPeriodKind`, `CanadianSourcePolicy`, `CanadianYieldConvention`, `CanadianYieldSelection`, `CanadianPriceBasis`. Their variants, ALL arrays where specified,
every signature/inspector, privacy/Copy/Clone/Eq/Hash/Debug/Send/Sync obligations and
non-exhaustive contextual behavior are the main §6/§7 definitions. No field becomes
public, no Default/numeric ordering/parser/implicit float-money conversion is added.
Price Eq/Hash uses canonical finite bits+basis. Bond/Schedule are owned Eq+Hash;
Plan borrows Bond and owns bounded bundles; Risk owns retained arrays and compact
Debug; reports/cash snapshots have their specified ownership. Arbitrary Clone/format
allocation has ordinary Rust qualifications and is not part of fallible-entry promises.

CanadianError has exactly33 domain variants, kind prefix `canadian.`; CanadianYieldError
has five, prefix `canadian_yield.`. Full fields/suffixes/causes/operation precedence are
main §8, with `guard discount range` and C0–C7 clarifications applied:

| Variant and fields | suffix / cause |
|---|---|
| InvalidDateCap{cap,minimum,maximum:usize} | invalid_date_cap / none |
| TooManyQuasiDates{count,cap:usize} | too_many_quasi_dates / none |
| TooFewQuasiDates{count,minimum:usize} | too_few_quasi_dates / none |
| TooManyCoupons{count,cap:usize} | too_many_coupons / none |
| NoCoupons | no_coupons / none |
| AccrualNotBeforeMaturity{accrual_start,stated_maturity:Date} | accrual_not_before_maturity / none |
| QuasiDateOrder{index:usize,previous,date:Date} | quasi_date_order / none |
| QuasiCoverage{accrual_start,stated_maturity,first,next,penultimate,last:Date} | quasi_coverage / none |
| QuasiDateMismatch{index:usize,supplied,expected,anchor:Date,frequency:CanadianFrequency,end_of_month:EndOfMonth} | quasi_date_mismatch / none |
| CouponDateOrder{index:usize,previous,date:Date} | coupon_date_order / none |
| CouponBoundary{last_coupon,stated_maturity:Date} | coupon_boundary / none |
| CouponBeforeAccrual{index:usize,date,accrual_start:Date} | coupon_before_accrual / none; includes equality |
| CouponOffCycle{index:usize,date:Date} | coupon_off_cycle / none |
| IrregularInteriorPeriod{index:usize,previous,date:Date} | irregular_interior_period / none |
| Date{index:Option<usize>,phase:&'static str,cause:DateError} | date / DateError |
| NegativeCoupon{coupon_per_100:DecimalAmount} | negative_coupon / none |
| DatedAfterIssue{accrual_start,original_issue:Date} | dated_after_issue / none |
| IssueNotBeforeFirstCoupon{original_issue,first_coupon:Date} | issue_not_before_first_coupon / none |
| TrueMaturityBeforeStated{true_maturity,stated_maturity:Date} | true_maturity_before_stated / none |
| InvalidBundleCap{cap,maximum:usize} | invalid_bundle_cap / none |
| TooManyBundles{count,cap:usize} | too_many_bundles / none |
| SettlementBeforeIssue{settlement,original_issue:Date} | settlement_before_issue / none |
| SettlementAtOrAfterMaturity{settlement,stated_maturity:Date} | settlement_at_or_after_maturity / none |
| InvalidCouponIndex{index,count:usize} | invalid_coupon_index / none |
| MoneyMarketDomain{yield_rate:f64,days_to_true_maturity:u32} | money_market_domain / none |
| Value{index:Option<usize>,phase:&'static str,cause:ValueError} | value / ValueError |
| Rate{index:Option<usize>,phase:&'static str,cause:RateError} | rate / RateError |
| Sum{phase:&'static str,cause:SumError} | sum / SumError |
| AllocationFailed{phase:&'static str,requested:usize} | allocation_failed / none |
| InternalCapacity{index:Option<usize>,phase,operation:&'static str} | internal_capacity / none |
| ZeroDirtyPrice{measure:&'static str,yield_rate:f64} | zero_dirty_price / none |
| NonPositiveNominal{nominal:DecimalAmount} | nonpositive_nominal / none |
| CashArithmetic{stage,endpoint:&'static str,cause:AmountError} | cash_arithmetic / AmountError |

CanadianYieldError:

```text
Target{target:CanadianPrice,phase:&'static str,cause:CanadianError} #target
Preparation{phase:&'static str,cause:CanadianError}              #preparation
Solver(SolverError)                                             #solver
Callback{stage:EvaluationStage,point:f64,evaluations:EvaluationCounts,
  cause:CanadianError}                                          #callback
NonPositiveTarget{target:CanadianPrice,model_dirty_target_per_100:f64,
  quote_accrued_per_100:f64,settlement_accrued_per_100:f64,
  source_policy:CanadianSourcePolicy,convention:CanadianYieldConvention,
  evaluations:EvaluationCounts}                                 #nonpositive_target
```

Every typed cause is immediate StdError::source; other variants have None. Root
Error::Canadian/CanadianYield exposes the wrapped domain error as its immediate source,
then that domain error's listed cause, preserving all layers. From conversions and
kind/Display delegation are required. No string-erased cause, sentinel, panic or
recursive allocation. Wrappers are not counted again as domain variants. Cash cause
is AmountError::Inexact(operation quantize) or OutOfRange(operation quantize, except
final cash-net `sum`); ResultScale is excluded by supported quantum scales.

Schedule/Bond/Plan/evaluate/risk/yield/cash validation order is the fixed numbered main
§8 order. Finite quote/domain checks precede PV reservation; shared g occurs AFTER
that reservation; per-original-ordinal factor precedes source PV precedes requested
risk terms; exact dirty reduction precedes reports. Shared failures use indexNone;
per-source failures Some(original ordinal). Unused source views/clean bridges/risk
getters cannot leak failure into an independent valid operation. Stable quantity,
phase, stage, endpoint and operation literals are binding, not free-form messages.

For the six independent getters, the exact phase and ValueError quantity are the
following space-separated strings; “getter names” in the incorporated proposal means
this mapping, not an implementation-selected spelling:

| Getter | Exact phase / ValueError quantity |
|---|---|
| price_derivative | price derivative |
| price_second_derivative | price second derivative |
| time_weighted_amount | time weighted amount |
| modified_duration | modified duration |
| macaulay_duration | macaulay duration |
| convexity | convexity |

## C9. Bounded resources and implementation acceptance

Use std only, no unsafe/input panic/I/O/global mutable state/heap bignum/recursive
fallback/arbitrary wider retry. Five-word source pairs are two magnitudes, not one
40-byte ratio. Keep represented exponents as metadata; normalized division aligns the
ratio relatively and scans at most53 quotient bits with a u64 result, not exponent-sized
loops or extra wide quotient arrays. Direct IEEE subnormal branches use reviewed
range-conditioned shifts. Numerical route widths and scratch/lifetime construction
are binding via the final incorporated resource reviews.

That53-bit/u64-quotient rule applies ONLY to normalized numeric observations, not to
arbitrary-quantum cash/view division. A cash quantum count can be much wider than u64;
the cash route retains its full eight-/40-word quotient and counts quotient,
remainder, alignment, subtract and result-construction slots in its call-chain budget.
Do not truncate a cash count, substitute an approximate reciprocal or import the
normalized numerical scan limit into the cash quantizer.

MM affine40 words, selected rational risk112, retained/normalization243, residual34,
compound guard20 are per-magnitude widths. The constructive traversal uses24 borrowed/
mutable-output112-word slots; old exact_sum's TWO34-word arrays are explicitly counted
as two additional padded slots (26<=32). Risk holds at most3 working+3 transfer arrays;
getters borrow retained arrays and own at most3 scratch. The numerical wide-payload
ceiling is32*112*8+6*243*8=40336bytes; conservatively adding12guard20-word slots gives
42256bytes<65536. Do not allocate all32 slots and overlook callee arrays. Count explicit
by-value copies and return temporaries while callers remain live; no optimizer-last-use
credit. The final cash-resource route binds its complete eight-/40-word call-chain
schedule, signed rounding/normalization and snapshot bounds with the same discipline.

The cash route uses branch-specific price decoding: binary p/2^1074 or the decimal
coefficient/10^scale pair, combined with ONE cached AI pair per endpoint. An uncancelled
transplant of Treasury's larger common basis is not the approved40-word route.
Pure cash/view uses ten8-word buffers plus at most four80-byte complete source-pair
copies:960<=1024bytes. Price uses twenty-four40-word buffers plus that ten-buffer
source subcall and four pair copies:8640<=10240bytes. These counts include the wide
quotient, remainder, shifted divisor, complement, retained endpoints and normalization.
Cash quotient scans are bounded by414/2501 dividend bits and final decimal normalization
by38 passes. Available complete field budgets are1024bytes per instruction and1088
per receipt, within the2KiB constraint; they are constructive, not measured layouts.

The metric excludes caller-owned buffers/cache storage/spare Vec capacity, allocator
headers/granted excess, prepared/PV heap payload, unrelated retained clones, scalar
metadata/references and compiler spills/ABI overhead. It does not claim that memory
disappears or that compiler stack frames equal logical wide payload. Caps count lengths,
reserve_exact describes requested slots and moved vectors retain capacity. Final layout,
actual allocation attempts, call-chain copies/liveness and relevant compiler frames
require post-implementation audit before claiming conformance. Source factories/views/
cash evaluation allocate0; Plan reserves one bundle vector, each evaluation/risk one
PV vector, each solve one reused PV vector; callbacks/per-flow work allocate0.

Prepared bundles contain one selected source coefficient and original ordinal/exact
coordinate-or-day metadata, with logical size<=128bytes. At most20000 slots give
2560000 initialized payload bytes; PV slots at most160000bytes. Cash snapshots/receipts
are owned, vector-free and<=2KiB each. Source constructor/preparation traversal is
bounded linear in admitted Q+actual count. Unknown future Rust expressions are not
certified merely because their proposed capacities fit a table.

## C10. Full requirements and preproduction-before-production gate

The following72 functional requirements are adopted together only with the complete
contract. Each FR-CAD-NN retains its CADNN semantic-family mapping. They are obligations,
not72 existing tests, a fixture count or acceptance by shape labels. Proof-excluded
financial errors get honest proof/primitive coverage as C6 requires; no requirement
authorizes forging an impossible public report to obtain a runtime branch hit.

| Requirement | Semantic family | Complete required behavior/evidence |
|---|---|---|
|FR-CAD-01|CAD01|All4 primary PDF hashes/version/scope; as-of/metadata/legacy-redirect limits; no current-market certification|
|FR-CAD-02|CAD02|Literal p16 wrong result preserved, independent48/73 exact reference and rounding distinct|
|FR-CAD-03|CAD03|p17 extra full coupon preserved; corrected local short-last formula differs by C/f|
|FR-CAD-04|CAD04|Appendix83.298 divergence; other20 cells; no hidden fix to retained source|
|FR-CAD-05|CAD05|1.1/1.2/1.3 lateAI differences; text-order differences not falsely mathematical|
|FR-CAD-06|CAD06|Monthly ActAct exception vs p42 fixed365; policy identity and local decision explicit|
|FR-CAD-07|CAD07|MMp49 terminal once atNC1/NC2 and zero/nonzero yield; direct exact coefficients|
|FR-CAD-08|CAD08|LongfirstDIC1 stub versus full-first-payment definition; diagram-based concrete difference|
|FR-CAD-09|CAD09|Every frequency2/4/12 and annual coupon/yield unit conversion; unknown frequencies absent, no defaults|
|FR-CAD-10|CAD10|All metadata enums/type census/traits/no implicit float-money conversion/new roots and errorcatalog|
|FR-CAD-11|CAD11|Schedulecap0/1/2/20000/20001, Qcount and actualcount precedence, no allocation before structure|
|FR-CAD-12|CAD12|Minimal outer coverage, duplicate/decreasing Q, redundant prefix/suffix and Date edge failures|
|FR-CAD-13|CAD13|Direct anchor vs recursive clamp/EOM/Ignore, leap/nonleap/monthend across all frequencies|
|FR-CAD-14|CAD14|Actual coupon order/boundary/offcycle/interior adjacency and original index errors|
|FR-CAD-15|CAD15|Regular first/last with C0/Cpositive; zero coupon amounts remain actual coupon dates|
|FR-CAD-16|CAD16|Short first dates at each endpoint/day; value/payable/day-denominator distinctions|
|FR-CAD-17|CAD17|Long first one/several/thousands full periods; issue after skipped virtual boundary; no invented payments|
|FR-CAD-18|CAD18|Short final including source48/183day cases and forward full quasi denominator|
|FR-CAD-19|CAD19|Long final local extension with0/1/many interior actual coupons and explicit source-deviation label|
|FR-CAD-20|CAD20|Composite one-coupon both ends offcycle, same/different quasi periods, no duplicated coupon/principal|
|FR-CAD-21|CAD21|Shortfirst+shortlast, longfirst+shortlast, longfirst+longlast crossproducts and single shared period|
|FR-CAD-22|CAD22|D<=I<F, reopenings, equality/invalidorder precedence before trueT/cache work|
|FR-CAD-23|CAD23|T=M/T>M/T<M, no extra coupon accrual or inferred calendar, MM horizons change only|
|FR-CAD-24|CAD24|S<I,S=I,S=D,S on skippedQ/actualcoupon,S=M-1/M; visible post-M rejection even T>M|
|FR-CAD-25|CAD25|ActAct source value from independent interval intersections, full and proper fragments, no overlapping single-quasi pieces|
|FR-CAD-26|CAD26|Canadian365 proper branch and -1/146 drop, exact threshold unreachable for2/4/12|
|FR-CAD-27|CAD27|Canonical completed181/182period exactlyC/f; literal partial-at-full counterexample retained|
|FR-CAD-28|CAD28|Monthly payable oddCB vs monthly AI=ActAct; don't merge coupon and accrued policies|
|FR-CAD-29|CAD29|A/AI reset only actual coupons, persist at skippedQ, C0 and zero accrued geometry metadata|
|FR-CAD-30|CAD30|C MIN positive/MAX/scales0..38, C/f scale39/40/nonterminating, no premature public amount gate|
|FR-CAD-31|CAD31|Source q/m views all7modes, exact nonterminating failure, OutOfRange independent of final cash|
|FR-CAD-32|CAD32|Full source AI normal/finiteness at min/day/max/cap plus exactzero, independent numerator/denominator bounds|
|FR-CAD-33|CAD33|Fractional compound alpha1/2 vs Treasury rational prefix, exact squared-factor discriminator|
|FR-CAD-34|CAD34|General chi exponent exact2partial coordinate vs rounded-lambda/annualtime substitutions|
|FR-CAD-35|CAD35|Compoundfloors and neighbors allf; exact δ steps; f12 argument rounding; invalid upper unqueried|
|FR-CAD-36|CAD36|MIN_SUB/tiny threshold/zero exponent identities, non-tiny branches and primitivecallcounts|
|FR-CAD-37|CAD37|Compound ln/ln1p/exp certified reference and new per-frequency profile; informative boundary and rejected cases|
|FR-CAD-38|CAD38|MM exact domain neighbors/non-dyadic floor and sharedaffine positivity; terminalrho0; valid y=-300 one-day MM outside compounddomain|
|FR-CAD-39|CAD39|MM intermediate overflow recoverable by exact ratio; all source factors observed including zeros|
|FR-CAD-40|CAD40|Direct source×observedfactor IEEE subnormal/overflow ties and double-rounding mutant kills|
|FR-CAD-41|CAD41|Dirty reduction errors, original coupon indices and strict operation precedence|
|FR-CAD-42|CAD42|Three typed price bases and reports; compoundA vs MMAI; no automatic BondPrice::Dirty ambiguity|
|FR-CAD-43|CAD43|Direct3-term settlementbridge and target reverse vs roundedclean/add, including signed/zero all-in targets|
|FR-CAD-44|CAD44|Negative all-in valid witness, no positive-price gate on bridge/cash; independent modelpositive denominator|
|FR-CAD-45|CAD45|All6 ideal compound derivatives/moment via independent dual2/quasi controls, allf/allstub shapes|
|FR-CAD-46|CAD46|All6 MM ideal quantities via positiveaggregateH and independent per-flow derivatives; trueT changes|
|FR-CAD-47|CAD47|Selectedobservedrisk PV-seeded terms vs exactsourceH route; exact one53bit rounding and all held ratios|
|FR-CAD-48|CAD48|Raw±overflow/underflow/exactzero vs independently finite normalized views; no getter dependency|
|FR-CAD-49|CAD49|Fractional rawD2normal/Cvunderflow witness; no borrowed Treasury range exclusions|
|FR-CAD-50|CAD50|Normalized denominator D_hat, not clean/allin; MM moment convention, no periodic Macaulay shortcut|
|FR-CAD-51|CAD51|Extended accumulation243word bit binding, directnorm, signedzero errors, sourcezero without invalid normalization|
|FR-CAD-52|CAD52|Risk/publicreport inspectors/provenance/Debug no giantbuffers; independent clean/bridge getter failures|
|FR-CAD-53|CAD53|Yield all3basis target resolution including externalnegative/zero but positiveModelTarget; shift before allocation|
|FR-CAD-54|CAD54|NonpositiveModelTarget0calls, both residualscales threshold/equality, exact signs/cancellation/zeroiff equality|
|FR-CAD-55|CAD55|Brent lowerzero1call, upperzero, same sign, countcap1/2/3, cachedselectedpayload no finalcall|
|FR-CAD-56|CAD56|Actual financial/numerical callback failures if reachable; exact stagepointcountcause; synthetic transport separate|
|FR-CAD-57|CAD57|Negativevalidy/compoundfloor/MMdomain errors counted onlywhenqueried, original index and immediate causes|
|FR-CAD-58|CAD58|MM positive lowerimage and unattainablepositivetargets; illconditioning/flat roundedimages/precisionexhausted/brackettolerance; no fabricated root certification|
|FR-CAD-59|CAD59|Roundtrip targets at cached observations, exactcertificate root brackets with slope-qualified informative/uninformative coordinate radii|
|FR-CAD-60|CAD60|MM/compound/strategy identity and branchswitch boundaries around last actual coupon, no hidden mode default|
|FR-CAD-61|CAD61|All4 cash recipes actualsource quantities/principal excludedcoupon; positiveN evenzero inputs|
|FR-CAD-62|CAD62|All7modes×2aggregations×signedpointprice/nominalscale/quantumscale and exact midpoint cases|
|FR-CAD-63|CAD63|Appendix certified sevennominals;1mcomponent-vs-total cent;1bnfullprice-vs6dp nonmidpoint discriminator|
|FR-CAD-64|CAD64|Continuousdeclared interval Exact unresolved evenendpointmultiples; equaldeclaredbounds point with declaredprovenance|
|FR-CAD-65|CAD65|Capital-first thenAI Exact precedence; intermediatewideoverflow+netcancellation; final lowerbeforeupper|
|FR-CAD-66|CAD66|Stageinputpointness vs intervalconstructor, quantizercollapse thennetpoint; no endpoint-only universalExact|
|FR-CAD-67|CAD67|Decimalrangeholes/canonicalcoefficientlimits/OutOfRange sign and operation; no clip or suggestion|
|FR-CAD-68|CAD68|Dirtyallin suppliedevidence vs observedbridge and exactAIinvoice, no unjustified cross-recipe guarantee|
|FR-CAD-69|CAD69|Owned cash snapshotgeometry/fullcount/source ratios/None semantics, sourceviewfailure independent fromcash|
|FR-CAD-70|CAD70|EqHash retainscomplete reconstructible instrument viaF/L/anchor/f/EOM; equalcashdifferentL counterexample; selection/evidencekind; determined vs nondetermined provenance|
|FR-CAD-71|CAD71|All33+5 structuredvariants/stablekind/sourcechain/stages/indices; capacityproof exclusions distinguished from valid runtime failures|
|FR-CAD-72|CAD72|Preproductionadmission guard/schemas/pins/mutants/debugrelease/Clippy/docs/targets/package/coverage; allocation/liveness review is not fixture metadata|

The incorporated full preproduction protocol requires FOUR independent corpora:
source/date/cash, prescribed numerical observations, certified ideal price/six-risk,
and target-driven inverse, plus separate blind source/cash and numerical hands.
Source/cash and numerical expectation producers are distinct from the ideal/inverse
route. Expected-result authors do not read future Canadian production. Reproduction is
labelled reproduction, not another independent derivation. All old frozen artifacts
stay unchanged. Exact schemas/canonical semantic input identities, actual --check,
unchanged shared-reader admission, complete pins/censuses and tool-integrity controls
must all pass. A subset is interim, never the complete gate.

New corpus bytes are canonical LF under their byte attestor; this does not narrow the
unchanged shared reader's own LF/CRLF acceptance. Predeclare informative comparisons
for each quantity/major ordinary regime before production, as the complete protocol
requires. Exact integer-stage references may use supplied represented g/factor seeds;
native elementary outputs instead use certified intervals plus their declared profile,
not one platform's exact libm bits on every target. Public ideal-bound consumers carry
that native variation through all observation/normalization stages.
An expected single exact composed value/error/guard class must be stable across the
allowed primitive error. Otherwise predeclare a justified bounded class/interval and
separate exact-seeded stage control; an unconstrained wildcard is not acceptance.

Ideal inverse references declare the external supplied price basis and by default use
exact source A/AI, distinct from a separately labelled cached-resolved ModelDirty
route. Conditioning/error budgets include basis conversion and target-shift RN errors
when relevant. Repeated derived model targets do not become independent financial
inputs. These reference distinctions do not change C6's actual cached target protocol.

The order is noncircular: (1) complete complementary review and adopt the entire
immutable source/model/API/cash/numeric/resource contract, including all final pins;
(2) author, independently review, attest and freeze ALL required preproduction evidence
from it; (3) begin Canadian Rust only after that complete gate passes; (4) bind actual
public operations/private kernel to all outputs, kill meaningful compiled semantic
mutants, and run full native debug/release, strict lint/fmt/docs, foreign compilation,
extracted package checks, measured production coverage and honest missing-region
classification. Actual foreign runtime, independent financial/owner acceptance,
maintained calendar/legal interpretation, release and wider M8 remain separate gates.

This complete local contract is adopted as one scope. The complete independent
preproduction gate remains mandatory before any Canadian production; adoption
alone is not an implementation result or financial/platform/release acceptance.
