# Canadian complete cash-resource design review

Independent full-route review promoted2026-10-23 Bangkok. Final index
`/private/tmp/sabiquant-canadian-cash-resource-review-v1/final-index.json`
SHA256 `a7e7a2cb4414d6217d822077837a9821c2850d2a865649ba46f03e6bd3906206`;
primary evidence SHA256
`b9274e40ef118c0f5e9b9cdf8b289feda9f2457d9e1cfe118dc9d9f1cde091f9`.
The original proof and both authoritative addenda are reproduced below.
The first addendum is preserved privately; its corrected v2 takes precedence.
All earlier scripts/outputs/snapshots and post-review byte differences remain
retained. This establishes mathematical capacities and available by-reference
routes for the complete proposed cash model, not actual Canadian Rust layout,
copy/allocation/stack measurements or financial acceptance.

The [independent complete resource challenge](M8_CANADIAN_RESOURCE_INTEGRATION_REVIEW.md)
found no remaining blocking design issue. Read with the
[complete local model](M8_CANADIAN_LOCAL_DESIGN.md),
[numeric/test companion](M8_CANADIAN_NUMERIC_AND_TEST_DESIGN.md),
[source/cash/API integration](M8_CANADIAN_CONTRACT_INTEGRATION_REVIEW.md) and
[preproduction protocol](M8_CANADIAN_PREPRODUCTION_PROTOCOL.md).


## Original complete independent cash proof

Original SHA256 `f37b484310e96c3c52b2a56d79cf22058def32655cf0b3fb44d9997af878fb53`.

### Canadian complete exact-cash capacity and resource design

PRIVATE independent review, 2026-10-23. No repository/Git/AGENTS edits. This
reviews the current full public proposal's four recipes, both aggregations,
seven signed modes, continuous evidence/Exact semantics, coefficient scales0..38,
and arbitrary admitted positive quantum. No fixed-coupon case is removed.
It proves a constructive bounded arithmetic route and available storage design;
it does not claim Canadian implementation/layout/allocation/stack measurements,
financial acceptance or contract adoption. Own SabiQuant Treasury/settlement/
amount/date source was read as design context; no QuantLib implementation was
accessed. The separately reviewed outward compound guard is outside this task.

## 1. Inputs, exact source forms and one shared source denominator

Write C=a_C/10^s_C, positive nominal N=a_N/10^s_N, quantum q=a_q/10^s_q.
Coupon/source C>=0, N>0 and q>0; 0<=s_C,s_N,s_q<=38, and positive coefficients
are <=i128MAX<2^127. Exact DECIMAL PRICE evidence also admits i128MIN, whose
magnitude is2^127, not strictly below2^127. Binary price bounds are finite and
may be signed/zero. Evidence retains its original decimal/binary and
supplied/declared kind, bits/bounds and canonical zeros.

Let B=b/d be one cached exact source coupon or source AI, b<2^160,d<2^148.
The cache form is available without an LCM over every quasi date:

- Valuation intervals are C/f times k plus at most TWO proper fractions a/s_1
  and e/s_2. A common numerator uses k*s_1*s_2+a*s_2+e*s_1; total quasi length
  <20,000 and s_i<=184 give magnitude<2^30. The denominator is
  10^s_C*f*s_1*s_2<2^148. Coupon coefficient multiplication gives numerator
  <2^157, with the declared160-bit allowance conservative.
- Canadian365 completed pieces use C/f. Proper pieces have the common
  denominator365*f*10^s_C, with bounded integer numerator per piece. Sum the
  full-count term and at most TWO proper pieces on this ONE denominator. Its
  numerator/denominator bounds are smaller than160/148.
- Monthly settlement AI uses the valuation form; monthly PAYABLE coupon still
  uses the proposal's payable form. The two amounts are never substituted.
- A same-quasi doubly partial interval uses its one actual fragment once.
  Empty accrued interval is exact0/1 with the prescribed empty geometry; C0
  with nonempty geometry remains distinguishable. Geometry is retained even
  when its amount is zero.

