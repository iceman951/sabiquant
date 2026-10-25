# Price enclosure resource proposal

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

Companion to REPORT.md/API_PROPOSAL.md; no repository/production/API/requirement
adoption. This is an abstract bounded implementation route with a complete operation
and lifetime schedule, not measured Rust stack, allocation or platform conformance.
All32 generic configurations, signed flows, full Gilt/Treasury/Canadian f2/4/12,
odd/composite/ex-coupon/trueT and every existing basis remain in scope.

## 1. Fixed parameters and representations

Candidate precisions p=64/128/256/512/1024, fixed per call. Let M=16 mantissa words,
E=33 exponent words, B=2176 exact-input bits (34 words), W=52 arithmetic words
(3328 bits). These are proposed capacities, not adopted constants.

An endpoint is sign * m * 2^(K-(p-1)), where nonzero m has exactly p significant
bits and K=floor(log2(abs(endpoint))). K is signed magnitude,33 u64 words; zero has
canonical zero sign/m/K. Use one flags u64 for sign/zero/exponent-sign and keep p
in the enclosing frame. Logical endpoint payload=(16+33+1)*8=400 bytes; interval800.
Do not confuse floor-log exponent K with the unit exponent e=K-(p-1).

A raw exact ratio is sign*N/D*2^e, with N,D34 words each, D>0, and two control
words including the small raw binary exponent. Payload70words=560bytes. Denominator
forms below are exact; no native result or observed cache substitutes for them.

Outputs borrow plans and use borrowed writes; no endpoint/interval/raw ratio is
returned or passed by value. All large working values live in one explicit arena.
Helpers take distinct borrowed input/output slots plus disjoint scalar scratch.
They return only a small status/error. Rotate slot indices, not hidden cloned arrays.

## 2. Exact source/input and positivity bounds

Finite binary64 x has x=sign*s*2^e, s<2^53, -1074<=e<=971 (unnormalized subnormal
significands permitted). Exact y*t has <=106 significand bits and -2148<=e<=1942.

| Input form | Constructive raw bound before normalized conversion |
|---|---|
| Generic amount/time/y/face/cache AI | Decode53-bit s plus e; no shortest-decimal interpretation. |
| Generic Simple q=1+y*t | With e0=min(e_product,0), N=2^-e0+sign*s_product*2^(e_product-e0), D=2^-e0. N,D fit2150bits, including carry. Exact sign check precedes conversion; a positive q is >=2^-2148 (deliberately loose). |
| Periodic b=(m+y)/m, m<=365 | N=m*2^-min(e_y,0)+sign*s_y*2^max(e_y,0); D=m*2^-min(e_y,0). Both fit1084bits. Exact N>0 is checked. |
| Treasury q=(2s+r*y)/(2s) | Same affine construction with2s<2^9, r<2^8;1084bits suffice. |
| Canadian MM1+y*Ddays/365 | Same construction with Ddays<2^22 and365<2^9;1084bits suffice. Exact numerator sign uses full integer subtraction. Applies to tau and original rho; trueT is retained. |
| Gilt source/AI | Exact source numerators<2^148, denominators<2^144; signed AI retained (gilt.rs source_rational/contract N5,C8). |
| Treasury raw source/selected AI | Raw source numerator<2^152, denominator<2^144; selected published AI is its exact publication count/decimal basis, not raw or binary cached AI. Five-word storage and explicit count form are borrowed. |
| Canadian source/A/AI | Canonical numerator<2^160, denominator<2^148; first/last/regular sources and terminal par retain exact ratios and original geometry. |

B=2176 dominates every raw affine above, including generic Simple2150bits. No
unchecked shift by a float's value or a huge computed exponent is used. An optional
sharper Simple-domain proof exists: when y<0 and1+y*t>0, the exact product is either
<1/2 or lies[1/2,1) on a grid no finer than2^-106, giving q>=2^-106. The coarse
2^-2148 route is sound and does not require this optimization.