Every proper payable fragment is positive and <C/f (the only late proper
fragment is f2,s184,d183). All nonterminal actual coupons are on Q, so the
source interval partitions at most Q.len()-1 cells. Both a selected coupon
and AI therefore satisfy

  0<=B<=C*(Q.len()-1)/f
       <=(2^127-1)*19999/2 <2^141.

Valuation source diagnostic views obey the same upper bound. These are broad
bounds, not claims that the civil-date domain attains the maximal count and
coefficient simultaneously. Canonical reduction of a stored source pair can
only reduce its bounds; canonical zero is0/1. A gcd may use bounded Euclidean
integer division, or a fixed deterministic unreduced nonzero form can be
retained if the adopted identity contract explicitly chooses it. No gcd is
needed during price cash to fit the proven widths.

A cash endpoint combines with THIS one cached AI denominator once. Capital
and accrued components are finally expressed at the SAME q scale, so their
net is signed integer addition. Neither route multiplies a list of denominators
or forms an LCM over full schedules/components. Five words is capacity PER
unsigned source magnitude; a b/d pair of two five-word values occupies80 bytes.

## 2. Pure source/view route: eight words and the390-bit result

For a coupon or AI cash point, form the exact quantum count ratio directly:

  n=b*a_N*10^s_q,
  v=d*10^s_N*100*a_q.

The unsigned numerator is <2^(160+127+127)=2^414. Denominator is
<2^(148+127+7+127)=2^409. Every prefix product is positive and smaller than
the final product. No conversion of b/d, C/f, nominal, or exact cash to f64 or
to a public DecimalAmount is allowed before quantization.

For source diagnostic views omit N/100:

  n=b*10^s_q<2^287,
  v=d*a_q<2^275.

Perform one exact integer div/rem, apply the selected mode to the quantum
count, then multiply by a_q to obtain an integer coefficient at scale s_q.
The count/remainder/complement/carry all fit512 bits. It is INVALID to prove
this final multiplication by combining independent414-bit count and127-bit
quantum maxima: that naive541-bit bound loses the numerical coupling.

Instead every successful mode satisfies |R(x)-x|<=q (Exact succeeds only at
zero error). For pure cash,

  |x|=N*B/100 <2^(127+141-6)=2^262,
  |coefficient|=|R(x)|*10^s_q
               <=|x|*10^s_q+a_q
               <2^389+2^127 <2^390.

For a source VIEW, the corresponding coefficient bound is <2^269. Thus the
full positive count*a_q product fits eight words even when the count would
be large; every positive schoolbook partial sum is bounded by that full
product. Intermediate output can be far beyond i128. The390-bit claim is
sound, provided it is justified this way, not by unrelated maxima or an early
public amount gate. All39 scales, f2/4/12 including nonterminating C/12,
maximum coefficients, zero sources and arbitrary positive quantum remain.

## 3. Explicit forty-word evidence route supporting the2501-bit claim

Use a branch-specific exact rational for EACH price endpoint p/P:

- Binary endpoint x: E=2^1074, p=x*E is an exact signed integer with
  |p|<2^2098, P=E. Decode represented significand and bounded exponent directly.
- Decimal endpoint: p is its signed canonical coefficient, |p|<=2^127,
  P=10^s_p<2^127. i128MIN is handled by unsigned magnitude2^127.

There is no need to put decimal and binary endpoints on one enormous basis.
The evidence type itself supplies valid endpoint order/pointness. Decimal
endpoints with different scales can be handled individually; their original
continuous interval/point tag is retained. Positive scaling and exact
translation preserve that order. No giant endpoint cross-product comparison
is necessary just to recover already validated evidence ordering.

For RoundTotal invoice with exact AI=b_A/d_A, form

  S=p*d_A+b_A*P       (SIGNED addition/subtraction),
  V=P*d_A,
  n=abs(S)*a_N*10^s_q,
  v=V*10^s_N*100*a_q.

The numerator and denominator remain exact even when p is negative or its
magnitude nearly cancels AI. Canonical zero has positive sign. Do not round AI
to the analytic AI_hat cache or pre-quantize the price/source.