Exact invalid sign is a domain error. The mandatory core denominator route performs
that exact Raw sign check and point conversion: a positive rational produces a
strictly positive normalized lower significand with extended exponent, so it never
underflows to0 internally. MM/Treasury/Simple divisors and positive exp reciprocals
therefore exclude0 in all admitted core executions. A zero-containing primitive
divisor remains a private checked-invariant defense, not an unused public unresolved
variant. A separately adopted conservative interval-affine alternative could make
such arithmetic ambiguity reachable; only then would refinement/exact fallback/
unresolved policy need public specification, never InvalidYield/assert/panic.
Near-unit log intervals can cross0 as arguments without creating this divisor case.
New admission remains independent of old native factor/PV representability gates.

For log reduction, exact k=floor(log2(N/D)) comes from bit lengths and one aligned
comparison. Normalize m=b/2^k in[1,2). The coupled shift makes both normalized ratio
members fit1085bits, not1084+abs(k). Then z=(m-1)/(m+1) fits1086bits. Thus exact z
construction fits the same raw slots. Source data are borrowed; no date/calendar,
geometry search or source cache cloning occurs during numerical reuse.

## 3. Normalization/division/multiplication widths

For exact N/D*2^e with input bit lengths<=B, determine k, and unit U=k-(p-1).
If U>=0, shifting D by U fits at most B+1bits by the bit-length relation. If U<0,
shifting N by -U fits at most B+p bits. Allow two spare bits for comparison/carry:
B+p+2<=3202<3328. W52 therefore covers the aligned remainder/divisor and corrections.
The quotient is p bits with at most one carry; p+1<=1025bits. A52-word quotient
buffer also supports raw exact-point divisions with <=B+1=2177 quotient iterations.

Normalized mantissa products have <=2p<=2048bits, with carry/comparison allowance
<=2050<3328. Signed aligned adds need <=p+2bits. A raw multiplication is admitted
only when the operand bit-length sum is <=B; otherwise an optional exact-identity
shortcut stops without claiming identity. All mandatory affine/source constructions
and the concrete shortcuts below have separate proven sums within this limit.

ScalarMul/Normalize have an explicit PAIRED-ROUNDING contract: one exact product
and one aligned quotient/remainder produce BOTH directed endpoints in the L15
candidate pair. Use retained remainder/sign/parity/carry to select floor and ceil;
write both mantissas AND both33-word exponents, including any power-of-two carry.
The proposed Normalize charge includes both writes and both metadata adjustments.
Four such paired products suffice for general signed interval multiplication. If an
actual helper returns only one directed endpoint or reruns division for the other,
it must be charged as8 products/normalizations instead; the4-call cost cannot be
claimed for that implementation. This choice must match actual code before adoption.

Bitwise long division uses a borrowed divisor shifted to the quotient's highest
possible bit, then repeated compare/subtract/right-shift. Normalization quotient
iterations<=p+1; raw integer quotient iterations<=B+1. No loop scans all absolute
exponents or every bit below the aligned window. Caller preconditions require
positive denominator and sufficient output width; no truncated high limb is success.

## 4. Signed alignment and BOTH exponent directions

For endpoint inputs a,b, set K=max(K_a,K_b), u=2^(K-p+1). Align each to u using
floor for the lower sum and ceil for the upper sum, respecting sign. If a shift
difference is large, compare its33-word value to p+2; a tiny positive operand is
in(0,u), hence floor0/ceil1; a tiny negative is in(-u,0), hence floor-1/ceil0.
Only differences<=p+2 become actual bit shifts. This is an exact sticky decision,
not a dropped tail. The two aligned signed integers fit p+2bits.

Let A=max(abs(a),abs(b)). Since u<=A/2 for p>=2, aligned magnitude<=2A+2u<=3A;
final outward normalization is within factor2, giving magnitude<6A<8A. Thus the
upper floor-log increases by at most3 per scalar signed add.

For a NONZERO aligned integer sum, magnitude>=u. Renormalization therefore gives
K_new>=K-p+1, and unit exponent e_new>=K-2p+2. It loses at most p-1 bits relative
to the larger input unit exponent. Exact zero is separate. This lower bound matters:
a positive-flow upper-growth bound alone does not cover deep signed cancellation.

Charge the loss against **all signed-add primitive calls**, not just flow count.
Every primitive consumes >=1 work tick and max_work<=u64::MAX; consequently
W_add<2^64 and accumulated cancellation loss <p*2^64<=2^74. The same bound covers
AI shifts, series and final source/unit arithmetic if counted. Native observations
or a rounded zero cannot waive domain checks or establish a constant function.

## 5. Whole operation-graph exponent bound and remaining caveat