For binary endpoints, the separate products satisfy
|p*d_A|<2^2246 and b_A*E<2^1234, hence |S|<2^2247. Scaling by nominal and
10^s_q gives n<2^2501. The denominator is

  v<2^(1074+148+127+7+127)=2^1483.

For decimal endpoints, |S|<2^288 and n<2^542, while v<2^536. Both branches
therefore fit forty words (2560 bits), including one conservative carry bit
for division/remainder work. Dirty and component CAPITAL use the same route
with b_A=0,d_A=1; their bounds are smaller. Recipe identity/provenance and
stage selection do not change merely because AI happens to be zero.

The2501-bit claim is SOUND FOR THIS CONSTRUCTED ROUTE. Blindly taking the old
Treasury R0=2^1074*5^38 decoder and then multiplying by the ENTIRE148-bit
Canadian source denominator gives a naive2589-bit numerator bound, above40
words. Such an implementation needs explicit cancellation of the already
covered decimal factor, or the branch-specific route above. This review
selects the latter because its complete forms and bounds are explicit; it
does not approve an unexamined transplantation of the older R0 algebra.

## 4. Rounded price coefficients and component cancellation

All endpoint prices obey |x|<=MAX, and source AI<2^141. The binary MAX gap to
2^1024 is2^971, so |x+AI|<2^1024 as well. With nominal<2^127 and division100,
raw capital/dirty/total cash magnitude is <2^1145. Therefore every successful
rounded price-stage coefficient satisfies

  |c|<=|cash|*10^s_q+a_q <2^1273.

Its quantum count is also bounded by cash/q plus1, since q>=10^-38>2^-127;
no multiplication of an actually2501-bit quotient by an unrelated127-bit
quantum coefficient is required. The full product count*a_q fits forty words.
Pure accrued component coefficients remain <2^390. A signed net sum of capital
and accrued coefficients is conservatively <2^1274, comfortably below2560.

RoundComponentsThenSum completes capital quantization first, then accrued
quantization, then adds their signed coefficients EXACTLY at the same q scale.
There is no net quantizer, denominator alignment or LCM. Opposite signs subtract
magnitudes and canonicalize zero; same signs add with one bounded carry. Do not
admit either component to i128. Net cancellation can rescue individually
unrepresentable components, but cannot excuse an earlier Exact Inexact result.

For a fully reconstructible dated example, let f2,C=i128MAX,
D=I2024-01-01, Q at Jan/Jul through2026-01-01, one actual coupon M=T2026-01-01,
and S2025-01-01. This long interval has two completed quasi pieces by S, so
exact AI=C. For supplied decimal clean X=-C, nominal200 and q1, the two exact
components are -2C and+2C, both outside public i128 range; net0 is valid. All
seven modes agree at these integer counts. At q3 with Exact, RoundTotal still
succeeds at0 while components fail first at `cash capital` Inexact. Required
stage domains survive cancellation.

## 5. Seven signed modes, continuous domains and endpoint precedence

For n/v with v>0, divide |n| into floor count a and remainder r. If r=0 retain
the exact signed count for all modes. Otherwise Exact gives Inexact at that
point. For the other modes compare r with v-r (no oversized2r buffer):

- TowardZero leaves a; AwayFromZero increments a.
- TowardPositiveInfinity increments only for nonnegative n;
  TowardNegativeInfinity increments only for negative n.
- NearestTiesEven increments when r>v-r or at equality when a is odd.
- NearestTiesAwayFromZero increments when r>=v-r.

Restore the sign after rounding, with positive zero. All these successful
quantizers are monotone for real signed inputs. For a nonpoint continuous
price interval, positive N and exact constant AI preserve nonpointness until
a quantizer. Exact on such a required stage returns ExactStageUnresolved:
endpoint divisibility is insufficient for every real interior. No enumeration
of representable binary64 interior points is substituted.

Stage and representation order remain precisely the proposal's:

1. Coupon factory checks original actual coupon ordinal then N>0. Other
   factories check N>0 first. Retain source/provenance without optional source
   DecimalAmount/f64 admission, allocation or external query.