REPORT proves exact exp argument magnitude<=2^2048, after intersecting with the
independently justified model envelope. Select s=ceil(log2(2abs(x))) if abs(x)>1/2,
otherwise0. For a power-of-two endpoint use K+1, otherwise K+2; the clipped maximum
2^2048 is exactly a power of two. Hence s<=2049, including that boundary.

Positive exp(a), a<=1/2, is intersected with[1,2]. Each directed square gives the
H recurrence H0=1,Hnext=2H+1; after s, H=2^(s+1)-1. A final reciprocal changes
this loose floor-log envelope by at most1. Thus completed exp endpoints have
abs(K)<=2^2050. K-(p-1), signs and all exponent sums/differences must use full-width
checked arithmetic;32-bit exponent shortcuts are invalid for the full generic scope.

Intermediate logarithm/Taylor terms also need lower bounds before that intersection:
exact nonzero z has numerator>=1/denominator<2^1086, so its converted positive lower
K>=-1087. z² is >=2^-2175 after loose directed rounding. At most p<=1024 recurrence
multiplications put log terms above2^(-2^22), allowing small coefficient divisions.
The final k*ln2 add/subtract can lose at most p-1 more bits. With finite t and m,
nonzero exp-input endpoints conservatively have K>=-2^24. Scaling by s<=2049 keeps
a above2^(-2^25) when nonzero. Taylor2p<=2048 steps, each multiplying by a and
dividing by j<=2048, conservatively keep nonzero term K>-2^37. Upper term/sum
magnitudes before the[1,2] intersection are also finite small powers (a loose2^14
floor-log bound suffices). Zero endpoints are canonical and do not require -infinity.
These bounds are deliberately wide and require independent line-by-line review.

Outside exp squaring, the operation graph only scales a completed factor by bounded
source/affine/unit terms; it never repeatedly multiplies unrelated enormous prices.
Generic Simple terms fit a loose floor-log envelope±4096. Canadian MM affines have
positive magnitude>2^-1083 and upper<2^1047, so source-scaled MM terms also fit±4096.
Gilt/Treasury/CAD exact source coefficients and source AI have small fixed envelopes;
generic100/face adds at most1082 floor-log units. Streaming reduction and all shifts
add at most O(p*W_add), bounded above by2^74, in either exponent direction.

Therefore completed and intermediate graph endpoint K fits a conservative
abs(K)<2^2051 envelope; exponent-operation temporary differences/sums/unit offsets
are safely below2^2053.33-word signed-magnitude storage has2112 magnitude bits,
leaving room for these exact operations and their one-bit carries. This closes a
constructive candidate range, not actual Rust carry code or simultaneous-frame proof.
The intermediate-term constants/source envelopes and every operation edge still
need independent review; a generic unrestricted interval API is NOT proposed.

## 6. Scalar scratch, output borrowing and lifetime schedule

One reusable scalar frame:

| Field | u64 words |
|---|---:|
| Two raw ratios, each N34+D34+control2 |140|
| Four arithmetic scratch arrays W52 |208|
| Quotient/output integer buffer W52 |52|
| Three exponent-operation scratch arrays E33 |99|
| Scalar counters, signs and precision controls |32|
| **Total** |**531=4,248bytes**|

Scratch roles are remainder, shifted divisor, product/aligned signed temporary,
and held correction/input. Division's quotient is separate. Source/point/raw loops
share this frame; do not create a second frame in a helper. Checked field splitting
must let inputs borrow Raw0/Raw1 while mutable arithmetic outputs borrow disjoint
scratch. Exponent temporaries never alias a live input. Normalization writes into
a distinct endpoint slot. Large by-value locals, closure captures, Copy/Clone and
helper return values are forbidden by the proposed schedule; actual compiler
moves/stack must still be inspected rather than inferred from this rule.

Reserve8 global interval slots G0..G7 and16 local kernel slots L0..L15.
Algorithm state uses only L0..L12. Permanently reserve L13 as the stable per-precision exact unit interval for
reciprocals (other coefficients write directly to L0..L12 or raw slots), L14 for
interval reciprocal composition, and L15 for one scalar candidate
floor/ceiling pair. Interval multiplication evaluates its four exact endpoint
products sequentially into L15, comparing/copying extrema to its distinct output.
No recursive interval primitive creates another local arena. Thus constructor and
candidate outputs are counted, not just named series variables:


- G0 dirty accumulator; G1 common log(base); G2 fixed Treasury q/MM d;
  remaining globals hold the current amount, coordinate/affine, argument/factor,
  product and replacement sum. At most8 globals are live.
- Log: Raw0 builds b; Raw1 holds normalized m; Raw0 receives z, releasing Raw1.
  Atanh uses local z,z²,term,sum,coefficient,product,replacement sum,replacement term
  (8 slots). Keep log(m) in L8 while reusing L0..L7 for ln2; k*ln2/final sum uses
  at most L9..L12. Output common log to G1; release all locals.
- Exp: borrow one argument endpoint; locals hold a,term,sum,coefficient,product,
  replacement term/sum,tail,combined answer,square destination,reciprocal destination
  (<=13 slots). After an endpoint is certified, write it to its G output endpoint.
  Retain that400-byte endpoint while evaluating the other; the same local16 slots
  and single scalar frame are reused. No recursive exponent or series calls.
- Compound flow: amount G3, coordinate G4, argument G5, exp factor G6, term G7;
  release G4 after argument construction and use it for sum replacement. Rotate
  accumulator indices; release all current-flow globals before next flow.
- Simple: exact affine Raw->G4, reciprocal G5, amount G3, product G6, new sum G7.
- MM: fixed d G2, current amount G3/numerator G4, ratio G5, product G6,new sum G7.
- Treasury: common log G1/q G2; compute exp(-jlogb) as above, release argument,
  divide factor by q into a freed global, then amount/product/new sum. The rational
  integer-power specialization can reuse the same globals and local square slots.
- Final basis: release G1/G2 caches. Convert exact A/AI (or their exact rational
  difference) into freed globals, shift G0, then generic100/face scaling. Canadian
  D-A+AI is exact-source arithmetic, never observed Ahat/AIhat. Endpoint export
  occurs only after the complete signed target has been formed.

The arithmetic needs no PV vector/source/calendar clone or per-flow heap allocation.
Existing plan vectors remain borrowed. The abstract implementation receives one
caller-loaned arena and writes all aggregates in place. Physical entrypoint placement
(stack or one explicitly reviewed fallible reservation) remains an unadopted API/
resource decision. Do not silently promise allocation freedom before that decision.
A safe Rust by-value aggregate constructor/return can make additional whole-frame
copies; borrowed helper signatures alone do not eliminate initial construction.
Actual constructor, repeat initialization, return transfer and layout/MIR measurements
must close that separate question. The resource proof does not assume optimizer copy
elision. The finite result must borrow only the plan, never this working arena.

## 7. Conservative logical payload arithmetic

| Simultaneously reserved component | Bytes |
|---|---:|
|8 global +16 local intervals, each800 |19,200|
|One scalar frame |4,248|
|Outer controls/input views/budget/counters reserve64words |512|
|Four typed finite/Outcome/Result/caller-return reserves, each512 |2,048|
|Typed small error/status/completion staging reserve |256|
|**Total proposed explicit payload** |**26,264**|
|Margin to32KiB comparison point |6,504|

This counts incoming borrowed outputs as arena slots, retained first exp endpoint,
point-detection scratch and four distinct512-byte return/construction positions:
report construction, Outcome construction, Result construction and caller ABI/result
transfer. Small completion/error construction has a separate256-byte reserve. Helpers
return only small completion/status; no helper returns a full finite report or arena.
These reservations include proposed discriminant/alignment headroom, not measured
actual public type sizes. Existing16-aligned SettlementEvidence/typed reports/errors
must be actual typed locals/fields outside the u64 working arena; no u64-byte buffer
is reinterpreted as those types. Their exact size/alignment/MIR must still be checked.
The earlier24,344-byte marker under-reserved a possible160-byte error and return
staging; this generous revision corrects it without assuming copy elision. It does not certify
Rust struct padding, ABI return slots, compiler copies, inlining, spill space,
formatting stack, thread stack, allocator behavior or platform conformance. Measure
actual safe Rust layouts/live frames and audit borrows/temporaries before adoption.
The32KiB comparison point is a proposed engineering target, not a silently adopted
budget from a different model. No denominator/exponent arrays escape in public errors.