2. Pure coupon/AI quantizes at `cash coupon`/`cash accrued`; dirty and total
   invoice at `cash total`.
3. Components complete `cash capital`, then `cash accrued`, then exact `cash net`.
4. Each point Exact stage checks divisibility before later work. Each nonpoint
   Exact stage returns its Unresolved diagnostic before endpoint quantization.
5. Only FINAL results normalize/admit. Point inputs use endpoint `point`;
   nonpoint final stage images admit lower before upper with those endpoint
   labels. Capital quantization may collapse a later NET image to a point;
   total/dirty quantization keeps the original stage-input pointness for its
   own final diagnostic even if both rounded extrema coincide.
6. Equal admitted final extrema give Determined, unequal give QuantumAmbiguous.
   Bounds remain ordered and attain the extrema by monotonicity. This does
   not assert every interior quantum amount survives DecimalAmount precision
   holes. Equal declared bounds are mathematical points while their declared
   provenance/guarantee remains retained.

Actual controls include an interval between adjacent binary floats at2^20
with q=2^-32 (a valid decimal quantum). Both endpoints are exact multiples;
Exact nevertheless returns `cash total` Unresolved for the continuous interval.
Another control maps a nonpoint price interval to one out-of-range bucket:
RoundTotal fails at `cash total/lower`, while components collapse the net and
fail at `cash net/point`, with operation `sum` only in the latter case.

## 6. Final canonicalization, asymmetric range and exact/binary AI

A rounded integer coefficient c at q scale0..38 remains wide. If c=0 return
canonical decimal0/scale0. Otherwise repeatedly divide by10 while the scale
is positive AND the coefficient is divisible by10; at most38 bounded passes.
Only after this normalization test the asymmetric signed limits: positive
magnitude<=i128MAX; negative magnitude<=2^127. Thus i128MIN is admitted, while
positive2^127 is rejected. Negative zero is absent. Source/total final failures
retain AmountError OutOfRange operation `quantize`; final component-net
failures use operation `sum`. Point/lower/upper and stage are preserved.

The source ratio is not replaced by an optional public view. For regular
monthly C1, U=1/12 has no exact q1 source view, but nominal1200 coupon cash is
exact1 and succeeds with Exact. A source diagnostic failure cannot gate it.

The invoice uses exact AI, independently of analytic AI_hat. A monthly long
first period D=I2025-01-01, Q monthly through first coupon M=T2025-07-01,
S2025-05-01 and C1 has exact settlement AI=4/12=1/3. With clean supplied point0,
nominal100 and q10^-38, exact-source invoice differs from dirty supplied-binary
point RN(1/3). Both final amounts are representable. This is an explicit
source-versus-binary staging discriminator, not an assertion that arbitrary
supplied points enclose the ideal model. Dirty cash always retains ALL-IN
SETTLEMENT evidence; no compound ModelDirty substitution or promised bridge/
invoice equality is introduced. Price guarantees apply only to the original
price ingredient, and pure source receipts have no price guarantee.

## 7. Constructive zero-heap by-reference schedules

These are proposed call/storage constraints that must be implemented and audited;
they are not measured existing Canadian lifetimes. Helpers borrow source inputs
and accept caller-supplied mutable output/scratch. No recursive precision retry,
heap bignum, Vec, Box, String or temporary vector is required. Capture compact
owned source/provenance from already admitted cached metadata. No full Q/coupon
clone, calendar generation, I/O, clock, callback or external query is needed.

One pure-source design uses TEN eight-word scratch buffers:
scaled numerator, scaled denominator, quotient, remainder, shifted divisor,
remainder complement, rounded coefficient, normalization quotient/copy,
power10/product scratch, and one product/shift temporary. The division helper
borrows numerator/divisor and writes quotient/remainder to disjoint supplied
slots; subtraction/normalization mutate existing slots. Small carry/sign/scale
metadata are scalar. Never return a wide expression tree by value.