## 8. IEEE outward export and compact reports/errors

Inspect endpoint sign and K using E33-word comparisons. K>1023 may require an
unavailable outward upper (or lower for a negative value); it is not automatically
final-price overflow. K<-1074 exports0/MIN_SUB toward the appropriate direction.
Otherwise choose IEEE unit max(K-52,-1074), inspect/discard at most p significand
bits with sticky/remainder direction, handle carry and MAX exactly. Only after K
is proved in this small range may it become a small signed integer. No shift by
a huge K is executed. Signed bounds/cancellation and negative clean remain valid.

Finite reports retain existing finite SettlementEvidence plus small immutable
model/basis/yield/budget/work metadata and a borrowed originating plan. Exact decimal
points are inspected through an encoding enum, not falsely presented as dyadics.
ExactRationalIdentity and IntervalCollapse are distinct proof facts. All exported
kinds remain DECLARED evidence; receipt guarantees stay unchanged.

Unresolved/errors retain only small reason/model/basis/stage/index/budget/work and
admission-complete status. Huge internal endpoints are discarded with the arena;
no33-word exponent, raw ratio or whole frame enters an error or owned unresolved
report. Target an actual Error layout<=160bytes (existing threshold161); the256-byte typed status reserve is NOT a measured guarantee. Finite report's existing
evidence size/layout and borrowed context must also be measured before freezing
its exact size. Do not return private huge endpoint views in the initial API.

## 9. Exact-point shortcuts without an unbounded rational engine

Mandatory uniform interval coverage never depends on detecting every rational/tie.
Cheap zero/unit-factor identities are allowed only after full exact admission.
Generic y0/t0 common-basis sum: each prepared amount on2^-1074 has integer numerator
<2^2098; <=2^64 terms plus cached AI fit2163bits. Multiplying by100 fits2170bits.
Divide by the face significand<2^53 and retain e=-1074-e_face in[-2045,0]. This fits
Raw34 without materializing a larger denominator or converting through a float.

Named y0: use a common denominator proved from the finite cache roles, not an
LCM over arbitrary caller rationals. Gilt has regular/one published first/one source
AI denominator; a product of these three bounds is <2^432. Treasury raw regular/
first and selected published AI likewise use finitely many cache forms. Canadian
selected regular/first/last plus A/AI give at most5 denominators<2^148, whose product
is <2^740; terminal par canonicalization only divides its coupon denominator.
Numerator N_source*(commonD/D_source)<2^900, summing<20000 terms and shifts fits
<918bits. Raw34 easily covers these constructions. Cache-role/divisibility mapping
must be checked against every actual plan branch before this shortcut is adopted.

To detect a fitting decimal/binary point, strip2/5 factors from the raw denominator;
let R be the remaining factor. A single exact division N/R must have remainder0
(otherwise no finite decimal/binary point exists). Strip remaining2/5 from numerator,
combine the small binary exponent with the2-power count, then test scale<=38 and
coefficient range. Reject a too-large coefficient BEFORE multiplying by extra decimal
powers. Each raw quotient has <=2177 iterations; factor stripping has <=2176 iterations
per prime/operand. No unrestricted gcd or giant integer power is required. Binary
point encoding additionally proves a<=53-bit significant odd part, exact subnormal
grid, and finite IEEE range. Exact zero uses canonical decimal0.

Arbitrary factor grouping/signed identities are optional and must have an explicit
bounded traversal (or only cheap adjacent exact pairs); no hidden map/sort/clone is
assumed. A failed shortcut falls back to intervals if budget remains, or valid
unresolved after complete admission. It never asserts an unproved exact identity.
This explains exact100.1 without promising every rational or boundary is decided.

## 10. Deterministic work proposal, not just a capacity table

Define an abstract work tick as one bounded limb-loop step, scalar metadata step,
or charged branch/slot update. A concrete implementation must refine this into a
complete auditable cost table and charge BEFORE a step/block. Counter arithmetic
uses checked u64; exhaustion stops before the operation, never after an overrun.
No promise equates ticks with processor cycles. Every primitive consumes>=1tick.