Count up to FOUR complete compact source-pair copies simultaneously, covering
incoming snapshot, an explicitly owned local/cache, receipt construction and
return transfer if needed. Each pair is2*5*8=80 bytes. The available bound is

  10*8*8 +4*80 =960 bytes <=16*8*8=1024 bytes.

Borrowed unrelated caller storage is not counted twice. The schedule has spare
headroom and does not rely on optimizer last-use elimination. A final code
review must either enforce this exact copy bound or provide another valid one.
A source-view entry can use the same frame with still fewer fields.

A price/interval design uses TWENTY-FOUR forty-word scratch buffers; one
available role partition is:

| Slots | Roles |
|---|---|
|0..3|price lower/upper exact numerator and denominator|
|4..7|constant exact AI and signed combined endpoint numerator/denominator work|
|8..11|scaled count numerator/divisor and product/decimal-power work|
|12..15|quotient, remainder, shifted divisor and complement/rounding work|
|16..19|retained rounded lower/upper capital and final lower/upper net coefficients|
|20..21|accrued coefficient and canonicalization output/copy|
|22..23|by-reference product/subtract/return work|

Only each role's fixed slot is owned; source cached five-word inputs remain
borrowed or appear in the separately counted compact copies. Per-endpoint
scaling/division reuses scratch after the lower rounded result is stored.
Component AI may call the ten-buffer pure quantizer while capital endpoint
buffers remain alive; resize its eight-word result INTO an existing price
slot by reference, then end the source scratch scope before final net work.
All numerator/denominator products proved above fit their assigned slots.

Including that source subcall and four complete source-pair copies gives

  24*40*8 +10*8*8 +4*80 =8640 bytes <=32*40*8=10,240 bytes.

This explicitly counts source subroutine scratch and snapshot copies rather
than presuming them free. The two endpoint pairs, cached outputs and final
normalization buffers are already in the24 slots. There are no extra by-value
forty-word numerator, denominator, quotient or subtract arrays hidden in a
callee. Use disjoint named fields/fixed borrow splits, not unsafe aliasing.
Existing old by-value helpers cannot be assumed to meet this new schedule
without their own call/copy census; the new design may use an equivalent
by-reference route without changing monetary observations.

Binary long division is bounded by the dividend bit count: at most414 bits
for pure source,2501 for price. Each step shifts an in-place remainder,
compares/subtracts the positive divisor and sets a quotient bit. Remainder
before subtraction is <2*divisor, so it needs at most410/1484 bits, below512/
2560. Quotient needs at most414/2501 bits. Subtracting r from divisor for
nearest ties needs no oversized2r. Schoolbook multiplication needs at most8^2
or40^2 limb multiply updates per product, one scalar u128 carry; positive
partial sums fit the proved full product. Canonicalization has at most38
small-divisor passes. These are finite algorithmic work bounds, not elapsed
runtime or compiled Rust allocation counts.

## 8. Available compact layouts, not measured Canadian layouts

All retained metadata are fixed scalars: exact C/nominal/q, original dates,
F/L/direct anchor/f/EOM/source policy, first/last kinds, optional S/selection/
convention, optional selected coupon ordinal/date/payment, interval/quasi pairs,
full count and at most two day ratios, source b/d, original evidence and
aggregation. Absence remains Option/tagged absence, not fake public0 dates.
No vector or borrowed instrument survives inside instruction/receipt.

An available conservative private field budget is:

| Group | Bytes reserved |
|---|---:|
|Common bond/report scalar identity, dates, options and enums|256|
|Selected source geometry and ordinal/payment metadata|192|
|Two five-word exact source magnitudes|80|
|Nominal, full rounding policy, recipe/aggregation tags|128|
|Original fixed-size SettlementEvidence, including both declared endpoints|128|
|Private alignment/wrapper reserve|240|
|Instruction total available budget|1024|