Initialization and terminal reserves are explicit. The complete logical payload has
3,283 u64-equivalent slots. Candidate C_init=16*3283+128=52,656 ticks is debited BEFORE arena
initialization/zeroing, including repeat initialization on every call. This is prepaid upper-charge
accounting for working-word initialization AND typed construction/transfer reserves;
it does not mean arbitrary typed values can be initialized by zeroing bytes. Candidate
C_exit=1,024 ticks is reserved before variable work for every finite/unresolved/error
construction and arena-loan return. Data/arithmetic steps cannot spend the exit
reserve. On return charged_work includes the fixed exit debit and stays<=max_work.
The exact output-copy implementation must be bounded by that prepaid reserve.

The proposed budget constructor therefore requires max_work>=C_init+C_exit=53,680,
not merely nonzero. max_work1 is a bounded configuration error with attempted fields
and work0; it creates no arena or producer run. This constant-time constructor/field
validation is explicitly outside the producer work counter. Once a valid call begins,
all runtime bookkeeping/report construction is covered. These candidate costs/minimum
are unadopted until matched to actual code, layout and work units. Formatting a returned
value later is a separate caller operation and does not alter numerical work/results.

A deliberately conservative proposed upper-charge table (M16,E33,W52):

    Compare =4E+4M+32.
    Normalize(p,q)=16W+16E+64+q*(8W+32).
    q<=p+1 for normalized division; q<=B+1 for raw quotient.
    ScalarAdd <=Normalize(p,p+1)+16E+16M+64.
    ScalarMul <=Normalize(p,p+1)+16M^2+32M+16E+64.
    ScalarDiv <=Normalize(p,p+1)+8E+8M+32.
    IntervalAdd <=2*ScalarAdd+4Compare.
    IntervalMul <=4*ScalarMul+8Compare.
    IntervalReciprocal <=2*ScalarDiv+4Compare.
    IntervalDiv <=IntervalReciprocal+IntervalMul.

These upper charges cover long-division compare/subtract/right-shift passes,
quotient steps, fixed scratch initialization/copies, exponent operations and carries
at the described algorithm level. Exact implementer word-operation constants remain
an explicit proof obligation; this is not yet a claim about unseen code. Raw affine,
source conversion and point shortcuts use their own B-word/bit-length work, never
are charged merely as one financial row. Avoid overflow in cost calculations by
checking remaining budget before multiplication/addition of counts.

Atanh N=p: one exact input conversion, one z², then N coefficient conversions,
2N interval multiplications,N interval adds, one tail construction/add. Log uses
at most two Atanh evaluations, one small k conversion/multiply and one signed add,
plus exact range reduction/raw work. Do not omit k<0 or a zero/tiny input branch.

Exp of one endpoint: range-reduction metadata work;2p coefficient conversions,
4p interval multiplications,2p interval adds, one tail/add/intersection; at most2049
interval squarings; at most one reciprocal. ExpInterval performs BOTH endpoints;
retaining the first output does not make the second free. Special exact0/unit paths
have smaller actual charges but preserve all admission checks.

Each flow pays exact domain/admission work (including zero coefficient), amount/
coordinate or affine conversion, the selected Simple/Continuous/Periodic/MM/Treasury
factor route, signed amount multiplication and accumulator add. Hybrid mixed flow
branches are selected from the frozen threshold bits and counted separately. Generic
finite quote/global floor validation happens even for empty plans. Gilt/Treasury/CAD
source caches are read/converted with original ordinal and exact source policy.
Final basis source conversion/shift, generic normalization, point detection if tried,
finite export and compact report construction are charged. Full source admission
must finish before a valid-domain unresolved result is possible; earlier exhaustion
is admission-incomplete, not InvalidYield. No partial sum is a full-price enclosure.

The entrypoint can stop at any max_work; it need not precompute or multiply a huge
n*worst_cost. A saturating preflight bound may be diagnostic only. Every loop has
both its mathematical cap and the decreasing work counter. This makes total work
bounded even with an enormous admitted generic vector. Runtime pricing is streaming
O(n) at fixed p; optional nontrivial grouping must declare its own bounded complexity.

## 11. Exact missing proofs before adoption

1. Independent review of the intermediate lower-exponent constants and all actual
   source/cache denominator roles; validate no missed graph edge multiplies enormous
   values outside the H recurrence.
2. A safe bounded scalar prototype implementing W52 relative alignment/division,
   E33 signed metadata and exact directed carries, then peer challenge at every
   extreme, including many close signed cancellations and huge exponent differences.
3. Exact work tick constants matched to EVERY real helper/loop/zeroing/normalization/
   conversion/report path; do not inherit a loose source q cap that ignores quotient
   iteration count. Test exhaustion before admission, series, squaring and export.
4. Actual Rust field/return layouts, explicit caller/borrowed-output slot schedule,
   call-site temporaries and stack/heap measurements on the selected implementation.
   The26,264-byte payload does not establish those measurements.
5. Complete API/error/traits/lifetime/encoding closure, with compact unresolved/error
   summaries and no caller proof construction; old APIs/failure order remain frozen.
6. Independent preproduction against exact rationals/root/series certificates,
   informative ordinary and legitimate unresolved extremes, all models/bases/source
   policies/EOM/period/entitlement classes and all exact-cash stage/mode cases.

No repository/Git/remote edits and no new production are authorized by this proposal.

## 12. Actual bounded-width research witness and retained revisions

`bounded_primitive_probe.py` is a private Python capacity-controlled prototype with
33 fixed u64 exponent limbs, p<=1024, Raw2176 and working3328-bit checks. Actual
controls passed105 exact rational rounding pairs,600 rational operations,20 huge
metadata/cancellation controls and34 exact affine seeds. There were1,476 normalization
calls; observed maxima were raw2149,working3172,quotient1025 and exponent-magnitude
2052bits. These are finite width/alignment controls, not a whole-model runtime,
Rust allocation/layout, builtin-division-loop or work-cost conformance proof.

The initial probe counted maximum exponent bits only at integer input construction;
its add/sub-result constructors were not reflected in that statistic. The correction
records every constructed exponent and changes the maximum2051->2052; arithmetic and
all pass counts are unchanged. Exact initial files are retained as
`bounded_primitive_probe.reviewed-v1.py` and `bounded-primitive-results.reviewed-v1.json`.
No expected production source or frozen fixture was edited.

The exact original resource draft remains `RESOURCE_PROPOSAL.reviewed-v1.md`, SHA256
`de58fa40833bf8866695731c328e4690d3e15ab9b2e4cfec2d3a034a2a05915f`.
This revision names primitive candidate slots and precharges initialization/terminal
work following the independent liveness challenge. Further exact-review reports and
pins are retained in the separate resource evidence manifest. The earlier complete
design/API evidence93fced558... remains unchanged under `frozen-design-api-v1/`.

## 13. Raw/source and whole-model cost closure formulas

To make the proposed work graph executable rather than charging a source row as a
single step, use R=34 raw words and the following additional conservative candidates:

    RawAdd <=16R+8E+64.
    RawShift <=8R+8E+32.
    RawMul <=16R^2+32W+16E+64 (only when admitted bit-length sum<=B).
    RawAffine <=4RawMul+32R+16E+64.
    RawQuotient <=Normalize(p,B+1).
    SourceRead <=8R+8E+64.
    SourceConvert <=SourceRead+Normalize(p,p+1).

These cover fixed-width decode/alignment/sign work and source reads; their constants
are still proposed algorithm-level charges requiring actual helper matching. Exact
affine admission uses RawAffine, including zero source coefficients; no calendar or
source reconstruction query is hidden there. A full admission scan precedes price
arithmetic and has its own decreasing budget. If it stops, the outcome is admission-
incomplete, not a valid-domain unresolved price.

Atanh/Log/Exp formulas in §10 and these raw charges give complete per-flow maxima:

- Simple: SourceRead+RawAffine+2SourceConvert+IntervalReciprocal+IntervalMul+IntervalAdd.
- Continuous/Periodic/Gilt/Compound: SourceRead+2SourceConvert+IntervalMul(argument)+
  ExpInterval+IntervalMul(amount)+IntervalAdd; Continuous can omit the log and use
  its exact product argument. For generic Periodic, the coordinate conversion
  constructs exact m*t as <=62 significand bits (53+9) with its separate binary
  exponent; G1 is ln(base), so precisely one interval multiplication forms the
  argument. It does not use plain t without m. A global common-base Log is charged
  separately.
- Treasury: that compound-flow bound plus IntervalDiv for q, or its separately
  bounded exact integer-power route; raw q admission/conversion is global.