Own Date contains one i32; own DecimalAmount contains i128 plus scale; existing
evidence contains at most two such amounts and tags. The generous group budgets
leave room for fixed tagged options and alignment on the existing target family.
This is a constructive field-budget design, not an observed Rust size_of result
or a stable ABI promise. A receipt with this1024-byte instruction,32-byte amount,
16-byte guarantee option and16-byte padding reserve totals1088 bytes<2KiB.
Actual new type sizes/copy placement must later be measured/asserted; no unknown
future enum variant is silently certified. Identity/provenance fields are not
dropped to obtain the bound.

These payload limits exclude scalar metadata from the separate WIDE-scratch
metric but include its fixed size in the instruction budget. They do not claim
actual compiler stack<=1024/10,240 bytes. Caller Vec excess capacity, allocator
headers, input-owned schedule/caches, unrelated caller copies, formatter/String
allocation, spills, ABI copies and compiler frame padding have their explicit
separate qualifications. Ordinary formatting/collection APIs keep standard
allocation behavior. The proposed factory/evaluate arithmetic path itself needs
zero heap operations by construction; actual allocation counters remain a later
implementation gate.

## 9. Actual private controls and scope

The exclusive-create scripts use only Python standard-library integer/Fraction
arithmetic. The runtime design uses fixed arrays; Python big integers here are
offline independent reference machinery, not a proposed production dependency.
No frozen source, reference fixture or prior report was overwritten.

The first actual run passed19 constants,39,312 source-scale/frequency/nominal/
quantum/mode controls,14,112 signed price-width controls,70 signed midpoint
controls and28 required-stage examples. All53,532 quantizer invocations matched
an independent signed-floor oracle instead of the magnitude/complement route.
It recorded canonicalization, i128MIN asymmetry, component rescue versus Exact
failure, lower-first errors, continuous endpoint-only Exact counterexample,
net-collapse context and admitted-extrema precision holes.

The separate boundary run passed1,260 pure source/view controls, with1,264
additional oracle crosschecks including source-view independence and exact-AI
versus binary-AI discrimination. Six explicit integer bit-division boundary
controls executed the bounded remainder algorithm and matched Python divmod.
Those division bounds are primitive supersets, not complete financial states.
Observed first-run price numerator width2500 and denominator1482 corroborate
(but do not establish) the2501/1483 universal ceilings. Boundary source numerator
reached414 bits; source rounded coefficient reached387, below the390 proof.
A finite maximum is not asserted exhaustive over dated inputs.

Initial arithmetic SHA256:
`b722c10dc0e670771345319ecf5ac61e6da55b437e6b5c14bf50e000af6640b1`.
Boundary-controls SHA256:
`3f2cd9f88852fae7bfd1d39737f2d2693c3f4a0dea27f3783c74e9df048d3024`.

The source/common-denominator and full arithmetic capacity gates reviewed here
are closed MATHEMATICALLY for the explicit route. Actual Canadian code,
public-constructor errors/causes, measured layouts/lifetimes/allocations/stack,
frozen whole-contract evidence and financial/platform/release acceptance remain
separate requirements. No native Rust test count is increased by this review.

## Authoritative canonical-source and quotient clarification

Original SHA256 `b9715e0d355d231937ef7a61b67608d481031e9f060da4f2625b18037e07153e`.

### Final cash-resource clarification and canonical source choice

This addendum accompanies `review.md` and preserves that first proof draft and
all earlier outputs unchanged.

The root's complete-contract candidate chooses ONE deterministic cached source
identity: every nonzero b/d is reduced by gcd, and zero is exactly0/1. The
review's conditional allowance for a deterministic unreduced form is therefore
not the chosen candidate. All arithmetic widths only decrease on reduction.
This is preparation/constructor work; cash evaluation does not recalculate a
gcd, re-query dates or require a DecimalAmount source view.

For initial numerator<2^160 and denominator<2^148, one Euclidean step leaves
the first operand equal to the initial denominator and the second strictly
below it; hence both are below2^148. Every two further nonterminal
steps at least halve the earlier positive remainder, giving a conservative
ceiling of297 exact divisions. Each integer has at most160 bits; bounded bit
division needs at most161-bit remainder scratch. Thus even five-word320-bit
magnitudes, and certainly the proposed eight-word preparation frame, suffice.
A positive canonical gcd is divided out exactly; zero is handled separately.
No variable-precision allocation is needed.