- Canadian MM: SourceRead+RawAffine+2SourceConvert+IntervalDiv+IntervalMul+IntervalAdd;
  tau/d admission/conversion is global, rho/n work remains per original flow.

Final source shift/unit/export work and optional exact-point work are additional.
Each named zero-yield common-denominator fold pays its finite cache denominator
products, then per source RawQuotient+RawMul+RawAdd+SourceRead, plus point-encoding
factor stripping and final raw quotient. A conservative stripping cap is4B divisions
by2/5 across numerator/denominator, each traversing R words; the small binary exponent
is handled as counts, never by an enormous shift. Before decimal coefficient growth,
check sign/exponent/coefficient bounds, including NEGATIVE net denominator exponents:
S=max(a,b,0), coefficient=oddN*2^(S-a)*5^(S-b). A too-large exponent/bit bound rejects
the shortcut before multiplication. Undetected exact points fall back to intervals.

No whole n*cost value needs to fit u64: runtime charges each bounded block before
executing it. A diagnostic preflight uses checked/saturating arithmetic and never
asserts a full result from an overflowed estimate. All32 configurations and each
mixed flow branch select one of these charged paths. These formulas close the
abstract traversal accounting; actual loop/carry/copy/constructor matching remains
an explicit missing production proof, not inferred from this cost specification.

## 14. Core divisor reachability closure

The exact Raw->positive extended-point denominator path is mandatory in this proposal.
Every required Simple/MM/Treasury denominator interval is strictly positive. Exp
reciprocal operands stay positive through the[1,2]/squaring route. Accordingly the
initial public API omits DenominatorEnclosureContainsZero rather than adopting an
unused placeholder. General signed interval division still has a checked internal
precondition; unexpected failure is not a financial domain verdict. A future
interval-affine alternative would need explicit adoption, rationale and actual
reachable controls before exporting its unresolved reason.

## 15. Raw/word/exponent clobber protocol before normalization

The capacity table alone does not resolve aliasing. The following concrete phase
rules are mandatory for a bounded core implementation:

1. A caller converts/releases live raw source operands before invoking an interval
   primitive. ScalarAdd/Mul/Div may clobber Raw0 and Raw1. Exact-point folds are a
   separate raw phase and do not call interval primitives while a raw accumulator
   or source operand is live. After a completed fold, point conversion is explicit.
2. ScalarMul forms its <=2p-bit signed product in W2; ScalarAdd forms its <=p+2-bit
   aligned signed integer in W2. Copy that exact integer to Raw0.N and set Raw0.D=1
   (2p<=2048<B2176). Only then may Normalize reuse W0..W3. Do not pass W2 as a
   read-only input while overwriting it as normalization scratch. Raw1 is free.
3. ScalarDiv stages signed SigA and positive abs(SigB) in Raw0; its exact unit
   exponent difference is in E0. ScalarMul's huge unit sum and ScalarAdd's huge
   alignment unit likewise stay in E0. They never enter Raw's small-e control field.
4. Normalize borrows Raw0 plus immutable external E_source=E0. Its bit-length k and
   relative unit U are small integers. E1 and E2 hold the directed lower/upper output
   K values; no fourth exponent array is required. E0 is unchanged until BOTH
   candidate endpoints have been committed. Input/raw exponent conversion from an
   ordinary source loads its small e into E0 first, then follows the same protocol.
5. For ScalarAdd, E0 is the selected large unit. E1/E2 temporarily hold each operand
   unit/difference, one operand at a time; after aligned integers are stored in W0/
   W1 and summed into W2/Raw0, those exponent temporaries are released to Normalize.
   Differences greater than p+2 are sticky decisions, never actual shifts.
6. The quotient remains in Q52. Derive floor/ceil and resolve a p+1-bit carry there
   BEFORE copying a <=p-bit significand into M16. Paired K outputs are written from
   E1/E2 to L15. Negative-sign bound ordering and exact-zero canonicalization are
   explicit. Four-product interval extrema reuse L15 sequentially; completed outer
   outputs are distinct G/L0..L12 slots. L14 holds reciprocal composition; L13 unit
   stays immutable for the duration of the primitive.

Actual safe Rust field/slice splitting, primitive call sites and return construction
must be compiled/reviewed against these phases. These rules provide a plausible
single-frame graph, not a claim that missing code has already satisfied the borrow
checker or has no compiler-created temporaries.