`canonical-source-controls.py` actually passed1,404 exact controls covering
all39 scales, zero/minimum/maximum coefficients, all3 frequencies, and4 source
forms. Its extreme common-denominator coordinates are primitive capacity
supersets, not claimed date-coupled monthly/quarterly geometries. All values
matched Fraction, gcd-reduced identity and zero0/1; the largest observed Euclid
count was87, below the universal297 ceiling. This finite maximum is not an
exhaustive Euclidean worst-case claim.

The source-view finite coverage is explicit: `boundary-controls.py` executed
252 `view=True` cases and1,008 pure cash cases (1,260 total), plus4 independent
source-view/coupon and exact/binary-AI discriminator checks. The first regular
source matrix alone did not execute source views. Their current evidence is
not inferred from the mere existence of the function.

Cash integer division requires a WIDE quotient, unlike numeric relative53
rounding. Both the ten-buffer eight-word and24-buffer forty-word proposed
frames assign a separate quotient slot, alongside remainder, shifted divisor
and complement. No u64 quotient or hidden extra wide returned array is assumed.
All by-reference schedules and explicit source-pair copy bounds remain design
constraints for later code audit, not measured Canadian liveness.

The2KiB instruction/receipt statements remain AVAILABLE FIELD BUDGETS. No
Canadian type or actual new Rust layout/stack/allocation counter was created.
The zero-heap route is an algorithmic construction with fixed buffers/metadata;
actual implementation compliance remains a later gate.

Independent peer wording correction: the first Euclidean operand equals,
rather than strictly falls below, the prior denominator. This v2 clarification
preserves the original addendum and changes no capacity or iteration bound.

## All-scale controls and evidence qualifications

Original SHA256 `0f5579fc9fa9c99b6066a7610e152eb474dd27d69102e4f3c43420ce5c2c9c09`.

### Additional all-scale grid controls and final evidence qualifications

`scale-grid-controls.py` actually executed31,941 pure cash and21,294 signed
price controls, each compared with the independent signed-floor oracle:
53,235 additional quantizer crosschecks. All39 nominal scales and all39
quantum scales are paired. The pure matrix repeats that39x39 grid for each
f2/4/12 and all7 modes, with coupon scale=(nominal_scale+7*quantum_scale)%39.
Every coupon scale is therefore visited, but this is NOT all39^3 independent
coupon/nominal/quantum scale triples. Coefficients vary between minimum/maximum
and the chosen small positive quantum5. The signed price grid uses +/-MAX
binary evidence plus exact AI1/3. These are additional offline resource/stage
controls, not an exhaustive coefficient/date census or Canadian Rust tests.

The root independently reran unchanged derive.py and boundary-controls.py in
`/private/tmp/sabiquant-canadian-root-cash-resource-replay-v1/`. Both exited0.
The mathematical payloads match: arithmetic.json excludes only created_utc and
input_snapshots; boundary-controls.json matches fully. That is reproduction,
not independent mathematical authorship or a new native run. The root evidence
SHA256 is222232bc16b343656051fc731fb6c99b95b3611c2ad2566b8b9bcf246c70c519.

The original verification.json correctly recorded all six current input files
matching their captured snapshots at its own creation time. Subsequently root
clarified numeric N2's actual native ell_hat versus represented-input rounding.
The NEW postreview-continuity.json records that numeric document's different
current bytes, the exact diff and the unchanged N5 cash-resource section. The
main cash/source proposal and four own Rust source context files still match
the original snapshots. Do not convert the earlier all-match observation into
a claim about the later numeric document. All arithmetic/scripts remain
unchanged; none of the financial math payloads were silently refreshed.

The complete mathematical review is review.md with final-addendum-v2.md. This
scale/count/continuity addendum supplements its evidence. Instruction/receipt
sizes and zero-heap/liveness remain available design constraints, not measured
implementation facts. Canonical cache gcd and the wide cash quotient remain
explicit as in final-addendum-v2.md.
